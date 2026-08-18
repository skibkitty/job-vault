use argon2::Argon2;
use ring::aead;
use ring::hmac;
use rand::RngCore;
use rand::rngs::OsRng;

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;
pub const TAG_LEN: usize = 16;
pub const WRAPPED_KEY_LEN: usize = KEY_LEN + TAG_LEN;

pub const ARGON2_MEMORY_KIB: u32 = 65536;
pub const ARGON2_ITERATIONS: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 4;

pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn generate_key() -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn derive_kek(password: &str, salt: &[u8; SALT_LEN]) -> [u8; KEY_LEN] {
    let mut kek = [0u8; KEY_LEN];
    let params = argon2::Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(KEY_LEN),
    ).expect("valid argon2 params");
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    argon2.hash_password_into(password.as_bytes(), salt, &mut kek)
        .expect("argon2 hash should not fail");
    kek
}

pub fn compute_verification_tag(kek: &[u8; KEY_LEN]) -> [u8; KEY_LEN] {
    let key = hmac::Key::new(hmac::HMAC_SHA256, kek);
    let tag = hmac::sign(&key, b"verify");
    let mut result = [0u8; KEY_LEN];
    result.copy_from_slice(tag.as_ref());
    result
}

pub fn verify_password(password: &str, salt: &[u8; SALT_LEN], expected_tag: &[u8; KEY_LEN]) -> bool {
    let kek = derive_kek(password, salt);
    hmac::verify(&hmac::Key::new(hmac::HMAC_SHA256, &kek), b"verify", expected_tag).is_ok()
}

pub fn wrap_key(dek: &[u8; KEY_LEN], kek: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN]) -> Result<[u8; WRAPPED_KEY_LEN], String> {
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, kek)
        .map_err(|e| format!("Invalid key: {e}"))?;
    let less_safe_key = aead::LessSafeKey::new(unbound_key);

    let mut in_out = dek.to_vec();
    let nonce = aead::Nonce::try_assume_unique_for_key(nonce)
        .map_err(|e| format!("Invalid nonce: {e}"))?;
    let aad = aead::Aad::empty();

    less_safe_key.seal_in_place_append_tag(nonce, aad, &mut in_out)
        .map_err(|e| format!("Encryption failed: {e}"))?;

    let mut result = [0u8; WRAPPED_KEY_LEN];
    result.copy_from_slice(&in_out);
    Ok(result)
}

pub fn unwrap_key(wrapped: &[u8; WRAPPED_KEY_LEN], kek: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN]) -> Result<[u8; KEY_LEN], String> {
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, kek)
        .map_err(|e| format!("Invalid key: {e}"))?;
    let less_safe_key = aead::LessSafeKey::new(unbound_key);

    let nonce = aead::Nonce::try_assume_unique_for_key(nonce)
        .map_err(|e| format!("Invalid nonce: {e}"))?;
    let aad = aead::Aad::empty();

    let mut in_out = wrapped.to_vec();
    let plaintext = less_safe_key.open_in_place(nonce, aad, &mut in_out)
        .map_err(|e| format!("Decryption failed (wrong password?): {e}"))?;

    let mut result = [0u8; KEY_LEN];
    result.copy_from_slice(plaintext);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salt_is_random() {
        let s1 = generate_salt();
        let s2 = generate_salt();
        assert_ne!(s1, s2);
    }

    #[test]
    fn derive_kek_deterministic() {
        let salt = generate_salt();
        let k1 = derive_kek("password", &salt);
        let k2 = derive_kek("password", &salt);
        assert_eq!(k1, k2);
    }

    #[test]
    fn derive_kek_different_passwords() {
        let salt = generate_salt();
        let k1 = derive_kek("password1", &salt);
        let k2 = derive_kek("password2", &salt);
        assert_ne!(k1, k2);
    }

    #[test]
    fn verification_tag_works() {
        let salt = generate_salt();
        let kek = derive_kek("password", &salt);
        let tag = compute_verification_tag(&kek);
        assert!(verify_password("password", &salt, &tag));
        assert!(!verify_password("wrong", &salt, &tag));
    }

    #[test]
    fn wrap_unwrap_roundtrip() {
        let dek = generate_key();
        let kek = derive_kek("password", &generate_salt());
        let nonce = generate_nonce();
        let wrapped = wrap_key(&dek, &kek, &nonce).unwrap();
        let unwrapped = unwrap_key(&wrapped, &kek, &nonce).unwrap();
        assert_eq!(dek, unwrapped);
    }

    #[test]
    fn unwrap_wrong_key_fails() {
        let dek = generate_key();
        let kek = derive_kek("password", &generate_salt());
        let wrong_kek = derive_kek("wrong", &generate_salt());
        let nonce = generate_nonce();
        let wrapped = wrap_key(&dek, &kek, &nonce).unwrap();
        assert!(unwrap_key(&wrapped, &wrong_kek, &nonce).is_err());
    }
}
