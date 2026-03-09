#[cfg(test)]
mod tests {
    use shard_parser::{
        build_dedup_response_shard, parse_shard,
        types::{
            ChunkInXorb, MDB_SHARD_HEADER_TAG, MDB_SHARD_HEADER_VERSION,
            MDB_FILE_FLAG_WITH_METADATA_EXT, MDB_FILE_FLAG_WITH_VERIFICATION,
            BOOKEND_HASH,
        },
    };

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn make_hash(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    fn write_u32_le(buf: &mut Vec<u8>, v: u32) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_u64_le(buf: &mut Vec<u8>, v: u64) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_bookend(buf: &mut Vec<u8>) {
        buf.extend_from_slice(&BOOKEND_HASH);
        buf.extend_from_slice(&[0u8; 16]);
    }

    /// Build a minimal valid shard (as sent by the upload API — no footer).
    fn build_test_shard(
        file_hash: [u8; 32],
        file_sha256: [u8; 32],
        xorb_hash: [u8; 32],
        chunk_hash: [u8; 32],
    ) -> Vec<u8> {
        let mut buf = Vec::new();

        // ── Header (48 bytes) ────────────────────────────────────────────────
        buf.extend_from_slice(&MDB_SHARD_HEADER_TAG);
        write_u64_le(&mut buf, MDB_SHARD_HEADER_VERSION);
        write_u64_le(&mut buf, 0); // footer_size = 0 (no footer)

        // ── File Info Section ────────────────────────────────────────────────
        // FileDataSequenceHeader
        buf.extend_from_slice(&file_hash);
        let flags = MDB_FILE_FLAG_WITH_VERIFICATION | MDB_FILE_FLAG_WITH_METADATA_EXT;
        write_u32_le(&mut buf, flags);
        write_u32_le(&mut buf, 1); // num_entries = 1
        buf.extend_from_slice(&[0u8; 8]); // _unused

        // FileDataSequenceEntry (1 term)
        buf.extend_from_slice(&xorb_hash); // cas_hash
        write_u32_le(&mut buf, 0);          // cas_flags
        write_u32_le(&mut buf, 65536);      // unpacked_segment_bytes
        write_u32_le(&mut buf, 0);          // chunk_index_start
        write_u32_le(&mut buf, 1);          // chunk_index_end

        // FileVerificationEntry (1 entry matching num_entries)
        buf.extend_from_slice(&make_hash(0xAA)); // range_hash
        buf.extend_from_slice(&[0u8; 16]);

        // FileMetadataExt
        buf.extend_from_slice(&file_sha256);
        buf.extend_from_slice(&[0u8; 16]);

        // File Info bookend
        write_bookend(&mut buf);

        // ── CAS Info Section ─────────────────────────────────────────────────
        // CASChunkSequenceHeader
        buf.extend_from_slice(&xorb_hash);
        write_u32_le(&mut buf, 0);      // cas_flags
        write_u32_le(&mut buf, 1);      // num_entries
        write_u32_le(&mut buf, 65536);  // num_bytes_in_cas
        write_u32_le(&mut buf, 66000);  // num_bytes_on_disk

        // CASChunkSequenceEntry
        buf.extend_from_slice(&chunk_hash);
        write_u32_le(&mut buf, 0);      // chunk_byte_range_start
        write_u32_le(&mut buf, 65536);  // unpacked_segment_bytes
        buf.extend_from_slice(&[0u8; 8]); // _unused

        // CAS Info bookend
        write_bookend(&mut buf);

        buf
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    #[test]
    fn parse_minimal_shard() {
        let file_hash  = make_hash(0x01);
        let sha256     = make_hash(0x02);
        let xorb_hash  = make_hash(0x03);
        let chunk_hash = make_hash(0x04);

        let data = build_test_shard(file_hash, sha256, xorb_hash, chunk_hash);
        let shard = parse_shard(&data).expect("parse failed");

        // File Info
        assert_eq!(shard.files.len(), 1);
        let f = &shard.files[0];
        assert_eq!(f.file_hash, file_hash);
        assert_eq!(f.sha256, Some(sha256));
        assert_eq!(f.terms.len(), 1);
        assert_eq!(f.terms[0].xorb_hash, xorb_hash);
        assert_eq!(f.terms[0].chunk_index_start, 0);
        assert_eq!(f.terms[0].chunk_index_end, 1);
        assert_eq!(f.terms[0].unpacked_length, 65536);

        // CAS Info
        assert_eq!(shard.xorbs.len(), 1);
        let x = &shard.xorbs[0];
        assert_eq!(x.xorb_hash, xorb_hash);
        assert_eq!(x.chunks.len(), 1);
        assert_eq!(x.chunks[0].chunk_hash, chunk_hash);
        assert_eq!(x.chunks[0].chunk_byte_range_start, 0);
        assert_eq!(x.chunks[0].unpacked_segment_bytes, 65536);
    }

    #[test]
    fn parse_rejects_bad_tag() {
        let mut data = build_test_shard(make_hash(1), make_hash(2), make_hash(3), make_hash(4));
        data[0] = 0xFF; // corrupt the tag
        let result = parse_shard(&data);
        assert!(result.is_err(), "expected BadTag error");
    }

    #[test]
    fn parse_rejects_bad_version() {
        let mut data = build_test_shard(make_hash(1), make_hash(2), make_hash(3), make_hash(4));
        // Version is at bytes [32..40]
        let bad_version: u64 = 99;
        data[32..40].copy_from_slice(&bad_version.to_le_bytes());
        let result = parse_shard(&data);
        assert!(result.is_err(), "expected BadVersion error");
    }

    #[test]
    fn parse_empty_shard() {
        // Shard with no files and no xorbs (just bookends)
        let mut buf = Vec::new();
        buf.extend_from_slice(&MDB_SHARD_HEADER_TAG);
        let mut tmp = [0u8; 16];
        tmp[..8].copy_from_slice(&MDB_SHARD_HEADER_VERSION.to_le_bytes());
        buf.extend_from_slice(&tmp);
        write_bookend(&mut buf); // file info bookend
        write_bookend(&mut buf); // cas info bookend

        let shard = parse_shard(&buf).expect("parse failed");
        assert!(shard.files.is_empty());
        assert!(shard.xorbs.is_empty());
    }

    #[test]
    fn dedup_shard_roundtrip_size() {
        use shard_parser::ParsedXorb;
        let xorb = ParsedXorb {
            xorb_hash: make_hash(0xAB),
            num_bytes_in_cas: 65536,
            num_bytes_on_disk: 66000,
            chunks: vec![
                ChunkInXorb { chunk_hash: make_hash(0x10), chunk_byte_range_start: 0,     unpacked_segment_bytes: 32768 },
                ChunkInXorb { chunk_hash: make_hash(0x11), chunk_byte_range_start: 32768, unpacked_segment_bytes: 32768 },
            ],
        };

        let shard_bytes = build_dedup_response_shard(&[&xorb]);

        // Must end with a 200-byte footer
        assert!(shard_bytes.len() >= 200 + 48, "shard too small");
        // Footer version is at footer_offset
        let footer_offset = u64::from_le_bytes(
            shard_bytes[shard_bytes.len() - 8..].try_into().unwrap()
        ) as usize;
        let footer_version = u64::from_le_bytes(
            shard_bytes[footer_offset..footer_offset + 8].try_into().unwrap()
        );
        assert_eq!(footer_version, 1, "footer version mismatch");
    }
}
