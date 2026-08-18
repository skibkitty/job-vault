use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultState {
    Locked,
    Unlocking,
    Unlocked,
    Locking,
    Error,
}

pub struct Vault {
    state: VaultState,
}

impl Vault {
    pub fn new() -> Self {
        Self {
            state: VaultState::Locked,
        }
    }

    pub fn state(&self) -> &VaultState {
        &self.state
    }

    pub fn unlock(&mut self, _password: &str, _vault_path: &Path) -> Result<(), String> {
        self.state = VaultState::Unlocking;
        // TODO: implement KDF + decryption
        self.state = VaultState::Unlocked;
        Ok(())
    }

    pub fn lock(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
