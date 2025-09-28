use zeroize::Zeroize;
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use dirs_next::config_dir;
use std::fs;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use rand::{RngCore, rngs::OsRng};
use base64::Engine;

fn key_material() -> [u8;32] {
    // If feature "kdf-argon2" enabled, derive with Argon2id using per-user salt; else fallback to SHA256(user||host||salt)
    let user = whoami::username();
    // Prefer fallible hostname API; fall back to placeholder without using deprecated call.
    let host = whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string());
    let salt_static = b"AEONMI_API_KEY_SALT_v1";
    #[cfg(feature = "kdf-argon2")]
    {
        use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, PasswordHash}};
        // Construct deterministic salt from user+host hashed to 16 bytes to avoid storing separate salt file
        let mut hasher = Sha256::new(); hasher.update(user.as_bytes()); hasher.update(host.as_bytes()); hasher.update(salt_static); let full = hasher.finalize();
        let salt_bytes = &full[..16];
        let salt_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(salt_bytes);
        let salt = SaltString::new(&salt_b64).unwrap_or_else(|_| SaltString::encode_b64(salt_bytes).unwrap());
        let argon = Argon2::default(); // default params (can be tuned)
        let mut key = [0u8;32];
        // Use password as user+host (not secret) purely to expand into key space; security relies on local file protection.
        if argon.hash_password_into(format!("{}:{}", user, host).as_bytes(), salt.as_salt(), &mut key).is_err() {
            return fallback_sha256(&user, &host, salt_static);
        }
        return key;
    }
    #[cfg(not(feature = "kdf-argon2"))]
    {
        fallback_sha256(&user, &host, salt_static)
    }
}

fn fallback_sha256(user: &str, host: &str, salt: &[u8]) -> [u8;32] {
    let mut hasher = Sha256::new(); hasher.update(user.as_bytes()); hasher.update(host.as_bytes()); hasher.update(salt); let out = hasher.finalize(); let mut arr=[0u8;32]; arr.copy_from_slice(&out[..32]); arr
}

fn storage_path() -> PathBuf {
    if let Ok(base) = std::env::var("AEONMI_CONFIG_DIR") { return PathBuf::from(base).join("keys.json"); }
    config_dir().unwrap_or(std::env::temp_dir()).join("aeonmi").join("keys.json")
}

const KEY_FORMAT_VERSION: u32 = 1; // increment if structure changes

pub fn set_api_key(provider: &str, key: &str) -> Result<(), String> {
    let mut data = load_all_raw();
    let key_bytes = key_material();
    let mut nonce = [0u8;12]; OsRng.fill_bytes(&mut nonce);
    let mut cipher = ChaCha20::new((&key_bytes).into(), (&nonce).into());
    let mut buf = key.as_bytes().to_vec();
    cipher.apply_keystream(&mut buf);
    let mut stored = Vec::with_capacity(12+buf.len());
    stored.extend_from_slice(&nonce); stored.extend_from_slice(&buf);
    buf.zeroize();
    // Store as JSON object (string) embedding version marker so future migrations possible
    let entry = serde_json::json!({
        "v": KEY_FORMAT_VERSION,
        "alg": "ChaCha20",
    "nonce_ct_b64": base64::engine::general_purpose::STANDARD.encode(&stored)
    });
    data.insert(provider.to_string(), entry.to_string());
    save_all_raw(&data)
}

pub fn get_api_key(provider: &str) -> Option<String> {
    let data = load_all_raw(); let raw = data.get(provider)?;
    // Backwards compatibility: either plain base64 (legacy) or JSON object
    let (b64, _ver) = if raw.trim_start().starts_with('{') {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
            if let Some(nc) = val.get("nonce_ct_b64").and_then(|v| v.as_str()) { (nc.to_string(), val.get("v").and_then(|v| v.as_u64()).unwrap_or(0) as u32) } else { ("".to_string(), 0) }
        } else { ("".to_string(), 0) }
    } else { (raw.clone(), 0) };
    if b64.is_empty() { return None; }
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?; if bytes.len()<13 { return None; }
    let mut nonce=[0u8;12]; nonce.copy_from_slice(&bytes[..12]);
    let mut ct = bytes[12..].to_vec();
    let key_bytes = key_material();
    let mut cipher = ChaCha20::new((&key_bytes).into(), (&nonce).into());
    cipher.apply_keystream(&mut ct);
    let s = String::from_utf8_lossy(&ct).to_string(); ct.zeroize(); Some(s)
}

pub fn delete_api_key(provider: &str) -> Result<(), String> { let mut data = load_all_raw(); data.remove(provider); save_all_raw(&data) }

pub fn list_providers() -> Vec<String> { let data = load_all_raw(); data.keys().cloned().collect() }

fn load_all_raw() -> std::collections::HashMap<String,String> {
    let path = storage_path();
    if let Ok(txt) = fs::read_to_string(path) { serde_json::from_str(&txt).unwrap_or_default() } else { Default::default() }
}
fn save_all_raw(map: &std::collections::HashMap<String,String>) -> Result<(), String> {
    let path = storage_path(); if let Some(parent)=path.parent() { let _=fs::create_dir_all(parent); }
    fs::write(path, serde_json::to_string_pretty(map).unwrap()).map_err(|e| e.to_string())
}

/// Re-encrypt all stored provider keys with current key_material (e.g., after enabling new KDF feature)
pub struct RotationReport { pub attempted: usize, pub rotated: usize, pub errors: Vec<(String,String)> }

pub fn rotate_all_keys() -> Result<RotationReport, String> {
    let data = load_all_raw();
    let providers: Vec<String> = data.keys().cloned().collect();
    let mut rotated = 0usize; let mut errors = Vec::new();
    for prov in providers.iter() {
        match get_api_key(prov) {
            Some(plain) => {
                if let Err(e) = set_api_key(prov, &plain) { errors.push((prov.clone(), e)); } else { rotated += 1; }
            },
            None => { errors.push((prov.clone(), "decrypt_failed".to_string())); }
        }
    }
    Ok(RotationReport { attempted: providers.len(), rotated, errors })
}
