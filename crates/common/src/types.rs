/// A 32-byte content hash used throughout the Xet protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MerkleHash(pub [u8; 32]);

impl MerkleHash {
    pub fn from_slice(b: &[u8]) -> Option<Self> {
        b.try_into().ok().map(Self)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Convert a raw 32-byte hash to the 64-char hex string used in API paths.
///
/// Per spec: split into 4 groups of 8 bytes, treat each group as a
/// little-endian u64, format as 16-char lowercase hex, concatenate.
pub fn hash_to_api_string(hash: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for i in 0..4 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&hash[i * 8..(i + 1) * 8]);
        let val = u64::from_le_bytes(buf);
        out.push_str(&format!("{val:016x}"));
    }
    out
}

/// Decode a 64-char API-encoded hex string back to raw 32 bytes.
pub fn api_string_to_hash(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for i in 0..4 {
        let hex_chunk = &s[i * 16..(i + 1) * 16];
        let val = u64::from_str_radix(hex_chunk, 16).ok()?;
        let bytes = val.to_le_bytes();
        out[i * 8..(i + 1) * 8].copy_from_slice(&bytes);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_known_vector() {
        // From spec: [0,1,2,...,31] → "07060504030201000f0e0d0c0b0a0908..."
        let input: [u8; 32] = std::array::from_fn(|i| i as u8);
        let encoded = hash_to_api_string(&input);
        assert_eq!(&encoded[..16], "0706050403020100");
        assert_eq!(&encoded[16..32], "0f0e0d0c0b0a0908");
        let decoded = api_string_to_hash(&encoded).unwrap();
        assert_eq!(decoded, input);
    }
}
