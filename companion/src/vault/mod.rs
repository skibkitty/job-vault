use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::crypto;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultState {
    Locked,
    Unlocking,
    Unlocked,
    Locking,
    Error,
}

impl VaultState {
    fn can_transition_to(&self, next: &VaultState) -> bool {
        matches!(
            (self, next),
            (VaultState::Locked, VaultState::Unlocking)
                | (VaultState::Unlocking, VaultState::Unlocked)
                | (VaultState::Unlocking, VaultState::Error)
                | (VaultState::Unlocked, VaultState::Locking)
                | (VaultState::Locking, VaultState::Locked)
                | (VaultState::Error, VaultState::Locked)
                | (VaultState::Error, VaultState::Unlocking)
                | (VaultState::Locked, VaultState::Locked)
        )
    }
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
    dek: Option<ZeroizeKey>,
    vault_dir: Option<std::path::PathBuf>,
    last_activity: Option<std::time::Instant>,
    auto_lock_timeout: Option<std::time::Duration>,
}

struct ZeroizeKey([u8; crypto::KEY_LEN]);

impl Zeroize for ZeroizeKey {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for ZeroizeKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl std::fmt::Debug for ZeroizeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZeroizeKey").finish_non_exhaustive()
    }
}

impl std::fmt::Debug for Vault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vault")
            .field("state", &self.state)
            .field("header", &self.header)
            .field("dek", &self.dek.as_ref().map(|_| "..."))
            .field("vault_dir", &self.vault_dir)
            .field("last_activity", &self.last_activity)
            .field("auto_lock_timeout", &self.auto_lock_timeout)
            .finish()
    }
}

impl Vault {
    pub fn new() -> Self {
        Self {
            state: VaultState::Locked,
            header: None,
            dek: None,
            vault_dir: None,
            last_activity: None,
            auto_lock_timeout: None,
        }
    }

    pub fn with_auto_lock_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.auto_lock_timeout = Some(timeout);
        self
    }

    pub fn state(&self) -> &VaultState {
        &self.state
    }

    fn transition(&mut self, next: VaultState) -> Result<(), String> {
        if !self.state.can_transition_to(&next) {
            return Err(format!(
                "Invalid state transition: {:?} -> {:?}",
                self.state, next
            ));
        }
        self.state = next;
        Ok(())
    }

    fn update_activity(&mut self) {
        self.last_activity = Some(std::time::Instant::now());
    }

    pub fn should_auto_lock(&self) -> bool {
        if let (Some(timeout), Some(last_activity)) = (self.auto_lock_timeout, self.last_activity) {
            last_activity.elapsed() >= timeout
        } else {
            false
        }
    }

    pub fn check_and_auto_lock(&mut self) -> Result<bool, String> {
        if self.should_auto_lock() && self.state == VaultState::Unlocked {
            self.lock()?;
            Ok(true)
        } else {
            Ok(false)
        }
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
            last_activity: None,
            auto_lock_timeout: None,
        })
    }

    pub fn open(vault_dir: &Path) -> Result<Self, String> {
        let header_path = vault_dir.join("vault.json");
        if !header_path.exists() {
            return Err("Vault does not exist".to_string());
        }

        let header_json = fs::read_to_string(&header_path)
            .map_err(|e| format!("Failed to read vault header: {e}"))?;
        let header: VaultHeader = serde_json::from_str(&header_json)
            .map_err(|e| format!("Failed to parse vault header: {e}"))?;

        Ok(Self {
            state: VaultState::Locked,
            header: Some(header),
            dek: None,
            vault_dir: Some(vault_dir.to_path_buf()),
            last_activity: None,
            auto_lock_timeout: None,
        })
    }

    pub fn unlock(&mut self, password: &str) -> Result<(), String> {
        self.transition(VaultState::Unlocking)?;

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
            let _ = self.transition(VaultState::Error);
            return Err("Invalid password".to_string());
        }

        let kek = crypto::derive_kek(password, &salt_arr);

        let mut wrapped_arr = [0u8; crypto::WRAPPED_KEY_LEN];
        wrapped_arr.copy_from_slice(&encrypted_dek);

        let mut nonce_arr = [0u8; crypto::NONCE_LEN];
        nonce_arr.copy_from_slice(&dek_nonce);

        let dek = crypto::unwrap_key(&wrapped_arr, &kek, &nonce_arr)?;

        self.header = Some(header);
        self.dek = Some(ZeroizeKey(dek));
        self.update_activity();
        self.transition(VaultState::Unlocked)?;
        Ok(())
    }

    pub fn lock(&mut self) -> Result<(), String> {
        match self.state {
            VaultState::Locked | VaultState::Error => {
                self.dek = None;
                self.state = VaultState::Locked;
                Ok(())
            }
            VaultState::Unlocked => {
                self.transition(VaultState::Locking)?;
                if let Some(mut key) = self.dek.take() {
                    key.zeroize();
                }
                self.transition(VaultState::Locked)?;
                Ok(())
            }
            _ => Err(format!(
                "Cannot lock in state {:?}",
                self.state
            )),
        }
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
        vault.lock().unwrap();
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

        vault.lock().unwrap();
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

    #[test]
    fn cannot_unlock_when_unlocked() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("password", vault_dir).unwrap();
        vault.unlock("password").unwrap();
        assert!(vault.is_unlocked());

        let result = vault.unlock("password");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid state transition"));
    }

    #[test]
    fn cannot_lock_when_locked() {
        let mut vault = Vault::new();
        let result = vault.lock();
        assert!(result.is_ok());
        assert_eq!(vault.state(), &VaultState::Locked);
    }

    #[test]
    fn cannot_unlock_when_locking() {
        let mut vault = Vault::new();
        vault.state = VaultState::Locking;
        let result = vault.unlock("password");
        assert!(result.is_err());
    }

    #[test]
    fn cannot_lock_when_unlocking() {
        let mut vault = Vault::new();
        vault.state = VaultState::Unlocking;
        let result = vault.lock();
        assert!(result.is_err());
    }

    #[test]
    fn error_can_transition_to_locked() {
        let mut vault = Vault::new();
        vault.state = VaultState::Error;
        vault.lock().unwrap();
        assert_eq!(vault.state(), &VaultState::Locked);
    }

    #[test]
    fn error_can_transition_to_unlocking() {
        let mut vault = Vault::new();
        vault.state = VaultState::Error;
        let result = vault.transition(VaultState::Unlocking);
        assert!(result.is_ok());
    }

    #[test]
    fn state_transition_validation() {
        assert!(VaultState::Locked.can_transition_to(&VaultState::Unlocking));
        assert!(VaultState::Locked.can_transition_to(&VaultState::Locked));
        assert!(!VaultState::Locked.can_transition_to(&VaultState::Unlocked));
        assert!(!VaultState::Locked.can_transition_to(&VaultState::Locking));

        assert!(VaultState::Unlocking.can_transition_to(&VaultState::Unlocked));
        assert!(VaultState::Unlocking.can_transition_to(&VaultState::Error));
        assert!(!VaultState::Unlocking.can_transition_to(&VaultState::Locked));

        assert!(VaultState::Unlocked.can_transition_to(&VaultState::Locking));
        assert!(!VaultState::Unlocked.can_transition_to(&VaultState::Locked));
        assert!(!VaultState::Unlocked.can_transition_to(&VaultState::Unlocking));

        assert!(VaultState::Locking.can_transition_to(&VaultState::Locked));
        assert!(!VaultState::Locking.can_transition_to(&VaultState::Unlocked));

        assert!(VaultState::Error.can_transition_to(&VaultState::Locked));
        assert!(VaultState::Error.can_transition_to(&VaultState::Unlocking));
        assert!(!VaultState::Error.can_transition_to(&VaultState::Unlocked));
    }

    #[test]
    fn vault_open_existing() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        Vault::create("password", vault_dir).unwrap();

        let vault = Vault::open(vault_dir).unwrap();
        assert_eq!(vault.state(), &VaultState::Locked);
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn vault_open_nonexistent_fails() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let result = Vault::open(vault_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn vault_open_and_unlock() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        Vault::create("password", vault_dir).unwrap();

        let mut vault = Vault::open(vault_dir).unwrap();
        vault.unlock("password").unwrap();
        assert!(vault.is_unlocked());

        vault.lock().unwrap();
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn auto_lock_timeout() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("password", vault_dir)
            .unwrap()
            .with_auto_lock_timeout(std::time::Duration::from_millis(50));

        vault.unlock("password").unwrap();
        assert!(vault.is_unlocked());
        assert!(!vault.should_auto_lock());

        std::thread::sleep(std::time::Duration::from_millis(100));

        assert!(vault.should_auto_lock());
        let locked = vault.check_and_auto_lock().unwrap();
        assert!(locked);
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn auto_lock_not_triggered_without_timeout() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("password", vault_dir).unwrap();
        vault.unlock("password").unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));

        assert!(!vault.should_auto_lock());
        let locked = vault.check_and_auto_lock().unwrap();
        assert!(!locked);
        assert!(vault.is_unlocked());
    }

    #[test]
    fn auto_lock_only_when_unlocked() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let vault = Vault::create("password", vault_dir)
            .unwrap()
            .with_auto_lock_timeout(std::time::Duration::from_millis(0));

        assert!(!vault.should_auto_lock());
    }

    #[test]
    fn multiple_wrong_passwords_fail_safely() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("password", vault_dir).unwrap();

        let wrong_passwords = ["wrong1", "wrong2", "wrong3", ""];
        for pw in wrong_passwords {
            let result = vault.unlock(pw);
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Invalid password"));
            assert_eq!(vault.state(), &VaultState::Error);
        }
    }

    #[test]
    fn recovery_from_error_via_lock() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("password", vault_dir).unwrap();
        vault.unlock("wrong").unwrap_err();
        assert_eq!(vault.state(), &VaultState::Error);

        vault.lock().unwrap();
        assert_eq!(vault.state(), &VaultState::Locked);

        vault.unlock("password").unwrap();
        assert!(vault.is_unlocked());
    }

    #[test]
    fn corrupted_header_json_fails() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        Vault::create("password", vault_dir).unwrap();
        let header_path = vault_dir.join("vault.json");
        fs::write(&header_path, "{invalid json").unwrap();

        let result = Vault::open(vault_dir);
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_salt_fails() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        Vault::create("password", vault_dir).unwrap();
        let header_path = vault_dir.join("vault.json");
        let content = fs::read_to_string(&header_path).unwrap();
        let mut header: VaultHeader = serde_json::from_str(&content).unwrap();
        header.salt = "not-valid-base64!!!".to_string();
        fs::write(&header_path, serde_json::to_string_pretty(&header).unwrap()).unwrap();

        let mut vault = Vault::open(vault_dir).unwrap();
        let result = vault.unlock("password");
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_verification_tag_fails() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        Vault::create("password", vault_dir).unwrap();
        let header_path = vault_dir.join("vault.json");
        let content = fs::read_to_string(&header_path).unwrap();
        let mut header: VaultHeader = serde_json::from_str(&content).unwrap();
        header.verification_tag = "!!!".to_string();
        fs::write(&header_path, serde_json::to_string_pretty(&header).unwrap()).unwrap();

        let mut vault = Vault::open(vault_dir).unwrap();
        let result = vault.unlock("password");
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_encrypted_dek_fails() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        Vault::create("password", vault_dir).unwrap();
        let header_path = vault_dir.join("vault.json");
        let content = fs::read_to_string(&header_path).unwrap();
        let mut header: VaultHeader = serde_json::from_str(&content).unwrap();
        header.encrypted_dek = "!!!".to_string();
        fs::write(&header_path, serde_json::to_string_pretty(&header).unwrap()).unwrap();

        let mut vault = Vault::open(vault_dir).unwrap();
        let result = vault.unlock("password");
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_encrypted_dek_nonce_fails() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        Vault::create("password", vault_dir).unwrap();
        let header_path = vault_dir.join("vault.json");
        let content = fs::read_to_string(&header_path).unwrap();
        let mut header: VaultHeader = serde_json::from_str(&content).unwrap();
        header.encrypted_dek_nonce = "!!!".to_string();
        fs::write(&header_path, serde_json::to_string_pretty(&header).unwrap()).unwrap();

        let mut vault = Vault::open(vault_dir).unwrap();
        let result = vault.unlock("password");
        assert!(result.is_err());
    }

    #[test]
    fn missing_header_file_fails() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("password", vault_dir).unwrap();
        let header_path = vault_dir.join("vault.json");
        fs::remove_file(&header_path).unwrap();

        let result = vault.unlock("password");
        assert!(result.is_err());
    }

    #[test]
    fn multiple_lock_unlock_cycles() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("password", vault_dir).unwrap();

        for i in 0..10 {
            vault.unlock("password").unwrap();
            assert!(vault.is_unlocked(), "Should be unlocked at cycle {i}");

            vault.lock().unwrap();
            assert!(!vault.is_unlocked(), "Should be locked at cycle {i}");
        }
    }

    #[test]
    fn lock_unlock_with_reopen() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        Vault::create("password", vault_dir).unwrap();

        for _ in 0..5 {
            let mut vault = Vault::open(vault_dir).unwrap();
            vault.unlock("password").unwrap();
            assert!(vault.is_unlocked());
            vault.lock().unwrap();
        }

        let mut vault = Vault::open(vault_dir).unwrap();
        vault.unlock("password").unwrap();
        assert!(vault.is_unlocked());
    }

    #[test]
    fn error_messages_do_not_leak_sensitive_data() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("mysecretpassword", vault_dir).unwrap();

        let wrong_result = vault.unlock("wrongpassword").unwrap_err();
        assert!(!wrong_result.contains("mysecretpassword"));
        assert!(!wrong_result.contains("wrongpassword"));
        assert!(!wrong_result.contains("dek"));
        assert!(!wrong_result.contains("kek"));
    }

    #[test]
    fn vault_state_consistent_after_failed_unlock() {
        let tmp = TempDir::new().unwrap();
        let vault_dir = tmp.path();

        let mut vault = Vault::create("password", vault_dir).unwrap();

        vault.unlock("wrong").unwrap_err();
        vault.lock().unwrap();

        vault.unlock("password").unwrap();
        assert!(vault.is_unlocked());

        vault.lock().unwrap();
        assert!(!vault.is_unlocked());

        vault.unlock("wrong").unwrap_err();
        vault.lock().unwrap();

        vault.unlock("password").unwrap();
        assert!(vault.is_unlocked());
    }
}
