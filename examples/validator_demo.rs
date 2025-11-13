use anyhow::Result;
use sui_core::{messages::ExecutionRequest, transaction::TransactionDigest, mock_signed_transfer};
use sui_storage::{
    CheckpointStore, EffectsStore, InMemoryCheckpointStore, InMemoryEffectsStore,
    InMemoryObjectStore, ObjectStore,
};
use sui_validator::ValidatorNode;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Sui Validator Demo\n");

    // Initialize storage layers
    let object_store: Arc<dyn ObjectStore> = Arc::new(InMemoryObjectStore::new());
    let effects_store: Arc<dyn EffectsStore> = Arc::new(InMemoryEffectsStore::new());
    let checkpoint_store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());

    // Create a validator node
    println!("📦 Creating validator node...");
    let validator = ValidatorNode::new(
        "validator-1",
        object_store.clone(),
        effects_store.clone(),
        checkpoint_store.clone(),
    )
    .await?;
    println!("✅ Validator '{}' created\n", validator.name());

    // Create a test transaction
    println!("📝 Creating test transaction...");
    let tx = mock_signed_transfer("alice", "bob", "coin-123");
    let request = ExecutionRequest {
        tx,
        digest: TransactionDigest::random(),
    };
    println!("✅ Transaction created: {}\n", request.digest.0);

    // Process the transaction
    println!("⚙️  Processing transaction...");
    let effects = validator.handle_transaction(request.clone()).await?;
    println!("✅ Transaction processed successfully!");
    println!("   - Created {} objects", effects.created.len());
    println!("   - Mutated {} objects", effects.mutated.len());
    println!("   - Emitted {} events\n", effects.events.len());

    // Check checkpoint
    if let Some(checkpoint) = validator.latest_checkpoint().await {
        println!("📊 Latest checkpoint:");
        println!("   - Sequence: {}", checkpoint.sequence_number);
        println!("   - Transactions: {}", checkpoint.transaction_count);
        println!("   - Root digest: {}\n", checkpoint.root_digest);
    }

    // Verify storage
    println!("💾 Verifying storage...");
    let stored_effects = effects_store.get_effects(&request.digest).await?;
    if stored_effects.is_some() {
        println!("✅ Effects stored successfully");
    }

    let latest_seq = checkpoint_store.get_latest_sequence().await?;
    if let Some(seq) = latest_seq {
        println!("✅ Checkpoint {} stored successfully\n", seq);
    }

    println!("🎉 Demo completed successfully!");

    Ok(())
}

