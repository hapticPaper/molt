//! Cryptographic primitives for `HardClaw` protocol.
//!
//! Uses audited, production-grade crates:
//! - ed25519-dalek for signatures (same as Solana)
//! - BLAKE3 for fast hashing
//! - SHA3-256 for commitment schemes
//! - bip39 for standard mnemonic seed phrases

mod hash;
mod signature;
mod commitment;
mod mnemonic;

pub use hash::{Hash, Hasher, hash_data, merkle_root};
pub use signature::{Keypair, PublicKey, SecretKey, Signature, sign, verify};
pub use commitment::{Commitment, CommitReveal};
pub use mnemonic::{
    generate_mnemonic, parse_mnemonic, keypair_from_mnemonic,
    keypair_from_phrase, mnemonic_to_words, MNEMONIC_WORD_COUNT,
};

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
