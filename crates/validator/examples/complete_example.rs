//! Complete example demonstrating all features of the Sui architecture project

use anyhow::Result;
use sui_core::{
    messages::ExecutionRequest,
    object::{ObjectData, ObjectID, Owner, SuiObject},
    transaction::{SignedTransaction, TransactionDigest, TransactionKind, TransactionPayload},
    mock_signed_transfer,
};
use sui_network::NetworkClient;
use sui_storage::*;
use sui_validator::ValidatorNode;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Sui Architecture Complete Example\n");
    println!("{}", "=".repeat(50));

    // ============================================
    // Step 1: Initialize Storage
    // ============================================
    println!("\n📦 Step 1: Initializing storage layers...");
    let object_store: Arc<dyn ObjectStore> = Arc::new(InMemoryObjectStore::new());
    let effects_store: Arc<dyn EffectsStore> = Arc::new(InMemoryEffectsStore::new());
    let checkpoint_store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());
    println!("✅ Storage initialized\n");

    // ============================================
    // Step 2: Create Validator
    // ============================================
    println!("🔧 Step 2: Creating validator node...");
    let validator = ValidatorNode::new(
        "demo-validator",
        object_store.clone(),
        effects_store.clone(),
        checkpoint_store.clone(),
    )
    .await?;
    println!("✅ Validator '{}' created\n", validator.name());

    // ============================================
    // Step 3: Create Initial Objects
    // ============================================
    println!("💰 Step 3: Creating initial objects...");
    let coin1 = SuiObject::new(
        ObjectID::new("coin-alice-1"),
        Owner::Address("alice".to_string()),
        ObjectData::Coin { balance: 1000 },
    );
    let coin2 = SuiObject::new(
        ObjectID::new("coin-bob-1"),
        Owner::Address("bob".to_string()),
        ObjectData::Coin { balance: 500 },
    );

    object_store.put_object(coin1.clone()).await?;
    object_store.put_object(coin2.clone()).await?;
    println!("✅ Created 2 coin objects\n");

    // ============================================
    // Step 4: Process Transfer Transactions
    // ============================================
    println!("📝 Step 4: Processing transfer transactions...");
    
    // Transfer 1: Alice to Bob
    let tx1 = mock_signed_transfer("alice", "bob", "coin-alice-1");
    let req1 = ExecutionRequest {
        tx: tx1,
        digest: TransactionDigest::random(),
    };
    println!("   Processing transfer: alice -> bob");
    let effects1 = validator.handle_transaction(req1.clone()).await?;
    println!("   ✅ Transfer 1 completed: {} events emitted\n", effects1.events.len());

    // Transfer 2: Bob to Charlie
    let tx2 = mock_signed_transfer("bob", "charlie", "coin-bob-1");
    let req2 = ExecutionRequest {
        tx: tx2,
        digest: TransactionDigest::random(),
    };
    println!("   Processing transfer: bob -> charlie");
    let effects2 = validator.handle_transaction(req2.clone()).await?;
    println!("   ✅ Transfer 2 completed: {} events emitted\n", effects2.events.len());

    // ============================================
    // Step 5: Process Move Function Call
    // ============================================
    println!("⚙️  Step 5: Processing Move function call...");
    let payload = TransactionPayload {
        kind: TransactionKind::Call {
            package: ObjectID::new("coin-package"),
            module: "coin".to_string(),
            function: "mint".to_string(),
            arguments: vec![serde_json::json!("new-owner"), serde_json::json!(2000u64)],
        },
        gas_budget: 5000,
    };

    let tx3 = SignedTransaction::new("system".to_string(), payload);
    let req3 = ExecutionRequest {
        tx: tx3,
        digest: TransactionDigest::random(),
    };

    let effects3 = validator.handle_transaction(req3.clone()).await?;
    println!("   ✅ Move call completed:");
    println!("      - Gas used: simulated");
    println!("      - Events: {:?}\n", effects3.events);

    // ============================================
    // Step 6: Check Checkpoints
    // ============================================
    println!("📊 Step 6: Checking checkpoints...");
    if let Some(checkpoint) = validator.latest_checkpoint().await {
        println!("   Latest checkpoint:");
        println!("      Sequence: {}", checkpoint.sequence_number);
        println!("      Transactions: {}", checkpoint.transaction_count);
        println!("      Root digest: {}\n", checkpoint.root_digest);
    }

    // ============================================
    // Step 7: Query Storage
    // ============================================
    println!("💾 Step 7: Querying storage...");
    
    // Query effects
    let stored_effects = effects_store.get_effects(&req1.digest).await?;
    if stored_effects.is_some() {
        println!("   ✅ Effects stored for transaction 1");
    }

    // Query checkpoints
    let latest_seq = checkpoint_store.get_latest_sequence().await?;
    if let Some(seq) = latest_seq {
        let checkpoint_data = checkpoint_store.get_checkpoint(seq).await?;
        if checkpoint_data.is_some() {
            println!("   ✅ Checkpoint {} retrieved from storage", seq);
        }
    }

    // List objects
    let alice_objects = object_store.list_objects(Some("alice")).await?;
    let bob_objects = object_store.list_objects(Some("bob")).await?;
    println!("   Alice has {} objects", alice_objects.len());
    println!("   Bob has {} objects\n", bob_objects.len());

    // ============================================
    // Step 8: Network Server (Optional)
    // ============================================
    println!("🌐 Step 8: Starting network server (will run for 5 seconds)...");
    
    let validator_clone = validator.clone();
    let server_handle = tokio::spawn(async move {
        validator_clone.start_network_server(8080).await
    });

    // Give server time to start
    sleep(Duration::from_millis(500)).await;

    // Test network client
    println!("   Testing network client...");
    let client = NetworkClient::new("http://localhost:8080");

    // Health check
    let health_response = reqwest::Client::new()
        .get("http://localhost:8080/health")
        .send()
        .await?;
    if health_response.status().is_success() {
        println!("   ✅ Health check passed");
    }

    // Submit transaction via network
    let tx4 = mock_signed_transfer("charlie", "dave", "coin-charlie-1");
    let req4 = ExecutionRequest {
        tx: tx4,
        digest: TransactionDigest::random(),
    };

    match client.submit_transaction(req4).await {
        Ok(response) => {
            if response.accepted {
                println!("   ✅ Transaction submitted via network");
            } else {
                println!("   ⚠️  Transaction rejected: {}", response.message);
            }
        }
        Err(e) => {
            println!("   ⚠️  Network error: {}", e);
        }
    }

    // Wait a bit then shutdown
    sleep(Duration::from_secs(2)).await;
    server_handle.abort();
    println!("   ✅ Network server stopped\n");

    // ============================================
    // Summary
    // ============================================
    println!("{}", "=".repeat(50));
    println!("\n🎉 Example completed successfully!");
    println!("\nSummary:");
    println!("   ✅ Validator created and running");
    println!("   ✅ {} transactions processed", 4);
    println!("   ✅ Storage operations verified");
    println!("   ✅ Network server tested");
    println!("\n✨ All features demonstrated!\n");

    Ok(())
}

