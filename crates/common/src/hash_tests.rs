/// Hash encoding roundtrip and known-vector tests from the Xet spec.
#[cfg(test)]
mod hash_tests {
    use crate::types::{api_string_to_hash, hash_to_api_string};

    #[test]
    fn spec_known_vector() {
        // From spec: bytes [0..32] → "0706050403020100..."
        let input: [u8; 32] = std::array::from_fn(|i| i as u8);
        let s = hash_to_api_string(&input);
        assert_eq!(s.len(), 64);
        assert_eq!(&s[0..16], "0706050403020100");
        assert_eq!(&s[16..32], "0f0e0d0c0b0a0908");
        assert_eq!(&s[32..48], "1716151413121110");
        assert_eq!(&s[48..64], "1f1e1d1c1b1a1918");
    }

    #[test]
    fn roundtrip_identity() {
        for seed in 0u8..=255 {
            let hash: [u8; 32] = std::array::from_fn(|i| seed.wrapping_add(i as u8));
            let encoded = hash_to_api_string(&hash);
            let decoded = api_string_to_hash(&encoded).unwrap();
            assert_eq!(decoded, hash, "roundtrip failed for seed={seed}");
        }
    }

    #[test]
    fn all_zeros() {
        let h = [0u8; 32];
        let s = hash_to_api_string(&h);
        assert_eq!(s, "0".repeat(64));
    }

    #[test]
    fn all_ones() {
        let h = [0xFFu8; 32];
        let s = hash_to_api_string(&h);
        assert_eq!(s, "ffffffffffffffff".repeat(4));
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(api_string_to_hash("tooshort").is_none());
        assert!(api_string_to_hash(&"a".repeat(63)).is_none());
        assert!(api_string_to_hash(&"a".repeat(65)).is_none());
    }
}
