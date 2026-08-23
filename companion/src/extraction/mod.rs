pub mod indeed;
pub mod linkedin;

use crate::model::{Job, JobSnapshot};

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub job: Job,
    pub snapshot: JobSnapshot,
    pub source_url: String,
    pub extractor_name: String,
}

#[derive(Debug, Clone)]
pub struct ExtractionContext {
    pub url: String,
    pub html: String,
    pub title: Option<String>,
}

pub trait JobExtractor: Send + Sync {
    fn name(&self) -> &str;

    fn can_extract(&self, context: &ExtractionContext) -> bool;

    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult, String>;
}

pub struct ExtractionPipeline {
    extractors: Vec<Box<dyn JobExtractor>>,
}

impl ExtractionPipeline {
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    pub fn register(&mut self, extractor: Box<dyn JobExtractor>) {
        self.extractors.push(extractor);
    }

    pub fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult, String> {
        for extractor in &self.extractors {
            if extractor.can_extract(context) {
                return extractor.extract(context);
            }
        }
        Err("No extractor found for this page".to_string())
    }

    pub fn extractor_names(&self) -> Vec<&str> {
        self.extractors.iter().map(|e| e.name()).collect()
    }
}

impl Default for ExtractionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    struct MockExtractor {
        name: String,
        should_match: bool,
    }

    impl MockExtractor {
        fn new(name: &str, should_match: bool) -> Self {
            Self {
                name: name.to_string(),
                should_match,
            }
        }
    }

    impl JobExtractor for MockExtractor {
        fn name(&self) -> &str {
            &self.name
        }

        fn can_extract(&self, _context: &ExtractionContext) -> bool {
            self.should_match
        }

        fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult, String> {
            if !self.should_match {
                return Err("Cannot extract".to_string());
            }

            let now = Utc::now().to_rfc3339();
            let job = Job::new(
                format!("job-{}", &context.url[..20.min(context.url.len())]),
                "Extracted Title".to_string(),
                now.clone(),
            );

            let snapshot = JobSnapshot::new(
                format!("snap-{}", &context.url[..20.min(context.url.len())]),
                job.id.clone(),
                now,
                "Extracted Title".to_string(),
                "Extracted description".to_string(),
            );

            Ok(ExtractionResult {
                job,
                snapshot,
                source_url: context.url.clone(),
                extractor_name: self.name.clone(),
            })
        }
    }

    #[test]
    fn pipeline_registers_extractors() {
        let mut pipeline = ExtractionPipeline::new();
        pipeline.register(Box::new(MockExtractor::new("test", true)));
        assert_eq!(pipeline.extractor_names(), vec!["test"]);
    }

    #[test]
    fn pipeline_extracts_matching_extractor() {
        let mut pipeline = ExtractionPipeline::new();
        pipeline.register(Box::new(MockExtractor::new("mock", true)));

        let context = ExtractionContext {
            url: "https://example.com/job/123".to_string(),
            html: "<html></html>".to_string(),
            title: None,
        };

        let result = pipeline.extract(&context).unwrap();
        assert_eq!(result.extractor_name, "mock");
        assert_eq!(result.source_url, "https://example.com/job/123");
    }

    #[test]
    fn pipeline_skips_non_matching_extractor() {
        let mut pipeline = ExtractionPipeline::new();
        pipeline.register(Box::new(MockExtractor::new("no-match", false)));
        pipeline.register(Box::new(MockExtractor::new("match", true)));

        let context = ExtractionContext {
            url: "https://example.com/job/123".to_string(),
            html: "<html></html>".to_string(),
            title: None,
        };

        let result = pipeline.extract(&context).unwrap();
        assert_eq!(result.extractor_name, "match");
    }

    #[test]
    fn pipeline_fails_when_no_extractor_matches() {
        let mut pipeline = ExtractionPipeline::new();
        pipeline.register(Box::new(MockExtractor::new("no-match", false)));

        let context = ExtractionContext {
            url: "https://example.com/job/123".to_string(),
            html: "<html></html>".to_string(),
            title: None,
        };

        assert!(pipeline.extract(&context).is_err());
    }

    #[test]
    fn pipeline_fails_when_empty() {
        let pipeline = ExtractionPipeline::new();
        let context = ExtractionContext {
            url: "https://example.com/job/123".to_string(),
            html: "<html></html>".to_string(),
            title: None,
        };

        assert!(pipeline.extract(&context).is_err());
    }
}
