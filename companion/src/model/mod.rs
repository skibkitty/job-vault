use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Job {
    pub id: String,
    pub company_id: Option<String>,
    pub title: String,
    pub canonical_url: Option<String>,
    pub location: Option<String>,
    pub employment_type: Option<String>,
    pub salary: Option<String>,
    pub source: Option<String>,
    pub external_job_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Job {
    pub fn new(id: String, title: String, created_at: String) -> Self {
        Self {
            id,
            company_id: None,
            title,
            canonical_url: None,
            location: None,
            employment_type: None,
            salary: None,
            source: None,
            external_job_id: None,
            created_at: created_at.clone(),
            updated_at: created_at,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Job id cannot be empty".to_string());
        }
        if self.title.is_empty() {
            return Err("Job title cannot be empty".to_string());
        }
        if self.created_at.is_empty() {
            return Err("Job created_at cannot be empty".to_string());
        }
        if self.updated_at.is_empty() {
            return Err("Job updated_at cannot be empty".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_creation() {
        let job = Job::new(
            "job-123".to_string(),
            "Software Engineer".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
        );

        assert_eq!(job.id, "job-123");
        assert_eq!(job.title, "Software Engineer");
        assert!(job.company_id.is_none());
        assert!(job.canonical_url.is_none());
        assert!(job.location.is_none());
        assert!(job.created_at == job.updated_at);
    }

    #[test]
    fn job_validation_passes() {
        let job = Job::new(
            "job-123".to_string(),
            "Software Engineer".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
        );
        assert!(job.validate().is_ok());
    }

    #[test]
    fn job_validation_empty_id_fails() {
        let job = Job::new(
            "".to_string(),
            "Software Engineer".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
        );
        let result = job.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("id"));
    }

    #[test]
    fn job_validation_empty_title_fails() {
        let job = Job::new(
            "job-123".to_string(),
            "".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
        );
        let result = job.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("title"));
    }

    #[test]
    fn job_validation_empty_created_at_fails() {
        let job = Job::new(
            "job-123".to_string(),
            "Software Engineer".to_string(),
            "".to_string(),
        );
        let result = job.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("created_at"));
    }

    #[test]
    fn job_serialization_roundtrip() {
        let job = Job::new(
            "job-123".to_string(),
            "Software Engineer".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
        );

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(job, deserialized);
    }

    #[test]
    fn job_clone() {
        let job = Job::new(
            "job-123".to_string(),
            "Software Engineer".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
        );

        let cloned = job.clone();
        assert_eq!(job, cloned);
    }

    #[test]
    fn job_with_optional_fields() {
        let job = Job {
            id: "job-123".to_string(),
            company_id: Some("company-456".to_string()),
            title: "Software Engineer".to_string(),
            canonical_url: Some("https://example.com/job/123".to_string()),
            location: Some("San Francisco, CA".to_string()),
            employment_type: Some("Full-time".to_string()),
            salary: Some("$100,000 - $150,000".to_string()),
            source: Some("linkedin".to_string()),
            external_job_id: Some("li-789".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-02T00:00:00Z".to_string(),
        };

        assert!(job.validate().is_ok());
        assert_eq!(job.company_id, Some("company-456".to_string()));
        assert_eq!(job.source, Some("linkedin".to_string()));
    }
}
