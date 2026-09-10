use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

pub const PROTOCOL: &str = "SEAM-Share-E2E-v1";

pub struct KeyPair {
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
}

pub fn generate_keypair() -> KeyPair {
    let secret = StaticSecret::random_from_rng(rand_core::OsRng);
    let public = PublicKey::from(&secret);
    KeyPair { private_key: secret.to_bytes(), public_key: public.to_bytes() }
}

pub fn derive_shared_key(private_key: &[u8; 32], peer_public_key: &[u8; 32]) -> [u8; 32] {
    let secret = StaticSecret::from(*private_key);
    let peer = PublicKey::from(*peer_public_key);
    let shared = secret.diffie_hellman(&peer);
    let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
    let mut out = [0u8; 32];
    hk.expand(PROTOCOL.as_bytes(), &mut out).expect("HKDF output size is valid");
    out
}

pub fn encrypt(key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    ChaCha20Poly1305::new(key.into())
        .encrypt(Nonce::from_slice(nonce), chacha20poly1305::aead::Payload { msg: plaintext, aad })
        .map_err(|_| "E2E encryption failed".to_string())
}

pub fn decrypt(key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    ChaCha20Poly1305::new(key.into())
        .decrypt(Nonce::from_slice(nonce), chacha20poly1305::aead::Payload { msg: ciphertext, aad })
        .map_err(|_| "E2E authentication failed".to_string())
}
