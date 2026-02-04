//! Cryptographic primitives for `HardClaw` protocol.
//!
//! Uses audited, production-grade crates:
//! - ed25519-dalek for signatures (same as Solana)
//! - BLAKE3 for fast hashing
//! - SHA3-256 for commitment schemes
//! - bip39 for standard mnemonic seed phrases

mod commitment;
mod hash;
mod mnemonic;
mod signature;

pub use commitment::{CommitReveal, Commitment};
pub use hash::{hash_data, merkle_root, Hash, Hasher};
pub use mnemonic::{
    generate_mnemonic, keypair_from_mnemonic, keypair_from_phrase, mnemonic_to_words,
    parse_mnemonic, MNEMONIC_WORD_COUNT,
};
pub use signature::{sign, verify, Keypair, PublicKey, SecretKey, Signature};

use thiserror::Error;

/// Cryptographic errors
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Invalid signature
    #[error("invalid signature")]
    InvalidSignature,
    /// Invalid public key format
    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),
    /// Invalid hash format
    #[error("invalid hash: {0}")]
    InvalidHash(String),
    /// Commitment verification failed
    #[error("commitment verification failed")]
    CommitmentMismatch,
    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(String),
    /// Invalid mnemonic phrase
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),
}

/// Result type for crypto operations
pub type CryptoResult<T> = Result<T, CryptoError>;
