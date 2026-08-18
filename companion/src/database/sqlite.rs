use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

use crate::database::schema;
use crate::model::{Job, JobSnapshot};

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    pub fn open(path: &Path) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("Failed to open database: {e}"))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("Failed to set pragmas: {e}"))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn initialize(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        schema::create_tables(&conn)
    }

    pub fn create_job(&self, job: &Job) -> Result<(), String> {
        job.validate()?;
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        conn.execute(
            "INSERT INTO job (id, company_id, title, canonical_url, location, employment_type, salary, source, external_job_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                job.id,
                job.company_id,
                job.title,
                job.canonical_url,
                job.location,
                job.employment_type,
                job.salary,
                job.source,
                job.external_job_id,
                job.created_at,
                job.updated_at,
            ],
        )
        .map_err(|e| format!("Failed to insert job: {e}"))?;
        Ok(())
    }

    pub fn get_job(&self, id: &str) -> Result<Job, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        conn.query_row(
            "SELECT id, company_id, title, canonical_url, location, employment_type, salary, source, external_job_id, created_at, updated_at FROM job WHERE id = ?1",
            [id],
            |row| {
                Ok(Job {
                    id: row.get(0)?,
                    company_id: row.get(1)?,
                    title: row.get(2)?,
                    canonical_url: row.get(3)?,
                    location: row.get(4)?,
                    employment_type: row.get(5)?,
                    salary: row.get(6)?,
                    source: row.get(7)?,
                    external_job_id: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        )
        .map_err(|e| format!("Failed to get job: {e}"))
    }

    pub fn update_job(&self, job: &Job) -> Result<(), String> {
        job.validate()?;
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        let rows = conn.execute(
            "UPDATE job SET company_id = ?2, title = ?3, canonical_url = ?4, location = ?5, employment_type = ?6, salary = ?7, source = ?8, external_job_id = ?9, updated_at = ?10 WHERE id = ?1",
            rusqlite::params![
                job.id,
                job.company_id,
                job.title,
                job.canonical_url,
                job.location,
                job.employment_type,
                job.salary,
                job.source,
                job.external_job_id,
                job.updated_at,
            ],
        )
        .map_err(|e| format!("Failed to update job: {e}"))?;
        if rows == 0 {
            return Err("Job not found".to_string());
        }
        Ok(())
    }

    pub fn delete_job(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        let rows = conn
            .execute("DELETE FROM job WHERE id = ?1", [id])
            .map_err(|e| format!("Failed to delete job: {e}"))?;
        if rows == 0 {
            return Err("Job not found".to_string());
        }
        Ok(())
    }

    pub fn list_jobs(&self) -> Result<Vec<Job>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        let mut stmt = conn
            .prepare("SELECT id, company_id, title, canonical_url, location, employment_type, salary, source, external_job_id, created_at, updated_at FROM job ORDER BY created_at DESC")
            .map_err(|e| format!("Failed to prepare statement: {e}"))?;
        let jobs = stmt
            .query_map([], |row| {
                Ok(Job {
                    id: row.get(0)?,
                    company_id: row.get(1)?,
                    title: row.get(2)?,
                    canonical_url: row.get(3)?,
                    location: row.get(4)?,
                    employment_type: row.get(5)?,
                    salary: row.get(6)?,
                    source: row.get(7)?,
                    external_job_id: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|e| format!("Failed to query jobs: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect jobs: {e}"))?;
        Ok(jobs)
    }

    pub fn create_snapshot(&self, snapshot: &JobSnapshot) -> Result<(), String> {
        snapshot.validate()?;
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        conn.execute(
            "INSERT INTO job_snapshot (id, job_id, captured_at, source_url, raw_text, normalized_text, title, company, location, salary, description, requirements, responsibilities, extraction_metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                snapshot.id,
                snapshot.job_id,
                snapshot.captured_at,
                snapshot.source_url,
                snapshot.raw_text,
                snapshot.normalized_text,
                snapshot.title,
                snapshot.company,
                snapshot.location,
                snapshot.salary,
                snapshot.description,
                snapshot.requirements,
                snapshot.responsibilities,
                snapshot.extraction_metadata,
            ],
        )
        .map_err(|e| format!("Failed to insert snapshot: {e}"))?;
        Ok(())
    }

    pub fn get_snapshot(&self, id: &str) -> Result<JobSnapshot, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        conn.query_row(
            "SELECT id, job_id, captured_at, source_url, raw_text, normalized_text, title, company, location, salary, description, requirements, responsibilities, extraction_metadata FROM job_snapshot WHERE id = ?1",
            [id],
            |row| {
                Ok(JobSnapshot {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    captured_at: row.get(2)?,
                    source_url: row.get(3)?,
                    raw_text: row.get(4)?,
                    normalized_text: row.get(5)?,
                    title: row.get(6)?,
                    company: row.get(7)?,
                    location: row.get(8)?,
                    salary: row.get(9)?,
                    description: row.get(10)?,
                    requirements: row.get(11)?,
                    responsibilities: row.get(12)?,
                    extraction_metadata: row.get(13)?,
                })
            },
        )
        .map_err(|e| format!("Failed to get snapshot: {e}"))
    }

    pub fn get_snapshots_for_job(&self, job_id: &str) -> Result<Vec<JobSnapshot>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        let mut stmt = conn
            .prepare("SELECT id, job_id, captured_at, source_url, raw_text, normalized_text, title, company, location, salary, description, requirements, responsibilities, extraction_metadata FROM job_snapshot WHERE job_id = ?1 ORDER BY captured_at DESC")
            .map_err(|e| format!("Failed to prepare statement: {e}"))?;
        let snapshots = stmt
            .query_map([job_id], |row| {
                Ok(JobSnapshot {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    captured_at: row.get(2)?,
                    source_url: row.get(3)?,
                    raw_text: row.get(4)?,
                    normalized_text: row.get(5)?,
                    title: row.get(6)?,
                    company: row.get(7)?,
                    location: row.get(8)?,
                    salary: row.get(9)?,
                    description: row.get(10)?,
                    requirements: row.get(11)?,
                    responsibilities: row.get(12)?,
                    extraction_metadata: row.get(13)?,
                })
            })
            .map_err(|e| format!("Failed to query snapshots: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect snapshots: {e}"))?;
        Ok(snapshots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory() {
        let storage = SqliteStorage::open(Path::new(":memory:")).unwrap();
        assert!(storage.initialize().is_ok());
    }

    fn create_test_storage() -> SqliteStorage {
        let storage = SqliteStorage::open(Path::new(":memory:")).unwrap();
        storage.initialize().unwrap();
        storage
    }

    fn test_job(id: &str) -> Job {
        Job::new(
            id.to_string(),
            "Software Engineer".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
        )
    }

    #[test]
    fn create_and_get_job() {
        let storage = create_test_storage();
        let job = test_job("job-1");

        storage.create_job(&job).unwrap();
        let retrieved = storage.get_job("job-1").unwrap();
        assert_eq!(retrieved.id, "job-1");
        assert_eq!(retrieved.title, "Software Engineer");
    }

    #[test]
    fn create_job_validates() {
        let storage = create_test_storage();
        let job = Job::new("".to_string(), "title".to_string(), "2026-01-01T00:00:00Z".to_string());
        assert!(storage.create_job(&job).is_err());
    }

    #[test]
    fn get_nonexistent_job_fails() {
        let storage = create_test_storage();
        assert!(storage.get_job("nonexistent").is_err());
    }

    #[test]
    fn update_job() {
        let storage = create_test_storage();
        let mut job = test_job("job-1");
        storage.create_job(&job).unwrap();

        job.title = "Senior Software Engineer".to_string();
        job.updated_at = "2026-01-02T00:00:00Z".to_string();
        storage.update_job(&job).unwrap();

        let retrieved = storage.get_job("job-1").unwrap();
        assert_eq!(retrieved.title, "Senior Software Engineer");
    }

    #[test]
    fn update_nonexistent_job_fails() {
        let storage = create_test_storage();
        let job = test_job("nonexistent");
        assert!(storage.update_job(&job).is_err());
    }

    #[test]
    fn delete_job() {
        let storage = create_test_storage();
        let job = test_job("job-1");
        storage.create_job(&job).unwrap();

        storage.delete_job("job-1").unwrap();
        assert!(storage.get_job("job-1").is_err());
    }

    #[test]
    fn delete_nonexistent_job_fails() {
        let storage = create_test_storage();
        assert!(storage.delete_job("nonexistent").is_err());
    }

    #[test]
    fn list_jobs() {
        let storage = create_test_storage();
        let job1 = test_job("job-1");
        let job2 = test_job("job-2");
        storage.create_job(&job1).unwrap();
        storage.create_job(&job2).unwrap();

        let jobs = storage.list_jobs().unwrap();
        assert_eq!(jobs.len(), 2);
    }

    #[test]
    fn list_jobs_empty() {
        let storage = create_test_storage();
        let jobs = storage.list_jobs().unwrap();
        assert!(jobs.is_empty());
    }

    fn test_snapshot(id: &str, job_id: &str) -> JobSnapshot {
        JobSnapshot::new(
            id.to_string(),
            job_id.to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            "Software Engineer".to_string(),
            "Description of the job".to_string(),
        )
    }

    #[test]
    fn create_and_get_snapshot() {
        let storage = create_test_storage();
        let job = test_job("job-1");
        storage.create_job(&job).unwrap();

        let snapshot = test_snapshot("snap-1", "job-1");
        storage.create_snapshot(&snapshot).unwrap();

        let retrieved = storage.get_snapshot("snap-1").unwrap();
        assert_eq!(retrieved.id, "snap-1");
        assert_eq!(retrieved.job_id, "job-1");
        assert_eq!(retrieved.title, "Software Engineer");
    }

    #[test]
    fn create_snapshot_validates() {
        let storage = create_test_storage();
        let snapshot = JobSnapshot::new(
            "".to_string(),
            "job-1".to_string(),
            "2026-01-01T00:00:00Z".to_string(),
            "title".to_string(),
            "description".to_string(),
        );
        assert!(storage.create_snapshot(&snapshot).is_err());
    }

    #[test]
    fn get_nonexistent_snapshot_fails() {
        let storage = create_test_storage();
        assert!(storage.get_snapshot("nonexistent").is_err());
    }

    #[test]
    fn get_snapshots_for_job() {
        let storage = create_test_storage();
        let job = test_job("job-1");
        storage.create_job(&job).unwrap();

        let snap1 = test_snapshot("snap-1", "job-1");
        let snap2 = test_snapshot("snap-2", "job-1");
        storage.create_snapshot(&snap1).unwrap();
        storage.create_snapshot(&snap2).unwrap();

        let snapshots = storage.get_snapshots_for_job("job-1").unwrap();
        assert_eq!(snapshots.len(), 2);
    }

    #[test]
    fn get_snapshots_for_job_empty() {
        let storage = create_test_storage();
        let snapshots = storage.get_snapshots_for_job("nonexistent").unwrap();
        assert!(snapshots.is_empty());
    }

    #[test]
    fn snapshots_ordered_by_captured_at() {
        let storage = create_test_storage();
        let job = test_job("job-1");
        storage.create_job(&job).unwrap();

        let mut snap1 = test_snapshot("snap-1", "job-1");
        snap1.captured_at = "2026-01-01T00:00:00Z".to_string();
        let mut snap2 = test_snapshot("snap-2", "job-1");
        snap2.captured_at = "2026-01-02T00:00:00Z".to_string();
        storage.create_snapshot(&snap1).unwrap();
        storage.create_snapshot(&snap2).unwrap();

        let snapshots = storage.get_snapshots_for_job("job-1").unwrap();
        assert_eq!(snapshots[0].id, "snap-2");
        assert_eq!(snapshots[1].id, "snap-1");
    }
}
