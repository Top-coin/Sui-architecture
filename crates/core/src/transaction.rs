use crate::object::ObjectID;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionKind {
    Transfer {
        object: ObjectID,
        recipient: String,
    },
    Call {
        package: ObjectID,
        module: String,
        function: String,
        arguments: Vec<serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPayload {
    pub kind: TransactionKind,
    pub gas_budget: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub signer: String,
    pub payload: TransactionPayload,
    pub signature: String,
}

impl SignedTransaction {
    pub fn new_transfer(sender: String, recipient: String, object: String) -> Self {
        let payload = TransactionPayload {
            kind: TransactionKind::Transfer {
                object: ObjectID(object),
                recipient,
            },
            gas_budget: 1_000,
        };
        Self::new(sender, payload)
    }

    pub fn new(sender: String, payload: TransactionPayload) -> Self {
        let signature = format!(
            "mock-signature-{:x}",
            rand::thread_rng().gen::<u64>()
        );
        Self { signer: sender, payload, signature }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasObject {
    pub id: ObjectID,
    pub balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TransactionDigest(pub String);

impl TransactionDigest {
    pub fn random() -> Self {
        Self(format!("tx-{:x}", rand::thread_rng().gen::<u128>()))
    }
}

