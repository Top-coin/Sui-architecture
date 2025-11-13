//! Core primitives shared across the simplified Sui architecture example.
//! The goal is to provide strongly typed building blocks that other crates
//! can compose when simulating validator and Nautilus behaviour.

pub mod object;
pub mod transaction;
pub mod messages;

pub use object::{ObjectData, ObjectID, Owner, SuiObject};
pub use transaction::{GasObject, SignedTransaction, TransactionDigest, TransactionKind, TransactionPayload};
pub use messages::{ConsensusMessage, CheckpointSummary, ExecutionRequest};

/// Helper used by examples and tests to fabricate a signed transaction without
/// implementing full cryptography.
pub fn mock_signed_transfer(from: &str, to: &str, object_id: &str) -> SignedTransaction {
    SignedTransaction::new_transfer(from.to_string(), to.to_string(), object_id.to_string())
}

