//! Binary MDB shard parser.
//!
//! Supports shards without a footer (as sent by clients on upload).
//! Format: Header(48) | FileInfoSection | CASInfoSection | [Footer(200)]

use crate::types::*;

#[derive(Debug, thiserror::Error)]
pub enum ShardParseError {
    #[error("Shard too short: need {need} bytes, have {have}")]
    TooShort { need: usize, have: usize },
    #[error("Invalid header tag: {0:?}")]
    BadTag([u8; 32]),
    #[error("Unsupported header version: {0}")]
    BadVersion(u64),
    #[error("Corrupt shard: {0}")]
    Corrupt(String),
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], ShardParseError> {
        if self.remaining() < n {
            return Err(ShardParseError::TooShort {
                need: n,
                have: self.remaining(),
            });
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn read_hash(&mut self) -> Result<[u8; 32], ShardParseError> {
        let b = self.read_bytes(32)?;
        Ok(b.try_into().unwrap())
    }

    fn read_u32_le(&mut self) -> Result<u32, ShardParseError> {
        let b = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_u64_le(&mut self) -> Result<u64, ShardParseError> {
        let b = self.read_bytes(8)?;
        Ok(u64::from_le_bytes(b.try_into().unwrap()))
    }

    fn skip(&mut self, n: usize) -> Result<(), ShardParseError> {
        self.read_bytes(n)?;
        Ok(())
    }

    fn is_bookend(&mut self) -> Result<bool, ShardParseError> {
        if self.remaining() < 48 {
            return Err(ShardParseError::Corrupt(
                "expected bookend, not enough bytes".into(),
            ));
        }
        let hash: [u8; 32] = self.data[self.pos..self.pos + 32].try_into().unwrap();
        Ok(hash == BOOKEND_HASH)
    }
}

/// Parse an MDB shard from raw bytes.
///
/// Accepts shards both with and without the optional footer (footer_size == 0
/// means no footer, as required by the shard upload API).
pub fn parse_shard(data: &[u8]) -> Result<ParsedShard, ShardParseError> {
    let mut cur = Cursor::new(data);

    // ── Header (48 bytes) ────────────────────────────────────────────────────
    let tag = cur.read_hash()?;
    if tag != MDB_SHARD_HEADER_TAG {
        return Err(ShardParseError::BadTag(tag));
    }
    let version = cur.read_u64_le()?;
    if version != MDB_SHARD_HEADER_VERSION {
        return Err(ShardParseError::BadVersion(version));
    }
    let _footer_size = cur.read_u64_le()?; // 0 = no footer (upload case)

    // ── File Info Section ────────────────────────────────────────────────────
    let mut shard = ParsedShard::default();

    loop {
        if cur.is_bookend()? {
            cur.skip(48)?; // consume the 32-byte bookend hash + 16 zeros
            break;
        }

        // FileDataSequenceHeader (48 bytes)
        let file_hash = cur.read_hash()?;
        let file_flags = cur.read_u32_le()?;
        let num_entries = cur.read_u32_le()?;
        cur.skip(8)?; // _unused

        // FileDataSequenceEntry × num_entries (each 48 bytes)
        let mut terms = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            let xorb_hash = cur.read_hash()?;
            let _cas_flags = cur.read_u32_le()?;
            let unpacked_length = cur.read_u32_le()?;
            let chunk_index_start = cur.read_u32_le()?;
            let chunk_index_end = cur.read_u32_le()?;
            terms.push(FileReconstructionTerm {
                xorb_hash,
                chunk_index_start,
                chunk_index_end,
                unpacked_length,
            });
        }

        // FileVerificationEntry × num_entries (each 48 bytes) — read and discard
        if file_flags & MDB_FILE_FLAG_WITH_VERIFICATION != 0 {
            for _ in 0..num_entries {
                cur.skip(48)?; // range_hash[32] + _unused[16]
            }
        }

        // FileMetadataExt × 1 (48 bytes) — sha256[32] + _unused[16]
        let mut sha256: Option<[u8; 32]> = None;
        if file_flags & MDB_FILE_FLAG_WITH_METADATA_EXT != 0 {
            sha256 = Some(cur.read_hash()?);
            cur.skip(16)?; // _unused
        }

        shard.files.push(ParsedFile {
            file_hash,
            sha256,
            terms,
        });
    }

    // ── CAS Info Section ─────────────────────────────────────────────────────
    loop {
        if cur.is_bookend()? {
            cur.skip(48)?;
            break;
        }

        // CASChunkSequenceHeader (48 bytes)
        let xorb_hash = cur.read_hash()?;
        let _cas_flags = cur.read_u32_le()?;
        let num_entries = cur.read_u32_le()?;
        let num_bytes_in_cas = cur.read_u32_le()?;
        let num_bytes_on_disk = cur.read_u32_le()?;

        // CASChunkSequenceEntry × num_entries (each 48 bytes)
        let mut chunks = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            let chunk_hash = cur.read_hash()?;
            let chunk_byte_range_start = cur.read_u32_le()?;
            let unpacked_segment_bytes = cur.read_u32_le()?;
            cur.skip(8)?; // _unused
            chunks.push(ChunkInXorb {
                chunk_hash,
                chunk_byte_range_start,
                unpacked_segment_bytes,
            });
        }

        shard.xorbs.push(ParsedXorb {
            xorb_hash,
            num_bytes_in_cas,
            num_bytes_on_disk,
            chunks,
        });
    }

    // Footer (if present) — ignore for our purposes.

    Ok(shard)
}
