use anyhow::{anyhow, Result};
use async_trait::async_trait;
use aws_nautilus_sdk::{EnclaveInfo, NautilusClient};
use serde_json::json;
use std::sync::Arc;
use sui_checkpoint::CheckpointAggregator;
use sui_core::{
    messages::{CheckpointSummary, ExecutionRequest},
    object::{ObjectData, ObjectID, Owner, SuiObject},
};
use sui_effects::EffectsBuilder;
use sui_locking::{LockManager, LockMode};
use sui_network::{NetworkServer, TransactionHandler};
use sui_precheck::PreCheckPipeline;
use sui_storage::{CheckpointStore, EffectsStore, ObjectStore};
use sui_vm::MoveVMExecutor;

pub struct ValidatorNode {
    name: String,
    precheck: PreCheckPipeline,
    lock_manager: Arc<LockManager>,
    vm: Arc<MoveVMExecutor>,
    checkpoints: Arc<tokio::sync::Mutex<CheckpointAggregator>>,
    sequence: Arc<tokio::sync::Mutex<u64>>,
    nautilus_client: Arc<NautilusClient>,
    nautilus_enclave_id: String,
    object_store: Arc<dyn ObjectStore>,
    effects_store: Arc<dyn EffectsStore>,
    checkpoint_store: Arc<dyn CheckpointStore>,
}

impl ValidatorNode {
    pub async fn new(
        name: impl Into<String>,
        object_store: Arc<dyn ObjectStore>,
        effects_store: Arc<dyn EffectsStore>,
        checkpoint_store: Arc<dyn CheckpointStore>,
    ) -> Result<Self> {
        let name = name.into();
        let client = Arc::new(NautilusClient::connect_sync()?);
        let enclave_id = client.create_enclave_sync(&EnclaveInfo {
            name: format!("{}-enclave", name),
            cpu_cores: 2,
            memory_mb: 4096,
        })?;

        let vm = Arc::new(MoveVMExecutor::with_object_store(
            Box::new(InMemoryObjectStoreWrapper {
                store: object_store.clone(),
            }),
        ));

        Ok(Self {
            name: name.clone(),
            precheck: PreCheckPipeline::default(),
            lock_manager: Arc::new(LockManager::new()),
            vm,
            checkpoints: Arc::new(tokio::sync::Mutex::new(CheckpointAggregator::new())),
            sequence: Arc::new(tokio::sync::Mutex::new(0)),
            nautilus_client: client,
            nautilus_enclave_id: enclave_id,
            object_store,
            effects_store,
            checkpoint_store,
        })
    }

    pub async fn handle_transaction(&self, request: ExecutionRequest) -> Result<sui_effects::TransactionEffects> {
        let report = self
            .precheck
            .run(&request)
            .map_err(|err| anyhow!("pre-check failed: {err}"))?;

        let simulated_object = SuiObject::new(
            ObjectID::new("object-to-lock"),
            Owner::Shared,
            ObjectData::Coin { balance: 0 },
        );

        if report.requires_shared_lock
            && !self
                .lock_manager
                .acquire(&simulated_object, LockMode::Exclusive)
        {
            return Err(anyhow!("unable to acquire lock for shared object"));
        }

        let exec_result = self.vm.execute(&request).await;

        for obj in &exec_result.touched_objects {
            self.object_store.put_object(obj.clone()).await?;
        }

        let mut builder = EffectsBuilder::new(request.digest.clone());
        for touched in exec_result.touched_objects {
            builder = builder.record_created(touched);
        }
        for log in exec_result.logs {
            builder = builder.record_event(log);
        }
        let effects = builder.build();

        let effects_json = serde_json::to_string(&effects)?;
        self.effects_store
            .save_effects(&request.digest, &effects_json)
            .await?;

        let mut seq = self.sequence.lock().await;
        *seq += 1;
        let current_seq = *seq;
        drop(seq);

        let checkpoint = CheckpointSummary {
            sequence_number: current_seq,
            transaction_count: 1,
            root_digest: request.digest.0.clone(),
        };

        let mut checkpoints = self.checkpoints.lock().await;
        checkpoints.record(checkpoint.clone());
        drop(checkpoints);

        let checkpoint_json = serde_json::to_string(&checkpoint)?;
        self.checkpoint_store
            .save_checkpoint(current_seq, &checkpoint_json)
            .await?;

        let payload = json!({
            "validator": self.name,
            "digest": request.digest.0,
            "event_count": effects.events.len(),
        });
        let _ = self
            .nautilus_client
            .send_transaction_sync(&self.nautilus_enclave_id, payload);

        if report.requires_shared_lock {
            self.lock_manager
                .release(&simulated_object, LockMode::Exclusive);
        }

        Ok(effects)
    }

    pub async fn latest_checkpoint(&self) -> Option<CheckpointSummary> {
        let checkpoints = self.checkpoints.lock().await;
        checkpoints.latest().cloned()
    }

    pub async fn start_network_server(&self, port: u16) -> Result<()> {
        let handler = ValidatorHandler {
            validator: Arc::new(self.clone()),
        };
        let server = NetworkServer::new(port);
        server.start(handler).await
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Clone for ValidatorNode {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            precheck: PreCheckPipeline::default(),
            lock_manager: Arc::clone(&self.lock_manager),
            vm: Arc::clone(&self.vm),
            checkpoints: Arc::clone(&self.checkpoints),
            sequence: Arc::clone(&self.sequence),
            nautilus_client: Arc::clone(&self.nautilus_client),
            nautilus_enclave_id: self.nautilus_enclave_id.clone(),
            object_store: Arc::clone(&self.object_store),
            effects_store: Arc::clone(&self.effects_store),
            checkpoint_store: Arc::clone(&self.checkpoint_store),
        }
    }
}

#[derive(Clone)]
struct ValidatorHandler {
    validator: Arc<ValidatorNode>,
}

#[async_trait]
impl TransactionHandler for ValidatorHandler {
    async fn handle_transaction(&self, request: ExecutionRequest) -> Result<sui_network::SubmitTransactionResponse> {
        match self.validator.handle_transaction(request).await {
            Ok(_effects) => Ok(sui_network::SubmitTransactionResponse {
                accepted: true,
                message: "Transaction processed successfully".to_string(),
            }),
            Err(e) => Ok(sui_network::SubmitTransactionResponse {
                accepted: false,
                message: format!("Transaction failed: {}", e),
            }),
        }
    }

    async fn get_object(&self, object_id: &str) -> Result<Option<serde_json::Value>> {
        match self.validator.object_store.get_object(object_id).await {
            Ok(Some(obj)) => Ok(Some(serde_json::to_value(obj)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow!("Failed to get object: {}", e)),
        }
    }
}

struct InMemoryObjectStoreWrapper {
    store: Arc<dyn ObjectStore>,
}

#[async_trait]
impl sui_storage::ObjectStore for InMemoryObjectStoreWrapper {
    async fn get_object(&self, id: &str) -> Result<Option<SuiObject>> {
        self.store.get_object(id).await
    }

    async fn put_object(&self, object: SuiObject) -> Result<()> {
        self.store.put_object(object).await
    }

    async fn delete_object(&self, id: &str) -> Result<()> {
        self.store.delete_object(id).await
    }

    async fn list_objects(&self, owner: Option<&str>) -> Result<Vec<SuiObject>> {
        self.store.list_objects(owner).await
    }
}
