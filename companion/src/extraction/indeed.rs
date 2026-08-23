use crate::extraction::{ExtractionContext, ExtractionResult, JobExtractor};
use crate::model::{Job, JobSnapshot};
use chrono::Utc;
use uuid::Uuid;

pub struct IndeedExtractor;

impl IndeedExtractor {
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
            "<div id=\"jobDescriptionText\"",
            "<div class=\"jobsearch-JobComponent-description",
            "<div class=\"jobDescription",
            "<div class=\"jobsearch-jobDescriptionText",
        ];

        for marker in markers {
            if let Some(start) = html.find(marker) {
                let after_marker = &html[start..];
                if let Some(tag_end) = after_marker.find('>') {
                    let content = &after_marker[tag_end + 1..];
                    if let Some(end_marker) = content.find("</div>") {
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

impl Default for IndeedExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl JobExtractor for IndeedExtractor {
    fn name(&self) -> &str {
        "indeed"
    }

    fn can_extract(&self, context: &ExtractionContext) -> bool {
        context.url.contains("indeed.com/viewjob")
            || context.url.contains("indeed.com/jobs")
    }

    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult, String> {
        let title = Self::extract_field(
            &context.html,
            &[
                "<h1 class=\"jobsearch-JobInfoHeader-title",
                "<h1 class=\"jobsearch-JobInfoHeader-titleContainer",
                "<h1 class=\"jobsearch-JobInfoHeader-titleText",
                "<h1 class=\"jobsearch-JobInfoHeader-title",
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
                "<div class=\"jobsearch-CompanyInfo",
                "<a class=\"jobsearch-CompanyInfoCompany",
                "<span class=\"jobsearch-CompanyInfoCompany",
            ],
        );

        let location = Self::extract_field(
            &context.html,
            &[
                "<div class=\"jobsearch-JobInfoHeader-location",
                "<div class=\"jobsearch-JobInfoHeader-locationIconContainer",
                "<span class=\"jobsearch-CompanyInfo-location",
            ],
        );

        let description = Self::extract_description(&context.html)
            .unwrap_or_else(|| "No description available".to_string());

        let external_job_id = context
            .url
            .split('?')
            .next()
            .and_then(|u| u.rsplit('/').next())
            .filter(|s| !s.is_empty())
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
            source: Some("indeed".to_string()),
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
                "extractor": "indeed",
                "version": "1.0"
            }).to_string()),
        };

        Ok(ExtractionResult {
            job,
            snapshot,
            source_url: context.url.clone(),
            extractor_name: "indeed".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn indeed_context(url: &str, html: &str) -> ExtractionContext {
        ExtractionContext {
            url: url.to_string(),
            html: html.to_string(),
            title: None,
        }
    }

    #[test]
    fn can_extract_indeed_viewjob_urls() {
        let extractor = IndeedExtractor::new();
        let ctx = indeed_context("https://www.indeed.com/viewjob?jk=abc123", "");
        assert!(extractor.can_extract(&ctx));
    }

    #[test]
    fn can_extract_indeed_jobs_urls() {
        let extractor = IndeedExtractor::new();
        let ctx = indeed_context("https://www.indeed.com/jobs?q=engineer", "");
        assert!(extractor.can_extract(&ctx));
    }

    #[test]
    fn cannot_extract_non_indeed_urls() {
        let extractor = IndeedExtractor::new();
        let ctx = indeed_context("https://www.linkedin.com/jobs/view/123", "");
        assert!(!extractor.can_extract(&ctx));
    }

    #[test]
    fn extract_title_from_html() {
        let extractor = IndeedExtractor::new();
        let html = r#"<h1 class="jobsearch-JobInfoHeader-title">Software Engineer</h1>"#;
        let ctx = indeed_context("https://www.indeed.com/viewjob?jk=abc123", html);
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Software Engineer");
    }

    #[test]
    fn extract_uses_page_title_fallback() {
        let extractor = IndeedExtractor::new();
        let ctx = ExtractionContext {
            url: "https://www.indeed.com/viewjob?jk=abc123".to_string(),
            html: "<html></html>".to_string(),
            title: Some("Fallback Title".to_string()),
        };
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Fallback Title");
    }

    #[test]
    fn extract_defaults_to_untitled() {
        let extractor = IndeedExtractor::new();
        let ctx = indeed_context("https://www.indeed.com/viewjob?jk=abc123", "<html></html>");
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Untitled Position");
    }

    #[test]
    fn extract_sets_source_indeed() {
        let extractor = IndeedExtractor::new();
        let ctx = indeed_context("https://www.indeed.com/viewjob?jk=abc123", "<html></html>");
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.source, Some("indeed".to_string()));
        assert_eq!(result.extractor_name, "indeed");
    }

    #[test]
    fn extract_sets_canonical_url() {
        let extractor = IndeedExtractor::new();
        let url = "https://www.indeed.com/viewjob?jk=abc123";
        let ctx = indeed_context(url, "<html></html>");
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.canonical_url, Some(url.to_string()));
    }

    #[test]
    fn strip_html_tags_basic() {
        let result = IndeedExtractor::strip_html_tags("<p>Hello <b>world</b></p>");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn strip_html_entities() {
        let result = IndeedExtractor::strip_html_tags("A &amp; B &lt; C &gt; D");
        assert_eq!(result, "A & B < C > D");
    }

    #[test]
    fn name_returns_indeed() {
        let extractor = IndeedExtractor::new();
        assert_eq!(extractor.name(), "indeed");
    }
}
