use serde::{Deserialize, Serialize};

/// Unique identifier for any on-chain object.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectID(pub String);

impl ObjectID {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn random() -> Self {
        use rand::Rng;
        Self(format!("obj-{:x}", rand::thread_rng().gen::<u128>()))
    }
}

/// Ownership model simplified from Sui.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Owner {
    Address(String),
    Shared,
    Immutable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectData {
    Coin { balance: u64 },
    Package { modules: Vec<String> },
    MoveStruct { type_name: String, fields: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiObject {
    pub id: ObjectID,
    pub version: u64,
    pub owner: Owner,
    pub data: ObjectData,
}

impl SuiObject {
    pub fn new(id: ObjectID, owner: Owner, data: ObjectData) -> Self {
        Self { id, version: 1, owner, data }
    }

    pub fn lock_key(&self) -> String {
        format!("{}::v{}", self.id.0, self.version)
    }

    pub fn is_shared(&self) -> bool {
        matches!(self.owner, Owner::Shared)
    }
}

