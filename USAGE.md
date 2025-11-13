# How to Use the Sui Architecture Project

This guide shows you how to use all the components of the Sui architecture project.

## 🚀 Quick Start

### 1. Build the Project

```bash
cd sui-architecture
cargo build
```

### 2. Run the Validator Demo

```bash
cargo run --example validator_demo -p sui-validator
```

This will:
- Create a validator node
- Process a sample transaction
- Show checkpoint information
- Demonstrate storage operations

## 📝 Basic Usage Examples

### Example 1: Creating a Validator Node

```rust
use sui_validator::ValidatorNode;
use sui_storage::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize storage layers
    let object_store: Arc<dyn ObjectStore> = Arc::new(InMemoryObjectStore::new());
    let effects_store: Arc<dyn EffectsStore> = Arc::new(InMemoryEffectsStore::new());
    let checkpoint_store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());

    // Create validator
    let validator = ValidatorNode::new(
        "my-validator",
        object_store,
        effects_store,
        checkpoint_store,
    ).await?;

    println!("Validator created: {}", validator.name());
    Ok(())
}
```

### Example 2: Processing a Transaction

```rust
use sui_core::{messages::ExecutionRequest, transaction::TransactionDigest, mock_signed_transfer};

// Create a transfer transaction
let tx = mock_signed_transfer("alice", "bob", "coin-123");
let request = ExecutionRequest {
    tx,
    digest: TransactionDigest::random(),
};

// Process it
let effects = validator.handle_transaction(request).await?;

println!("Transaction processed!");
println!("Created {} objects", effects.created.len());
println!("Events: {:?}", effects.events);
```

### Example 3: Starting a Network Server

```rust
// Start HTTP server on port 8080
let validator_clone = validator.clone();
tokio::spawn(async move {
    validator_clone.start_network_server(8080).await.unwrap();
});

println!("Server running on http://localhost:8080");
```

### Example 4: Submitting Transaction via Network

```rust
use sui_network::NetworkClient;
use sui_core::{messages::ExecutionRequest, transaction::TransactionDigest, mock_signed_transfer};

// Create client
let client = NetworkClient::new("http://localhost:8080");

// Create transaction
let tx = mock_signed_transfer("alice", "bob", "coin-456");
let request = ExecutionRequest {
    tx,
    digest: TransactionDigest::random(),
};

// Submit via network
let response = client.submit_transaction(request).await?;

if response.accepted {
    println!("Transaction accepted: {}", response.message);
} else {
    println!("Transaction rejected: {}", response.message);
}
```

### Example 5: Querying Objects

```rust
// Query object via network
let response = client.get_object("coin-123").await?;

if response.found {
    println!("Object found: {:?}", response.object);
} else {
    println!("Object not found");
}
```

### Example 6: Working with Storage Directly

```rust
use sui_storage::*;
use sui_core::object::{SuiObject, ObjectID, Owner, ObjectData};

// Create an object
let object = SuiObject::new(
    ObjectID::new("my-object"),
    Owner::Address("alice".to_string()),
    ObjectData::Coin { balance: 1000 },
);

// Store it
object_store.put_object(object.clone()).await?;

// Retrieve it
let retrieved = object_store.get_object("my-object").await?;
println!("Retrieved: {:?}", retrieved);

// List all objects owned by alice
let alice_objects = object_store.list_objects(Some("alice")).await?;
println!("Alice has {} objects", alice_objects.len());
```

### Example 7: Creating Move Function Calls

```rust
use sui_core::{
    messages::ExecutionRequest,
    transaction::{SignedTransaction, TransactionPayload, TransactionKind, TransactionDigest},
    object::ObjectID,
};
use serde_json::json;

// Create a Move function call
let payload = TransactionPayload {
    kind: TransactionKind::Call {
        package: ObjectID::new("package-123"),
        module: "coin".to_string(),
        function: "transfer".to_string(),
        arguments: vec![
            json!("coin-object-id"),
            json!("recipient-address"),
            json!(1000u64),
        ],
    },
    gas_budget: 5000,
};

let tx = SignedTransaction::new("sender-address".to_string(), payload);
let request = ExecutionRequest {
    tx,
    digest: TransactionDigest::random(),
};

// Execute it
let effects = validator.handle_transaction(request).await?;
```

### Example 8: Using Nautilus Host

```rust
use aws_nautilus_sdk::{EnclaveInfo, NautilusClient};
use nautilus_enclave::handle_request;
use nautilus_shared::EnclaveRequest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to AWS Nautilus
    let client = NautilusClient::connect().await?;
    
    // Create an enclave
    let enclave_id = client.create_enclave(&EnclaveInfo {
        name: "my-enclave".into(),
        cpu_cores: 2,
        memory_mb: 2048,
    }).await?;
    
    // Get attestation
    let attestation = client.attest(&enclave_id).await?;
    
    // Send request to enclave
    let request = EnclaveRequest {
        nonce: 12345,
        payload: json!({
            "action": "process_transaction",
            "data": "transaction-data"
        }).to_string(),
    };
    
    let response = handle_request(&attestation, request);
    println!("Enclave response: {}", response.message);
    
    Ok(())
}
```

## 🔧 Complete Working Example

Here's a complete example that demonstrates the full flow:

```rust
use anyhow::Result;
use sui_core::{messages::ExecutionRequest, transaction::TransactionDigest, mock_signed_transfer};
use sui_storage::*;
use sui_validator::ValidatorNode;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Sui Validator\n");

    // 1. Initialize storage
    let object_store: Arc<dyn ObjectStore> = Arc::new(InMemoryObjectStore::new());
    let effects_store: Arc<dyn EffectsStore> = Arc::new(InMemoryEffectsStore::new());
    let checkpoint_store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());

    // 2. Create validator
    let validator = ValidatorNode::new(
        "validator-1",
        object_store.clone(),
        effects_store.clone(),
        checkpoint_store.clone(),
    ).await?;
    println!("✅ Validator created: {}\n", validator.name());

    // 3. Create and process multiple transactions
    for i in 1..=3 {
        let tx = mock_signed_transfer(
            &format!("sender-{}", i),
            &format!("recipient-{}", i),
            &format!("coin-{}", i),
        );
        
        let request = ExecutionRequest {
            tx,
            digest: TransactionDigest::random(),
        };

        println!("📝 Processing transaction {}...", i);
        let effects = validator.handle_transaction(request.clone()).await?;
        
        println!("   ✅ Created {} objects", effects.created.len());
        println!("   ✅ Emitted {} events\n", effects.events.len());
    }

    // 4. Check latest checkpoint
    if let Some(checkpoint) = validator.latest_checkpoint().await {
        println!("📊 Latest checkpoint:");
        println!("   Sequence: {}", checkpoint.sequence_number);
        println!("   Transactions: {}", checkpoint.transaction_count);
    }

    // 5. Query storage
    let latest_seq = checkpoint_store.get_latest_sequence().await?;
    println!("\n💾 Latest checkpoint sequence: {:?}", latest_seq);

    println!("\n🎉 All done!");
    Ok(())
}
```

## 🌐 Running a Network Server

To run a validator with a network server:

```rust
use sui_validator::ValidatorNode;
use sui_storage::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup storage and validator (same as above)
    let object_store: Arc<dyn ObjectStore> = Arc::new(InMemoryObjectStore::new());
    let effects_store: Arc<dyn EffectsStore> = Arc::new(InMemoryEffectsStore::new());
    let checkpoint_store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());

    let validator = ValidatorNode::new(
        "network-validator",
        object_store,
        effects_store,
        checkpoint_store,
    ).await?;

    // Start network server (this blocks)
    println!("🌐 Starting network server on port 8080...");
    validator.start_network_server(8080).await?;
}
```

Then test it with curl:

```bash
# Health check
curl http://localhost:8080/health

# Submit transaction (using a JSON payload)
curl -X POST http://localhost:8080/submit_transaction \
  -H "Content-Type: application/json" \
  -d '{
    "transaction": {
      "tx": {
        "signer": "alice",
        "payload": {
          "kind": {
            "Transfer": {
              "object": {"0": "coin-123"},
              "recipient": "bob"
            }
          },
          "gas_budget": 1000
        },
        "signature": "sig-123"
      },
      "digest": {"0": "tx-digest-123"}
    }
  }'
```

## 📦 Project Commands

```bash
# Check all code compiles
cargo check

# Build everything
cargo build

# Build in release mode
cargo build --release

# Run tests (when you add them)
cargo test

# Run the validator demo
cargo run --example validator_demo -p sui-validator

# Run nautilus host
cargo run -p nautilus-host

# Check specific crate
cargo check -p sui-validator
cargo check -p sui-vm
cargo check -p sui-network
```

## 🎯 Common Use Cases

### Use Case 1: Local Development
- Use in-memory storage
- Run single validator
- Test transaction processing

### Use Case 2: Network Testing
- Start validator with network server
- Connect multiple clients
- Test distributed scenarios

### Use Case 3: Storage Integration
- Replace `InMemoryObjectStore` with database-backed implementation
- Use PostgreSQL, RocksDB, or other storage

### Use Case 4: Move Development
- Test Move function calls
- Simulate Move bytecode execution
- Develop and test smart contracts

## 🔍 Debugging

Enable debug logging:

```rust
// Add to your Cargo.toml
[dependencies]
env_logger = "0.11"

// In your main.rs
env_logger::init();
```

Then run with:
```bash
RUST_LOG=debug cargo run --example validator_demo -p sui-validator
```

## 📚 Next Steps

1. **Extend Storage**: Implement database backends
2. **Add Tests**: Write unit and integration tests
3. **Enhance VM**: Add more Move instructions
4. **Add Consensus**: Implement consensus protocol
5. **Real AWS**: Connect to actual AWS Nautilus

## ❓ Troubleshooting

**Problem**: "Cannot find crate"
- **Solution**: Run `cargo build` first to download dependencies

**Problem**: "Port already in use"
- **Solution**: Change the port number in `start_network_server()`

**Problem**: "Async runtime not found"
- **Solution**: Make sure your main function is `#[tokio::main]`

**Problem**: "Trait bound not satisfied"
- **Solution**: Check that you're using `Arc<dyn Trait>` for trait objects

