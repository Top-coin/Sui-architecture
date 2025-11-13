# Quick Start Guide

## 🚀 Get Started in 3 Steps

### Step 1: Build the Project

```bash
cd sui-architecture
cargo build
```

### Step 2: Run a Simple Example

```bash
# Basic validator demo
cargo run --example validator_demo -p sui-validator

# Complete example with all features
cargo run --example complete_example -p sui-validator
```

### Step 3: Create Your Own Validator

Create a file `my_validator.rs`:

```rust
use sui_validator::ValidatorNode;
use sui_storage::*;
use sui_core::{messages::ExecutionRequest, transaction::TransactionDigest, mock_signed_transfer};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup storage
    let object_store: Arc<dyn ObjectStore> = Arc::new(InMemoryObjectStore::new());
    let effects_store: Arc<dyn EffectsStore> = Arc::new(InMemoryEffectsStore::new());
    let checkpoint_store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());

    // 2. Create validator
    let validator = ValidatorNode::new(
        "my-validator",
        object_store,
        effects_store,
        checkpoint_store,
    ).await?;

    // 3. Process a transaction
    let tx = mock_signed_transfer("alice", "bob", "coin-123");
    let request = ExecutionRequest {
        tx,
        digest: TransactionDigest::random(),
    };

    let effects = validator.handle_transaction(request).await?;
    println!("✅ Transaction processed! Events: {:?}", effects.events);

    Ok(())
}
```

Then run it:
```bash
rustc --edition 2021 my_validator.rs --extern sui_validator=target/debug/libsui_validator.rlib
# Or better: add it as an example in Cargo.toml
```

## 📚 What Can You Do?

### ✅ Process Transactions
- Transfer objects between addresses
- Execute Move function calls
- Track transaction effects

### ✅ Use Storage
- Store and retrieve objects
- Save transaction effects
- Manage checkpoints

### ✅ Network Communication
- Start HTTP server
- Submit transactions via REST API
- Query objects remotely

### ✅ TEE Integration
- Create AWS Nautilus enclaves
- Perform secure attestation
- Send transactions to enclaves

## 🎯 Common Commands

```bash
# Check everything compiles
cargo check

# Run tests
cargo test

# Build release version
cargo build --release

# Run specific example
cargo run --example validator_demo -p sui-validator
cargo run --example complete_example -p sui-validator

# Check specific crate
cargo check -p sui-validator
cargo check -p sui-vm
cargo check -p sui-network
```

## 📖 Next Steps

1. Read [USAGE.md](USAGE.md) for detailed examples
2. Read [README.md](README.md) for architecture overview
3. Explore the examples in `crates/validator/examples/`
4. Modify and extend the code!

## ❓ Need Help?

- Check the examples in `crates/validator/examples/`
- See `USAGE.md` for detailed usage patterns
- Review `README.md` for architecture details

