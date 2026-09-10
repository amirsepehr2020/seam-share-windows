pub const VERSION: u32 = 1;
pub const PLAINTEXT_CHUNK: usize = 256 * 1024;
pub const TAG_SIZE: usize = 16;
pub const NONCE_PREFIX_SIZE: usize = 4;
pub const NONCE_SIZE: usize = 12;

pub fn ciphertext_size(plaintext_size: u64) -> u64 {
    assert!(plaintext_size >= 0);
    let chunks = if plaintext_size == 0 { 1 } else { (plaintext_size + PLAINTEXT_CHUNK as u64 - 1) / PLAINTEXT_CHUNK as u64 };
    plaintext_size + chunks * TAG_SIZE as u64
}

pub fn aad(transfer_id: &str, index: u64, plaintext_len: usize) -> Vec<u8> {
    let id = transfer_id.as_bytes();
    let mut out = Vec::with_capacity(4 + 8 + 4 + id.len());
    out.extend_from_slice(&VERSION.to_be_bytes());
    out.extend_from_slice(&index.to_be_bytes());
    out.extend_from_slice(&(plaintext_len as u32).to_be_bytes());
    out.extend_from_slice(id);
    out
}
