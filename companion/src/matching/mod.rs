use crate::diff::TextDiffEngine;
use crate::model::Job;
use crate::normalization::TextNormalizer;
use sha2::{Digest, Sha256};

pub const FINGERPRINT_MATCH_CONFIDENCE: f64 = 0.90;
pub const CONTENT_MATCH_CONFIDENCE_BASE: f64 = 0.50;
pub const CONTENT_MATCH_CONFIDENCE_FACTOR: f64 = 0.39;
pub const DEFAULT_CONTENT_SIMILARITY_THRESHOLD: f64 = 0.70;

#[derive(Debug, Clone, PartialEq)]
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
    Similarity,
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

pub struct JobCandidate<'a> {
    pub job: &'a Job,
    pub latest_description: Option<&'a str>,
}

pub struct SimilarityMatcher {
    similarity_threshold: f64,
}

impl SimilarityMatcher {
    pub fn new() -> Self {
        Self::with_similarity_threshold(DEFAULT_CONTENT_SIMILARITY_THRESHOLD)
    }

    pub fn with_similarity_threshold(threshold: f64) -> Self {
        Self {
            similarity_threshold: threshold,
        }
    }

    fn posting_text(job: &Job, description: Option<&str>) -> String {
        let mut parts = Vec::new();
        parts.push(TextNormalizer::normalize_for_comparison(&job.title));
        if let Some(company) = &job.company_id {
            parts.push(TextNormalizer::normalize_for_comparison(company));
        }
        if let Some(location) = &job.location {
            parts.push(TextNormalizer::normalize_for_comparison(location));
        }
        if let Some(desc) = description {
            parts.push(TextNormalizer::normalize_for_comparison(desc));
        }
        TextNormalizer::remove_filler_words(&parts.join(" "))
    }

    pub fn content_similarity(
        &self,
        new_job: &Job,
        new_description: Option<&str>,
        existing_job: &Job,
        existing_description: Option<&str>,
    ) -> f64 {
        let a = Self::posting_text(new_job, new_description);
        let b = Self::posting_text(existing_job, existing_description);
        TextDiffEngine::similarity(&a, &b)
    }

    fn specificity(match_type: &MatchType) -> u8 {
        match match_type {
            MatchType::ExactUrl => 4,
            MatchType::ExternalJobId => 3,
            MatchType::CanonicalUrl => 2,
            MatchType::Fingerprint => 2,
            MatchType::Similarity => 1,
        }
    }

    fn merge_match(current: Option<MatchResult>, incoming: &MatchResult) -> MatchResult {
        match current {
            None => incoming.clone(),
            Some(cur) => {
                if incoming.confidence > cur.confidence
                    || (incoming.confidence == cur.confidence
                        && Self::specificity(&incoming.match_type) > Self::specificity(&cur.match_type))
                {
                    incoming.clone()
                } else {
                    cur
                }
            }
        }
    }

    pub fn find_similar(
        &self,
        new_job: &Job,
        new_description: Option<&str>,
        candidates: &[JobCandidate],
    ) -> Vec<MatchResult> {
        let url_matcher = UrlMatcher::new();
        let existing_jobs: Vec<Job> = candidates.iter().map(|c| c.job.clone()).collect();
        let url_matches = url_matcher.find_matches(new_job, &existing_jobs);

        let new_fingerprint = FingerprintGenerator::new().generate(new_job);

        let mut results: Vec<MatchResult> = Vec::new();

        for candidate in candidates {
            let mut best: Option<MatchResult> = None;

            for url_match in &url_matches {
                if url_match.matched_job_id == candidate.job.id {
                    best = Some(Self::merge_match(best, url_match));
                }
            }

            if FingerprintGenerator::new().generate(candidate.job) == new_fingerprint {
                let fingerprint_match = MatchResult {
                    job_id: new_job.id.clone(),
                    matched_job_id: candidate.job.id.clone(),
                    confidence: FINGERPRINT_MATCH_CONFIDENCE,
                    match_type: MatchType::Fingerprint,
                };
                best = Some(Self::merge_match(best, &fingerprint_match));
            }

            let similarity = self.content_similarity(
                new_job,
                new_description,
                candidate.job,
                candidate.latest_description,
            );
            if similarity >= self.similarity_threshold {
                let similarity_match = MatchResult {
                    job_id: new_job.id.clone(),
                    matched_job_id: candidate.job.id.clone(),
                    confidence: CONTENT_MATCH_CONFIDENCE_BASE
                        + CONTENT_MATCH_CONFIDENCE_FACTOR * similarity,
                    match_type: MatchType::Similarity,
                };
                best = Some(Self::merge_match(best, &similarity_match));
            }

            if let Some(match_result) = best {
                results.push(match_result);
            }
        }

        results.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.matched_job_id.cmp(&b.matched_job_id))
        });

        results
    }
}

impl Default for SimilarityMatcher {
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

    fn create_job(
        id: &str,
        title: &str,
        company: Option<&str>,
        location: Option<&str>,
        url: Option<&str>,
        external_id: Option<&str>,
    ) -> Job {
        Job {
            id: id.to_string(),
            company_id: company.map(|c| c.to_string()),
            title: title.to_string(),
            canonical_url: url.map(|u| u.to_string()),
            location: location.map(|l| l.to_string()),
            employment_type: None,
            salary: None,
            source: Some("linkedin".to_string()),
            external_job_id: external_id.map(|e| e.to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn candidate<'a>(job: &'a Job, description: Option<&'a str>) -> JobCandidate<'a> {
        JobCandidate {
            job,
            latest_description: description,
        }
    }

    #[test]
    fn content_similarity_identical() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "Software Engineer", Some("Acme"), Some("Remote"), None, None);
        let existing_job = create_job("2", "Software Engineer", Some("Acme"), Some("Remote"), None, None);
        let sim = matcher.content_similarity(
            &new_job,
            Some("Build scalable backend services using Rust."),
            &existing_job,
            Some("Build scalable backend services using Rust."),
        );
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn content_similarity_disjoint() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "Plumber", Some("Acme"), Some("Austin"), None, None);
        let existing_job = create_job("2", "Graphic Designer", Some("Globex"), Some("Berlin"), None, None);
        let sim = matcher.content_similarity(
            &new_job,
            Some("Install and repair pipes and fixtures."),
            &existing_job,
            Some("Create brand identity assets and illustrations."),
        );
        assert!((sim - 0.0).abs() < 1e-9);
    }

    #[test]
    fn content_similarity_partial_overlap() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "Backend Engineer", None, None, None, None);
        let existing_job = create_job("2", "Backend Engineer", None, None, None, None);
        let sim = matcher.content_similarity(
            &new_job,
            Some("Design APIs and databases."),
            &existing_job,
            Some("Design APIs and build client SDKs."),
        );
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn content_similarity_punctuation_tolerant() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "Software Engineer", Some("Acme"), Some("Remote"), None, None);
        let existing_job = create_job("2", "Software Engineer", Some("Acme"), Some("Remote"), None, None);
        let sim = matcher.content_similarity(
            &new_job,
            Some("Rust, databases!"),
            &existing_job,
            Some("Rust databases,"),
        );
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn content_similarity_case_tolerant() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "SOFTWARE ENGINEER", None, None, None, None);
        let existing_job = create_job("2", "software engineer", None, None, None, None);
        let sim = matcher.content_similarity(&new_job, Some("Distributed Systems"), &existing_job, Some("distributed systems"));
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn content_similarity_identical_metadata_without_description() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "Data Scientist", Some("Acme"), Some("Chicago"), None, None);
        let existing_job = create_job("2", "Data Scientist", Some("Acme"), Some("Chicago"), None, None);
        let sim = matcher.content_similarity(&new_job, None, &existing_job, None);
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn find_similar_fingerprint_match_dominates_content() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "DevOps Engineer", Some("Acme"), Some("Remote"), Some("https://a.com/jobs/1"), None);
        let existing_job = create_job("2", "DevOps Engineer", Some("Acme"), Some("Remote"), Some("https://b.net/jobs/99"), None);
        let matches = matcher.find_similar(
            &new_job,
            Some("Manage CI/CD pipelines."),
            &[candidate(&existing_job, Some("Manage CI/CD pipelines."))],
        );
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::Fingerprint);
        assert_eq!(matches[0].confidence, FINGERPRINT_MATCH_CONFIDENCE);
    }

    #[test]
    fn find_similar_content_match_with_new_url() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "Frontend Engineer", Some("Acme"), Some("Remote"), Some("https://a.com/jobs/1"), None);
        let existing_job = create_job("2", "Frontend Engineer", Some("Globex"), Some("Remote"), Some("https://b.net/jobs/2"), None);
        let matches = matcher.find_similar(
            &new_job,
            Some("Build accessible React components and design systems."),
            &[candidate(&existing_job, Some("Build accessible React components and design systems."))],
        );
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::Similarity);
        assert!(matches[0].confidence >= 0.70);
        assert!(matches[0].confidence < FINGERPRINT_MATCH_CONFIDENCE);
    }

    #[test]
    fn find_similar_exact_url_dominates() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "DevOps Engineer", Some("Acme"), Some("Remote"), Some("https://a.com/jobs/1"), None);
        let existing_job = create_job("2", "QA Engineer", Some("Globex"), Some("Bengaluru"), Some("https://a.com/jobs/1"), None);
        let matches = matcher.find_similar(
            &new_job,
            Some("Automate tests."),
            &[candidate(&existing_job, Some("Manually verify tickets."))],
        );
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::ExactUrl);
        assert_eq!(matches[0].confidence, 1.0);
    }

    #[test]
    fn find_similar_external_id_still_matches() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "Security Analyst", None, None, Some("https://a.com/jobs/1"), Some("ext-77"));
        let existing_job = create_job("2", "Product Manager", None, None, Some("https://b.net/jobs/2"), Some("ext-77"));
        let matches = matcher.find_similar(
            &new_job,
            Some("Run threat models."),
            &[candidate(&existing_job, Some("Own the roadmap."))],
        );
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].match_type, MatchType::ExternalJobId);
        assert_eq!(matches[0].confidence, 0.95);
    }

    #[test]
    fn find_similar_below_threshold_no_match() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "Plumber", Some("Acme"), Some("Austin"), None, None);
        let existing_job = create_job("2", "Graphic Designer", Some("Globex"), Some("Berlin"), None, None);
        let matches = matcher.find_similar(
            &new_job,
            Some("Install and repair pipes and fixtures."),
            &[candidate(&existing_job, Some("Create brand identity assets and illustrations."))],
        );
        assert!(matches.is_empty());
    }

    #[test]
    fn find_similar_empty_candidates() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "DevOps Engineer", None, None, None, None);
        let matches = matcher.find_similar(&new_job, Some("Manage pipelines."), &[]);
        assert!(matches.is_empty());
    }

    #[test]
    fn find_similar_different_title_no_description_no_match() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "DevOps Engineer", Some("Acme"), Some("Remote"), None, None);
        let existing_job = create_job("2", "Graphic Designer", Some("Globex"), Some("London"), None, None);
        let matches = matcher.find_similar(&new_job, None, &[candidate(&existing_job, None)]);
        assert!(matches.is_empty());
    }

    #[test]
    fn find_similar_sorted_by_confidence_descending() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job(
            "1",
            "DevOps Engineer",
            Some("Acme"),
            Some("Remote"),
            Some("https://a.com/jobs/1"),
            None,
        );
        let url_job = create_job("2", "QA Engineer", Some("Globex"), Some("Bengaluru"), Some("https://a.com/jobs/1"), None);
        let fp_job = create_job("3", "DevOps Engineer", Some("Acme"), Some("Remote"), None, None);
        let sim_job = create_job("4", "DevOps Engineer", Some("Umbrella"), Some("Remote"), None, None);
        let matches = matcher.find_similar(
            &new_job,
            Some("Manage Kubernetes clusters and Terraform."),
            &[
                candidate(&url_job, Some("Unrelated content.")),
                candidate(&fp_job, Some("Manage Kubernetes clusters and Terraform.")),
                candidate(&sim_job, Some("Manage Kubernetes clusters and Terraform.")),
            ],
        );
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].matched_job_id, "2");
        assert_eq!(matches[0].confidence, 1.0);
        assert_eq!(matches[1].matched_job_id, "3");
        assert_eq!(matches[1].confidence, FINGERPRINT_MATCH_CONFIDENCE);
        assert!(matches[2].confidence < FINGERPRINT_MATCH_CONFIDENCE);
    }

    #[test]
    fn find_similar_deterministic() {
        let matcher = SimilarityMatcher::new();
        let new_job = create_job("1", "DevOps Engineer", Some("Acme"), Some("Remote"), None, None);
        let existing_job = create_job("2", "DevOps Engineer", Some("Umbrella"), Some("Remote"), None, None);
        let candidates = [candidate(&existing_job, Some("Manage Kubernetes clusters."))];
        let first = matcher.find_similar(&new_job, Some("Manage Kubernetes clusters."), &candidates);
        let second = matcher.find_similar(&new_job, Some("Manage Kubernetes clusters."), &candidates);
        assert_eq!(first, second);
    }

    #[test]
    fn find_similar_threshold_configurable() {
        let strict = SimilarityMatcher::with_similarity_threshold(0.95);
        let lenient = SimilarityMatcher::with_similarity_threshold(0.20);
        let new_job = create_job("1", "SRE", None, None, None, None);
        let existing_job = create_job("2", "SRE", Some("Acme"), Some("Paris"), None, None);
        let candidates = [candidate(&existing_job, Some("Improve reliability and observability."))];
        assert!(strict.find_similar(&new_job, Some("Improve reliability."), &candidates).is_empty());
        assert_eq!(lenient.find_similar(&new_job, Some("Improve reliability."), &candidates).len(), 1);
    }
}
