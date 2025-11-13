use nautilus_shared::{EnclaveRequest, EnclaveResponse};

pub fn handle_request(attestation_token: &str, request: EnclaveRequest) -> EnclaveResponse {
    if attestation_token.is_empty() {
        return EnclaveResponse {
            accepted: false,
            message: "Missing attestation".into(),
        };
    }

    let message = match serde_json::from_str::<serde_json::Value>(&request.payload) {
        Ok(parsed) => format!("Enclave confirmed action: {}", parsed.get("action").and_then(|a| a.as_str()).unwrap_or("unknown")),
        Err(_) => "Unable to read payload".into(),
    };

    EnclaveResponse {
        accepted: true,
        message,
    }
}

