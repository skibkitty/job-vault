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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobSnapshot {
    pub id: String,
    pub job_id: String,
    pub captured_at: String,
    pub source_url: Option<String>,
    pub raw_text: Option<String>,
    pub normalized_text: Option<String>,
    pub title: String,
    pub company: Option<String>,
    pub location: Option<String>,
    pub salary: Option<String>,
    pub description: String,
    pub requirements: Option<String>,
    pub responsibilities: Option<String>,
    pub extraction_metadata: Option<String>,
}

impl JobSnapshot {
    pub fn new(
        id: String,
        job_id: String,
        captured_at: String,
        title: String,
        description: String,
    ) -> Self {
        Self {
            id,
            job_id,
            captured_at,
            source_url: None,
            raw_text: None,
            normalized_text: None,
            title,
            company: None,
            location: None,
            salary: None,
            description,
            requirements: None,
            responsibilities: None,
            extraction_metadata: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("JobSnapshot id cannot be empty".to_string());
        }
        if self.job_id.is_empty() {
            return Err("JobSnapshot job_id cannot be empty".to_string());
        }
        if self.captured_at.is_empty() {
            return Err("JobSnapshot captured_at cannot be empty".to_string());
        }
        if self.title.is_empty() {
            return Err("JobSnapshot title cannot be empty".to_string());
        }
        if self.description.is_empty() {
            return Err("JobSnapshot description cannot be empty".to_string());
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

    #[test]
    fn job_snapshot_creation() {
        let snapshot = JobSnapshot::new(
            "snap-123".to_string(),
            "job-456".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            "Software Engineer".to_string(),
            "Description of the job".to_string(),
        );

        assert_eq!(snapshot.id, "snap-123");
        assert_eq!(snapshot.job_id, "job-456");
        assert_eq!(snapshot.title, "Software Engineer");
        assert!(snapshot.company.is_none());
        assert!(snapshot.description == "Description of the job");
    }

    #[test]
    fn job_snapshot_validation_passes() {
        let snapshot = JobSnapshot::new(
            "snap-123".to_string(),
            "job-456".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            "Software Engineer".to_string(),
            "Description of the job".to_string(),
        );
        assert!(snapshot.validate().is_ok());
    }

    #[test]
    fn job_snapshot_validation_empty_id_fails() {
        let snapshot = JobSnapshot::new(
            "".to_string(),
            "job-456".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            "Software Engineer".to_string(),
            "Description of the job".to_string(),
        );
        let result = snapshot.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("id"));
    }

    #[test]
    fn job_snapshot_validation_empty_job_id_fails() {
        let snapshot = JobSnapshot::new(
            "snap-123".to_string(),
            "".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            "Software Engineer".to_string(),
            "Description of the job".to_string(),
        );
        let result = snapshot.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("job_id"));
    }

    #[test]
    fn job_snapshot_validation_empty_title_fails() {
        let snapshot = JobSnapshot::new(
            "snap-123".to_string(),
            "job-456".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            "".to_string(),
            "Description of the job".to_string(),
        );
        let result = snapshot.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("title"));
    }

    #[test]
    fn job_snapshot_validation_empty_description_fails() {
        let snapshot = JobSnapshot::new(
            "snap-123".to_string(),
            "job-456".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            "Software Engineer".to_string(),
            "".to_string(),
        );
        let result = snapshot.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("description"));
    }

    #[test]
    fn job_snapshot_serialization_roundtrip() {
        let snapshot = JobSnapshot::new(
            "snap-123".to_string(),
            "job-456".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            "Software Engineer".to_string(),
            "Description of the job".to_string(),
        );

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: JobSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, deserialized);
    }

    #[test]
    fn job_snapshot_clone() {
        let snapshot = JobSnapshot::new(
            "snap-123".to_string(),
            "job-456".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            "Software Engineer".to_string(),
            "Description of the job".to_string(),
        );

        let cloned = snapshot.clone();
        assert_eq!(snapshot, cloned);
    }

    #[test]
    fn job_snapshot_with_optional_fields() {
        let snapshot = JobSnapshot {
            id: "snap-123".to_string(),
            job_id: "job-456".to_string(),
            captured_at: "2026-01-01T00:00:00Z".to_string(),
            source_url: Some("https://example.com/job/123".to_string()),
            raw_text: Some("Raw job posting text".to_string()),
            normalized_text: Some("Normalized job posting text".to_string()),
            title: "Software Engineer".to_string(),
            company: Some("Example Corp".to_string()),
            location: Some("San Francisco, CA".to_string()),
            salary: Some("$100,000 - $150,000".to_string()),
            description: "Description of the job".to_string(),
            requirements: Some("5+ years experience".to_string()),
            responsibilities: Some("Build software".to_string()),
            extraction_metadata: Some("{\"source\": \"linkedin\"}".to_string()),
        };

        assert!(snapshot.validate().is_ok());
        assert_eq!(snapshot.company, Some("Example Corp".to_string()));
        assert_eq!(snapshot.requirements, Some("5+ years experience".to_string()));
    }
}
