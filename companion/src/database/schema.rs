use rusqlite::Connection;

pub fn create_tables(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS job (
            id TEXT PRIMARY KEY,
            company_id TEXT,
            title TEXT NOT NULL,
            canonical_url TEXT,
            location TEXT,
            employment_type TEXT,
            salary TEXT,
            source TEXT,
            external_job_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS job_snapshot (
            id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL,
            captured_at TEXT NOT NULL,
            source_url TEXT,
            raw_text TEXT,
            normalized_text TEXT,
            title TEXT NOT NULL,
            company TEXT,
            location TEXT,
            salary TEXT,
            description TEXT NOT NULL,
            requirements TEXT,
            responsibilities TEXT,
            extraction_metadata TEXT,
            FOREIGN KEY (job_id) REFERENCES job(id)
        );

        CREATE TABLE IF NOT EXISTS job_changeset (
            id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL,
            from_snapshot_id TEXT NOT NULL,
            to_snapshot_id TEXT NOT NULL,
            added TEXT,
            removed TEXT,
            modified TEXT,
            moved TEXT,
            reordered TEXT,
            added_requirements TEXT,
            removed_requirements TEXT,
            metadata TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (job_id) REFERENCES job(id),
            FOREIGN KEY (from_snapshot_id) REFERENCES job_snapshot(id),
            FOREIGN KEY (to_snapshot_id) REFERENCES job_snapshot(id)
        );

        CREATE INDEX IF NOT EXISTS idx_job_snapshot_job_id ON job_snapshot(job_id);
        CREATE INDEX IF NOT EXISTS idx_job_changeset_job_id ON job_changeset(job_id);
        "
    )
    .map_err(|e| format!("Failed to create tables: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn tables_are_created() {
        let conn = Connection::open(":memory:").unwrap();
        create_tables(&conn).unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('job', 'job_snapshot', 'job_changeset')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn indexes_are_created() {
        let conn = Connection::open(":memory:").unwrap();
        create_tables(&conn).unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count >= 2);
    }

    #[test]
    fn tables_are_idempotent() {
        let conn = Connection::open(":memory:").unwrap();
        create_tables(&conn).unwrap();
        create_tables(&conn).unwrap();
    }
}
