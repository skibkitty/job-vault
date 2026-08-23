use crate::extraction::{ExtractionContext, ExtractionResult, JobExtractor};
use crate::model::{Job, JobSnapshot};
use chrono::Utc;
use uuid::Uuid;

pub struct ManualExtractor;

impl ManualExtractor {
    pub fn new() -> Self {
        Self
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

impl Default for ManualExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl JobExtractor for ManualExtractor {
    fn name(&self) -> &str {
        "manual"
    }

    fn can_extract(&self, _context: &ExtractionContext) -> bool {
        true
    }

    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult, String> {
        let title = context
            .title
            .as_ref()
            .filter(|t| !t.trim().is_empty())
            .cloned()
            .ok_or_else(|| "Title is required for manual extraction".to_string())?;

        let description = Self::strip_html_tags(&context.html);
        if description.trim().is_empty() {
            return Err("Description is required for manual extraction".to_string());
        }

        let now = Utc::now().to_rfc3339();
        let job_id = Uuid::new_v4().to_string();
        let snapshot_id = Uuid::new_v4().to_string();

        let job = Job {
            id: job_id.clone(),
            company_id: None,
            title,
            canonical_url: Some(context.url.clone()),
            location: None,
            employment_type: None,
            salary: None,
            source: Some("manual".to_string()),
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
            company: None,
            location: None,
            salary: None,
            description,
            requirements: None,
            responsibilities: None,
            extraction_metadata: Some(serde_json::json!({
                "extractor": "manual",
                "version": "1.0"
            }).to_string()),
        };

        Ok(ExtractionResult {
            job,
            snapshot,
            source_url: context.url.clone(),
            extractor_name: "manual".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manual_context(url: &str, html: &str) -> ExtractionContext {
        ExtractionContext {
            url: url.to_string(),
            html: html.to_string(),
            title: None,
        }
    }

    #[test]
    fn can_extract_any_url() {
        let extractor = ManualExtractor::new();
        let ctx = manual_context("https://example.com/anywhere", "");
        assert!(extractor.can_extract(&ctx));
    }

    #[test]
    fn extract_with_title_and_description() {
        let extractor = ManualExtractor::new();
        let ctx = ExtractionContext {
            url: "https://example.com/job".to_string(),
            html: "This is a job description with some details about the position.".to_string(),
            title: Some("Software Engineer".to_string()),
        };
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.title, "Software Engineer");
        assert_eq!(
            result.snapshot.description,
            "This is a job description with some details about the position."
        );
    }

    #[test]
    fn extract_strips_html_from_description() {
        let extractor = ManualExtractor::new();
        let ctx = ExtractionContext {
            url: "https://example.com/job".to_string(),
            html: "<p>This is a <b>job</b> description.</p>".to_string(),
            title: Some("Developer".to_string()),
        };
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.snapshot.description, "This is a job description.");
    }

    #[test]
    fn extract_fails_without_title() {
        let extractor = ManualExtractor::new();
        let ctx = manual_context("https://example.com/job", "Some description");
        let result = extractor.extract(&ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Title is required"));
    }

    #[test]
    fn extract_fails_with_empty_title() {
        let extractor = ManualExtractor::new();
        let ctx = ExtractionContext {
            url: "https://example.com/job".to_string(),
            html: "Some description".to_string(),
            title: Some("   ".to_string()),
        };
        let result = extractor.extract(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn extract_fails_without_description() {
        let extractor = ManualExtractor::new();
        let ctx = ExtractionContext {
            url: "https://example.com/job".to_string(),
            html: "".to_string(),
            title: Some("Software Engineer".to_string()),
        };
        let result = extractor.extract(&ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Description is required"));
    }

    #[test]
    fn extract_fails_with_empty_description() {
        let extractor = ManualExtractor::new();
        let ctx = ExtractionContext {
            url: "https://example.com/job".to_string(),
            html: "   ".to_string(),
            title: Some("Software Engineer".to_string()),
        };
        let result = extractor.extract(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn extract_sets_source_manual() {
        let extractor = ManualExtractor::new();
        let ctx = ExtractionContext {
            url: "https://example.com/job".to_string(),
            html: "Job description".to_string(),
            title: Some("Developer".to_string()),
        };
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.source, Some("manual".to_string()));
        assert_eq!(result.extractor_name, "manual");
    }

    #[test]
    fn extract_sets_canonical_url() {
        let extractor = ManualExtractor::new();
        let url = "https://example.com/job/123";
        let ctx = ExtractionContext {
            url: url.to_string(),
            html: "Job description".to_string(),
            title: Some("Developer".to_string()),
        };
        let result = extractor.extract(&ctx).unwrap();
        assert_eq!(result.job.canonical_url, Some(url.to_string()));
    }

    #[test]
    fn name_returns_manual() {
        let extractor = ManualExtractor::new();
        assert_eq!(extractor.name(), "manual");
    }
}
