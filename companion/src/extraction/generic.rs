use crate::extraction::{ExtractionContext, ExtractionResult, JobExtractor};
use crate::model::{Job, JobSnapshot};
use chrono::Utc;
use uuid::Uuid;

pub struct GenericCareerExtractor;

impl GenericCareerExtractor {
    pub fn new() -> Self {
        Self
    }

    fn extract_field(html: &str, selectors: &[&str]) -> Option<String> {
        for selector in selectors {
            if let Some(start) = html.find(selector) {
                let after_selector = &html[start + selector.len()..];
                if let Some(tag_end) = after_selector.find('>') {
                    let content = &after_selector[tag_end + 1..];
                    if let Some(close_tag) = content.find('<') {
                        let value = content[..close_tag].trim();
                        if !value.is_empty() {
                            return Some(value.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_meta_content(html: &str, name: &str) -> Option<String> {
        let patterns = [
            format!("<meta name=\"{}\" content=\"", name),
            format!("<meta content=\"{}\" name=\"", name),
            format!("<meta property=\"{}\" content=\"", name),
            format!("<meta content=\"{}\" property=\"", name),
        ];

        for pattern in &patterns {
            if let Some(start) = html.find(pattern) {
                let after_pattern = &html[start + pattern.len()..];
                if let Some(end_quote) = after_pattern.find('"') {
                    let value = after_pattern[..end_quote].trim();
                    if !value.is_empty() {
                        return Some(value.to_string());
                    }
                }
            }
        }
        None
    }

    fn extract_title(html: &str) -> Option<String> {
        let selectors = [
            "<h1 class=\"job-title",
            "<h1 class=\"position-title",
            "<h1 class=\"title",
            "<h1 class=\"jobposting-title",
            "<h2 class=\"job-title",
            "<h2 class=\"position-title",
            "<div class=\"job-title",
            "<div class=\"position-title",
        ];

        Self::extract_field(html, &selectors)
            .or_else(|| Self::extract_meta_content(html, "og:title"))
            .or_else(|| Self::extract_meta_content(html, "title"))
    }

    fn extract_company(html: &str) -> Option<String> {
        let selectors = [
            "<span class=\"company-name",
            "<div class=\"company-name",
            "<a class=\"company-name",
            "<span class=\"employer",
            "<div class=\"employer",
            "<span class=\"organization",
            "<div class=\"organization",
        ];

        Self::extract_field(html, &selectors)
            .or_else(|| Self::extract_meta_content(html, "og:site_name"))
    }

    fn extract_location(html: &str) -> Option<String> {
        let selectors = [
            "<span class=\"location",
            "<div class=\"location",
            "<span class=\"job-location",
            "<div class=\"job-location",
            "<span class=\"address",
            "<div class=\"address",
        ];

        Self::extract_field(html, &selectors)
            .or_else(|| Self::extract_meta_content(html, "og:latitude"))
            .and_then(|loc| {
                if let Some(lat) = Self::extract_meta_content(html, "og:longitude") {
                    Some(format!("{}, {}", loc, lat))
                } else {
                    Some(loc)
                }
            })
    }

    fn extract_description(html: &str) -> Option<String> {
        let selectors = [
            "<div class=\"job-description",
            "<div class=\"jobposting-description",
            "<div class=\"description",
            "<div class=\"content",
            "<article class=\"job-description",
            "<section class=\"job-description",
        ];

        for selector in &selectors {
            if let Some(start) = html.find(selector) {
                let after_selector = &html[start..];
                if let Some(tag_end) = after_selector.find('>') {
                    let content = &after_selector[tag_end + 1..];
                    if let Some(end_marker) = content.find("</div>")
                        .or_else(|| content.find("</article>"))
                        .or_else(|| content.find("</section>"))
                    {
                        let raw = &content[..end_marker];
                        let cleaned = Self::strip_html_tags(raw);
                        if !cleaned.trim().is_empty() {
                            return Some(cleaned.trim().to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn strip_html_tags(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        let mut in_entity = false;
        let mut entity = String::new();

        for ch in html.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                '&' => {
                    in_entity = true;
                    entity.clear();
                    entity.push(ch);
                }
                ';' if in_entity => {
                    in_entity = false;
                    entity.push(ch);
                    match entity.as_str() {
                        "&amp;" => result.push('&'),
                        "&lt;" => result.push('<'),
                        "&gt;" => result.push('>'),
                        "&nbsp;" => result.push(' '),
                        "&quot;" => result.push('"'),
                        _ => result.push_str(&entity),
                    }
                    entity.clear();
                }
                _ if in_entity => entity.push(ch),
                _ if !in_tag => result.push(ch),
                _ => {}
            }
        }

        result.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    fn is_career_page(url: &str) -> bool {
        let lower_url = url.to_lowercase();
        let career_patterns = [
            "/careers", "/jobs", "/positions", "/openings",
            "/opportunities", "/vacancies", "/employment",
        ];

        career_patterns.iter().any(|pattern| lower_url.contains(pattern))
    }
}

impl Default for GenericCareerExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl JobExtractor for GenericCareerExtractor {
    fn name(&self) -> &str {
        "generic"
    }

    fn can_extract(&self, context: &ExtractionContext) -> bool {
        Self::is_career_page(&context.url)
    }

    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult, String> {
        let title = Self::extract_title(&context.html)
            .unwrap_or_else(|| {
                context
                    .title
                    .clone()
                    .unwrap_or_else(|| "Untitled Position".to_string())
            });

        let company = Self::extract_company(&context.html);
        let location = Self::extract_location(&context.html);

        let description = Self::extract_description(&context.html)
            .unwrap_or_else(|| "No description available".to_string());

        let now = Utc::now().to_rfc3339();
        let job_id = Uuid::new_v4().to_string();
        let snapshot_id = Uuid::new_v4().to_string();

        let job = Job {
            id: job_id.clone(),
            company_id: None,
            title,
            canonical_url: Some(context.url.clone()),
            location,
            employment_type: None,
            salary: None,
            source: Some("generic".to_string()),
            external_job_id: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        let snapshot = JobSnapshot {
            id: snapshot_id,
            job_id,
            captured_at: now,
            source_url: Some(context.url.clone()),
            raw_text: Some(context.html.clone()),
            normalized_text: Some(description.clone()),
            title: job.title.clone(),
            company,
            location: job.location.clone(),
            salary: None,
            description,
            requirements: None,
            responsibilities: None,
            extraction_metadata: Some(serde_json::json!({
                "extractor": "generic",
                "version": "1.0"
            }).to_string()),
        };

        Ok(ExtractionResult {
            job,
            snapshot,
            source_url: context.url.clone(),
            extractor_name: "generic".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn career_page_context(url: &str, html: &str) -> ExtractionContext {
        ExtractionContext {
            url: url.to_string(),
            html: html.to_string(),
            title: None,
        }
    }

    #[test]
    fn can_extract_careers_urls() {
        let extractor = GenericCareerExtractor::new();
        let ctx = career_page_context("https://example.com/careers/software-engineer", "");
        assert!(extractor.can_extract(&ctx));
    }

    #[test]
    fn can_extract_jobs_urls() {
        let extractor = GenericCareerExtractor::new();
        let ctx = career_page_context("https://example.com/jobs/123", "");
        assert!(extractor.can_extract(&ctx));
    }

    #[test]
    fn can_extract_positions_urls() {
        let extractor = GenericCareerExtractor::new();
        let ctx = career_page_context("https://example.com/positions/frontend", "");
        assert!(extractor.can_extract(&ctx));
    }

    #[test]
    fn cannot_extract_non_career_urls() {
        let extractor = GenericCareerExtractor::new();
        let ctx = career_page_context("https://example.com/about", "");
        assert!(!extractor.can_extract(&ctx));
    }

    #[test]
    fn extract_title_from_h1() {
        let extractor = GenericCareerExtractor::new();
        let html = r#"<h1 class="job-title">Software Engineer</h1>"#;
        let ctx = career_page_context("https://example.com/careers/se", html);
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Software Engineer");
    }

    #[test]
    fn extract_title_from_meta() {
        let extractor = GenericCareerExtractor::new();
        let html = r#"<meta property="og:title" content="Backend Developer">"#;
        let ctx = career_page_context("https://example.com/careers/be", html);
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Backend Developer");
    }

    #[test]
    fn extract_uses_page_title_fallback() {
        let extractor = GenericCareerExtractor::new();
        let ctx = ExtractionContext {
            url: "https://example.com/careers/dev".to_string(),
            html: "<html></html>".to_string(),
            title: Some("Fallback Title".to_string()),
        };
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Fallback Title");
    }

    #[test]
    fn extract_defaults_to_untitled() {
        let extractor = GenericCareerExtractor::new();
        let ctx = career_page_context("https://example.com/careers/dev", "<html></html>");
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Untitled Position");
    }

    #[test]
    fn extract_sets_source_generic() {
        let extractor = GenericCareerExtractor::new();
        let ctx = career_page_context("https://example.com/careers/dev", "<html></html>");
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.source, Some("generic".to_string()));
        assert_eq!(result.extractor_name, "generic");
    }

    #[test]
    fn extract_sets_canonical_url() {
        let extractor = GenericCareerExtractor::new();
        let url = "https://example.com/careers/software-engineer";
        let ctx = career_page_context(url, "<html></html>");
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.canonical_url, Some(url.to_string()));
    }

    #[test]
    fn strip_html_tags_basic() {
        let result = GenericCareerExtractor::strip_html_tags("<p>Hello <b>world</b></p>");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn strip_html_entities() {
        let result = GenericCareerExtractor::strip_html_tags("A &amp; B &lt; C &gt; D");
        assert_eq!(result, "A & B < C > D");
    }

    #[test]
    fn name_returns_generic() {
        let extractor = GenericCareerExtractor::new();
        assert_eq!(extractor.name(), "generic");
    }

    #[test]
    fn extract_company_from_html() {
        let extractor = GenericCareerExtractor::new();
        let html = r#"<span class="company-name">Acme Corp</span>"#;
        let ctx = career_page_context("https://example.com/careers/dev", html);
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.snapshot.company, Some("Acme Corp".to_string()));
    }

    #[test]
    fn extract_location_from_html() {
        let extractor = GenericCareerExtractor::new();
        let html = r#"<span class="location">San Francisco, CA</span>"#;
        let ctx = career_page_context("https://example.com/careers/dev", html);
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.location, Some("San Francisco, CA".to_string()));
    }
}
