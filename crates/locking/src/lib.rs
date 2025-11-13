use parking_lot::Mutex;
use std::collections::HashMap;

use sui_core::object::SuiObject;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    Shared,
    Exclusive,
}

#[derive(Debug, Default)]
struct LockState {
    shared_count: usize,
    exclusive: bool,
}

#[derive(Default)]
pub struct LockManager {
    inner: Mutex<HashMap<String, LockState>>,
}

impl LockManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(&self, object: &SuiObject, mode: LockMode) -> bool {
        let mut map = self.inner.lock();
        let state = map.entry(object.lock_key()).or_default();

        match mode {
            LockMode::Shared => {
                if state.exclusive {
                    false
                } else {
                    state.shared_count += 1;
                    true
                }
            }
            LockMode::Exclusive => {
                if state.exclusive || state.shared_count > 0 {
                    false
                } else {
                    state.exclusive = true;
                    true
                }
            }
        }
    }

    pub fn release(&self, object: &SuiObject, mode: LockMode) {
        let mut map = self.inner.lock();
        if let Some(state) = map.get_mut(&object.lock_key()) {
            match mode {
                LockMode::Shared => state.shared_count = state.shared_count.saturating_sub(1),
                LockMode::Exclusive => state.exclusive = false,
            }

            if state.shared_count == 0 && !state.exclusive {
                map.remove(&object.lock_key());
            }
        }
    }
}

