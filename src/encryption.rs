use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use pqcrypto_kyber::kyber1024;
use pqcrypto_sphincsplus::sphincssha2256ssimple as sphincs;
use pqcrypto_traits::kem::{Ciphertext as _, PublicKey as _, SecretKey as _, SharedSecret as _};
use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as _, SecretKey as _};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use zeroize::Zeroize;

/// Persistent key material for the Domain Quantum Vault.
///
/// Keys are stored as raw byte blobs to keep the serialized structure simple.
/// Runtime helpers reconstruct strongly typed key objects on demand.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VaultKeyMaterial {
    pub aes_key: [u8; 32],
    pub kyber_public: Vec<u8>,
    pub kyber_secret: Vec<u8>,
    pub sphincs_public: Vec<u8>,
    pub sphincs_secret: Vec<u8>,
}

impl VaultKeyMaterial {
    pub fn runtime(&self) -> Result<VaultRuntimeKeys> {
        let kyber_public = kyber1024::PublicKey::from_bytes(&self.kyber_public)
            .map_err(|e| anyhow!("kyber public key decode failed: {:?}", e))?;
        let kyber_secret = kyber1024::SecretKey::from_bytes(&self.kyber_secret)
            .map_err(|e| anyhow!("kyber secret key decode failed: {:?}", e))?;
        let sphincs_public = sphincs::PublicKey::from_bytes(&self.sphincs_public)
            .map_err(|e| anyhow!("sphincs public key decode failed: {:?}", e))?;
        let sphincs_secret = sphincs::SecretKey::from_bytes(&self.sphincs_secret)
            .map_err(|e| anyhow!("sphincs secret key decode failed: {:?}", e))?;

        Ok(VaultRuntimeKeys {
            aes_key: self.aes_key,
            kyber_public,
            kyber_secret,
            sphincs_public,
            sphincs_secret,
        })
    }
}

#[derive(Clone)]
pub struct VaultRuntimeKeys {
    pub aes_key: [u8; 32],
    pub kyber_public: kyber1024::PublicKey,
    pub kyber_secret: kyber1024::SecretKey,
    pub sphincs_public: sphincs::PublicKey,
    pub sphincs_secret: sphincs::SecretKey,
}

/// Encrypted payload stored by the vault. Uses base64 strings to remain JSON-friendly.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EncryptedPayload {
    pub nonce_b64: String,
    pub ciphertext_b64: String,
    pub kem_ciphertext_b64: String,
    pub binding_tag_b64: String,
    pub signature_b64: String,
}

impl EncryptedPayload {
    fn decode_nonce(&self) -> Result<[u8; 12]> {
        let bytes = general_purpose::STANDARD
            .decode(&self.nonce_b64)
            .context("nonce decode")?;
        if bytes.len() != 12 {
            return Err(anyhow!("nonce must be 96 bits"));
        }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&bytes);
        Ok(nonce)
    }

    fn decode_ciphertext(&self) -> Result<Vec<u8>> {
        general_purpose::STANDARD
            .decode(&self.ciphertext_b64)
            .context("ciphertext decode")
    }

    fn decode_kem_ciphertext(&self) -> Result<kyber1024::Ciphertext> {
        let bytes = general_purpose::STANDARD
            .decode(&self.kem_ciphertext_b64)
            .context("kem ciphertext decode")?;
        kyber1024::Ciphertext::from_bytes(&bytes)
            .map_err(|e| anyhow!("kem ciphertext decode failed: {:?}", e))
    }

    fn decode_binding_tag(&self) -> Result<Vec<u8>> {
        general_purpose::STANDARD
            .decode(&self.binding_tag_b64)
            .context("binding tag decode")
    }

    fn decode_signature(&self) -> Result<Vec<u8>> {
        general_purpose::STANDARD
            .decode(&self.signature_b64)
            .context("signature decode")
    }
}

/// Generate fresh vault key material (AES-256 + Kyber + Sphincs+).
pub fn vault_keygen() -> Result<VaultKeyMaterial> {
    let mut aes_key = [0u8; 32];
    OsRng.fill_bytes(&mut aes_key);
    let (kyber_public, kyber_secret) = kyber1024::keypair();
    let (sphincs_public, sphincs_secret) = sphincs::keypair();

    Ok(VaultKeyMaterial {
        aes_key,
        kyber_public: kyber_public.as_bytes().to_vec(),
        kyber_secret: kyber_secret.as_bytes().to_vec(),
        sphincs_public: sphincs_public.as_bytes().to_vec(),
        sphincs_secret: sphincs_secret.as_bytes().to_vec(),
    })
}

/// Encrypt data using AES-256-GCM while deriving post-quantum binding material via Kyber KEM.
pub fn quantum_encrypt(
    keys: &VaultKeyMaterial,
    plaintext: &[u8],
    aad: &[u8],
) -> Result<EncryptedPayload> {
    let runtime = keys.runtime()?;
    let cipher = Aes256Gcm::new_from_slice(&runtime.aes_key)
        .map_err(|e| anyhow!("unable to init AES-GCM cipher: {e}"))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|e| anyhow!("aes-gcm encryption failure: {e}"))?;

    let (kem_ciphertext, shared_secret) = kyber1024::encapsulate(&runtime.kyber_public);

    // Bind ciphertext + shared secret to produce a tamper-evident tag.
    let mut hasher = Sha512::new();
    hasher.update(shared_secret.as_bytes());
    hasher.update(&nonce_bytes);
    hasher.update(aad);
    hasher.update(&ciphertext);
    let mut binding_tag = hasher.finalize().to_vec();

    let signature_bytes = hybrid_sign(keys, &binding_tag)?;

    let payload = EncryptedPayload {
        nonce_b64: general_purpose::STANDARD.encode(nonce_bytes),
        ciphertext_b64: general_purpose::STANDARD.encode(ciphertext),
        kem_ciphertext_b64: general_purpose::STANDARD.encode(kem_ciphertext.as_bytes()),
        binding_tag_b64: general_purpose::STANDARD.encode(&binding_tag),
        signature_b64: general_purpose::STANDARD.encode(&signature_bytes),
    };

    binding_tag.zeroize();

    Ok(payload)
}

/// Decrypt a payload produced by [`quantum_encrypt`].
pub fn quantum_decrypt(
    keys: &VaultKeyMaterial,
    payload: &EncryptedPayload,
    aad: &[u8],
) -> Result<Vec<u8>> {
    let runtime = keys.runtime()?;
    let nonce_bytes = payload.decode_nonce()?;
    let ciphertext = payload.decode_ciphertext()?;
    let kem_ciphertext = payload.decode_kem_ciphertext()?;
    let binding_tag = payload.decode_binding_tag()?;
    let signature = payload.decode_signature()?;

    hybrid_verify(keys, &binding_tag, &signature)?;

    let shared_secret = kyber1024::decapsulate(&kem_ciphertext, &runtime.kyber_secret);
    let mut hasher = Sha512::new();
    hasher.update(shared_secret.as_bytes());
    hasher.update(&nonce_bytes);
    hasher.update(aad);
    hasher.update(&ciphertext);
    let computed_binding = hasher.finalize().to_vec();

    if computed_binding != binding_tag {
        return Err(anyhow!(
            "binding tag mismatch; ciphertext integrity failure"
        ));
    }

    let cipher = Aes256Gcm::new_from_slice(&runtime.aes_key)
        .map_err(|e| anyhow!("unable to init AES-GCM cipher: {e}"))?;

    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: &ciphertext,
                aad,
            },
        )
        .map_err(|e| anyhow!("aes-gcm decryption failure: {e}"))?;

    Ok(plaintext)
}

/// Produce a detached post-quantum signature over a message using Sphincs+.
pub fn hybrid_sign(keys: &VaultKeyMaterial, message: &[u8]) -> Result<Vec<u8>> {
    let runtime = keys.runtime()?;
    let signature = sphincs::detached_sign(message, &runtime.sphincs_secret);
    Ok(signature.as_bytes().to_vec())
}

/// Verify a detached Sphincs+ signature.
pub fn hybrid_verify(keys: &VaultKeyMaterial, message: &[u8], signature: &[u8]) -> Result<()> {
    let runtime = keys.runtime()?;
    let sig = sphincs::DetachedSignature::from_bytes(signature)
        .map_err(|e| anyhow!("sphincs signature decode failed: {:?}", e))?;
    sphincs::verify_detached_signature(&sig, message, &runtime.sphincs_public)
        .map_err(|_| anyhow!("sphincs signature verification failed"))
}
