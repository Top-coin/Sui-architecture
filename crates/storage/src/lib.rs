use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use sui_core::{object::SuiObject, transaction::TransactionDigest};

#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn get_object(&self, id: &str) -> anyhow::Result<Option<SuiObject>>;
    async fn put_object(&self, object: SuiObject) -> anyhow::Result<()>;
    async fn delete_object(&self, id: &str) -> anyhow::Result<()>;
    async fn list_objects(&self, owner: Option<&str>) -> anyhow::Result<Vec<SuiObject>>;
}

#[async_trait]
pub trait EffectsStore: Send + Sync {
    async fn save_effects(&self, digest: &TransactionDigest, effects_json: &str) -> anyhow::Result<()>;
    async fn get_effects(&self, digest: &TransactionDigest) -> anyhow::Result<Option<String>>;
}

#[async_trait]
pub trait CheckpointStore: Send + Sync {
    async fn save_checkpoint(&self, sequence: u64, checkpoint_json: &str) -> anyhow::Result<()>;
    async fn get_checkpoint(&self, sequence: u64) -> anyhow::Result<Option<String>>;
    async fn get_latest_sequence(&self) -> anyhow::Result<Option<u64>>;
}

pub struct InMemoryObjectStore {
    objects: Arc<RwLock<HashMap<String, SuiObject>>>,
}

impl InMemoryObjectStore {
    pub fn new() -> Self {
        Self {
            objects: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ObjectStore for InMemoryObjectStore {
    async fn get_object(&self, id: &str) -> anyhow::Result<Option<SuiObject>> {
        Ok(self.objects.read().get(id).cloned())
    }

    async fn put_object(&self, object: SuiObject) -> anyhow::Result<()> {
        self.objects.write().insert(object.id.0.clone(), object);
        Ok(())
    }

    async fn delete_object(&self, id: &str) -> anyhow::Result<()> {
        self.objects.write().remove(id);
        Ok(())
    }

    async fn list_objects(&self, owner: Option<&str>) -> anyhow::Result<Vec<SuiObject>> {
        let objects = self.objects.read();
        if let Some(owner_addr) = owner {
            Ok(objects
                .values()
                .filter(|obj| match &obj.owner {
                    sui_core::Owner::Address(addr) => addr == owner_addr,
                    _ => false,
                })
                .cloned()
                .collect())
        } else {
            Ok(objects.values().cloned().collect())
        }
    }
}

pub struct InMemoryEffectsStore {
    effects: Arc<RwLock<HashMap<String, String>>>,
}

impl InMemoryEffectsStore {
    pub fn new() -> Self {
        Self {
            effects: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EffectsStore for InMemoryEffectsStore {
    async fn save_effects(&self, digest: &TransactionDigest, effects_json: &str) -> anyhow::Result<()> {
        self.effects.write().insert(digest.0.clone(), effects_json.to_string());
        Ok(())
    }

    async fn get_effects(&self, digest: &TransactionDigest) -> anyhow::Result<Option<String>> {
        Ok(self.effects.read().get(&digest.0).cloned())
    }
}

pub struct InMemoryCheckpointStore {
    checkpoints: Arc<RwLock<HashMap<u64, String>>>,
    latest: Arc<RwLock<Option<u64>>>,
}

impl InMemoryCheckpointStore {
    pub fn new() -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
            latest: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait]
impl CheckpointStore for InMemoryCheckpointStore {
    async fn save_checkpoint(&self, sequence: u64, checkpoint_json: &str) -> anyhow::Result<()> {
        self.checkpoints.write().insert(sequence, checkpoint_json.to_string());
        *self.latest.write() = Some(sequence);
        Ok(())
    }

    async fn get_checkpoint(&self, sequence: u64) -> anyhow::Result<Option<String>> {
        Ok(self.checkpoints.read().get(&sequence).cloned())
    }

    async fn get_latest_sequence(&self) -> anyhow::Result<Option<u64>> {
        Ok(*self.latest.read())
    }
}

