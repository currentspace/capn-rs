// Nested Capability Creation for Cap'n Web Protocol
// Enables dynamic capability graphs and advanced capability composition patterns

use super::tables::Value;
use crate::RpcTarget;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Capability factory for creating nested capabilities at runtime
#[async_trait::async_trait]
pub trait CapabilityFactory: Send + Sync + std::fmt::Debug {
    /// Create a new capability instance
    async fn create_capability(
        &self,
        capability_type: &str,
        config: Value,
    ) -> Result<Arc<dyn RpcTarget>, CapabilityError>;

    /// List available capability types
    fn list_capability_types(&self) -> Vec<String>;

    /// Get capability type metadata
    fn get_capability_metadata(&self, capability_type: &str) -> Option<CapabilityMetadata>;
}

/// Metadata about a capability type
#[derive(Debug, Clone)]
pub struct CapabilityMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub methods: Vec<MethodMetadata>,
    pub config_schema: Option<Value>, // JSON schema for configuration
}

/// Metadata about a capability method
#[derive(Debug, Clone)]
pub struct MethodMetadata {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterMetadata>,
    pub return_type: String,
}

/// Metadata about a method parameter
#[derive(Debug, Clone)]
pub struct ParameterMetadata {
    pub name: String,
    pub type_name: String,
    pub required: bool,
    pub description: String,
}

/// Enhanced RPC target that can create nested capabilities
#[async_trait::async_trait]
pub trait NestedCapableRpcTarget: RpcTarget {
    /// Create a sub-capability
    async fn create_sub_capability(
        &self,
        capability_type: &str,
        config: Value,
    ) -> Result<Value, crate::RpcError>;

    /// List available capability types
    async fn list_capability_types(&self) -> Result<Value, crate::RpcError>;

    /// Get capability metadata
    async fn get_capability_metadata(&self, capability_type: &str) -> Result<Value, crate::RpcError>;

    /// Get all child capabilities
    async fn list_child_capabilities(&self) -> Result<Value, crate::RpcError>;

    /// Dispose of a child capability
    async fn dispose_child_capability(&self, capability_id: &str) -> Result<Value, crate::RpcError>;
}

/// Capability graph manager for tracking nested capability relationships
#[derive(Debug)]
pub struct CapabilityGraph {
    /// Graph of capability relationships
    nodes: Arc<RwLock<HashMap<String, CapabilityNode>>>,
    /// Edges representing parent-child relationships
    edges: Arc<RwLock<HashMap<String, Vec<String>>>>, // parent_id -> [child_ids]
    /// Reference counting for capabilities
    reference_counts: Arc<RwLock<HashMap<String, usize>>>,
}

/// Node in the capability graph
#[derive(Debug, Clone)]
pub struct CapabilityNode {
    pub id: String,
    pub capability_type: String,
    pub parent_id: Option<String>,
    pub created_at: u64,
    pub config: Value,
    pub metadata: CapabilityMetadata,
}

impl CapabilityGraph {
    /// Create a new capability graph
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(HashMap::new())),
            reference_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a capability to the graph
    pub async fn add_capability(
        &self,
        node: CapabilityNode,
    ) -> Result<(), CapabilityError> {
        let mut nodes = self.nodes.write().await;
        let mut edges = self.edges.write().await;
        let mut ref_counts = self.reference_counts.write().await;

        tracing::debug!(
            capability_id = %node.id,
            capability_type = %node.capability_type,
            parent_id = ?node.parent_id,
            "Adding capability to graph"
        );

        // Add the node
        nodes.insert(node.id.clone(), node.clone());

        // Initialize reference count
        ref_counts.insert(node.id.clone(), 1);

        // Add parent-child relationship
        if let Some(parent_id) = &node.parent_id {
            edges.entry(parent_id.clone())
                .or_insert_with(Vec::new)
                .push(node.id.clone());
        }

        Ok(())
    }

    /// Get a capability node
    pub async fn get_capability(&self, id: &str) -> Option<CapabilityNode> {
        let nodes = self.nodes.read().await;
        nodes.get(id).cloned()
    }

    /// Get children of a capability
    pub async fn get_children(&self, parent_id: &str) -> Vec<String> {
        let edges = self.edges.read().await;
        edges.get(parent_id).cloned().unwrap_or_default()
    }

    /// Get all descendants (recursive children) of a capability
    pub async fn get_descendants(&self, parent_id: &str) -> Vec<String> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![parent_id.to_string()];

        while let Some(current_id) = to_visit.pop() {
            let children = self.get_children(&current_id).await;
            for child_id in children {
                descendants.push(child_id.clone());
                to_visit.push(child_id);
            }
        }

        descendants
    }

    /// Increment reference count
    pub async fn add_reference(&self, id: &str) -> Result<usize, CapabilityError> {
        let mut ref_counts = self.reference_counts.write().await;
        match ref_counts.get_mut(id) {
            Some(count) => {
                *count += 1;
                Ok(*count)
            }
            None => Err(CapabilityError::CapabilityNotFound(id.to_string())),
        }
    }

    /// Decrement reference count and remove if zero
    pub async fn remove_reference(&self, id: &str) -> Result<bool, CapabilityError> {
        let mut should_dispose = false;

        {
            let mut ref_counts = self.reference_counts.write().await;
            match ref_counts.get_mut(id) {
                Some(count) => {
                    *count -= 1;
                    if *count == 0 {
                        should_dispose = true;
                        ref_counts.remove(id);
                    }
                }
                None => return Err(CapabilityError::CapabilityNotFound(id.to_string())),
            }
        }

        if should_dispose {
            self.remove_capability(id).await?;
        }

        Ok(should_dispose)
    }

    /// Remove a capability and its descendants
    pub async fn remove_capability(&self, id: &str) -> Result<(), CapabilityError> {
        tracing::debug!(capability_id = %id, "Removing capability from graph");

        // Get all descendants first
        let descendants = self.get_descendants(id).await;

        // Remove all descendants
        {
            let mut nodes = self.nodes.write().await;
            let mut edges = self.edges.write().await;

            for desc_id in &descendants {
                nodes.remove(desc_id);
                edges.remove(desc_id);
            }

            // Remove the capability itself
            nodes.remove(id);
            edges.remove(id);
        }

        // Remove from parent's edge list
        {
            let nodes = self.nodes.read().await;
            if let Some(node) = nodes.get(id) {
                if let Some(parent_id) = &node.parent_id {
                    let mut edges = self.edges.write().await;
                    if let Some(children) = edges.get_mut(parent_id) {
                        children.retain(|child_id| child_id != id);
                    }
                }
            }
        }

        tracing::debug!(
            capability_id = %id,
            descendants_count = descendants.len(),
            "Capability and descendants removed from graph"
        );

        Ok(())
    }

    /// Get graph statistics
    pub async fn get_stats(&self) -> CapabilityGraphStats {
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;

        let total_capabilities = nodes.len();
        let root_capabilities = nodes.values().filter(|node| node.parent_id.is_none()).count();
        let max_depth = self.calculate_max_depth(&nodes, &edges).await;

        CapabilityGraphStats {
            total_capabilities,
            root_capabilities,
            max_depth,
            total_edges: edges.values().map(|children| children.len()).sum(),
        }
    }

    async fn calculate_max_depth(
        &self,
        nodes: &HashMap<String, CapabilityNode>,
        edges: &HashMap<String, Vec<String>>,
    ) -> usize {
        let mut max_depth = 0;

        for (id, node) in nodes {
            if node.parent_id.is_none() {
                let depth = self.calculate_depth_recursive(id, edges, 0);
                max_depth = max_depth.max(depth);
            }
        }

        max_depth
    }

    fn calculate_depth_recursive(
        &self,
        node_id: &str,
        edges: &HashMap<String, Vec<String>>,
        current_depth: usize,
    ) -> usize {
        let mut max_child_depth = current_depth;

        if let Some(children) = edges.get(node_id) {
            for child_id in children {
                let child_depth = self.calculate_depth_recursive(child_id, edges, current_depth + 1);
                max_child_depth = max_child_depth.max(child_depth);
            }
        }

        max_child_depth
    }
}

impl Default for CapabilityGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the capability graph
#[derive(Debug, Clone)]
pub struct CapabilityGraphStats {
    pub total_capabilities: usize,
    pub root_capabilities: usize,
    pub max_depth: usize,
    pub total_edges: usize,
}

/// Default implementation of nested-capable RPC target
#[derive(Debug)]
pub struct DefaultNestedCapableTarget {
    id: String,
    capability_type: String,
    factory: Arc<dyn CapabilityFactory>,
    graph: Arc<CapabilityGraph>,
    children: Arc<RwLock<HashMap<String, Arc<dyn RpcTarget>>>>,
    delegate: Arc<dyn RpcTarget>, // Delegate for base functionality
}

impl DefaultNestedCapableTarget {
    /// Create a new nested-capable target
    pub fn new(
        capability_type: String,
        factory: Arc<dyn CapabilityFactory>,
        graph: Arc<CapabilityGraph>,
        delegate: Arc<dyn RpcTarget>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            capability_type,
            factory,
            graph,
            children: Arc::new(RwLock::new(HashMap::new())),
            delegate,
        }
    }

    /// Get the capability ID
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[async_trait::async_trait]
impl NestedCapableRpcTarget for DefaultNestedCapableTarget {
    async fn create_sub_capability(
        &self,
        capability_type: &str,
        config: Value,
    ) -> Result<Value, crate::RpcError> {
        tracing::debug!(
            parent_id = %self.id,
            capability_type = %capability_type,
            "Creating sub-capability"
        );

        // Create the capability using the factory
        let capability = self.factory.create_capability(capability_type, config.clone()).await
            .map_err(|e| crate::RpcError::internal(e.to_string()))?;

        let capability_id = Uuid::new_v4().to_string();

        // Create metadata (simplified)
        let metadata = self.factory.get_capability_metadata(capability_type)
            .unwrap_or_else(|| CapabilityMetadata {
                name: capability_type.to_string(),
                description: format!("Dynamically created {} capability", capability_type),
                version: "1.0.0".to_string(),
                methods: Vec::new(),
                config_schema: None,
            });

        // Create graph node
        let node = CapabilityNode {
            id: capability_id.clone(),
            capability_type: capability_type.to_string(),
            parent_id: Some(self.id.clone()),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            config,
            metadata,
        };

        // Add to graph
        self.graph.add_capability(node).await
            .map_err(|e| crate::RpcError::internal(e.to_string()))?;

        // Store the capability
        self.children.write().await.insert(capability_id.clone(), capability);

        tracing::debug!(
            parent_id = %self.id,
            child_id = %capability_id,
            capability_type = %capability_type,
            "Sub-capability created successfully"
        );

        // Return capability reference
        Ok(Value::Object({
            let mut obj = std::collections::HashMap::new();
            obj.insert("capability_id".to_string(), Box::new(Value::String(capability_id)));
            obj.insert("capability_type".to_string(), Box::new(Value::String(capability_type.to_string())));
            obj.insert("parent_id".to_string(), Box::new(Value::String(self.id.clone())));
            obj
        }))
    }

    async fn list_capability_types(&self) -> Result<Value, crate::RpcError> {
        let types = self.factory.list_capability_types();
        let values: Vec<Value> = types.into_iter().map(Value::String).collect();
        Ok(Value::Array(values))
    }

    async fn get_capability_metadata(&self, capability_type: &str) -> Result<Value, crate::RpcError> {
        match self.factory.get_capability_metadata(capability_type) {
            Some(metadata) => {
                let mut obj = std::collections::HashMap::new();
                obj.insert("name".to_string(), Box::new(Value::String(metadata.name)));
                obj.insert("description".to_string(), Box::new(Value::String(metadata.description)));
                obj.insert("version".to_string(), Box::new(Value::String(metadata.version)));

                let methods: Vec<Value> = metadata.methods.into_iter().map(|method| {
                    let mut method_obj = std::collections::HashMap::new();
                    method_obj.insert("name".to_string(), Box::new(Value::String(method.name)));
                    method_obj.insert("description".to_string(), Box::new(Value::String(method.description)));
                    method_obj.insert("return_type".to_string(), Box::new(Value::String(method.return_type)));

                    let params: Vec<Value> = method.parameters.into_iter().map(|param| {
                        let mut param_obj = std::collections::HashMap::new();
                        param_obj.insert("name".to_string(), Box::new(Value::String(param.name)));
                        param_obj.insert("type".to_string(), Box::new(Value::String(param.type_name)));
                        param_obj.insert("required".to_string(), Box::new(Value::Bool(param.required)));
                        param_obj.insert("description".to_string(), Box::new(Value::String(param.description)));
                        Value::Object(param_obj)
                    }).collect();

                    method_obj.insert("parameters".to_string(), Box::new(Value::Array(params)));
                    Value::Object(method_obj)
                }).collect();

                obj.insert("methods".to_string(), Box::new(Value::Array(methods)));

                if let Some(schema) = metadata.config_schema {
                    obj.insert("config_schema".to_string(), Box::new(schema));
                }

                Ok(Value::Object(obj))
            }
            None => Err(crate::RpcError::not_found(format!("Capability type not found: {}", capability_type))),
        }
    }

    async fn list_child_capabilities(&self) -> Result<Value, crate::RpcError> {
        let children_ids = self.graph.get_children(&self.id).await;

        let mut children_info = Vec::new();

        for child_id in children_ids {
            if let Some(node) = self.graph.get_capability(&child_id).await {
                let mut child_obj = std::collections::HashMap::new();
                child_obj.insert("id".to_string(), Box::new(Value::String(node.id)));
                child_obj.insert("type".to_string(), Box::new(Value::String(node.capability_type)));
                child_obj.insert("created_at".to_string(), Box::new(Value::Number(serde_json::Number::from(node.created_at))));
                children_info.push(Value::Object(child_obj));
            }
        }

        Ok(Value::Array(children_info))
    }

    async fn dispose_child_capability(&self, capability_id: &str) -> Result<Value, crate::RpcError> {
        tracing::debug!(
            parent_id = %self.id,
            child_id = %capability_id,
            "Disposing child capability"
        );

        // Remove from our children map
        let removed = self.children.write().await.remove(capability_id).is_some();

        if removed {
            // Remove from graph (this will handle reference counting)
            self.graph.remove_reference(capability_id).await
                .map_err(|e| crate::RpcError::internal(e.to_string()))?;

            tracing::debug!(
                parent_id = %self.id,
                child_id = %capability_id,
                "Child capability disposed successfully"
            );

            Ok(Value::Bool(true))
        } else {
            Err(crate::RpcError::not_found(format!("Child capability not found: {}", capability_id)))
        }
    }
}

#[async_trait::async_trait]
impl RpcTarget for DefaultNestedCapableTarget {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, crate::RpcError> {
        match method {
            "createSubCapability" => {
                if args.len() != 2 {
                    return Err(crate::RpcError::bad_request("createSubCapability requires 2 arguments: capability_type, config"));
                }

                let capability_type = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(crate::RpcError::bad_request("capability_type must be a string")),
                };

                self.create_sub_capability(&capability_type, args[1].clone()).await
            }

            "listCapabilityTypes" => {
                if !args.is_empty() {
                    return Err(crate::RpcError::bad_request("listCapabilityTypes takes no arguments"));
                }
                self.list_capability_types().await
            }

            "getCapabilityMetadata" => {
                if args.len() != 1 {
                    return Err(crate::RpcError::bad_request("getCapabilityMetadata requires 1 argument: capability_type"));
                }

                let capability_type = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(crate::RpcError::bad_request("capability_type must be a string")),
                };

                self.get_capability_metadata(&capability_type).await
            }

            "listChildCapabilities" => {
                if !args.is_empty() {
                    return Err(crate::RpcError::bad_request("listChildCapabilities takes no arguments"));
                }
                self.list_child_capabilities().await
            }

            "disposeChildCapability" => {
                if args.len() != 1 {
                    return Err(crate::RpcError::bad_request("disposeChildCapability requires 1 argument: capability_id"));
                }

                let capability_id = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(crate::RpcError::bad_request("capability_id must be a string")),
                };

                self.dispose_child_capability(&capability_id).await
            }

            // Forward to child capabilities if the method contains a capability reference
            _ if method.starts_with("child:") => {
                let parts: Vec<&str> = method.splitn(3, ':').collect();
                if parts.len() == 3 {
                    let child_id = parts[1];
                    let child_method = parts[2];

                    let children = self.children.read().await;
                    if let Some(child) = children.get(child_id) {
                        child.call(child_method, args).await
                    } else {
                        Err(crate::RpcError::not_found(format!("Child capability not found: {}", child_id)))
                    }
                } else {
                    Err(crate::RpcError::bad_request("Invalid child method format: use 'child:id:method'"))
                }
            }

            // Delegate other methods to the base capability
            _ => self.delegate.call(method, args).await,
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, crate::RpcError> {
        match property {
            "capability_id" => Ok(Value::String(self.id.clone())),
            "capability_type" => Ok(Value::String(self.capability_type.clone())),
            _ => self.delegate.get_property(property).await,
        }
    }
}

/// Errors related to capability operations
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),

    #[error("Invalid capability type: {0}")]
    InvalidCapabilityType(String),

    #[error("Capability creation failed: {0}")]
    CreationFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Graph operation failed: {0}")]
    GraphOperationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Number;

    // Mock capability factory for testing
    #[derive(Debug)]
    struct MockCapabilityFactory;

    #[async_trait::async_trait]
    impl CapabilityFactory for MockCapabilityFactory {
        async fn create_capability(
            &self,
            capability_type: &str,
            _config: Value,
        ) -> Result<Arc<dyn RpcTarget>, CapabilityError> {
            match capability_type {
                "calculator" => Ok(Arc::new(crate::MockRpcTarget::new())),
                _ => Err(CapabilityError::InvalidCapabilityType(capability_type.to_string())),
            }
        }

        fn list_capability_types(&self) -> Vec<String> {
            vec!["calculator".to_string()]
        }

        fn get_capability_metadata(&self, capability_type: &str) -> Option<CapabilityMetadata> {
            match capability_type {
                "calculator" => Some(CapabilityMetadata {
                    name: "Calculator".to_string(),
                    description: "Basic calculator capability".to_string(),
                    version: "1.0.0".to_string(),
                    methods: vec![
                        MethodMetadata {
                            name: "add".to_string(),
                            description: "Add two numbers".to_string(),
                            parameters: vec![
                                ParameterMetadata {
                                    name: "a".to_string(),
                                    type_name: "number".to_string(),
                                    required: true,
                                    description: "First number".to_string(),
                                },
                                ParameterMetadata {
                                    name: "b".to_string(),
                                    type_name: "number".to_string(),
                                    required: true,
                                    description: "Second number".to_string(),
                                },
                            ],
                            return_type: "number".to_string(),
                        },
                    ],
                    config_schema: None,
                }),
                _ => None,
            }
        }
    }

    #[tokio::test]
    async fn test_capability_graph() {
        let graph = CapabilityGraph::new();

        let node = CapabilityNode {
            id: "test-cap-1".to_string(),
            capability_type: "calculator".to_string(),
            parent_id: None,
            created_at: 1234567890,
            config: Value::Object(HashMap::new()),
            metadata: CapabilityMetadata {
                name: "Test Calculator".to_string(),
                description: "Test capability".to_string(),
                version: "1.0.0".to_string(),
                methods: Vec::new(),
                config_schema: None,
            },
        };

        graph.add_capability(node.clone()).await.unwrap();

        let retrieved = graph.get_capability("test-cap-1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-cap-1");

        let stats = graph.get_stats().await;
        assert_eq!(stats.total_capabilities, 1);
        assert_eq!(stats.root_capabilities, 1);
        assert_eq!(stats.max_depth, 0);
    }

    #[tokio::test]
    async fn test_nested_capability_creation() {
        let factory = Arc::new(MockCapabilityFactory);
        let graph = Arc::new(CapabilityGraph::new());
        let delegate = Arc::new(crate::MockRpcTarget::new());

        let nested_target = DefaultNestedCapableTarget::new(
            "parent".to_string(),
            factory,
            graph.clone(),
            delegate,
        );

        // Test listing capability types
        let types = nested_target.list_capability_types().await.unwrap();
        match types {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 1);
                match &arr[0] {
                    Value::String(s) => assert_eq!(s, "calculator"),
                    _ => panic!("Expected string"),
                }
            }
            _ => panic!("Expected array"),
        }

        // Test creating a sub-capability
        let config = Value::Object({
            let mut obj = HashMap::new();
            obj.insert("precision".to_string(), Box::new(Value::Number(Number::from(2))));
            obj
        });

        let result = nested_target.create_sub_capability("calculator", config).await.unwrap();

        // Verify the result contains the expected structure
        match result {
            Value::Object(obj) => {
                assert!(obj.contains_key("capability_id"));
                assert!(obj.contains_key("capability_type"));
                assert!(obj.contains_key("parent_id"));
            }
            _ => panic!("Expected object result"),
        }

        // Test listing child capabilities
        let children = nested_target.list_child_capabilities().await.unwrap();
        match children {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 1);
            }
            _ => panic!("Expected array of children"),
        }

        // Check graph stats
        let stats = graph.get_stats().await;
        assert_eq!(stats.total_capabilities, 2); // parent + child
        assert_eq!(stats.root_capabilities, 1); // only parent is root
        assert_eq!(stats.max_depth, 1); // child is at depth 1
    }

    #[tokio::test]
    async fn test_capability_disposal() {
        let factory = Arc::new(MockCapabilityFactory);
        let graph = Arc::new(CapabilityGraph::new());
        let delegate = Arc::new(crate::MockRpcTarget::new());

        let nested_target = DefaultNestedCapableTarget::new(
            "parent".to_string(),
            factory,
            graph.clone(),
            delegate,
        );

        // Create a sub-capability
        let config = Value::Object(HashMap::new());
        let result = nested_target.create_sub_capability("calculator", config).await.unwrap();

        let capability_id = match result {
            Value::Object(ref obj) => {
                match obj.get("capability_id").unwrap().as_ref() {
                    Value::String(id) => id.clone(),
                    _ => panic!("Expected string capability_id"),
                }
            }
            _ => panic!("Expected object result"),
        };

        // Verify it was created
        let children_before = nested_target.list_child_capabilities().await.unwrap();
        match children_before {
            Value::Array(arr) => assert_eq!(arr.len(), 1),
            _ => panic!("Expected array"),
        }

        // Dispose the capability
        let disposed = nested_target.dispose_child_capability(&capability_id).await.unwrap();
        match disposed {
            Value::Bool(true) => {},
            _ => panic!("Expected true"),
        }

        // Verify it was removed
        let children_after = nested_target.list_child_capabilities().await.unwrap();
        match children_after {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected empty array"),
        }
    }
}