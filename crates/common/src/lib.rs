pub mod config;
pub mod error;
pub mod types;

pub use config::AppConfig;
pub use error::AppError;
pub use types::{MerkleHash, hash_to_api_string, api_string_to_hash};

#[cfg(test)]
mod hash_tests;
