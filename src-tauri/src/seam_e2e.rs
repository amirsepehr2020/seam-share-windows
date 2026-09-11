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

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(value: &str) -> Vec<u8> {
        (0..value.len()).step_by(2).map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap()).collect()
    }

    #[test]
    fn cross_platform_vector_matches() {
        let sender_private: [u8; 32] = (1u8..=32).collect::<Vec<_>>().try_into().unwrap();
        let receiver_private: [u8; 32] = (101u8..=132).collect::<Vec<_>>().try_into().unwrap();
        let sender_public = PublicKey::from(&StaticSecret::from(sender_private)).to_bytes();
        assert_eq!(sender_public.to_vec(), hex("07a37cbc142093c8b755dc1b10e86cb426374ad16aa853ed0bdfc0b2b86d1c7c"));

        let receiver_public = PublicKey::from(&StaticSecret::from(receiver_private)).to_bytes();
        assert_eq!(receiver_public.to_vec(), hex("5714769d116bf76436ae74bc793d2c30ad1903c59ac5273805c7e2698b410c36"));

        let key = derive_shared_key(&sender_private, &receiver_public);
        assert_eq!(key.to_vec(), hex("18d1c47d0296028a6dae78c8f723af54b80f1b9d6a4b7362f6ec34dfe3f979ce"));

        let nonce: [u8; 12] = hex("102030400000000000000002").try_into().unwrap();
        let aad = hex("00000001000000000000000200000024766563746f722d7472616e736665722d3031");
        let ciphertext = hex("bfb9465ab07e48e9acbc450e8e471a8720bddf5193e5b68f9ac09cc0a373f3d943d4ac634d71cf7d6daeac546177e81ebda939f5");
        let plaintext = b"SEAM Share cross-platform E2E vector";

        assert_eq!(encrypt(&key, &nonce, plaintext, &aad).unwrap(), ciphertext);
        assert_eq!(decrypt(&key, &nonce, &ciphertext, &aad).unwrap(), plaintext);
    }

    #[test]
    fn tampering_is_rejected() {
        let key = [7u8; 32];
        let nonce = [8u8; 12];
        let aad = b"transfer-id";
        let mut ciphertext = encrypt(&key, &nonce, b"hello", aad).unwrap();
        ciphertext[0] ^= 1;
        assert!(decrypt(&key, &nonce, &ciphertext, aad).is_err());
    }
}
