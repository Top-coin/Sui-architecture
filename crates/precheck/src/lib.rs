use sui_core::{messages::ExecutionRequest, transaction::TransactionKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PreCheckError {
    #[error("gas budget must be positive")]
    InvalidGasBudget,
    #[error("transfer recipient missing")]
    MissingRecipient,
    #[error("move call is missing target module or function")]
    InvalidCall,
}

#[derive(Debug, Clone)]
pub struct PreCheckReport {
    pub is_move_call: bool,
    pub requires_shared_lock: bool,
}

#[derive(Default)]
pub struct PreCheckPipeline;

impl PreCheckPipeline {
    pub fn run(&self, request: &ExecutionRequest) -> Result<PreCheckReport, PreCheckError> {
        let payload = &request.tx.payload;

        if payload.gas_budget == 0 {
            return Err(PreCheckError::InvalidGasBudget);
        }

        let (is_move_call, requires_shared_lock) = match &payload.kind {
            TransactionKind::Transfer { recipient, .. } => {
                if recipient.trim().is_empty() {
                    return Err(PreCheckError::MissingRecipient);
                }
                (false, false)
            }
            TransactionKind::Call { module, function, .. } => {
                if module.is_empty() || function.is_empty() {
                    return Err(PreCheckError::InvalidCall);
                }
                (true, true)
            }
        };

        Ok(PreCheckReport { is_move_call, requires_shared_lock })
    }
}

