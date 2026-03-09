//! Builds minimal shards for the Global Deduplication API response.
//!
//! A dedup response shard has:
//!   - Header  (footer_size = 200)
//!   - Empty File Info section (just a bookend)
//!   - CAS Info section with the relevant xorbs
//!   - Footer  (HMAC key = zeros → no protection for simplicity)

use crate::types::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_u32_le(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_u64_le(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_bookend(out: &mut Vec<u8>) {
    out.extend_from_slice(&BOOKEND_HASH); // 32 × 0xFF
    out.extend_from_slice(&[0u8; 16]); // 16 × 0x00
}

/// Build a minimal deduplication-response shard for the given xorbs.
///
/// The returned bytes include the footer and are safe to return directly as
/// the body of `GET /v1/chunks/default-merkledb/{hash}`.
pub fn build_dedup_response_shard(xorbs: &[&ParsedXorb]) -> Vec<u8> {
    let mut out = Vec::new();

    // ── Placeholder for header (filled in at the end) ────────────────────────
    let header_offset = 0usize;
    out.extend_from_slice(&[0u8; 48]); // header placeholder

    // ── File Info Section: empty (just bookend) ───────────────────────────────
    let file_info_offset = out.len() as u64;
    write_bookend(&mut out);

    // ── CAS Info Section ──────────────────────────────────────────────────────
    let cas_info_offset = out.len() as u64;
    for xorb in xorbs {
        // CASChunkSequenceHeader (48 bytes)
        out.extend_from_slice(&xorb.xorb_hash);
        write_u32_le(&mut out, 0); // cas_flags
        write_u32_le(&mut out, xorb.chunks.len() as u32);
        write_u32_le(&mut out, xorb.num_bytes_in_cas);
        write_u32_le(&mut out, xorb.num_bytes_on_disk);

        // CASChunkSequenceEntry × n (each 48 bytes)
        for chunk in &xorb.chunks {
            out.extend_from_slice(&chunk.chunk_hash);
            write_u32_le(&mut out, chunk.chunk_byte_range_start);
            write_u32_le(&mut out, chunk.unpacked_segment_bytes);
            out.extend_from_slice(&[0u8; 8]); // _unused
        }
    }
    write_bookend(&mut out);

    // ── Footer (200 bytes) ───────────────────────────────────────────────────
    let footer_offset = out.len() as u64;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expiry = now + 7 * 24 * 3600; // 7 days

    write_u64_le(&mut out, MDB_SHARD_FOOTER_VERSION); // version
    write_u64_le(&mut out, file_info_offset); // file_info_offset
    write_u64_le(&mut out, cas_info_offset); // cas_info_offset
    out.extend_from_slice(&[0u8; 48]); // _buffer
    out.extend_from_slice(&[0u8; 32]); // chunk_hash_hmac_key (zeros = no HMAC)
    write_u64_le(&mut out, now); // shard_creation_timestamp
    write_u64_le(&mut out, expiry); // shard_key_expiry
    out.extend_from_slice(&[0u8; 72]); // _buffer2
    write_u64_le(&mut out, footer_offset); // footer_offset
                                           // Footer is exactly 200 bytes: 8+8+8+48+32+8+8+72+8 = 200 ✓

    // ── Back-fill header ─────────────────────────────────────────────────────
    out[header_offset..header_offset + 32].copy_from_slice(&MDB_SHARD_HEADER_TAG);
    let version_bytes = MDB_SHARD_HEADER_VERSION.to_le_bytes();
    out[header_offset + 32..header_offset + 40].copy_from_slice(&version_bytes);
    let footer_size_bytes = MDB_SHARD_FOOTER_SIZE.to_le_bytes();
    out[header_offset + 40..header_offset + 48].copy_from_slice(&footer_size_bytes);

    out
}
