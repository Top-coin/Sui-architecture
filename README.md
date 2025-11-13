# Sui Architecture & Nautilus Project

A comprehensive educational implementation of Sui blockchain architecture components including validator logic, Move VM execution, storage, networking, and AWS Nautilus TEE integration.

## 🏗️ Architecture Overview

This project demonstrates the core components of a Sui blockchain validator:

### Core Components

1. **sui-core** - Fundamental data types (objects, transactions, messages)
2. **sui-precheck** - Transaction validation before execution
3. **sui-locking** - Object locking mechanism for concurrent access
4. **sui-vm** - Move bytecode execution engine
5. **sui-effects** - Transaction effects tracking
6. **sui-checkpoint** - Checkpoint aggregation and storage
7. **sui-storage** - Persistent storage abstractions
8. **sui-network** - HTTP-based networking layer
9. **sui-validator** - Main validator node orchestrating all components
10. **aws-nautilus-sdk** - AWS Nautilus TEE integration

### Nautilus TEE Components

- **nautilus/shared** - Shared types between host and enclave
- **nautilus/host** - Host application managing enclave lifecycle
- **nautilus/enclave** - Secure enclave binary for trusted execution

## 🚀 Features

### ✅ Storage Layer
- **ObjectStore**: Persistent storage for Sui objects
- **EffectsStore**: Transaction effects persistence
- **CheckpointStore**: Checkpoint sequence storage
- In-memory implementations provided (can be extended to databases)

### ✅ Networking
- **HTTP REST API** using Axum
- Transaction submission endpoint
- Object query endpoint
- Health check endpoint
- Async/await support throughout

### ✅ Move VM Execution
- Move bytecode parsing and interpretation
- Instruction-level execution simulation
- Support for:
  - Transfer operations
  - Move function calls
  - Coin operations (transfer, mint)
  - Custom module execution

### ✅ AWS Nautilus Integration
- Enclave creation and management
- Attestation support
- Secure transaction submission
- Both async and sync APIs

## 📦 Project Structure

```
sui-architecture/
├── crates/
│   ├── core/              # Core data types
│   ├── validator/         # Main validator node
│   ├── precheck/          # Pre-execution validation
│   ├── locking/           # Object locking
│   ├── vm/                # Move VM execution
│   ├── effects/           # Transaction effects
│   ├── checkpoint/        # Checkpoint management
│   ├── storage/           # Storage abstractions
│   ├── network/           # Networking layer
│   └── aws-nautilus-sdk/  # AWS Nautilus SDK
└── nautilus/
    ├── shared/            # Shared types
    ├── host/              # Host application
    └── enclave/           # Enclave binary
```

## 🔧 Building

```bash
# Check all crates compile
cargo check

# Build everything
cargo build --release

# Run the validator demo
cargo run --example validator_demo -p sui-validator
```

## 📖 Usage Example

```rust
use sui_validator::ValidatorNode;
use sui_storage::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize storage
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

    // Process transactions...
    // Start network server...
    Ok(())
}
```

## 🌐 Network API

The validator exposes a REST API:

- `GET /health` - Health check
- `POST /submit_transaction` - Submit a transaction for processing
- `POST /get_object` - Query object by ID

## 🔐 Nautilus TEE

The Nautilus components provide:

- **Enclave Creation**: Secure enclave instances
- **Attestation**: Verify enclave integrity
- **Secure Communication**: Encrypted channel between host and enclave

## 🎓 Educational Value

This project demonstrates:

1. **Modular Architecture**: Each component is independently testable
2. **Async Programming**: Full async/await support with Tokio
3. **Storage Abstractions**: Trait-based storage for flexibility
4. **Network Protocols**: HTTP-based communication
5. **Move Execution**: Simplified Move bytecode interpreter
6. **TEE Integration**: AWS Nautilus enclave management

## 🔄 Transaction Flow

1. **Pre-check**: Validate transaction format and requirements
2. **Locking**: Acquire necessary object locks
3. **Execution**: Run transaction in Move VM
4. **Effects**: Record all state changes
5. **Storage**: Persist objects, effects, and checkpoints
6. **Nautilus**: Send to secure enclave for additional processing

## 📝 Notes

- This is an **educational implementation** - not production-ready
- Storage uses in-memory implementations (extend for production)
- Move VM is a simplified interpreter (not full Move VM)
- AWS Nautilus integration uses mock implementations (add real AWS SDK calls)

## 🤝 Contributing

This is a learning project. Feel free to extend:
- Add database backends (PostgreSQL, RocksDB, etc.)
- Implement full Move VM
- Add consensus protocol
- Enhance networking (gRPC, WebSocket)
- Real AWS Nautilus integration

## 📄 License

Apache-2.0

