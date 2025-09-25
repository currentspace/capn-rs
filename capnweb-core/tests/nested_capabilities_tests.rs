// Cap'n Web Nested Capabilities and Advanced Features Tests
// Tests capability nesting, composition, and advanced protocol features
// Specification: Capabilities can contain and return other capabilities

use capnweb_core::{
    Message, Expression, ImportId, ExportId, CapId,
    ImportTable, ExportTable, IdAllocator, RpcTarget, RpcError,
    protocol::{Value, nested_capabilities::{NestedCapabilityManager, CapabilityGraph}},
};
use serde_json::Number;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use tokio::sync::{Mutex, RwLock};

#[cfg(test)]
mod nested_capability_tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct CompositeCapability {
        name: String,
        sub_capabilities: HashMap<String, Arc<dyn RpcTarget>>,
        parent: Option<Arc<dyn RpcTarget>>,
    }

    #[async_trait::async_trait]
    impl RpcTarget for CompositeCapability {
        async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
            match method {
                "getSubCapability" => {
                    if args.is_empty() {
                        return Err(RpcError::InvalidArguments("Missing capability name".into()));
                    }
                    match &args[0] {
                        Value::String(name) => {
                            self.sub_capabilities.get(name)
                                .map(|cap| Value::Capability(CapId(name.len() as i64)))
                                .ok_or_else(|| RpcError::NotFound(format!("Capability {} not found", name)))
                        }
                        _ => Err(RpcError::InvalidArguments("Name must be string".into()))
                    }
                }
                "listCapabilities" => {
                    let names: Vec<Value> = self.sub_capabilities.keys()
                        .map(|k| Value::String(k.clone()))
                        .collect();
                    Ok(Value::Array(names))
                }
                "getParent" => {
                    if self.parent.is_some() {
                        Ok(Value::Capability(CapId(-1)))
                    } else {
                        Ok(Value::Null)
                    }
                }
                _ => Err(RpcError::MethodNotFound(method.to_string()))
            }
        }

        async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
            match property {
                "name" => Ok(Value::String(self.name.clone())),
                "depth" => {
                    let mut depth = 0;
                    let mut current = self.parent.as_ref();
                    while current.is_some() {
                        depth += 1;
                        // In real implementation, would traverse parent chain
                        break;
                    }
                    Ok(Value::Number(Number::from(depth)))
                }
                _ => Err(RpcError::PropertyNotFound(property.to_string()))
            }
        }
    }

    /// Test basic nested capability creation
    #[tokio::test]
    async fn test_nested_capability_creation() {
        println!("🧪 Testing Nested Capability Creation");

        let allocator = Arc::new(IdAllocator::new());
        let import_table = ImportTable::new(allocator.clone());

        // Create parent capability
        let parent = Arc::new(CompositeCapability {
            name: "parent".to_string(),
            sub_capabilities: HashMap::new(),
            parent: None,
        });

        // Create child capabilities
        let child1 = Arc::new(CompositeCapability {
            name: "child1".to_string(),
            sub_capabilities: HashMap::new(),
            parent: Some(parent.clone()),
        });

        let child2 = Arc::new(CompositeCapability {
            name: "child2".to_string(),
            sub_capabilities: HashMap::new(),
            parent: Some(parent.clone()),
        });

        // Create grandchild capability
        let grandchild = Arc::new(CompositeCapability {
            name: "grandchild".to_string(),
            sub_capabilities: HashMap::new(),
            parent: Some(child1.clone()),
        });

        // Add to import table
        let parent_id = import_table.add_import(parent.clone()).await;
        let child1_id = import_table.add_import(child1.clone()).await;
        let child2_id = import_table.add_import(child2.clone()).await;
        let grandchild_id = import_table.add_import(grandchild.clone()).await;

        assert!(parent_id.0 > 0);
        assert!(child1_id.0 > parent_id.0);
        assert!(child2_id.0 > child1_id.0);
        assert!(grandchild_id.0 > child2_id.0);

        println!("✅ Nested capability hierarchy created");

        // Test retrieval
        let retrieved_parent = import_table.get_import(&parent_id).await;
        assert!(retrieved_parent.is_some());

        let retrieved_grandchild = import_table.get_import(&grandchild_id).await;
        assert!(retrieved_grandchild.is_some());

        println!("✅ Nested capability retrieval verified");
    }

    /// Test capability graph and relationships
    #[tokio::test]
    async fn test_capability_graph() {
        println!("🧪 Testing Capability Graph");

        let graph = CapabilityGraph::new();

        // Build capability graph
        let root_id = CapId(1);
        let service_id = CapId(2);
        let database_id = CapId(3);
        let auth_id = CapId(4);
        let cache_id = CapId(5);

        graph.add_node(root_id, "root".to_string()).await;
        graph.add_node(service_id, "service".to_string()).await;
        graph.add_node(database_id, "database".to_string()).await;
        graph.add_node(auth_id, "auth".to_string()).await;
        graph.add_node(cache_id, "cache".to_string()).await;

        // Add edges (dependencies)
        graph.add_edge(root_id, service_id).await;
        graph.add_edge(root_id, auth_id).await;
        graph.add_edge(service_id, database_id).await;
        graph.add_edge(service_id, cache_id).await;
        graph.add_edge(auth_id, database_id).await;

        // Test reachability
        assert!(graph.is_reachable(root_id, database_id).await);
        assert!(graph.is_reachable(root_id, cache_id).await);
        assert!(graph.is_reachable(service_id, database_id).await);
        assert!(!graph.is_reachable(database_id, root_id).await);
        assert!(!graph.is_reachable(cache_id, auth_id).await);

        println!("✅ Capability graph relationships verified");

        // Test dependency detection
        let service_deps = graph.get_dependencies(service_id).await;
        assert!(service_deps.contains(&database_id));
        assert!(service_deps.contains(&cache_id));

        let root_deps = graph.get_all_dependencies(root_id).await;
        assert_eq!(root_deps.len(), 4); // All other nodes

        println!("✅ Capability dependency tracking verified");

        // Test cycle detection
        assert!(!graph.has_cycle().await);
        graph.add_edge(database_id, service_id).await; // Create cycle
        assert!(graph.has_cycle().await);

        println!("✅ Capability cycle detection verified");
    }

    /// Test capability composition and delegation
    #[tokio::test]
    async fn test_capability_composition() {
        println!("🧪 Testing Capability Composition");

        // Create composite capability with multiple sub-capabilities
        let mut sub_caps = HashMap::new();

        let logger = Arc::new(CompositeCapability {
            name: "logger".to_string(),
            sub_capabilities: HashMap::new(),
            parent: None,
        });

        let metrics = Arc::new(CompositeCapability {
            name: "metrics".to_string(),
            sub_capabilities: HashMap::new(),
            parent: None,
        });

        let storage = Arc::new(CompositeCapability {
            name: "storage".to_string(),
            sub_capabilities: HashMap::new(),
            parent: None,
        });

        sub_caps.insert("logger".to_string(), logger.clone() as Arc<dyn RpcTarget>);
        sub_caps.insert("metrics".to_string(), metrics.clone() as Arc<dyn RpcTarget>);
        sub_caps.insert("storage".to_string(), storage.clone() as Arc<dyn RpcTarget>);

        let composite = Arc::new(CompositeCapability {
            name: "application".to_string(),
            sub_capabilities: sub_caps,
            parent: None,
        });

        // Test sub-capability access
        let list_result = composite.call("listCapabilities", vec![]).await.unwrap();
        match list_result {
            Value::Array(caps) => {
                assert_eq!(caps.len(), 3);
                let cap_names: HashSet<String> = caps.iter()
                    .filter_map(|v| match v {
                        Value::String(s) => Some(s.clone()),
                        _ => None
                    })
                    .collect();
                assert!(cap_names.contains("logger"));
                assert!(cap_names.contains("metrics"));
                assert!(cap_names.contains("storage"));
            }
            _ => panic!("Should return array of capabilities")
        }

        println!("✅ Capability composition verified");

        // Test sub-capability retrieval
        let logger_cap = composite.call("getSubCapability", vec![Value::String("logger".to_string())]).await.unwrap();
        match logger_cap {
            Value::Capability(_) => {} // Success
            _ => panic!("Should return capability")
        }

        let invalid_cap = composite.call("getSubCapability", vec![Value::String("invalid".to_string())]).await;
        assert!(invalid_cap.is_err());

        println!("✅ Sub-capability retrieval verified");
    }

    /// Test capability lifetime and disposal
    #[tokio::test]
    async fn test_capability_lifetime() {
        println!("🧪 Testing Capability Lifetime");

        let manager = NestedCapabilityManager::new();

        // Create capability hierarchy
        let root = Arc::new(CompositeCapability {
            name: "root".to_string(),
            sub_capabilities: HashMap::new(),
            parent: None,
        });

        let child1 = Arc::new(CompositeCapability {
            name: "child1".to_string(),
            sub_capabilities: HashMap::new(),
            parent: Some(root.clone()),
        });

        let child2 = Arc::new(CompositeCapability {
            name: "child2".to_string(),
            sub_capabilities: HashMap::new(),
            parent: Some(root.clone()),
        });

        // Register capabilities
        let root_id = manager.register_capability(root.clone()).await;
        let child1_id = manager.register_capability(child1.clone()).await;
        let child2_id = manager.register_capability(child2.clone()).await;

        // Add parent-child relationships
        manager.add_child(root_id, child1_id).await;
        manager.add_child(root_id, child2_id).await;

        // Test disposal cascade
        let children_before = manager.get_children(root_id).await;
        assert_eq!(children_before.len(), 2);

        // Dispose parent (should cascade to children)
        let disposed = manager.dispose_with_children(root_id).await;
        assert_eq!(disposed.len(), 3); // root + 2 children

        // Verify all are disposed
        assert!(manager.get_capability(root_id).await.is_none());
        assert!(manager.get_capability(child1_id).await.is_none());
        assert!(manager.get_capability(child2_id).await.is_none());

        println!("✅ Capability disposal cascade verified");
    }

    /// Test capability revocation and invalidation
    #[tokio::test]
    async fn test_capability_revocation() {
        println!("🧪 Testing Capability Revocation");

        let manager = NestedCapabilityManager::new();

        // Create capabilities
        let cap1 = Arc::new(CompositeCapability {
            name: "cap1".to_string(),
            sub_capabilities: HashMap::new(),
            parent: None,
        });

        let cap2 = Arc::new(CompositeCapability {
            name: "cap2".to_string(),
            sub_capabilities: HashMap::new(),
            parent: None,
        });

        let id1 = manager.register_capability(cap1.clone()).await;
        let id2 = manager.register_capability(cap2.clone()).await;

        // Grant access
        assert!(manager.get_capability(id1).await.is_some());
        assert!(manager.get_capability(id2).await.is_some());

        // Revoke cap1
        manager.revoke_capability(id1).await;
        assert!(manager.get_capability(id1).await.is_none());
        assert!(manager.is_revoked(id1).await);

        // cap2 should still be accessible
        assert!(manager.get_capability(id2).await.is_some());
        assert!(!manager.is_revoked(id2).await);

        println!("✅ Selective capability revocation verified");

        // Test batch revocation
        let mut batch_ids = vec![];
        for i in 0..5 {
            let cap = Arc::new(CompositeCapability {
                name: format!("batch_cap_{}", i),
                sub_capabilities: HashMap::new(),
                parent: None,
            });
            batch_ids.push(manager.register_capability(cap).await);
        }

        manager.batch_revoke(&batch_ids).await;
        for id in batch_ids {
            assert!(manager.is_revoked(id).await);
        }

        println!("✅ Batch capability revocation verified");
    }
}

#[cfg(test)]
mod advanced_capability_features {
    use super::*;

    /// Test capability proxying and forwarding
    #[tokio::test]
    async fn test_capability_proxying() {
        println!("🧪 Testing Capability Proxying");

        #[derive(Debug)]
        struct ProxyCapability {
            target: Arc<dyn RpcTarget>,
            interceptor: Arc<dyn Fn(&str, &[Value]) -> bool + Send + Sync>,
        }

        #[async_trait::async_trait]
        impl RpcTarget for ProxyCapability {
            async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
                // Check if call should be intercepted
                if (self.interceptor)(method, &args) {
                    return Err(RpcError::PermissionDenied("Call intercepted".into()));
                }
                // Forward to target
                self.target.call(method, args).await
            }

            async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
                // Always forward property access
                self.target.get_property(property).await
            }
        }

        // Create target capability
        let target = Arc::new(CompositeCapability {
            name: "target".to_string(),
            sub_capabilities: HashMap::new(),
            parent: None,
        });

        // Create proxy with interceptor
        let interceptor = Arc::new(|method: &str, _args: &[Value]| {
            // Block certain methods
            method.starts_with("admin_")
        });

        let proxy = Arc::new(ProxyCapability {
            target: target.clone(),
            interceptor,
        });

        // Test allowed call
        let result = proxy.call("listCapabilities", vec![]).await;
        assert!(result.is_ok());

        // Test blocked call
        let blocked_result = proxy.call("admin_delete", vec![]).await;
        assert!(blocked_result.is_err());
        match blocked_result.unwrap_err() {
            RpcError::PermissionDenied(_) => {} // Expected
            _ => panic!("Should be permission denied error")
        }

        println!("✅ Capability proxying and interception verified");
    }

    /// Test capability permissions and access control
    #[tokio::test]
    async fn test_capability_permissions() {
        println!("🧪 Testing Capability Permissions");

        #[derive(Debug)]
        struct SecureCapability {
            permissions: HashSet<String>,
            data: HashMap<String, Value>,
        }

        #[async_trait::async_trait]
        impl RpcTarget for SecureCapability {
            async fn call(&self, method: &str, _args: Vec<Value>) -> Result<Value, RpcError> {
                if !self.permissions.contains(method) {
                    return Err(RpcError::PermissionDenied(format!("No permission for method: {}", method)));
                }
                Ok(Value::String(format!("Executed: {}", method)))
            }

            async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
                let read_permission = format!("read:{}", property);
                if !self.permissions.contains(&read_permission) {
                    return Err(RpcError::PermissionDenied(format!("No read permission for: {}", property)));
                }
                self.data.get(property)
                    .cloned()
                    .ok_or_else(|| RpcError::PropertyNotFound(property.to_string()))
            }
        }

        // Create capability with specific permissions
        let mut permissions = HashSet::new();
        permissions.insert("read".to_string());
        permissions.insert("list".to_string());
        permissions.insert("read:public_data".to_string());

        let mut data = HashMap::new();
        data.insert("public_data".to_string(), Value::String("visible".to_string()));
        data.insert("private_data".to_string(), Value::String("hidden".to_string()));

        let secure_cap = Arc::new(SecureCapability {
            permissions,
            data,
        });

        // Test allowed operations
        let read_result = secure_cap.call("read", vec![]).await;
        assert!(read_result.is_ok());

        let list_result = secure_cap.call("list", vec![]).await;
        assert!(list_result.is_ok());

        // Test denied operations
        let write_result = secure_cap.call("write", vec![]).await;
        assert!(write_result.is_err());

        let delete_result = secure_cap.call("delete", vec![]).await;
        assert!(delete_result.is_err());

        // Test property permissions
        let public_prop = secure_cap.get_property("public_data").await;
        assert!(public_prop.is_ok());
        assert_eq!(public_prop.unwrap(), Value::String("visible".to_string()));

        let private_prop = secure_cap.get_property("private_data").await;
        assert!(private_prop.is_err());

        println!("✅ Capability permission system verified");
    }

    /// Test capability versioning and migration
    #[tokio::test]
    async fn test_capability_versioning() {
        println!("🧪 Testing Capability Versioning");

        #[derive(Debug)]
        struct VersionedCapability {
            version: u32,
            features: HashMap<u32, HashSet<String>>,
        }

        #[async_trait::async_trait]
        impl RpcTarget for VersionedCapability {
            async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
                // Check if method is supported in current version
                let current_features = self.features.get(&self.version)
                    .ok_or_else(|| RpcError::InternalError("Version not found".into()))?;

                if !current_features.contains(method) {
                    // Try to find the version that supports this method
                    for (version, features) in &self.features {
                        if features.contains(method) {
                            return Err(RpcError::VersionMismatch(format!(
                                "Method '{}' requires version {}, current version is {}",
                                method, version, self.version
                            )));
                        }
                    }
                    return Err(RpcError::MethodNotFound(method.to_string()));
                }

                // Execute versioned method
                match (self.version, method) {
                    (1, "getData") => Ok(Value::String("v1 data".to_string())),
                    (2, "getData") => Ok(Value::Object({
                        let mut obj = HashMap::new();
                        obj.insert("data".to_string(), Value::String("v2 data".to_string()));
                        obj.insert("version".to_string(), Value::Number(Number::from(2)));
                        obj
                    })),
                    (2, "newFeature") => Ok(Value::String("v2 only feature".to_string())),
                    _ => Ok(Value::String(format!("Generic response for {}", method)))
                }
            }

            async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
                match property {
                    "version" => Ok(Value::Number(Number::from(self.version))),
                    "supported_versions" => {
                        let versions: Vec<Value> = self.features.keys()
                            .map(|v| Value::Number(Number::from(*v)))
                            .collect();
                        Ok(Value::Array(versions))
                    }
                    _ => Err(RpcError::PropertyNotFound(property.to_string()))
                }
            }
        }

        // Create versioned capability
        let mut features = HashMap::new();

        let mut v1_features = HashSet::new();
        v1_features.insert("getData".to_string());
        v1_features.insert("basicOp".to_string());
        features.insert(1, v1_features);

        let mut v2_features = HashSet::new();
        v2_features.insert("getData".to_string());
        v2_features.insert("basicOp".to_string());
        v2_features.insert("newFeature".to_string());
        v2_features.insert("enhancedOp".to_string());
        features.insert(2, v2_features);

        // Test v1 capability
        let v1_cap = Arc::new(VersionedCapability {
            version: 1,
            features: features.clone(),
        });

        let v1_data = v1_cap.call("getData", vec![]).await.unwrap();
        assert_eq!(v1_data, Value::String("v1 data".to_string()));

        let v1_new_feature = v1_cap.call("newFeature", vec![]).await;
        assert!(v1_new_feature.is_err());

        // Test v2 capability
        let v2_cap = Arc::new(VersionedCapability {
            version: 2,
            features,
        });

        let v2_data = v2_cap.call("getData", vec![]).await.unwrap();
        match v2_data {
            Value::Object(obj) => {
                assert_eq!(obj.get("version"), Some(&Value::Number(Number::from(2))));
            }
            _ => panic!("v2 should return object")
        }

        let v2_new_feature = v2_cap.call("newFeature", vec![]).await.unwrap();
        assert_eq!(v2_new_feature, Value::String("v2 only feature".to_string()));

        println!("✅ Capability versioning verified");
    }

    /// Test capability event handling and notifications
    #[tokio::test]
    async fn test_capability_events() {
        println!("🧪 Testing Capability Event Handling");

        #[derive(Debug)]
        struct EventCapability {
            events: Arc<RwLock<Vec<(String, Value)>>>,
            listeners: Arc<RwLock<HashMap<String, Vec<Arc<dyn RpcTarget>>>>>,
        }

        impl EventCapability {
            async fn emit(&self, event: String, data: Value) {
                // Store event
                self.events.write().await.push((event.clone(), data.clone()));

                // Notify listeners
                if let Some(listeners) = self.listeners.read().await.get(&event) {
                    for listener in listeners {
                        // In real implementation, would call listener
                        let _ = listener.call("onEvent", vec![Value::String(event.clone()), data.clone()]).await;
                    }
                }
            }
        }

        #[async_trait::async_trait]
        impl RpcTarget for EventCapability {
            async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
                match method {
                    "emit" => {
                        if args.len() < 2 {
                            return Err(RpcError::InvalidArguments("emit requires event and data".into()));
                        }
                        match (&args[0], &args[1]) {
                            (Value::String(event), data) => {
                                self.emit(event.clone(), data.clone()).await;
                                Ok(Value::Bool(true))
                            }
                            _ => Err(RpcError::InvalidArguments("Invalid event format".into()))
                        }
                    }
                    "getEvents" => {
                        let events = self.events.read().await;
                        let event_values: Vec<Value> = events.iter()
                            .map(|(name, data)| Value::Object({
                                let mut obj = HashMap::new();
                                obj.insert("event".to_string(), Value::String(name.clone()));
                                obj.insert("data".to_string(), data.clone());
                                obj
                            }))
                            .collect();
                        Ok(Value::Array(event_values))
                    }
                    _ => Err(RpcError::MethodNotFound(method.to_string()))
                }
            }

            async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
                match property {
                    "event_count" => {
                        let count = self.events.read().await.len();
                        Ok(Value::Number(Number::from(count)))
                    }
                    _ => Err(RpcError::PropertyNotFound(property.to_string()))
                }
            }
        }

        // Create event capability
        let event_cap = Arc::new(EventCapability {
            events: Arc::new(RwLock::new(Vec::new())),
            listeners: Arc::new(RwLock::new(HashMap::new())),
        });

        // Emit events
        event_cap.emit("start".to_string(), Value::String("System starting".to_string())).await;
        event_cap.emit("data".to_string(), Value::Number(Number::from(42))).await;
        event_cap.emit("complete".to_string(), Value::Bool(true)).await;

        // Check events were recorded
        let events = event_cap.call("getEvents", vec![]).await.unwrap();
        match events {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Should return array of events")
        }

        let event_count = event_cap.get_property("event_count").await.unwrap();
        assert_eq!(event_count, Value::Number(Number::from(3)));

        println!("✅ Capability event handling verified");
    }
}