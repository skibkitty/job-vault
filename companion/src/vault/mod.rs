use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::crypto;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultState {
    Locked,
    Unlocking,
    Unlocked,
    Locking,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultHeader {
    pub version: u32,
    pub kdf: String,
    pub kdf_params: KdfParams,
    pub salt: String,
    pub verification_tag: String,
    pub encrypted_dek_nonce: String,
    pub encrypted_dek: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KdfParams {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

pub struct Vault {
    state: VaultState,
    header: Option<VaultHeader>,
    dek: Option<[u8; crypto::KEY_LEN]>,
    vault_dir: Option<std::path::PathBuf>,
}

impl Vault {
    pub fn new() -> Self {
        Self {
            state: VaultState::Locked,
            header: None,
            dek: None,
            vault_dir: None,
        }
    }

    pub fn state(&self) -> &VaultState {
        &self.state
    }

    pub fn create(password: &str, vault_dir: &Path) -> Result<Self, String> {
        let header_path = vault_dir.join("vault.json");
        if header_path.exists() {
            return Err("Vault already exists".to_string());
        }

        let salt = crypto::generate_salt();
        let kek = crypto::derive_kek(password, &salt);
        let verification_tag = crypto::compute_verification_tag(&kek);

        let dek = crypto::generate_key();
        let dek_nonce = crypto::generate_nonce();
        let encrypted_dek = crypto::wrap_key(&dek, &kek, &dek_nonce)?;

        let header = VaultHeader {
            version: 1,
            kdf: "argon2id".to_string(),
            kdf_params: KdfParams {
                memory_kib: crypto::ARGON2_MEMORY_KIB,
                iterations: crypto::ARGON2_ITERATIONS,
                parallelism: crypto::ARGON2_PARALLELISM,
            },
            salt: base64_encode(&salt),
            verification_tag: base64_encode(&verification_tag),
            encrypted_dek_nonce: base64_encode(&dek_nonce),
            encrypted_dek: base64_encode(&encrypted_dek),
            created_at: format!("{:?}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()),
        };

        let header_json = serde_json::to_string_pretty(&header)
            .map_err(|e| format!("Failed to serialize header: {e}"))?;
        fs::write(&header_path, header_json)
            .map_err(|e| format!("Failed to write vault header: {e}"))?;

        Ok(Self {
            state: VaultState::Locked,
            header: Some(header),
            dek: None,
            vault_dir: Some(vault_dir.to_path_buf()),
        })
    }

    pub fn unlock(&mut self, password: &str) -> Result<(), String> {
        self.state = VaultState::Unlocking;

        let vault_dir = self.vault_dir.as_ref().ok_or("No vault directory")?;
        let header_path = vault_dir.join("vault.json");
        let header_json = fs::read_to_string(&header_path)
            .map_err(|e| format!("Failed to read vault header: {e}"))?;
        let header: VaultHeader = serde_json::from_str(&header_json)
            .map_err(|e| format!("Failed to parse vault header: {e}"))?;

        let salt = base64_decode(&header.salt)?;
        let verification_tag = base64_decode(&header.verification_tag)?;
        let encrypted_dek = base64_decode(&header.encrypted_dek)?;
        let dek_nonce = base64_decode(&header.encrypted_dek_nonce)?;

        let mut salt_arr = [0u8; crypto::SALT_LEN];
        salt_arr.copy_from_slice(&salt);
        let mut tag_arr = [0u8; crypto::KEY_LEN];
        tag_arr.copy_from_slice(&verification_tag);

        if !crypto::verify_password(password, &salt_arr, &tag_arr) {
            self.state = VaultState::Error;
            return Err("Invalid password".to_string());
        }

        let kek = crypto::derive_kek(password, &salt_arr);

        let mut wrapped_arr = [0u8; crypto::WRAPPED_KEY_LEN];
        wrapped_arr.copy_from_slice(&encrypted_dek);

        let mut nonce_arr = [0u8; crypto::NONCE_LEN];
        nonce_arr.copy_from_slice(&dek_nonce);

        let dek = crypto::unwrap_key(&wrapped_arr, &kek, &nonce_arr)?;

        self.header = Some(header);
        self.dek = Some(dek);
        self.state = VaultState::Unlocked;
        Ok(())
    }

    pub fn lock(&mut self) {
        self.dek = None;
        self.state = VaultState::Locked;
    }

    pub fn is_unlocked(&self) -> bool {
        self.state == VaultState::Unlocked
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64ct::{Base64, Encoding};
    let mut buf = vec![0u8; data.len() * 4 / 3 + 4];
    let encoded = Base64::encode(data, &mut buf).unwrap();
    encoded.to_string()
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    use base64ct::{Base64, Encoding};
    let max_len = s.len();
    let mut buf = vec![0u8; max_len];
    let decoded = Base64::decode(s, &mut buf)
        .map_err(|e| format!("Base64 decode failed: {e}"))?;
    Ok(decoded.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn vault_starts_locked() {
        let vault = Vault::new();
        assert_eq!(vault.state(), &VaultState::Locked);
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn vault_locks() {
        let mut vault = Vault::new();
        vault.lock();
        assert_eq!(vault.state(), &VaultState::Locked);
    }

    #[test]
    fn create_and_unlock() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("password", vault_dir).unwrap();
        assert!(!vault.is_unlocked());

        vault.unlock("password").unwrap();
        assert!(vault.is_unlocked());

        vault.lock();
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn wrong_password_fails() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("password", vault_dir).unwrap();
        let result = vault.unlock("wrong");
        assert!(result.is_err());
        assert_eq!(vault.state(), &VaultState::Error);
    }

    #[test]
    fn create_fails_if_exists() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        Vault::create("password", vault_dir).unwrap();
        let result = Vault::create("password", vault_dir);
        assert!(result.is_err());
    }

    #[test]
    fn vault_header_written() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        Vault::create("password", vault_dir).unwrap();
        let header_path = vault_dir.join("vault.json");
        assert!(header_path.exists());

        let content = fs::read_to_string(&header_path).unwrap();
        let header: VaultHeader = serde_json::from_str(&content).unwrap();
        assert_eq!(header.version, 1);
        assert_eq!(header.kdf, "argon2id");
    }
}
