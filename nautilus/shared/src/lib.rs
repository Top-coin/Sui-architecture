use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EnclaveRequest {
    pub nonce: u64,
    pub payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnclaveResponse {
    pub accepted: bool,
    pub message: String,
}

