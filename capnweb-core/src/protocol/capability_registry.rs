// Capability Registry for bidirectional capability marshaling
// This enables real capability passing across HTTP batch requests and WebSocket connections

use std::{
    collections::HashMap,
    sync::{Arc, RwLock, atomic::{AtomicI64, Ordering}},
};
use crate::{RpcTarget, RpcError};
use crate::protocol::tables::StubReference;
use tracing::{debug, warn, info};

/// Registry for managing capability references across protocol boundaries
/// Supports both import and export of capabilities with proper lifecycle management
#[derive(Debug)]
pub struct CapabilityRegistry {
    /// Map from capability ID to the actual capability implementation
    capabilities: RwLock<HashMap<i64, Arc<dyn RpcTarget>>>,

    /// Map from Arc pointer address to capability ID (for reverse lookup)
    reverse_map: RwLock<HashMap<usize, i64>>,

    /// Next capability ID to assign
    next_id: AtomicI64,

    /// Reference count for each capability
    ref_counts: RwLock<HashMap<i64, u32>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities: RwLock::new(HashMap::new()),
            reverse_map: RwLock::new(HashMap::new()),
            next_id: AtomicI64::new(1), // Start from 1, 0 is reserved for main capability
            ref_counts: RwLock::new(HashMap::new()),
        }
    }

    /// Export a capability and return its ID for wire marshaling
    pub fn export_capability(&self, capability: Arc<dyn RpcTarget>) -> i64 {
        let ptr_addr = Arc::as_ptr(&capability) as *const () as usize;

        // Check if this capability is already exported
        if let Ok(reverse_map) = self.reverse_map.read() {
            if let Some(&existing_id) = reverse_map.get(&ptr_addr) {
                // Increment reference count
                if let Ok(mut ref_counts) = self.ref_counts.write() {
                    *ref_counts.entry(existing_id).or_insert(0) += 1;
                }
                debug!("Reusing existing capability export: ID {}", existing_id);
                return existing_id;
            }
        }

        // Assign new ID
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        // Store both mappings
        if let (Ok(mut capabilities), Ok(mut reverse_map), Ok(mut ref_counts)) = (
            self.capabilities.write(),
            self.reverse_map.write(),
            self.ref_counts.write()
        ) {
            capabilities.insert(id, capability);
            reverse_map.insert(ptr_addr, id);
            ref_counts.insert(id, 1);

            info!("Exported new capability: ID {}", id);
        }

        id
    }

    /// Import a capability by ID for method calls
    pub fn import_capability(&self, id: i64) -> Option<Arc<dyn RpcTarget>> {
        if let Ok(capabilities) = self.capabilities.read() {
            capabilities.get(&id).cloned()
        } else {
            None
        }
    }

    /// Check if a capability ID exists
    pub fn has_capability(&self, id: i64) -> bool {
        if let Ok(capabilities) = self.capabilities.read() {
            capabilities.contains_key(&id)
        } else {
            false
        }
    }

    /// Release a capability reference (decrement ref count)
    pub fn release_capability(&self, id: i64) -> bool {
        if let Ok(mut ref_counts) = self.ref_counts.write() {
            if let Some(count) = ref_counts.get_mut(&id) {
                *count = count.saturating_sub(1);

                if *count == 0 {
                    // Remove from all maps
                    ref_counts.remove(&id);

                    if let (Ok(mut capabilities), Ok(mut reverse_map)) = (
                        self.capabilities.write(),
                        self.reverse_map.write()
                    ) {
                        if let Some(capability) = capabilities.remove(&id) {
                            let ptr_addr = Arc::as_ptr(&capability) as *const () as usize;
                            reverse_map.remove(&ptr_addr);
                        }
                    }

                    info!("Released capability: ID {}", id);
                    true
                } else {
                    debug!("Decremented capability ref count: ID {} (count: {})", id, count);
                    false
                }
            } else {
                warn!("Attempted to release unknown capability: ID {}", id);
                false
            }
        } else {
            false
        }
    }

    /// Get current reference count for a capability
    pub fn get_ref_count(&self, id: i64) -> u32 {
        if let Ok(ref_counts) = self.ref_counts.read() {
            ref_counts.get(&id).copied().unwrap_or(0)
        } else {
            0
        }
    }

    /// Get all exported capability IDs
    pub fn get_exported_ids(&self) -> Vec<i64> {
        if let Ok(capabilities) = self.capabilities.read() {
            capabilities.keys().copied().collect()
        } else {
            Vec::new()
        }
    }

    /// Create a stub reference for a capability (for import table integration)
    pub fn create_stub_reference(&self, id: i64) -> Option<StubReference> {
        if let Some(capability) = self.import_capability(id) {
            Some(StubReference::new(capability))
        } else {
            None
        }
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for capabilities that can be registered in the registry
pub trait RegistrableCapability: RpcTarget {
    /// Get a display name for this capability (for debugging)
    fn name(&self) -> &str {
        "Unknown"
    }

    /// Called when the capability is exported
    fn on_export(&self, _id: i64) {}

    /// Called when the capability is imported
    fn on_import(&self, _id: i64) {}

    /// Called when the capability is released
    fn on_release(&self, _id: i64) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockRpcTarget;

    #[test]
    fn test_capability_export_import() {
        let registry = CapabilityRegistry::new();
        let capability = Arc::new(MockRpcTarget::new());

        // Export capability
        let id = registry.export_capability(capability.clone());
        assert!(id > 0);

        // Import capability
        let imported = registry.import_capability(id);
        assert!(imported.is_some());

        // Verify it's the same capability
        let imported = imported.unwrap();
        assert_eq!(
            Arc::as_ptr(&capability) as *const (),
            Arc::as_ptr(&imported) as *const ()
        );
    }

    #[test]
    fn test_capability_ref_counting() {
        let registry = CapabilityRegistry::new();
        let capability = Arc::new(MockRpcTarget::new());

        // Export same capability twice
        let id1 = registry.export_capability(capability.clone());
        let id2 = registry.export_capability(capability.clone());

        // Should get same ID
        assert_eq!(id1, id2);

        // Should have ref count of 2
        assert_eq!(registry.get_ref_count(id1), 2);

        // Release once - should still exist
        assert!(!registry.release_capability(id1));
        assert_eq!(registry.get_ref_count(id1), 1);
        assert!(registry.has_capability(id1));

        // Release again - should be removed
        assert!(registry.release_capability(id1));
        assert_eq!(registry.get_ref_count(id1), 0);
        assert!(!registry.has_capability(id1));
    }

    #[test]
    fn test_stub_reference_creation() {
        let registry = CapabilityRegistry::new();
        let capability = Arc::new(MockRpcTarget::new());

        let id = registry.export_capability(capability);
        let stub_ref = registry.create_stub_reference(id);

        assert!(stub_ref.is_some());

        // Test with non-existent ID
        let invalid_stub = registry.create_stub_reference(999);
        assert!(invalid_stub.is_none());
    }
}