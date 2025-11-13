use aws_nautilus_sdk::{EnclaveInfo, NautilusClient};
use nautilus_enclave::handle_request;
use nautilus_shared::EnclaveRequest;
use rand::Rng;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = NautilusClient::connect().await?;
    let enclave_id = client.create_enclave(&EnclaveInfo {
        name: "demo".into(),
        cpu_cores: 2,
        memory_mb: 2048,
    }).await?;
    let attestation = client.attest(&enclave_id).await?;

    let payload = serde_json::json!({
        "action": "process_effects",
        "digest": "example",
    })
    .to_string();

    let request = EnclaveRequest {
        nonce: rand::thread_rng().gen(),
        payload,
    };

    let response = handle_request(&attestation, request);
    println!("Host received response: {}", response.message);

    Ok(())
}

