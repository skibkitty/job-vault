use crate::model::Job;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub job_id: String,
    pub matched_job_id: String,
    pub confidence: f64,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    ExactUrl,
    ExternalJobId,
    CanonicalUrl,
    Fingerprint,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fingerprint {
    pub hash: String,
}

impl Fingerprint {
    pub fn new(hash: String) -> Self {
        Self { hash }
    }
}

pub struct FingerprintGenerator;

impl FingerprintGenerator {
    pub fn new() -> Self {
        Self
    }

    fn normalize_text(text: &str) -> String {
        text.to_lowercase()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn generate(&self, job: &Job) -> Fingerprint {
        let mut hasher = Sha256::new();

        let normalized_title = Self::normalize_text(&job.title);
        hasher.update(normalized_title.as_bytes());
        hasher.update(b"\0");

        if let Some(company) = &job.company_id {
            let normalized_company = Self::normalize_text(company);
            hasher.update(normalized_company.as_bytes());
        }
        hasher.update(b"\0");

        if let Some(location) = &job.location {
            let normalized_location = Self::normalize_text(location);
            hasher.update(normalized_location.as_bytes());
        }

        let result = hasher.finalize();
        let hash = format!("{:x}", result);

        Fingerprint::new(hash)
    }
}

impl Default for FingerprintGenerator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct UrlMatcher;

impl UrlMatcher {
    pub fn new() -> Self {
        Self
    }

    fn normalize_url(url: &str) -> String {
        let mut normalized = url.to_lowercase();

        normalized = normalized.trim_end_matches('/').to_string();

        if let Some(pos) = normalized.find('?') {
            normalized = normalized[..pos].to_string();
        }

        if let Some(pos) = normalized.find('#') {
            normalized = normalized[..pos].to_string();
        }

        normalized
    }

    pub fn find_matches(&self, new_job: &Job, existing_jobs: &[Job]) -> Vec<MatchResult> {
        let mut matches = Vec::new();

        for existing_job in existing_jobs {
            if let Some(match_result) = self.check_url_match(new_job, existing_job) {
                matches.push(match_result);
            }

            if let Some(match_result) = self.check_external_id_match(new_job, existing_job) {
                matches.push(match_result);
            }
        }

        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        matches
    }

    fn check_url_match(&self, new_job: &Job, existing_job: &Job) -> Option<MatchResult> {
        if let (Some(new_url), Some(existing_url)) = (&new_job.canonical_url, &existing_job.canonical_url) {
            let normalized_new = Self::normalize_url(new_url);
            let normalized_existing = Self::normalize_url(existing_url);

            if normalized_new == normalized_existing {
                return Some(MatchResult {
                    job_id: new_job.id.clone(),
                    matched_job_id: existing_job.id.clone(),
                    confidence: 1.0,
                    match_type: MatchType::ExactUrl,
                });
            }
        }

        None
    }

    fn check_external_id_match(&self, new_job: &Job, existing_job: &Job) -> Option<MatchResult> {
        if let (Some(new_ext_id), Some(existing_ext_id)) = (&new_job.external_job_id, &existing_job.external_job_id) {
            if new_ext_id == existing_ext_id {
                return Some(MatchResult {
                    job_id: new_job.id.clone(),
                    matched_job_id: existing_job.id.clone(),
                    confidence: 0.95,
                    match_type: MatchType::ExternalJobId,
                });
            }
        }

        None
    }
}

impl Default for UrlMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_job(id: &str, url: Option<&str>, external_id: Option<&str>) -> Job {
        Job {
            id: id.to_string(),
            company_id: None,
            title: "Software Engineer".to_string(),
            canonical_url: url.map(|u| u.to_string()),
            location: None,
            employment_type: None,
            salary: None,
            source: Some("linkedin".to_string()),
            external_job_id: external_id.map(|e| e.to_string()),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn exact_url_match() {
        let matcher = UrlMatcher::new();
        let new_job = create_test_job("1", Some("https://example.com/jobs/123"), None);
        let existing_jobs = vec![create_test_job("2", Some("https://example.com/jobs/123"), None)];

        let matches = matcher.find_matches(&new_job, &existing_jobs);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::ExactUrl);
        assert_eq!(matches[0].confidence, 1.0);
    }

    #[test]
    fn url_match_with_trailing_slash() {
        let matcher = UrlMatcher::new();
        let new_job = create_test_job("1", Some("https://example.com/jobs/123"), None);
        let existing_jobs = vec![create_test_job("2", Some("https://example.com/jobs/123/"), None)];

        let matches = matcher.find_matches(&new_job, &existing_jobs);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::ExactUrl);
    }

    #[test]
    fn url_match_with_query_params() {
        let matcher = UrlMatcher::new();
        let new_job = create_test_job("1", Some("https://example.com/jobs/123?ref=linkedin"), None);
        let existing_jobs = vec![create_test_job("2", Some("https://example.com/jobs/123"), None)];

        let matches = matcher.find_matches(&new_job, &existing_jobs);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::ExactUrl);
    }

    #[test]
    fn url_match_case_insensitive() {
        let matcher = UrlMatcher::new();
        let new_job = create_test_job("1", Some("https://Example.COM/Jobs/123"), None);
        let existing_jobs = vec![create_test_job("2", Some("https://example.com/jobs/123"), None)];

        let matches = matcher.find_matches(&new_job, &existing_jobs);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn external_id_match() {
        let matcher = UrlMatcher::new();
        let new_job = create_test_job("1", None, Some("ext-123"));
        let existing_jobs = vec![create_test_job("2", None, Some("ext-123"))];

        let matches = matcher.find_matches(&new_job, &existing_jobs);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::ExternalJobId);
        assert_eq!(matches[0].confidence, 0.95);
    }

    #[test]
    fn no_match_different_urls() {
        let matcher = UrlMatcher::new();
        let new_job = create_test_job("1", Some("https://example.com/jobs/123"), None);
        let existing_jobs = vec![create_test_job("2", Some("https://example.com/jobs/456"), None)];

        let matches = matcher.find_matches(&new_job, &existing_jobs);
        assert!(matches.is_empty());
    }

    #[test]
    fn no_match_empty_existing() {
        let matcher = UrlMatcher::new();
        let new_job = create_test_job("1", Some("https://example.com/jobs/123"), None);
        let existing_jobs = vec![];

        let matches = matcher.find_matches(&new_job, &existing_jobs);
        assert!(matches.is_empty());
    }

    #[test]
    fn multiple_matches_sorted_by_confidence() {
        let matcher = UrlMatcher::new();
        let new_job = create_test_job("1", Some("https://example.com/jobs/123"), Some("ext-123"));
        let existing_jobs = vec![
            create_test_job("2", Some("https://other.com/jobs/456"), Some("ext-123")),
            create_test_job("3", Some("https://example.com/jobs/123"), Some("ext-456")),
        ];

        let matches = matcher.find_matches(&new_job, &existing_jobs);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].confidence, 1.0);
        assert_eq!(matches[1].confidence, 0.95);
    }

    #[test]
    fn normalize_url_removes_fragment() {
        let matcher = UrlMatcher::new();
        let new_job = create_test_job("1", Some("https://example.com/jobs/123#section"), None);
        let existing_jobs = vec![create_test_job("2", Some("https://example.com/jobs/123"), None)];

        let matches = matcher.find_matches(&new_job, &existing_jobs);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn fingerprint_deterministic() {
        let generator = FingerprintGenerator::new();
        let job = create_test_job("1", None, None);
        
        let fp1 = generator.generate(&job);
        let fp2 = generator.generate(&job);
        
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn fingerprint_same_content_same_hash() {
        let generator = FingerprintGenerator::new();
        let job1 = create_test_job("1", None, None);
        let job2 = create_test_job("2", None, None);
        
        let fp1 = generator.generate(&job1);
        let fp2 = generator.generate(&job2);
        
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn fingerprint_different_title_different_hash() {
        let generator = FingerprintGenerator::new();
        let mut job1 = create_test_job("1", None, None);
        job1.title = "Software Engineer".to_string();
        let mut job2 = create_test_job("2", None, None);
        job2.title = "Backend Developer".to_string();
        
        let fp1 = generator.generate(&job1);
        let fp2 = generator.generate(&job2);
        
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn fingerprint_case_insensitive() {
        let generator = FingerprintGenerator::new();
        let mut job1 = create_test_job("1", None, None);
        job1.title = "Software Engineer".to_string();
        let mut job2 = create_test_job("2", None, None);
        job2.title = "software engineer".to_string();
        
        let fp1 = generator.generate(&job1);
        let fp2 = generator.generate(&job2);
        
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn fingerprint_whitespace_insensitive() {
        let generator = FingerprintGenerator::new();
        let mut job1 = create_test_job("1", None, None);
        job1.title = "Software  Engineer".to_string();
        let mut job2 = create_test_job("2", None, None);
        job2.title = "Software Engineer".to_string();
        
        let fp1 = generator.generate(&job1);
        let fp2 = generator.generate(&job2);
        
        assert_eq!(fp1, fp2);
    }
}
