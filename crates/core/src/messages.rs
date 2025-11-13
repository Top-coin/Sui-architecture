use crate::transaction::{SignedTransaction, TransactionDigest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub tx: SignedTransaction,
    pub digest: TransactionDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    SubmitTransaction(ExecutionRequest),
    Vote { digest: TransactionDigest, validator: String },
    Certified { digest: TransactionDigest },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointSummary {
    pub sequence_number: u64,
    pub transaction_count: usize,
    pub root_digest: String,
}

