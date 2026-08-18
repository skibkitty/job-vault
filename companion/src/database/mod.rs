use std::path::Path;

pub struct Database {
    _path: std::path::PathBuf,
}

impl Database {
    pub fn new(_path: &Path) -> Self {
        Self {
            _path: _path.to_path_buf(),
        }
    }

    pub fn initialize(&self) -> Result<(), String> {
        // TODO: create schema
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn database_creates() {
        let db = Database::new(&PathBuf::from(":memory:"));
        assert!(db.initialize().is_ok());
    }
}
