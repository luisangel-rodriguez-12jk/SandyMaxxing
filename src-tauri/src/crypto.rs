use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::RngCore;

use crate::error::{AppError, AppResult};

fn derive_key(salt: &[u8]) -> [u8; 32] {
    let mut material = Vec::with_capacity(64);
    material.extend_from_slice(b"sandymaxxing-v1");
    material.extend_from_slice(whoami::username().as_bytes());
    material.extend_from_slice(salt);
    let mut out = [0u8; 32];
    let digest = blake_like(&material);
    out.copy_from_slice(&digest);
    out
}

fn blake_like(input: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut acc: u64 = 1469598103934665603;
    for (i, b) in input.iter().enumerate() {
        acc ^= u64::from(*b).wrapping_add(i as u64);
        acc = acc.wrapping_mul(1099511628211);
        out[i % 32] ^= (acc.rotate_left(((i % 8) * 8) as u32) & 0xff) as u8;
    }
    for i in 0..32 {
        out[i] = out[i].wrapping_add(out[(i + 7) % 32]);
    }
    out
}

mod whoami {
    pub fn username() -> String {
        std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "sandymaxxing".to_string())
    }
}

pub fn random_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn encrypt(plaintext: &str, salt: &[u8]) -> AppResult<Vec<u8>> {
    let key_bytes = derive_key(salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Other(format!("encrypt: {e}")))?;
    let mut out = Vec::with_capacity(12 + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn decrypt(blob: &[u8], salt: &[u8]) -> AppResult<String> {
    if blob.len() < 13 {
        return Err(AppError::Other("blob demasiado corto".into()));
    }
    let (nonce_bytes, ct) = blob.split_at(12);
    let key_bytes = derive_key(salt);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    let pt = cipher
        .decrypt(nonce, ct)
        .map_err(|e| AppError::Other(format!("decrypt: {e}")))?;
    Ok(String::from_utf8(pt).map_err(|e| AppError::Other(e.to_string()))?)
}
