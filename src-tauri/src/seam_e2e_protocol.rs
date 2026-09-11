pub const VERSION: u32 = 1;
pub const PLAINTEXT_CHUNK: usize = 256 * 1024;
pub const TAG_SIZE: usize = 16;
pub const NONCE_PREFIX_SIZE: usize = 4;
pub const NONCE_SIZE: usize = 12;

pub fn ciphertext_size(plaintext_size: u64) -> u64 {
    let chunks = if plaintext_size == 0 { 1 } else { (plaintext_size + PLAINTEXT_CHUNK as u64 - 1) / PLAINTEXT_CHUNK as u64 };
    plaintext_size + chunks * TAG_SIZE as u64
}

pub fn nonce(prefix: &[u8; NONCE_PREFIX_SIZE], index: u64) -> [u8; NONCE_SIZE] {
    let mut out = [0u8; NONCE_SIZE];
    out[..NONCE_PREFIX_SIZE].copy_from_slice(prefix);
    out[NONCE_PREFIX_SIZE..].copy_from_slice(&index.to_be_bytes());
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ciphertext_size_handles_zero_and_chunk_boundaries() {
        assert_eq!(ciphertext_size(0), 16);
        assert_eq!(ciphertext_size(1), 17);
        assert_eq!(ciphertext_size(PLAINTEXT_CHUNK as u64), PLAINTEXT_CHUNK as u64 + 16);
        assert_eq!(ciphertext_size(PLAINTEXT_CHUNK as u64 + 1), PLAINTEXT_CHUNK as u64 + 33);
    }

    #[test]
    fn nonce_uses_big_endian_chunk_index() {
        let prefix = [0x10, 0x20, 0x30, 0x40];
        assert_eq!(nonce(&prefix, 0), [0x10,0x20,0x30,0x40,0,0,0,0,0,0,0,0]);
        assert_eq!(nonce(&prefix, 2), [0x10,0x20,0x30,0x40,0,0,0,0,0,0,0,2]);
    }
}
