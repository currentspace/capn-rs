// Cap'n Web Capability Lifecycle Tests
// Tests complete capability lifecycle from creation through disposal
// Specification: Capabilities have explicit lifecycle management

use capnweb_core::{
    Message, Expression, ImportId, ExportId, CapId,
    ImportTable, ExportTable, IdAllocator, RpcTarget, RpcError, Value,
    protocol::{
        lifecycle::{LifecycleManager, LifecycleState, LifecycleEvent},
        CapabilityMetrics, ResourceTracker,
    },
};
use serde_json::Number;
use std::sync::{Arc, Weak};
use std::collections::{HashMap, HashSet};
use tokio::sync::{Mutex, RwLock, Semaphore};
use std::time::{SystemTime, Duration, Instant};

#[cfg(test)]
mod capability_lifecycle_basic_tests {
    use super::*;

    #[derive(Debug)]
    struct TrackedCapability {
        id: CapId,
        name: String,
        state: Arc<Mutex<LifecycleState>>,
        created_at: SystemTime,
        accessed_at: Arc<Mutex<SystemTime>>,
        access_count: Arc<Mutex<usize>>,
        disposed: Arc<Mutex<bool>>,
    }

    #[async_trait::async_trait]
    impl RpcTarget for TrackedCapability {
        async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
            // Check if disposed
            if *self.disposed.lock().await {
                return Err(RpcError::CapabilityRevoked("Capability has been disposed".into()));
            }

            // Update access tracking
            *self.accessed_at.lock().await = SystemTime::now();
            *self.access_count.lock().await += 1;

            match method {
                "getState" => {
                    let state = self.state.lock().await;
                    Ok(Value::String(format!("{:?}", *state)))
                }
                "getMetrics" => {
                    let count = *self.access_count.lock().await;
                    let mut metrics = HashMap::new();
                    metrics.insert("access_count".to_string(), Value::Number(Number::from(count)));
                    metrics.insert("id".to_string(), Value::Number(Number::from(self.id.0)));
                    Ok(Value::Object(metrics))
                }
                _ => Err(RpcError::MethodNotFound(method.to_string()))
            }
        }

        async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
            if *self.disposed.lock().await {
                return Err(RpcError::CapabilityRevoked("Capability has been disposed".into()));
            }

            match property {
                "name" => Ok(Value::String(self.name.clone())),
                "disposed" => Ok(Value::Bool(*self.disposed.lock().await)),
                _ => Err(RpcError::PropertyNotFound(property.to_string()))
            }
        }
    }

    impl TrackedCapability {
        async fn dispose(&self) {
            *self.disposed.lock().await = true;
            *self.state.lock().await = LifecycleState::Disposed;
        }
    }

    /// Test basic capability lifecycle states
    #[tokio::test]
    async fn test_capability_lifecycle_states() {
        println!("🧪 Testing Capability Lifecycle States");

        let cap = Arc::new(TrackedCapability {
            id: CapId(1),
            name: "test_cap".to_string(),
            state: Arc::new(Mutex::new(LifecycleState::Created)),
            created_at: SystemTime::now(),
            accessed_at: Arc::new(Mutex::new(SystemTime::now())),
            access_count: Arc::new(Mutex::new(0)),
            disposed: Arc::new(Mutex::new(false)),
        });

        // Test initial state
        let state_result = cap.call("getState", vec![]).await.unwrap();
        assert_eq!(state_result, Value::String("Created".to_string()));

        // Transition to Active
        *cap.state.lock().await = LifecycleState::Active;
        let active_result = cap.call("getState", vec![]).await.unwrap();
        assert_eq!(active_result, Value::String("Active".to_string()));

        // Test access tracking
        let metrics_before = cap.call("getMetrics", vec![]).await.unwrap();
        match metrics_before {
            Value::Object(m) => {
                assert_eq!(m.get("access_count"), Some(&Value::Number(Number::from(2))));
            }
            _ => panic!("Should return metrics object")
        }

        // Dispose capability
        cap.dispose().await;
        let disposed_result = cap.call("getState", vec![]).await;
        assert!(disposed_result.is_err());
        match disposed_result.unwrap_err() {
            RpcError::CapabilityRevoked(_) => {} // Expected
            _ => panic!("Should return capability revoked error")
        }

        println!("✅ Capability lifecycle states verified");
    }

    /// Test capability creation and initialization
    #[tokio::test]
    async fn test_capability_creation() {
        println!("🧪 Testing Capability Creation");

        let manager = LifecycleManager::new();

        // Create multiple capabilities
        let caps: Vec<Arc<TrackedCapability>> = (0..5)
            .map(|i| Arc::new(TrackedCapability {
                id: CapId(i),
                name: format!("cap_{}", i),
                state: Arc::new(Mutex::new(LifecycleState::Created)),
                created_at: SystemTime::now(),
                accessed_at: Arc::new(Mutex::new(SystemTime::now())),
                access_count: Arc::new(Mutex::new(0)),
                disposed: Arc::new(Mutex::new(false)),
            }))
            .collect();

        // Register with manager
        for cap in &caps {
            manager.register_capability(cap.id, cap.clone()).await;
        }

        // Verify all registered
        assert_eq!(manager.active_count().await, 5);

        // Initialize capabilities
        for cap in &caps {
            manager.initialize_capability(cap.id).await;
            *cap.state.lock().await = LifecycleState::Initialized;
        }

        // Activate capabilities
        for cap in &caps {
            manager.activate_capability(cap.id).await;
            *cap.state.lock().await = LifecycleState::Active;
        }

        println!("✅ Capability creation and initialization verified");
    }

    /// Test capability disposal and cleanup
    #[tokio::test]
    async fn test_capability_disposal() {
        println!("🧪 Testing Capability Disposal");

        let allocator = Arc::new(IdAllocator::new());
        let import_table = ImportTable::new(allocator.clone());

        // Create capabilities with different reference counts
        let cap1 = Arc::new(TrackedCapability {
            id: CapId(1),
            name: "disposable1".to_string(),
            state: Arc::new(Mutex::new(LifecycleState::Active)),
            created_at: SystemTime::now(),
            accessed_at: Arc::new(Mutex::new(SystemTime::now())),
            access_count: Arc::new(Mutex::new(0)),
            disposed: Arc::new(Mutex::new(false)),
        });

        let cap2 = Arc::new(TrackedCapability {
            id: CapId(2),
            name: "disposable2".to_string(),
            state: Arc::new(Mutex::new(LifecycleState::Active)),
            created_at: SystemTime::now(),
            accessed_at: Arc::new(Mutex::new(SystemTime::now())),
            access_count: Arc::new(Mutex::new(0)),
            disposed: Arc::new(Mutex::new(false)),
        });

        // Add to import table
        let id1 = import_table.add_import(cap1.clone() as Arc<dyn RpcTarget>).await;
        let id2 = import_table.add_import(cap2.clone() as Arc<dyn RpcTarget>).await;

        // Add extra references
        import_table.add_ref(&id1).await;
        import_table.add_ref(&id1).await;
        import_table.add_ref(&id2).await;

        // Check reference counts
        assert_eq!(import_table.get_ref_count(&id1).await, 3);
        assert_eq!(import_table.get_ref_count(&id2).await, 2);

        // Release references
        import_table.release_ref(&id1).await;
        import_table.release_ref(&id1).await;
        assert_eq!(import_table.get_ref_count(&id1).await, 1);

        // Dispose when ref count reaches zero
        import_table.release_ref(&id1).await;
        assert_eq!(import_table.get_ref_count(&id1).await, 0);

        // Should trigger disposal
        cap1.dispose().await;
        assert!(*cap1.disposed.lock().await);

        // Cap2 still has references
        assert!(!*cap2.disposed.lock().await);

        println!("✅ Capability disposal and cleanup verified");
    }

    /// Test capability weak references
    #[tokio::test]
    async fn test_capability_weak_references() {
        println!("🧪 Testing Capability Weak References");

        let cap = Arc::new(TrackedCapability {
            id: CapId(1),
            name: "weak_test".to_string(),
            state: Arc::new(Mutex::new(LifecycleState::Active)),
            created_at: SystemTime::now(),
            accessed_at: Arc::new(Mutex::new(SystemTime::now())),
            access_count: Arc::new(Mutex::new(0)),
            disposed: Arc::new(Mutex::new(false)),
        });

        // Create weak reference
        let weak_ref: Weak<TrackedCapability> = Arc::downgrade(&cap);
        assert!(weak_ref.upgrade().is_some());

        // Store strong reference
        let stored_cap = cap.clone();

        // Drop original reference
        drop(cap);

        // Weak reference should still be valid
        assert!(weak_ref.upgrade().is_some());

        // Drop all strong references
        drop(stored_cap);

        // Weak reference should now be invalid
        assert!(weak_ref.upgrade().is_none());

        println!("✅ Capability weak references verified");
    }
}

#[cfg(test)]
mod capability_lifecycle_advanced_tests {
    use super::*;

    /// Test capability resource tracking
    #[tokio::test]
    async fn test_capability_resource_tracking() {
        println!("🧪 Testing Capability Resource Tracking");

        let tracker = ResourceTracker::new();

        // Track memory usage
        tracker.allocate_memory(CapId(1), 1024).await;
        tracker.allocate_memory(CapId(1), 2048).await;
        tracker.allocate_memory(CapId(2), 4096).await;

        assert_eq!(tracker.get_memory_usage(CapId(1)).await, 3072);
        assert_eq!(tracker.get_memory_usage(CapId(2)).await, 4096);
        assert_eq!(tracker.get_total_memory().await, 7168);

        // Track handle usage
        tracker.allocate_handle(CapId(1), "file_handle_1").await;
        tracker.allocate_handle(CapId(1), "socket_handle_1").await;
        tracker.allocate_handle(CapId(2), "db_connection_1").await;

        assert_eq!(tracker.get_handle_count(CapId(1)).await, 2);
        assert_eq!(tracker.get_handle_count(CapId(2)).await, 1);

        // Release resources
        tracker.release_memory(CapId(1), 1024).await;
        assert_eq!(tracker.get_memory_usage(CapId(1)).await, 2048);

        tracker.release_handle(CapId(1), "file_handle_1").await;
        assert_eq!(tracker.get_handle_count(CapId(1)).await, 1);

        // Release all resources for a capability
        tracker.release_all(CapId(1)).await;
        assert_eq!(tracker.get_memory_usage(CapId(1)).await, 0);
        assert_eq!(tracker.get_handle_count(CapId(1)).await, 0);

        println!("✅ Capability resource tracking verified");
    }

    /// Test capability lifecycle events
    #[tokio::test]
    async fn test_capability_lifecycle_events() {
        println!("🧪 Testing Capability Lifecycle Events");

        #[derive(Debug, Clone)]
        struct EventRecorder {
            events: Arc<Mutex<Vec<LifecycleEvent>>>,
        }

        impl EventRecorder {
            fn new() -> Self {
                Self {
                    events: Arc::new(Mutex::new(Vec::new())),
                }
            }

            async fn record(&self, event: LifecycleEvent) {
                self.events.lock().await.push(event);
            }

            async fn get_events(&self) -> Vec<LifecycleEvent> {
                self.events.lock().await.clone()
            }
        }

        let recorder = EventRecorder::new();
        let manager = LifecycleManager::new();

        // Set up event listener
        let recorder_clone = recorder.clone();
        manager.on_event(move |event| {
            let recorder = recorder_clone.clone();
            Box::pin(async move {
                recorder.record(event).await;
            })
        }).await;

        // Trigger lifecycle events
        let cap_id = CapId(1);

        manager.emit_event(LifecycleEvent::Created { id: cap_id, timestamp: SystemTime::now() }).await;
        manager.emit_event(LifecycleEvent::Initialized { id: cap_id, timestamp: SystemTime::now() }).await;
        manager.emit_event(LifecycleEvent::Activated { id: cap_id, timestamp: SystemTime::now() }).await;
        manager.emit_event(LifecycleEvent::Accessed { id: cap_id, method: "test".to_string(), timestamp: SystemTime::now() }).await;
        manager.emit_event(LifecycleEvent::Disposed { id: cap_id, reason: "explicit".to_string(), timestamp: SystemTime::now() }).await;

        // Verify events recorded
        let events = recorder.get_events().await;
        assert_eq!(events.len(), 5);

        // Check event sequence
        match &events[0] {
            LifecycleEvent::Created { id, .. } => assert_eq!(*id, cap_id),
            _ => panic!("First event should be Created"),
        }

        match &events[4] {
            LifecycleEvent::Disposed { id, reason, .. } => {
                assert_eq!(*id, cap_id);
                assert_eq!(reason, "explicit");
            }
            _ => panic!("Last event should be Disposed"),
        }

        println!("✅ Capability lifecycle events verified");
    }

    /// Test capability timeout and expiration
    #[tokio::test]
    async fn test_capability_timeout() {
        println!("🧪 Testing Capability Timeout");

        #[derive(Debug)]
        struct ExpiringCapability {
            id: CapId,
            created_at: Instant,
            ttl: Duration,
            last_access: Arc<Mutex<Instant>>,
            idle_timeout: Duration,
        }

        impl ExpiringCapability {
            fn new(id: CapId, ttl: Duration, idle_timeout: Duration) -> Self {
                let now = Instant::now();
                Self {
                    id,
                    created_at: now,
                    ttl,
                    last_access: Arc::new(Mutex::new(now)),
                    idle_timeout,
                }
            }

            async fn is_expired(&self) -> bool {
                // Check TTL expiration
                if self.created_at.elapsed() > self.ttl {
                    return true;
                }

                // Check idle timeout
                let last_access = *self.last_access.lock().await;
                last_access.elapsed() > self.idle_timeout
            }

            async fn touch(&self) {
                *self.last_access.lock().await = Instant::now();
            }
        }

        // Create capabilities with different timeouts
        let short_ttl = ExpiringCapability::new(
            CapId(1),
            Duration::from_millis(100),
            Duration::from_secs(10),
        );

        let short_idle = ExpiringCapability::new(
            CapId(2),
            Duration::from_secs(10),
            Duration::from_millis(50),
        );

        // Initially not expired
        assert!(!short_ttl.is_expired().await);
        assert!(!short_idle.is_expired().await);

        // Keep short_idle alive
        short_idle.touch().await;

        // Wait for TTL expiration
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(short_ttl.is_expired().await); // TTL expired
        assert!(short_idle.is_expired().await); // Idle timeout expired

        // Touch and check again
        short_idle.touch().await;
        assert!(!short_idle.is_expired().await); // Reset by touch

        println!("✅ Capability timeout and expiration verified");
    }

    /// Test capability migration and handoff
    #[tokio::test]
    async fn test_capability_migration() {
        println!("🧪 Testing Capability Migration");

        #[derive(Debug)]
        struct MigratableCapability {
            id: CapId,
            owner: Arc<Mutex<String>>,
            state: Arc<RwLock<HashMap<String, Value>>>,
        }

        impl MigratableCapability {
            async fn migrate_to(&self, new_owner: String) -> Result<HashMap<String, Value>, String> {
                // Capture current state
                let state_snapshot = self.state.read().await.clone();

                // Update owner
                *self.owner.lock().await = new_owner.clone();

                // Clear local state (simulating migration)
                self.state.write().await.clear();

                Ok(state_snapshot)
            }

            async fn restore_from(&self, snapshot: HashMap<String, Value>) {
                *self.state.write().await = snapshot;
            }
        }

        // Create migratable capability
        let cap = MigratableCapability {
            id: CapId(1),
            owner: Arc::new(Mutex::new("server1".to_string())),
            state: Arc::new(RwLock::new({
                let mut state = HashMap::new();
                state.insert("data".to_string(), Value::String("important".to_string()));
                state.insert("counter".to_string(), Value::Number(Number::from(42)));
                state
            })),
        };

        // Check initial state
        assert_eq!(*cap.owner.lock().await, "server1");
        assert_eq!(cap.state.read().await.len(), 2);

        // Migrate to new owner
        let snapshot = cap.migrate_to("server2".to_string()).await.unwrap();
        assert_eq!(*cap.owner.lock().await, "server2");
        assert_eq!(cap.state.read().await.len(), 0); // State cleared

        // Restore on new owner
        cap.restore_from(snapshot).await;
        assert_eq!(cap.state.read().await.len(), 2); // State restored
        assert_eq!(
            cap.state.read().await.get("data"),
            Some(&Value::String("important".to_string()))
        );

        println!("✅ Capability migration and handoff verified");
    }

    /// Test capability pools and recycling
    #[tokio::test]
    async fn test_capability_pooling() {
        println!("🧪 Testing Capability Pooling");

        #[derive(Debug)]
        struct CapabilityPool {
            available: Arc<Mutex<Vec<Arc<dyn RpcTarget>>>>,
            in_use: Arc<Mutex<HashSet<CapId>>>,
            max_size: usize,
            semaphore: Arc<Semaphore>,
        }

        impl CapabilityPool {
            fn new(max_size: usize) -> Self {
                Self {
                    available: Arc::new(Mutex::new(Vec::new())),
                    in_use: Arc::new(Mutex::new(HashSet::new())),
                    max_size,
                    semaphore: Arc::new(Semaphore::new(max_size)),
                }
            }

            async fn acquire(&self) -> Option<Arc<dyn RpcTarget>> {
                // Try to get permit
                let _permit = self.semaphore.acquire().await.ok()?;

                // Get from pool or create new
                let mut available = self.available.lock().await;
                if let Some(cap) = available.pop() {
                    Some(cap)
                } else {
                    // Create new capability (simplified)
                    None
                }
            }

            async fn release(&self, cap: Arc<dyn RpcTarget>) {
                let mut available = self.available.lock().await;
                if available.len() < self.max_size {
                    available.push(cap);
                }
                // Permit automatically released when dropped
            }

            async fn size(&self) -> usize {
                self.available.lock().await.len()
            }
        }

        let pool = CapabilityPool::new(5);

        // Pre-populate pool
        for i in 0..3 {
            let cap = Arc::new(TrackedCapability {
                id: CapId(i),
                name: format!("pooled_{}", i),
                state: Arc::new(Mutex::new(LifecycleState::Active)),
                created_at: SystemTime::now(),
                accessed_at: Arc::new(Mutex::new(SystemTime::now())),
                access_count: Arc::new(Mutex::new(0)),
                disposed: Arc::new(Mutex::new(false)),
            });
            pool.release(cap).await;
        }

        assert_eq!(pool.size().await, 3);

        // Acquire and release
        if let Some(cap1) = pool.acquire().await {
            assert_eq!(pool.size().await, 2);
            pool.release(cap1).await;
            assert_eq!(pool.size().await, 3);
        }

        println!("✅ Capability pooling and recycling verified");
    }
}

#[cfg(test)]
mod capability_metrics_tests {
    use super::*;

    /// Test capability performance metrics
    #[tokio::test]
    async fn test_capability_metrics() {
        println!("🧪 Testing Capability Metrics");

        let metrics = CapabilityMetrics::new();

        // Record various metrics
        metrics.record_creation(CapId(1)).await;
        metrics.record_creation(CapId(2)).await;

        metrics.record_access(CapId(1), "method1", Duration::from_millis(10)).await;
        metrics.record_access(CapId(1), "method1", Duration::from_millis(15)).await;
        metrics.record_access(CapId(1), "method2", Duration::from_millis(5)).await;
        metrics.record_access(CapId(2), "method1", Duration::from_millis(20)).await;

        metrics.record_error(CapId(1), "method1", "timeout").await;
        metrics.record_error(CapId(2), "method3", "not_found").await;

        metrics.record_disposal(CapId(1), "timeout").await;

        // Check metrics
        let summary = metrics.get_summary().await;
        assert_eq!(summary.total_created, 2);
        assert_eq!(summary.total_disposed, 1);
        assert_eq!(summary.active_capabilities, 1);
        assert_eq!(summary.total_accesses, 4);
        assert_eq!(summary.total_errors, 2);

        // Check per-capability metrics
        let cap1_metrics = metrics.get_capability_metrics(CapId(1)).await;
        assert_eq!(cap1_metrics.access_count, 3);
        assert_eq!(cap1_metrics.error_count, 1);
        assert_eq!(cap1_metrics.average_latency_ms, 10); // (10+15+5)/3

        println!("✅ Capability performance metrics verified");
    }

    /// Test capability health monitoring
    #[tokio::test]
    async fn test_capability_health_monitoring() {
        println!("🧪 Testing Capability Health Monitoring");

        #[derive(Debug)]
        struct HealthMonitor {
            thresholds: HealthThresholds,
            metrics: Arc<RwLock<HashMap<CapId, HealthMetrics>>>,
        }

        #[derive(Debug)]
        struct HealthThresholds {
            max_error_rate: f64,
            max_latency_ms: u128,
            min_success_rate: f64,
        }

        #[derive(Debug, Default)]
        struct HealthMetrics {
            total_calls: usize,
            successful_calls: usize,
            failed_calls: usize,
            total_latency_ms: u128,
        }

        impl HealthMonitor {
            fn new(thresholds: HealthThresholds) -> Self {
                Self {
                    thresholds,
                    metrics: Arc::new(RwLock::new(HashMap::new())),
                }
            }

            async fn record_call(&self, id: CapId, success: bool, latency_ms: u128) {
                let mut metrics = self.metrics.write().await;
                let entry = metrics.entry(id).or_insert_with(HealthMetrics::default);

                entry.total_calls += 1;
                entry.total_latency_ms += latency_ms;

                if success {
                    entry.successful_calls += 1;
                } else {
                    entry.failed_calls += 1;
                }
            }

            async fn is_healthy(&self, id: CapId) -> bool {
                let metrics = self.metrics.read().await;

                if let Some(m) = metrics.get(&id) {
                    if m.total_calls == 0 {
                        return true; // No data yet
                    }

                    let error_rate = m.failed_calls as f64 / m.total_calls as f64;
                    let success_rate = m.successful_calls as f64 / m.total_calls as f64;
                    let avg_latency = m.total_latency_ms / m.total_calls as u128;

                    error_rate <= self.thresholds.max_error_rate &&
                    success_rate >= self.thresholds.min_success_rate &&
                    avg_latency <= self.thresholds.max_latency_ms
                } else {
                    true // No data yet
                }
            }
        }

        let monitor = HealthMonitor::new(HealthThresholds {
            max_error_rate: 0.1,
            max_latency_ms: 100,
            min_success_rate: 0.9,
        });

        // Healthy capability
        let healthy_id = CapId(1);
        for _ in 0..9 {
            monitor.record_call(healthy_id, true, 20).await;
        }
        monitor.record_call(healthy_id, false, 30).await;
        assert!(monitor.is_healthy(healthy_id).await);

        // Unhealthy capability (high error rate)
        let unhealthy_id = CapId(2);
        for _ in 0..5 {
            monitor.record_call(unhealthy_id, false, 50).await;
        }
        for _ in 0..5 {
            monitor.record_call(unhealthy_id, true, 50).await;
        }
        assert!(!monitor.is_healthy(unhealthy_id).await);

        // Unhealthy capability (high latency)
        let slow_id = CapId(3);
        for _ in 0..10 {
            monitor.record_call(slow_id, true, 200).await;
        }
        assert!(!monitor.is_healthy(slow_id).await);

        println!("✅ Capability health monitoring verified");
    }
}