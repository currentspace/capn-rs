use async_trait::async_trait;
use currentspace_capnweb_core::{CapId, RpcError};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Trait for objects that need cleanup when disposed
#[async_trait]
pub trait Disposable: Send + Sync {
    /// Called when the capability is being disposed
    async fn dispose(&self) -> Result<(), RpcError>;
}

/// Tracks the lifecycle of capabilities
pub struct CapabilityLifecycle {
    /// Reference counts for capabilities
    ref_counts: Arc<DashMap<CapId, usize>>,
    /// Disposal callbacks for capabilities
    dispose_callbacks: Arc<DashMap<CapId, Arc<dyn Disposable>>>,
    /// Capabilities owned by each session
    session_caps: Arc<RwLock<DashMap<String, Vec<CapId>>>>,
}

impl CapabilityLifecycle {
    pub fn new() -> Self {
        Self {
            ref_counts: Arc::new(DashMap::new()),
            dispose_callbacks: Arc::new(DashMap::new()),
            session_caps: Arc::new(RwLock::new(DashMap::new())),
        }
    }

    /// Register a new capability with optional disposal callback
    pub async fn register(
        &self,
        cap_id: CapId,
        session_id: Option<String>,
        disposable: Option<Arc<dyn Disposable>>,
    ) {
        // Initialize reference count
        self.ref_counts.insert(cap_id, 1);

        // Register disposal callback if provided
        if let Some(disposable) = disposable {
            self.dispose_callbacks.insert(cap_id, disposable);
        }

        // Track session ownership if provided
        if let Some(session_id) = session_id {
            let session_caps = self.session_caps.write().await;
            session_caps
                .entry(session_id)
                .or_insert_with(Vec::new)
                .push(cap_id);
        }

        debug!("Registered capability {:?}", cap_id);
    }

    /// Increment reference count for a capability
    pub fn retain(&self, cap_id: &CapId) -> Result<(), RpcError> {
        if let Some(mut count) = self.ref_counts.get_mut(cap_id) {
            *count += 1;
            debug!("Retained capability {:?}, ref_count = {}", cap_id, *count);
            Ok(())
        } else {
            Err(RpcError::not_found(format!(
                "Capability {:?} not found",
                cap_id
            )))
        }
    }

    /// Decrement reference count and dispose if it reaches zero
    pub async fn release(&self, cap_id: &CapId) -> Result<bool, RpcError> {
        let should_dispose = {
            let mut should_dispose = false;

            if let Some(mut count) = self.ref_counts.get_mut(cap_id) {
                *count = count.saturating_sub(1);
                debug!("Released capability {:?}, ref_count = {}", cap_id, *count);

                if *count == 0 {
                    should_dispose = true;
                }
            } else {
                return Err(RpcError::not_found(format!(
                    "Capability {:?} not found",
                    cap_id
                )));
            }

            should_dispose
        };

        if should_dispose {
            self.dispose_internal(cap_id).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Force disposal of a capability regardless of reference count
    pub async fn dispose(&self, cap_id: &CapId) -> Result<(), RpcError> {
        debug!("Force disposing capability {:?}", cap_id);
        self.dispose_internal(cap_id).await
    }

    /// Internal disposal implementation
    async fn dispose_internal(&self, cap_id: &CapId) -> Result<(), RpcError> {
        // Remove from reference counts
        self.ref_counts.remove(cap_id);

        // Call disposal callback if exists
        if let Some((_, disposable)) = self.dispose_callbacks.remove(cap_id) {
            debug!("Calling disposal callback for capability {:?}", cap_id);
            if let Err(e) = disposable.dispose().await {
                warn!(
                    "Disposal callback failed for capability {:?}: {}",
                    cap_id, e
                );
                return Err(e);
            }
        }

        // Remove from session tracking
        let session_caps = self.session_caps.write().await;
        for mut caps in session_caps.iter_mut() {
            caps.retain(|&id| id != *cap_id);
        }

        debug!("Disposed capability {:?}", cap_id);
        Ok(())
    }

    /// Dispose all capabilities owned by a session
    pub async fn dispose_session(&self, session_id: &str) -> Result<(), RpcError> {
        debug!("Disposing all capabilities for session {}", session_id);

        let cap_ids = {
            let session_caps = self.session_caps.read().await;
            session_caps
                .get(session_id)
                .map(|caps| caps.clone())
                .unwrap_or_default()
        };

        let mut errors = Vec::new();
        for cap_id in cap_ids {
            if let Err(e) = self.dispose(&cap_id).await {
                errors.push(format!("{:?}: {}", cap_id, e));
            }
        }

        // Remove session from tracking
        {
            let session_caps = self.session_caps.write().await;
            session_caps.remove(session_id);
        }

        if !errors.is_empty() {
            Err(RpcError::internal(format!(
                "Failed to dispose some capabilities: {}",
                errors.join(", ")
            )))
        } else {
            Ok(())
        }
    }

    /// Get reference count for a capability
    pub fn ref_count(&self, cap_id: &CapId) -> Option<usize> {
        self.ref_counts.get(cap_id).map(|count| *count)
    }

    /// Check if a capability is alive (has references)
    pub fn is_alive(&self, cap_id: &CapId) -> bool {
        self.ref_count(cap_id)
            .map(|count| count > 0)
            .unwrap_or(false)
    }

    /// Get all capabilities for a session
    pub async fn session_capabilities(&self, session_id: &str) -> Vec<CapId> {
        let session_caps = self.session_caps.read().await;
        session_caps
            .get(session_id)
            .map(|caps| caps.clone())
            .unwrap_or_default()
    }

    /// Get lifecycle statistics
    pub async fn stats(&self) -> LifecycleStats {
        let session_caps = self.session_caps.read().await;
        let total_sessions = session_caps.len();
        let total_caps = self.ref_counts.len();
        let with_callbacks = self.dispose_callbacks.len();

        LifecycleStats {
            total_capabilities: total_caps,
            with_dispose_callbacks: with_callbacks,
            total_sessions,
        }
    }
}

impl Default for CapabilityLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LifecycleStats {
    pub total_capabilities: usize,
    pub with_dispose_callbacks: usize,
    pub total_sessions: usize,
}

/// Example disposable resource
pub struct DisposableResource {
    name: String,
}

impl DisposableResource {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl Disposable for DisposableResource {
    async fn dispose(&self) -> Result<(), RpcError> {
        debug!("Disposing resource: {}", self.name);
        // Cleanup logic here (close files, connections, etc.)
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capability_registration() {
        let lifecycle = CapabilityLifecycle::new();
        let cap_id = CapId::new(1);

        lifecycle.register(cap_id, None, None).await;
        assert_eq!(lifecycle.ref_count(&cap_id), Some(1));
        assert!(lifecycle.is_alive(&cap_id));
    }

    #[tokio::test]
    async fn test_retain_and_release() {
        let lifecycle = CapabilityLifecycle::new();
        let cap_id = CapId::new(1);

        lifecycle.register(cap_id, None, None).await;
        assert_eq!(lifecycle.ref_count(&cap_id), Some(1));

        lifecycle.retain(&cap_id).unwrap();
        assert_eq!(lifecycle.ref_count(&cap_id), Some(2));

        let disposed = lifecycle.release(&cap_id).await.unwrap();
        assert!(!disposed);
        assert_eq!(lifecycle.ref_count(&cap_id), Some(1));

        let disposed = lifecycle.release(&cap_id).await.unwrap();
        assert!(disposed);
        assert_eq!(lifecycle.ref_count(&cap_id), None);
    }

    #[tokio::test]
    async fn test_disposal_callback() {
        let lifecycle = CapabilityLifecycle::new();
        let cap_id = CapId::new(1);

        let resource = Arc::new(DisposableResource::new("test".to_string()));
        lifecycle.register(cap_id, None, Some(resource)).await;

        lifecycle.dispose(&cap_id).await.unwrap();
        assert!(!lifecycle.is_alive(&cap_id));
    }

    #[tokio::test]
    async fn test_session_management() {
        let lifecycle = CapabilityLifecycle::new();
        let session_id = "session1".to_string();

        let cap1 = CapId::new(1);
        let cap2 = CapId::new(2);

        lifecycle
            .register(cap1, Some(session_id.clone()), None)
            .await;
        lifecycle
            .register(cap2, Some(session_id.clone()), None)
            .await;

        let caps = lifecycle.session_capabilities(&session_id).await;
        assert_eq!(caps.len(), 2);

        lifecycle.dispose_session(&session_id).await.unwrap();

        assert!(!lifecycle.is_alive(&cap1));
        assert!(!lifecycle.is_alive(&cap2));

        let caps = lifecycle.session_capabilities(&session_id).await;
        assert_eq!(caps.len(), 0);
    }

    #[tokio::test]
    async fn test_force_dispose() {
        let lifecycle = CapabilityLifecycle::new();
        let cap_id = CapId::new(1);

        lifecycle.register(cap_id, None, None).await;
        lifecycle.retain(&cap_id).unwrap();
        lifecycle.retain(&cap_id).unwrap();
        assert_eq!(lifecycle.ref_count(&cap_id), Some(3));

        lifecycle.dispose(&cap_id).await.unwrap();
        assert!(!lifecycle.is_alive(&cap_id));
    }

    #[tokio::test]
    async fn test_lifecycle_stats() {
        let lifecycle = CapabilityLifecycle::new();

        lifecycle
            .register(CapId::new(1), Some("s1".to_string()), None)
            .await;
        lifecycle
            .register(
                CapId::new(2),
                Some("s1".to_string()),
                Some(Arc::new(DisposableResource::new("r1".to_string()))),
            )
            .await;
        lifecycle
            .register(CapId::new(3), Some("s2".to_string()), None)
            .await;

        let stats = lifecycle.stats().await;
        assert_eq!(stats.total_capabilities, 3);
        assert_eq!(stats.with_dispose_callbacks, 1);
        assert_eq!(stats.total_sessions, 2);
    }
}
