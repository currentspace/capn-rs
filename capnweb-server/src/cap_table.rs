use crate::RpcTarget;
use currentspace_capnweb_core::CapId;
use dashmap::DashMap;
use std::sync::Arc;

pub struct CapTable {
    caps: DashMap<CapId, Arc<dyn RpcTarget>>,
}

impl CapTable {
    pub fn new() -> Self {
        CapTable {
            caps: DashMap::new(),
        }
    }

    pub fn insert(&self, id: CapId, target: Arc<dyn RpcTarget>) {
        self.caps.insert(id, target);
    }

    pub fn lookup(&self, id: &CapId) -> Option<Arc<dyn RpcTarget>> {
        self.caps.get(id).map(|entry| Arc::clone(&*entry))
    }

    pub fn remove(&self, id: &CapId) -> Option<Arc<dyn RpcTarget>> {
        self.caps.remove(id).map(|(_, v)| v)
    }

    pub fn clear(&self) {
        self.caps.clear();
    }

    pub fn len(&self) -> usize {
        self.caps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.caps.is_empty()
    }
}

impl Default for CapTable {
    fn default() -> Self {
        Self::new()
    }
}
