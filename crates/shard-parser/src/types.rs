/// MDB Shard binary format constants.
///
/// Tag bytes: b"HFRepoMetaData0" + 17 random bytes
/// (from xet-core/mdb_shard/src/shard_format.rs)
pub const MDB_SHARD_HEADER_TAG: [u8; 32] = [
    b'H', b'F', b'R', b'e', b'p', b'o', b'M', b'e', b't', b'a', b'D', b'a', b't', b'a', 0, 85, 105,
    103, 69, 106, 123, 129, 87, 131, 165, 189, 217, 92, 205, 209, 74, 169,
];

pub const MDB_SHARD_HEADER_VERSION: u64 = 2;
pub const MDB_SHARD_FOOTER_VERSION: u64 = 1;
pub const MDB_SHARD_FOOTER_SIZE: u64 = 200;

// File flags
pub const MDB_FILE_FLAG_WITH_VERIFICATION: u32 = 0x8000_0000;
pub const MDB_FILE_FLAG_WITH_METADATA_EXT: u32 = 0x4000_0000;

// Bookend marker: first 32 bytes all 0xFF, last 16 bytes all 0x00
pub const BOOKEND_HASH: [u8; 32] = [0xFF; 32];

/// All data extracted from a parsed shard upload (no footer).
#[derive(Debug, Default)]
pub struct ParsedShard {
    /// Files listed in the File Info section.
    pub files: Vec<ParsedFile>,
    /// Xorbs listed in the CAS Info section.
    pub xorbs: Vec<ParsedXorb>,
}

/// A single file's reconstruction info from the File Info section.
#[derive(Debug)]
pub struct ParsedFile {
    /// 32-byte file hash.
    pub file_hash: [u8; 32],
    /// SHA-256 of file contents (from FileMetadataExt).
    pub sha256: Option<[u8; 32]>,
    /// Ordered list of reconstruction terms.
    pub terms: Vec<FileReconstructionTerm>,
}

/// A single reconstruction term (one entry from FileDataSequenceEntry).
#[derive(Debug, Clone)]
pub struct FileReconstructionTerm {
    /// 32-byte xorb hash.
    pub xorb_hash: [u8; 32],
    pub chunk_index_start: u32,
    pub chunk_index_end: u32,
    pub unpacked_length: u32,
}

/// A xorb's chunk metadata from the CAS Info section.
#[derive(Debug)]
pub struct ParsedXorb {
    /// 32-byte xorb hash.
    pub xorb_hash: [u8; 32],
    pub num_bytes_in_cas: u32,
    pub num_bytes_on_disk: u32,
    pub chunks: Vec<ChunkInXorb>,
}

/// A single chunk's metadata within a xorb.
#[derive(Debug, Clone)]
pub struct ChunkInXorb {
    /// 32-byte chunk hash.
    pub chunk_hash: [u8; 32],
    pub chunk_byte_range_start: u32,
    pub unpacked_segment_bytes: u32,
}
