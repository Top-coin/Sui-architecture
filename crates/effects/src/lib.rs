use serde::Serialize;
use sui_core::{object::SuiObject, transaction::TransactionDigest};

#[derive(Debug, Serialize)]
pub struct TransactionEffects {
    pub digest: TransactionDigest,
    pub created: Vec<SuiObject>,
    pub mutated: Vec<SuiObject>,
    pub events: Vec<String>,
}

impl TransactionEffects {
    pub fn new(digest: TransactionDigest) -> Self {
        Self {
            digest,
            created: Vec::new(),
            mutated: Vec::new(),
            events: Vec::new(),
        }
    }
}

pub struct EffectsBuilder {
    effects: TransactionEffects,
}

impl EffectsBuilder {
    pub fn new(digest: TransactionDigest) -> Self {
        Self {
            effects: TransactionEffects::new(digest),
        }
    }

    pub fn record_created(mut self, object: SuiObject) -> Self {
        self.effects.created.push(object);
        self
    }

    pub fn record_mutated(mut self, object: SuiObject) -> Self {
        self.effects.mutated.push(object);
        self
    }

    pub fn record_event(mut self, event: impl Into<String>) -> Self {
        self.effects.events.push(event.into());
        self
    }

    pub fn build(self) -> TransactionEffects {
        self.effects
    }
}

