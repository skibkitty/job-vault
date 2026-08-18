pub mod schema;
pub mod sqlite;

pub use sqlite::SqliteStorage;

use std::path::Path;

pub struct Database {
    storage: Option<SqliteStorage>,
    path: std::path::PathBuf,
}

impl Database {
    pub fn new(path: &Path) -> Self {
        Self {
            storage: None,
            path: path.to_path_buf(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        let storage = SqliteStorage::open(&self.path)?;
        storage.initialize()?;
        self.storage = Some(storage);
        Ok(())
    }

    pub fn storage(&self) -> Result<&SqliteStorage, String> {
        self.storage
            .as_ref()
            .ok_or_else(|| "Database not initialized".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn database_creates() {
        let mut db = Database::new(&PathBuf::from(":memory:"));
        assert!(db.initialize().is_ok());
        assert!(db.storage().is_ok());
    }

    #[test]
    fn database_uninitialized_returns_error() {
        let db = Database::new(&PathBuf::from(":memory:"));
        assert!(db.storage().is_err());
    }
}
