use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};
use hkdf::Hkdf;
use rand_core::OsRng;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

pub struct E2eKeyPair {
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
}

pub fn generate_key_pair() -> E2eKeyPair {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    E2eKeyPair { private_key: secret.to_bytes(), public_key: public.to_bytes() }
}

pub fn derive_shared_key(private_key: &[u8; 32], peer_public_key: &[u8; 32]) -> Result<[u8; 32], String> {
    let secret = StaticSecret::from(*private_key);
    let peer = PublicKey::from(*peer_public_key);
    let shared = secret.diffie_hellman(&peer);
    if shared.as_bytes().iter().all(|b| *b == 0) { return Err("invalid peer public key".into()); }
    let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
    let mut key = [0u8; 32];
    hk.expand(b"SEAM-Share-E2E-v1", &mut key).map_err(|_| "HKDF failed".to_string())?;
    Ok(key)
}

pub fn encrypt(key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    ChaCha20Poly1305::new(key.into()).encrypt(Nonce::from_slice(nonce), chacha20poly1305::aead::Payload { msg: plaintext, aad }).map_err(|_| "encryption failed".into())
}

pub fn decrypt(key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    ChaCha20Poly1305::new(key.into()).decrypt(Nonce::from_slice(nonce), chacha20poly1305::aead::Payload { msg: ciphertext, aad }).map_err(|_| "authentication failed".into())
}
