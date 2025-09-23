use capnweb_core::{CallId, PromiseId, PendingPromise, PromiseDependencyGraph, RpcError};
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::RwLock;

/// Table for tracking pending promises and their resolution
pub struct PromiseTable {
    /// Maps PromiseId to PendingPromise
    promises: Arc<DashMap<PromiseId, Arc<RwLock<PendingPromise>>>>,
    /// Maps CallId to PromiseId for result correlation
    call_to_promise: Arc<DashMap<CallId, PromiseId>>,
    /// Dependency graph for topological sorting
    dependency_graph: Arc<RwLock<PromiseDependencyGraph>>,
    /// Set of resolved promise IDs
    resolved_promises: Arc<RwLock<HashSet<PromiseId>>>,
}

impl PromiseTable {
    pub fn new() -> Self {
        Self {
            promises: Arc::new(DashMap::new()),
            call_to_promise: Arc::new(DashMap::new()),
            dependency_graph: Arc::new(RwLock::new(PromiseDependencyGraph::new())),
            resolved_promises: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Register a new pending promise
    pub async fn register_promise(&self, promise_id: PromiseId, call_id: CallId) {
        let promise = Arc::new(RwLock::new(PendingPromise::new(promise_id, call_id)));
        self.promises.insert(promise_id, promise);
        self.call_to_promise.insert(call_id, promise_id);
    }

    /// Add a dependency between promises
    pub async fn add_dependency(
        &self,
        promise_id: PromiseId,
        depends_on: PromiseId,
    ) -> Result<(), RpcError> {
        let mut graph = self.dependency_graph.write().await;

        // Check for cycles
        if graph.would_create_cycle(promise_id, depends_on) {
            return Err(RpcError::bad_request(format!(
                "Adding dependency from {:?} to {:?} would create a cycle",
                promise_id, depends_on
            )));
        }

        graph.add_dependency(promise_id, depends_on);

        // Update the promise's dependencies
        if let Some(promise_arc) = self.promises.get(&promise_id) {
            let mut promise = promise_arc.write().await;
            promise.add_dependency(depends_on);
        }

        Ok(())
    }

    /// Resolve a promise by call ID
    pub async fn resolve_by_call(&self, call_id: CallId, result: Value) -> Option<PromiseId> {
        if let Some(promise_id) = self.call_to_promise.remove(&call_id) {
            let promise_id = promise_id.1; // Extract value from DashMap entry
            self.resolve_promise(promise_id, result).await;
            Some(promise_id)
        } else {
            None
        }
    }

    /// Resolve a promise directly
    pub async fn resolve_promise(&self, promise_id: PromiseId, result: Value) {
        if let Some(promise_arc) = self.promises.get(&promise_id) {
            let mut promise = promise_arc.write().await;
            promise.resolve(result);
        }

        let mut resolved = self.resolved_promises.write().await;
        resolved.insert(promise_id);
    }

    /// Get promises that are ready to execute (all dependencies resolved)
    pub async fn get_ready_promises(&self) -> Vec<PromiseId> {
        let resolved = self.resolved_promises.read().await;
        let mut ready = Vec::new();

        for entry in self.promises.iter() {
            let promise_id = *entry.key();
            let promise = entry.value().read().await;

            if !promise.resolved && promise.is_ready(&resolved) {
                ready.push(promise_id);
            }
        }

        ready
    }

    /// Get the result of a resolved promise
    pub async fn get_result(&self, promise_id: &PromiseId) -> Option<Value> {
        if let Some(promise_arc) = self.promises.get(promise_id) {
            let promise = promise_arc.read().await;
            promise.result.clone()
        } else {
            None
        }
    }

    /// Get all promises in topologically sorted order
    pub async fn get_execution_order(&self) -> Option<Vec<PromiseId>> {
        let graph = self.dependency_graph.read().await;
        graph.topological_sort()
    }

    /// Check if a promise is resolved
    pub async fn is_resolved(&self, promise_id: &PromiseId) -> bool {
        let resolved = self.resolved_promises.read().await;
        resolved.contains(promise_id)
    }

    /// Clear all resolved promises (cleanup)
    pub async fn clear_resolved(&self) {
        let resolved = self.resolved_promises.read().await;
        for promise_id in resolved.iter() {
            self.promises.remove(promise_id);
        }
        drop(resolved);

        let mut resolved = self.resolved_promises.write().await;
        resolved.clear();
    }

    /// Get statistics about the promise table
    pub async fn stats(&self) -> PromiseTableStats {
        let resolved = self.resolved_promises.read().await;
        PromiseTableStats {
            total_promises: self.promises.len(),
            resolved_promises: resolved.len(),
            pending_promises: self.promises.len() - resolved.len(),
        }
    }
}

impl Default for PromiseTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PromiseTableStats {
    pub total_promises: usize,
    pub resolved_promises: usize,
    pub pending_promises: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_promise_registration() {
        let table = PromiseTable::new();
        let promise_id = PromiseId::new(1);
        let call_id = CallId::new(1);

        table.register_promise(promise_id, call_id).await;

        let stats = table.stats().await;
        assert_eq!(stats.total_promises, 1);
        assert_eq!(stats.pending_promises, 1);
        assert_eq!(stats.resolved_promises, 0);
    }

    #[tokio::test]
    async fn test_promise_resolution() {
        let table = PromiseTable::new();
        let promise_id = PromiseId::new(1);
        let call_id = CallId::new(1);

        table.register_promise(promise_id, call_id).await;
        table.resolve_by_call(call_id, json!({"result": true})).await;

        assert!(table.is_resolved(&promise_id).await);

        let result = table.get_result(&promise_id).await;
        assert_eq!(result, Some(json!({"result": true})));
    }

    #[tokio::test]
    async fn test_dependencies() {
        let table = PromiseTable::new();

        let p1 = PromiseId::new(1);
        let p2 = PromiseId::new(2);
        let p3 = PromiseId::new(3);

        table.register_promise(p1, CallId::new(1)).await;
        table.register_promise(p2, CallId::new(2)).await;
        table.register_promise(p3, CallId::new(3)).await;

        // p2 depends on p1
        table.add_dependency(p2, p1).await.unwrap();
        // p3 depends on p2
        table.add_dependency(p3, p2).await.unwrap();

        // Initially, only p1 should be ready
        let ready = table.get_ready_promises().await;
        assert_eq!(ready, vec![p1]);

        // After resolving p1, p2 should be ready
        table.resolve_promise(p1, json!(1)).await;
        let ready = table.get_ready_promises().await;
        assert_eq!(ready, vec![p2]);

        // After resolving p2, p3 should be ready
        table.resolve_promise(p2, json!(2)).await;
        let ready = table.get_ready_promises().await;
        assert_eq!(ready, vec![p3]);
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let table = PromiseTable::new();

        let p1 = PromiseId::new(1);
        let p2 = PromiseId::new(2);

        table.register_promise(p1, CallId::new(1)).await;
        table.register_promise(p2, CallId::new(2)).await;

        // p2 depends on p1
        table.add_dependency(p2, p1).await.unwrap();

        // p1 depends on p2 would create a cycle
        let result = table.add_dependency(p1, p2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execution_order() {
        let table = PromiseTable::new();

        let p1 = PromiseId::new(1);
        let p2 = PromiseId::new(2);
        let p3 = PromiseId::new(3);

        table.register_promise(p1, CallId::new(1)).await;
        table.register_promise(p2, CallId::new(2)).await;
        table.register_promise(p3, CallId::new(3)).await;

        table.add_dependency(p2, p1).await.unwrap();
        table.add_dependency(p3, p2).await.unwrap();

        let order = table.get_execution_order().await.unwrap();
        assert_eq!(order, vec![p1, p2, p3]);
    }

    #[tokio::test]
    async fn test_clear_resolved() {
        let table = PromiseTable::new();

        for i in 1..=3 {
            let pid = PromiseId::new(i);
            let cid = CallId::new(i);
            table.register_promise(pid, cid).await;
            table.resolve_promise(pid, json!(i)).await;
        }

        let stats = table.stats().await;
        assert_eq!(stats.resolved_promises, 3);

        table.clear_resolved().await;

        let stats = table.stats().await;
        assert_eq!(stats.total_promises, 0);
        assert_eq!(stats.resolved_promises, 0);
    }
}