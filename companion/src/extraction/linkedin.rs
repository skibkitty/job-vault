use crate::extraction::{ExtractionContext, ExtractionResult, JobExtractor};
use crate::model::{Job, JobSnapshot};
use chrono::Utc;
use uuid::Uuid;

pub struct LinkedInExtractor;

impl LinkedInExtractor {
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

    fn extract_description(html: &str) -> Option<String> {
        let markers = [
            "<div class=\"show-more-less-html__markup",
            "<div class=\"description__text",
            "<section class=\"description",
        ];

        for marker in markers {
            if let Some(start) = html.find(marker) {
                let after_marker = &html[start..];
                if let Some(tag_end) = after_marker.find('>') {
                    let content = &after_marker[tag_end + 1..];
                    if let Some(end_marker) = content.find("</section>")
                        .or_else(|| content.find("</div>"))
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
}

impl Default for LinkedInExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl JobExtractor for LinkedInExtractor {
    fn name(&self) -> &str {
        "linkedin"
    }

    fn can_extract(&self, context: &ExtractionContext) -> bool {
        context.url.contains("linkedin.com/jobs")
            || context.url.contains("linkedin.com/jobs/view")
    }

    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult, String> {
        let title = Self::extract_field(
            &context.html,
            &[
                "<h1 class=\"top-card-layout__title",
                "<h1 class=\"t-24 t-bold",
                "<h1 class=\"job-details-jobs-unified-top-card__job-title",
            ],
        )
        .unwrap_or_else(|| {
            context
                .title
                .clone()
                .unwrap_or_else(|| "Untitled Position".to_string())
        });

        let company = Self::extract_field(
            &context.html,
            &[
                "<a class=\"topcard__org-name-link",
                "<span class=\"topcard__flavor--bullet",
                "<a class=\"job-details-jobs-unified-top-card__primary-description-link",
            ],
        );

        let location = Self::extract_field(
            &context.html,
            &[
                "<span class=\"topcard__flavor--bullet",
                "<span class=\"job-details-jobs-unified-top-card__bullet",
            ],
        );

        let description = Self::extract_description(&context.html)
            .unwrap_or_else(|| "No description available".to_string());

        let external_job_id = context
            .url
            .split('/')
            .rev()
            .find(|s| !s.is_empty() && s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
            .map(|s| s.to_string());

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
            source: Some("linkedin".to_string()),
            external_job_id,
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
                "extractor": "linkedin",
                "version": "1.0"
            }).to_string()),
        };

        Ok(ExtractionResult {
            job,
            snapshot,
            source_url: context.url.clone(),
            extractor_name: "linkedin".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linkedin_context(url: &str, html: &str) -> ExtractionContext {
        ExtractionContext {
            url: url.to_string(),
            html: html.to_string(),
            title: None,
        }
    }

    #[test]
    fn can_extract_linkedin_urls() {
        let extractor = LinkedInExtractor::new();
        let ctx = linkedin_context("https://www.linkedin.com/jobs/view/software-engineer-123", "");
        assert!(extractor.can_extract(&ctx));
    }

    #[test]
    fn cannot_extract_non_linkedin_urls() {
        let extractor = LinkedInExtractor::new();
        let ctx = linkedin_context("https://www.indeed.com/jobs?q=engineer", "");
        assert!(!extractor.can_extract(&ctx));
    }

    #[test]
    fn extract_title_from_html() {
        let extractor = LinkedInExtractor::new();
        let html = r#"<h1 class="top-card-layout__title">Software Engineer</h1>"#;
        let ctx = linkedin_context("https://www.linkedin.com/jobs/view/123", html);
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Software Engineer");
    }

    #[test]
    fn extract_uses_page_title_fallback() {
        let extractor = LinkedInExtractor::new();
        let ctx = ExtractionContext {
            url: "https://www.linkedin.com/jobs/view/123".to_string(),
            html: "<html></html>".to_string(),
            title: Some("Fallback Title".to_string()),
        };
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Fallback Title");
    }

    #[test]
    fn extract_defaults_to_untitled() {
        let extractor = LinkedInExtractor::new();
        let ctx = linkedin_context("https://www.linkedin.com/jobs/view/123", "<html></html>");
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Untitled Position");
    }

    #[test]
    fn extract_sets_source_linkedin() {
        let extractor = LinkedInExtractor::new();
        let ctx = linkedin_context("https://www.linkedin.com/jobs/view/123", "<html></html>");
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.source, Some("linkedin".to_string()));
        assert_eq!(result.extractor_name, "linkedin");
    }

    #[test]
    fn extract_sets_canonical_url() {
        let extractor = LinkedInExtractor::new();
        let url = "https://www.linkedin.com/jobs/view/software-engineer-123";
        let ctx = linkedin_context(url, "<html></html>");
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.canonical_url, Some(url.to_string()));
    }

    #[test]
    fn strip_html_tags_basic() {
        let result = LinkedInExtractor::strip_html_tags("<p>Hello <b>world</b></p>");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn strip_html_entities() {
        let result = LinkedInExtractor::strip_html_tags("A &amp; B &lt; C &gt; D");
        assert_eq!(result, "A & B < C > D");
    }

    #[test]
    fn name_returns_linkedin() {
        let extractor = LinkedInExtractor::new();
        assert_eq!(extractor.name(), "linkedin");
    }
}
