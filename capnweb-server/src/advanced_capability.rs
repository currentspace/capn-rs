// Advanced Capability Implementation
// Exposes Resume Tokens, Nested Capabilities, and IL Plan Runner via RPC

use async_trait::async_trait;
use capnweb_core::{
    RpcTarget, RpcError, CapId, Value,
    Plan, Op, Source,
    il::{CallOp, ObjectOp, ArrayOp, CaptureRef, ResultRef, ParamRef, ValueRef},
    protocol::{
        resume_tokens::{ResumeTokenManager, PersistentSessionManager, SessionSnapshot, ResumeToken},
        nested_capabilities::{
            CapabilityGraph,
            CapabilityFactory as CapabilityFactoryTrait,
            CapabilityError,
            CapabilityMetadata, MethodMetadata, ParameterMetadata
        },
        il_runner::{PlanRunner, PlanBuilder, ExecutionContext},
        ids::{IdAllocator, ImportId, ExportId},
        tables::{ImportTable, ExportTable},
    }
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tokio::time::Duration;
use chrono::Utc;

/// Session state for resume tokens
#[derive(Clone, Debug)]
struct SessionState {
    variables: HashMap<String, Value>,
    operations: Vec<String>,
    last_result: Option<Value>,
    created_at: i64,
    last_accessed: i64,
}

/// Concrete implementation of CapabilityFactory
#[derive(Debug, Clone)]
struct SimpleCapabilityFactory {
    max_capabilities: usize,
    created_count: Arc<Mutex<usize>>,
}

impl SimpleCapabilityFactory {
    fn new(max_capabilities: usize) -> Self {
        Self {
            max_capabilities,
            created_count: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl CapabilityFactoryTrait for SimpleCapabilityFactory {
    async fn create_capability(
        &self,
        capability_type: &str,
        config: Value,
    ) -> Result<Arc<dyn RpcTarget>, CapabilityError> {
        let mut count = self.created_count.lock().await;
        if *count >= self.max_capabilities {
            return Err(CapabilityError::InvalidConfiguration(
                format!("Maximum capabilities limit ({}) exceeded", self.max_capabilities)
            ));
        }
        *count += 1;

        // Create a nested capability implementation
        let nested_cap = Arc::new(NestedCapabilityImpl {
            name: capability_type.to_string(),
            config,
            parent: None,
        });

        Ok(nested_cap as Arc<dyn RpcTarget>)
    }

    fn list_capability_types(&self) -> Vec<String> {
        vec![
            "validator".to_string(),
            "aggregator".to_string(),
            "processor".to_string(),
            "analyzer".to_string(),
            "transformer".to_string(),
        ]
    }

    fn get_capability_metadata(&self, capability_type: &str) -> Option<CapabilityMetadata> {
        match capability_type {
            "validator" => Some(CapabilityMetadata {
                name: "validator".to_string(),
                description: "Validates input data according to rules".to_string(),
                version: "1.0.0".to_string(),
                methods: vec![
                    MethodMetadata {
                        name: "validate".to_string(),
                        description: "Validates input against schema".to_string(),
                        parameters: vec![
                            ParameterMetadata {
                                name: "data".to_string(),
                                type_name: "any".to_string(),
                                description: "Data to validate".to_string(),
                                required: true,
                            },
                            ParameterMetadata {
                                name: "rules".to_string(),
                                type_name: "object".to_string(),
                                description: "Validation rules".to_string(),
                                required: false,
                            },
                        ],
                        return_type: "ValidationResult".to_string(),
                    },
                    MethodMetadata {
                        name: "setRules".to_string(),
                        description: "Configure validation rules".to_string(),
                        parameters: vec![
                            ParameterMetadata {
                                name: "rules".to_string(),
                                type_name: "object".to_string(),
                                description: "New validation rules".to_string(),
                                required: true,
                            },
                        ],
                        return_type: "void".to_string(),
                    },
                ],
                config_schema: Some(Value::Object({
                    let mut obj = std::collections::HashMap::new();
                    obj.insert("type".to_string(), Box::new(Value::String("object".to_string())));
                    let mut props = std::collections::HashMap::new();
                    let mut strict_prop = std::collections::HashMap::new();
                    strict_prop.insert("type".to_string(), Box::new(Value::String("boolean".to_string())));
                    props.insert("strict".to_string(), Box::new(Value::Object(strict_prop)));
                    let mut allow_prop = std::collections::HashMap::new();
                    allow_prop.insert("type".to_string(), Box::new(Value::String("boolean".to_string())));
                    props.insert("allowExtraFields".to_string(), Box::new(Value::Object(allow_prop)));
                    let mut depth_prop = std::collections::HashMap::new();
                    depth_prop.insert("type".to_string(), Box::new(Value::String("number".to_string())));
                    props.insert("maxDepth".to_string(), Box::new(Value::Object(depth_prop)));
                    obj.insert("properties".to_string(), Box::new(Value::Object(props)));
                    obj
                })),
            }),
            "aggregator" => Some(CapabilityMetadata {
                name: "aggregator".to_string(),
                description: "Aggregates and processes data streams".to_string(),
                version: "1.0.0".to_string(),
                methods: vec![
                    MethodMetadata {
                        name: "add".to_string(),
                        description: "Add data to aggregation".to_string(),
                        parameters: vec![
                            ParameterMetadata {
                                name: "data".to_string(),
                                type_name: "any".to_string(),
                                description: "Data to aggregate".to_string(),
                                required: true,
                            },
                        ],
                        return_type: "void".to_string(),
                    },
                    MethodMetadata {
                        name: "compute".to_string(),
                        description: "Compute aggregation result".to_string(),
                        parameters: vec![],
                        return_type: "AggregationResult".to_string(),
                    },
                    MethodMetadata {
                        name: "reset".to_string(),
                        description: "Reset aggregation state".to_string(),
                        parameters: vec![],
                        return_type: "void".to_string(),
                    },
                ],
                config_schema: None,
            }),
            "processor" => Some(CapabilityMetadata {
                name: "processor".to_string(),
                description: "Processes data through transformation pipeline".to_string(),
                version: "1.0.0".to_string(),
                methods: vec![
                    MethodMetadata {
                        name: "process".to_string(),
                        description: "Process input data".to_string(),
                        parameters: vec![
                            ParameterMetadata {
                                name: "input".to_string(),
                                type_name: "any".to_string(),
                                description: "Input data".to_string(),
                                required: true,
                            },
                        ],
                        return_type: "ProcessedData".to_string(),
                    },
                ],
                config_schema: None,
            }),
            "analyzer" => Some(CapabilityMetadata {
                name: "analyzer".to_string(),
                description: "Analyzes data patterns and metrics".to_string(),
                version: "1.0.0".to_string(),
                methods: vec![
                    MethodMetadata {
                        name: "analyze".to_string(),
                        description: "Analyze data".to_string(),
                        parameters: vec![
                            ParameterMetadata {
                                name: "data".to_string(),
                                type_name: "any".to_string(),
                                description: "Data to analyze".to_string(),
                                required: true,
                            },
                        ],
                        return_type: "AnalysisResult".to_string(),
                    },
                ],
                config_schema: None,
            }),
            "transformer" => Some(CapabilityMetadata {
                name: "transformer".to_string(),
                description: "Transforms data between formats".to_string(),
                version: "1.0.0".to_string(),
                methods: vec![
                    MethodMetadata {
                        name: "transform".to_string(),
                        description: "Transform data".to_string(),
                        parameters: vec![
                            ParameterMetadata {
                                name: "input".to_string(),
                                type_name: "any".to_string(),
                                description: "Input data".to_string(),
                                required: true,
                            },
                            ParameterMetadata {
                                name: "format".to_string(),
                                type_name: "string".to_string(),
                                description: "Target format".to_string(),
                                required: false,
                            },
                        ],
                        return_type: "TransformedData".to_string(),
                    },
                ],
                config_schema: None,
            }),
            _ => None
        }
    }
}

/// Advanced capability that exposes all protocol features
#[derive(Debug)]
pub struct AdvancedCapability {
    // Resume token management
    resume_manager: Arc<ResumeTokenManager>,
    persistent_manager: Arc<PersistentSessionManager>,
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,

    // Nested capabilities
    capability_graph: Arc<CapabilityGraph>,
    capability_factory: Arc<SimpleCapabilityFactory>,
    capability_counter: Arc<Mutex<u64>>,

    // IL Plan runner
    plan_runner: Arc<PlanRunner>,
    plan_cache: Arc<RwLock<HashMap<String, Plan>>>,

    // Session management
    id_allocator: Arc<IdAllocator>,
    import_table: Arc<ImportTable>,
    export_table: Arc<ExportTable>,

    // State tracking
    call_count: Arc<Mutex<usize>>,
    nested_capabilities: Arc<RwLock<HashMap<String, Arc<dyn RpcTarget>>>>,
}

/// Configuration for AdvancedCapability
#[derive(Debug, Clone)]
pub struct AdvancedCapabilityConfig {
    /// Secret key for token encryption (32 bytes)
    pub secret_key: Option<Vec<u8>>,
    /// Token time-to-live in seconds
    pub token_ttl: u64,
    /// Maximum session age in seconds
    pub max_session_age: u64,
    /// Maximum number of nested capabilities
    pub max_capabilities: usize,
    /// Maximum operations per plan
    pub max_plan_operations: usize,
    /// Plan execution timeout in milliseconds
    pub plan_timeout_ms: u64,
}

impl Default for AdvancedCapabilityConfig {
    fn default() -> Self {
        Self {
            secret_key: None,
            token_ttl: 3600,           // 1 hour
            max_session_age: 86400,     // 24 hours
            max_capabilities: 1000,
            max_plan_operations: 1000,
            plan_timeout_ms: 30000,     // 30 seconds
        }
    }
}

impl AdvancedCapability {
    /// Create a new advanced capability with default configuration
    pub fn new() -> Self {
        Self::with_config(AdvancedCapabilityConfig::default())
    }

    /// Create a new advanced capability with custom configuration
    pub fn with_config(config: AdvancedCapabilityConfig) -> Self {
        // Use provided secret key or generate a new one
        use rand::RngCore;
        let secret_key = config.secret_key.unwrap_or_else(|| {
            let mut key = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            key
        });

        let resume_manager = Arc::new(ResumeTokenManager::with_settings(
            secret_key.clone(),
            config.token_ttl,
            config.max_session_age,
        ));

        let persistent_manager = Arc::new(PersistentSessionManager::new(
            ResumeTokenManager::with_settings(
                secret_key,
                config.token_ttl,
                config.max_session_age,
            )
        ));

        // Create ID allocator for tables
        let id_allocator = Arc::new(IdAllocator::new());
        let import_table = Arc::new(ImportTable::new(id_allocator.clone()));
        let export_table = Arc::new(ExportTable::new(id_allocator.clone()));

        // Create plan runner with custom settings
        let plan_runner = PlanRunner::with_settings(
            import_table.clone(),
            export_table.clone(),
            config.plan_timeout_ms,
            config.max_plan_operations,
        );

        Self {
            resume_manager,
            persistent_manager,
            sessions: Arc::new(RwLock::new(HashMap::new())),

            capability_graph: Arc::new(CapabilityGraph::new()),
            capability_factory: Arc::new(SimpleCapabilityFactory::new(config.max_capabilities)),
            capability_counter: Arc::new(Mutex::new(1)),

            plan_runner: Arc::new(plan_runner),
            plan_cache: Arc::new(RwLock::new(HashMap::new())),

            id_allocator,
            import_table,
            export_table,

            call_count: Arc::new(Mutex::new(0)),
            nested_capabilities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Parse plan from JSON representation
    fn parse_plan(&self, json: &JsonValue) -> Result<Plan, RpcError> {
        let mut ops = Vec::new();
        let mut captures = Vec::new();

        if let Some(operations) = json.get("operations").and_then(|o| o.as_array()) {
            for op_json in operations {
                let operation = self.parse_operation(op_json)?;
                ops.push(operation);
            }
        }

        // Parse captures if present
        if let Some(capture_array) = json.get("captures").and_then(|c| c.as_array()) {
            for cap_json in capture_array {
                if let Some(cap_id) = cap_json.as_u64() {
                    captures.push(CapId::new(cap_id as u64));
                }
            }
        }

        // Get result source (default to last operation result)
        let result = if let Some(result_json) = json.get("result") {
            self.parse_source(Some(result_json))?
        } else {
            Source::Result { result: ResultRef { index: ops.len().saturating_sub(1) as u32 } }
        };

        Ok(Plan {
            captures,
            ops,
            result,
        })
    }

    /// Parse a single operation from JSON
    fn parse_operation(&self, json: &JsonValue) -> Result<Op, RpcError> {
        // Check if it's a call operation
        if let Some(call_obj) = json.get("call") {
            let target = self.parse_source(call_obj.get("target"))?;
            let member = call_obj.get("member")
                .and_then(|m| m.as_str())
                .ok_or_else(|| RpcError::bad_request("Call missing member"))?
                .to_string();
            let args = self.parse_sources(call_obj.get("args"))?;
            let result = call_obj.get("result")
                .and_then(|r| r.as_u64())
                .ok_or_else(|| RpcError::bad_request("Call missing result index"))?
                as u32;

            Ok(Op::Call {
                call: CallOp {
                    target,
                    member,
                    args,
                    result,
                }
            })
        }
        // Check if it's an object operation
        else if let Some(obj_obj) = json.get("object") {
            let mut fields = std::collections::BTreeMap::new();
            if let Some(fields_obj) = obj_obj.get("fields").and_then(|f| f.as_object()) {
                for (key, val) in fields_obj {
                    fields.insert(key.clone(), self.parse_source(Some(val))?);
                }
            }
            let result = obj_obj.get("result")
                .and_then(|r| r.as_u64())
                .ok_or_else(|| RpcError::bad_request("Object missing result index"))?
                as u32;

            Ok(Op::Object {
                object: ObjectOp {
                    fields,
                    result,
                }
            })
        }
        // Check if it's an array operation
        else if let Some(array_obj) = json.get("array") {
            let items = self.parse_sources(array_obj.get("items"))?;
            let result = array_obj.get("result")
                .and_then(|r| r.as_u64())
                .ok_or_else(|| RpcError::bad_request("Array missing result index"))?
                as u32;

            Ok(Op::Array {
                array: ArrayOp {
                    items,
                    result,
                }
            })
        }
        // For backward compatibility, also check direct type field
        else if let Some(op_type) = json.get("type").and_then(|t| t.as_str()) {
            match op_type {
                "call" => {
                    let target = self.parse_source(json.get("target"))?;
                    let member = json.get("member")
                        .and_then(|m| m.as_str())
                        .ok_or_else(|| RpcError::bad_request("Call missing member"))?
                        .to_string();
                    let args = self.parse_sources(json.get("args"))?;
                    let result = json.get("result")
                        .and_then(|r| r.as_u64())
                        .unwrap_or(0) as u32;

                    Ok(Op::Call {
                        call: CallOp {
                            target,
                            member,
                            args,
                            result,
                        }
                    })
                }
                _ => Err(RpcError::bad_request(&format!("Unknown operation type: {}", op_type)))
            }
        }
        else {
            Err(RpcError::bad_request("Operation must have 'call', 'object', or 'array' field"))
        }
    }

    /// Parse sources array
    fn parse_sources(&self, json: Option<&JsonValue>) -> Result<Vec<Source>, RpcError> {
        if let Some(array) = json.and_then(|j| j.as_array()) {
            array.iter()
                .map(|s| self.parse_source(Some(s)))
                .collect()
        } else {
            Ok(vec![])
        }
    }

    /// Parse a source from JSON
    fn parse_source(&self, json: Option<&JsonValue>) -> Result<Source, RpcError> {
        let json = json.ok_or_else(|| RpcError::bad_request("Missing source"))?;

        // Check for capture
        if let Some(capture_obj) = json.get("capture") {
            let index = capture_obj.get("index")
                .and_then(|i| i.as_u64())
                .ok_or_else(|| RpcError::bad_request("Capture missing index"))?
                as u32;
            Ok(Source::Capture {
                capture: CaptureRef { index }
            })
        }
        // Check for result
        else if let Some(result_obj) = json.get("result") {
            let index = result_obj.get("index")
                .and_then(|i| i.as_u64())
                .ok_or_else(|| RpcError::bad_request("Result missing index"))?
                as u32;
            Ok(Source::Result {
                result: ResultRef { index }
            })
        }
        // Check for param
        else if let Some(param_obj) = json.get("param") {
            let path = if let Some(path_array) = param_obj.get("path").and_then(|p| p.as_array()) {
                path_array.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            } else {
                vec![]
            };
            Ok(Source::Param {
                param: ParamRef { path }
            })
        }
        // Check for byValue
        else if let Some(value_obj) = json.get("byValue") {
            let value = value_obj.get("value")
                .ok_or_else(|| RpcError::bad_request("ByValue missing value"))?;
            Ok(Source::ByValue {
                by_value: ValueRef {
                    value: value.clone()
                }
            })
        }
        // For backward compatibility, check type field
        else if let Some(source_type) = json.get("type").and_then(|t| t.as_str()) {
            match source_type {
                "capture" => {
                    let index = json.get("index")
                        .and_then(|i| i.as_u64())
                        .unwrap_or(0) as u32;
                    Ok(Source::Capture {
                        capture: CaptureRef { index }
                    })
                }
                "result" => {
                    let index = json.get("index")
                        .and_then(|i| i.as_u64())
                        .unwrap_or(0) as u32;
                    Ok(Source::Result {
                        result: ResultRef { index }
                    })
                }
                "param" | "parameter" => {
                    let path = if let Some(path_str) = json.get("path").and_then(|p| p.as_str()) {
                        vec![path_str.to_string()]
                    } else if let Some(path_array) = json.get("path").and_then(|p| p.as_array()) {
                        path_array.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    } else {
                        vec![]
                    };
                    Ok(Source::Param {
                        param: ParamRef { path }
                    })
                }
                "literal" | "value" => {
                    let value = json.get("value")
                        .ok_or_else(|| RpcError::bad_request("Literal source missing value"))?;
                    Ok(Source::ByValue {
                        by_value: ValueRef {
                            value: value.clone()
                        }
                    })
                }
                _ => Err(RpcError::bad_request(&format!("Unknown source type: {}", source_type)))
            }
        } else {
            // If no recognized structure, treat as literal value
            Ok(Source::ByValue {
                by_value: ValueRef {
                    value: json.clone()
                }
            })
        }
    }

    /// Convert JSON value to protocol Value
    fn json_to_value(&self, json: &JsonValue) -> Result<Value, RpcError> {
        match json {
            JsonValue::Null => Ok(Value::Null),
            JsonValue::Bool(b) => Ok(Value::Bool(*b)),
            JsonValue::Number(n) => {
                Ok(Value::Number(n.clone()))
            }
            JsonValue::String(s) => Ok(Value::String(s.clone())),
            JsonValue::Array(arr) => {
                let values = arr.iter()
                    .map(|v| self.json_to_value(v))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::Array(values))
            }
            JsonValue::Object(obj) => {
                let mut map = std::collections::HashMap::new();
                for (key, val) in obj.iter() {
                    map.insert(key.clone(), Box::new(self.json_to_value(val)?));
                }
                Ok(Value::Object(map))
            }
        }
    }

    /// Convert protocol Value to JSON
    fn value_to_json(&self, value: &Value) -> JsonValue {
        match value {
            Value::Null => JsonValue::Null,
            Value::Bool(b) => JsonValue::Bool(*b),
            Value::Number(n) => JsonValue::Number(n.clone()),
            Value::String(s) => JsonValue::String(s.clone()),
            Value::Array(arr) => {
                JsonValue::Array(arr.iter().map(|v| self.value_to_json(v)).collect())
            }
            Value::Object(obj) => {
                let mut json_obj = serde_json::Map::new();
                for (key, val) in obj.iter() {
                    json_obj.insert(key.clone(), self.value_to_json(val));
                }
                JsonValue::Object(json_obj)
            }
            Value::Date(timestamp) => json!(timestamp),
            Value::Error { error_type, message, stack } => json!({
                "error": error_type,
                "message": message,
                "stack": stack
            }),
            Value::Stub(_) => JsonValue::String("__stub__".to_string()),
            Value::Promise(_) => JsonValue::String("__promise__".to_string()),
        }
    }
}

// Mock nested capability for testing
#[derive(Clone, Debug)]
struct NestedCapabilityImpl {
    name: String,
    config: Value,
    parent: Option<CapId>,
}

#[async_trait]
impl RpcTarget for NestedCapabilityImpl {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "getName" => Ok(Value::String(self.name.clone())),
            "getConfig" => Ok(self.config.clone()),
            "process" => {
                // Simple processing for nested capability
                if let Some(input) = args.first() {
                    Ok(Value::String(format!("{} processed: {:?}", self.name, input)))
                } else {
                    Err(RpcError::bad_request("Process requires input"))
                }
            }
            "validate" => {
                // Validation logic
                Ok(Value::Bool(true))
            }
            _ => Err(RpcError::not_found(&format!("Method {} not found on nested capability", method)))
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String(self.name.clone())),
            "config" => Ok(self.config.clone()),
            "parent" => Ok(match &self.parent {
                Some(id) => Value::String(id.as_u64().to_string()),
                None => Value::Null,
            }),
            _ => Err(RpcError::not_found(&format!("Property {} not found", property)))
        }
    }
}

#[async_trait]
impl RpcTarget for AdvancedCapability {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        // Increment call counter
        {
            let mut count = self.call_count.lock().await;
            *count += 1;
        }

        match method {
            // ============================================================================
            // RESUME TOKEN METHODS
            // ============================================================================

            "createResumeToken" => {
                // Parse arguments
                let config = args.first()
                    .and_then(|v| match v {
                        Value::Object(obj) => Some(obj),
                        _ => None
                    })
                    .ok_or_else(|| RpcError::bad_request("createResumeToken requires config object"))?;

                let session_id = config.get("sessionId")
                    .and_then(|v| match v.as_ref() {
                        Value::String(s) => Some(s.as_str()),
                        _ => None
                    })
                    .ok_or_else(|| RpcError::bad_request("Missing sessionId"))?;

                let include_state = config.get("includeState")
                    .and_then(|v| match v.as_ref() {
                        Value::Bool(b) => Some(*b),
                        _ => None
                    })
                    .unwrap_or(true);

                let expiration_minutes = config.get("expirationMinutes")
                    .and_then(|v| match v.as_ref() {
                        Value::Number(n) => n.as_u64(),
                        _ => None
                    })
                    .unwrap_or(60);

                // Create session snapshot
                let snapshot = self.resume_manager.create_snapshot(
                    session_id.to_string(),
                    &self.id_allocator,
                    &self.import_table,
                    &self.export_table,
                    None, // No variable state manager for now
                ).await.map_err(|e| RpcError::internal(&format!("Failed to create snapshot: {:?}", e)))?;

                // Store session state if requested
                if include_state {
                    let mut sessions = self.sessions.write().await;
                    let session_state = sessions.entry(session_id.to_string()).or_insert_with(|| {
                        SessionState {
                            variables: HashMap::new(),
                            operations: Vec::new(),
                            last_result: None,
                            created_at: Utc::now().timestamp(),
                            last_accessed: Utc::now().timestamp(),
                        }
                    });
                    session_state.last_accessed = Utc::now().timestamp();
                }

                // Generate token
                let token = self.resume_manager.generate_token(snapshot.clone())
                    .map_err(|e| RpcError::internal(&format!("Failed to generate token: {:?}", e)))?;

                // Persistent manager will handle storage internally

                let response_json = json!({
                    "token": token,
                    "sessionId": session_id,
                    "expiresAt": Utc::now().timestamp() + (expiration_minutes as i64 * 60),
                    "includesState": include_state
                });
                self.json_to_value(&response_json)
            }

            "restoreSession" => {
                let config = if let Some(Value::Object(obj)) = args.first() {
                    obj
                } else {
                    return Err(RpcError::bad_request("restoreSession requires config object"));
                };

                let token = if let Some(val) = config.get("token") {
                    if let Value::String(s) = &**val {
                        s.as_str()
                    } else {
                        return Err(RpcError::bad_request("Token must be a string"));
                    }
                } else {
                    return Err(RpcError::bad_request("Missing token"));
                };

                // Parse token from JSON
                let resume_token: ResumeToken = serde_json::from_str(token)
                    .map_err(|e| RpcError::bad_request(&format!("Invalid token format: {}", e)))?;

                // Parse and validate token
                let snapshot = self.resume_manager.parse_token(&resume_token)
                    .map_err(|e| RpcError::bad_request(&format!("Invalid token: {:?}", e)))?;

                // Restore session state
                let session_id = format!("restored_{}", Utc::now().timestamp());
                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), SessionState {
                    variables: HashMap::new(),
                    operations: Vec::new(),
                    last_result: None,
                    created_at: snapshot.created_at as i64,
                    last_accessed: Utc::now().timestamp(),
                });

                let response = json!({
                    "sessionId": session_id,
                    "restored": true,
                    "createdAt": snapshot.created_at,
                    "version": snapshot.version
                });
                self.json_to_value(&response)
            }

            "setVariable" => {
                let var_name = if let Some(Value::String(s)) = args.first() {
                    s.as_str()
                } else {
                    return Err(RpcError::bad_request("setVariable requires variable name"));
                };

                let var_value = args.get(1)
                    .ok_or_else(|| RpcError::bad_request("setVariable requires value"))?;

                // Find or create current session
                let mut sessions = self.sessions.write().await;
                let session = sessions.entry("default".to_string()).or_insert_with(|| {
                    SessionState {
                        variables: HashMap::new(),
                        operations: Vec::new(),
                        last_result: None,
                        created_at: Utc::now().timestamp(),
                        last_accessed: Utc::now().timestamp(),
                    }
                });

                session.variables.insert(var_name.to_string(), var_value.clone());
                session.last_accessed = Utc::now().timestamp();

                Ok(Value::Bool(true))
            }

            "getVariable" => {
                let var_name = if let Some(Value::String(s)) = args.first() {
                    s.as_str()
                } else {
                    return Err(RpcError::bad_request("getVariable requires variable name"));
                };

                let sessions = self.sessions.read().await;
                let session = sessions.get("default")
                    .ok_or_else(|| RpcError::not_found("No active session"))?;

                session.variables.get(var_name)
                    .cloned()
                    .ok_or_else(|| RpcError::not_found(&format!("Variable {} not found", var_name)))
            }

            // ============================================================================
            // NESTED CAPABILITY METHODS
            // ============================================================================

            "createSubCapability" => {
                let cap_type = if let Some(Value::String(s)) = args.first() {
                    s.as_str()
                } else {
                    return Err(RpcError::bad_request("createSubCapability requires type"));
                };

                let config = args.get(1)
                    .cloned()
                    .unwrap_or(Value::Object(std::collections::HashMap::new()));

                // Generate unique capability ID and name
                let mut counter = self.capability_counter.lock().await;
                let cap_id = CapId::new(*counter);
                let cap_name = format!("{}-{}", cap_type, *counter);
                *counter += 1;

                // Create nested capability implementation
                let nested_cap = Arc::new(NestedCapabilityImpl {
                    name: cap_type.to_string(),
                    config: config.clone(),
                    parent: None,  // Could track parent hierarchy
                });

                // Add to capability graph
                use capnweb_core::protocol::nested_capabilities::CapabilityNode;
                let node = CapabilityNode {
                    id: cap_name.clone(),
                    capability_type: cap_type.to_string(),
                    parent_id: None,
                    created_at: chrono::Utc::now().timestamp() as u64,
                    config: config.clone(),
                    metadata: self.capability_factory.get_capability_metadata(cap_type)
                        .unwrap_or(CapabilityMetadata {
                            name: cap_type.to_string(),
                            description: format!("{} capability", cap_type),
                            version: "1.0.0".to_string(),
                            methods: vec![],
                            config_schema: None,
                        }),
                };
                self.capability_graph.add_capability(node).await
                    .map_err(|e| RpcError::internal(&format!("Failed to add capability: {:?}", e)))?;

                // Store in local registry
                let mut capabilities = self.nested_capabilities.write().await;
                capabilities.insert(cap_name.clone(), nested_cap);

                let response = json!({
                    "capabilityId": cap_id.as_u64(),
                    "type": cap_type,
                    "name": cap_name,
                    "config": self.value_to_json(&config)
                });
                self.json_to_value(&response)
            }

            "callSubCapability" => {
                let cap_name = if let Some(Value::String(s)) = args.first() {
                    s.as_str()
                } else {
                    return Err(RpcError::bad_request("callSubCapability requires capability name"));
                };

                let sub_method = if let Some(Value::String(s)) = args.get(1) {
                    s.as_str()
                } else {
                    return Err(RpcError::bad_request("callSubCapability requires method name"));
                };

                let sub_args = args.get(2..)
                    .map(|a| a.to_vec())
                    .unwrap_or_else(Vec::new);

                // Find and call the sub-capability
                let capabilities = self.nested_capabilities.read().await;
                let capability = capabilities.get(cap_name)
                    .ok_or_else(|| RpcError::not_found(&format!("Capability {} not found", cap_name)))?;

                capability.call(sub_method, sub_args).await
            }

            "disposeSubCapability" => {
                let cap_name = if let Some(Value::String(s)) = args.first() {
                    s.as_str()
                } else {
                    return Err(RpcError::bad_request("disposeSubCapability requires capability name"));
                };

                // Remove from registry
                let mut capabilities = self.nested_capabilities.write().await;
                capabilities.remove(cap_name)
                    .ok_or_else(|| RpcError::not_found(&format!("Capability {} not found", cap_name)))?;

                Ok(Value::Bool(true))
            }

            "listSubCapabilities" => {
                let capabilities = self.nested_capabilities.read().await;
                let cap_list: Vec<String> = capabilities.keys().cloned().collect();

                Ok(Value::Array(
                    cap_list.into_iter()
                        .map(|name| Value::String(name))
                        .collect()
                ))
            }

            // ============================================================================
            // IL PLAN RUNNER METHODS
            // ============================================================================

            "executePlan" => {
                let plan_json = args.first()
                    .ok_or_else(|| RpcError::bad_request("executePlan requires plan"))?;

                let parameters = args.get(1)
                    .cloned()
                    .unwrap_or(Value::Object(std::collections::HashMap::new()));

                // Convert JSON to plan
                let json_value = self.value_to_json(plan_json);
                let plan = self.parse_plan(&json_value)?;

                // Get captures (capabilities to use in plan)
                let captures = if let Some(Value::Array(cap_array)) = args.get(2) {
                    let mut captured_caps = Vec::new();
                    for cap_ref in cap_array {
                        if let Value::String(cap_name) = cap_ref {
                            let capabilities = self.nested_capabilities.read().await;
                            if let Some(cap) = capabilities.get(cap_name) {
                                captured_caps.push(cap.clone() as Arc<dyn RpcTarget>);
                            }
                        }
                    }
                    captured_caps
                } else {
                    vec![Arc::new(self.clone()) as Arc<dyn RpcTarget>]
                };

                // Execute the plan
                let result = self.plan_runner.execute_plan(&plan, parameters, captures).await
                    .map_err(|e| RpcError::internal(&format!("Plan execution failed: {:?}", e)))?;

                // Store result in session
                {
                    let mut sessions = self.sessions.write().await;
                    let session = sessions.entry("default".to_string()).or_insert_with(|| {
                        SessionState {
                            variables: HashMap::new(),
                            operations: Vec::new(),
                            last_result: None,
                            created_at: Utc::now().timestamp(),
                            last_accessed: Utc::now().timestamp(),
                        }
                    });
                    session.last_result = Some(result.clone());
                    session.operations.push("executePlan".to_string());
                }

                Ok(result)
            }

            "createPlan" => {
                let plan_name = if let Some(Value::String(s)) = args.first() {
                    s.as_str()
                } else {
                    return Err(RpcError::bad_request("createPlan requires name"));
                };

                let operations = args.get(1)
                    .ok_or_else(|| RpcError::bad_request("createPlan requires operations"))?;

                // Parse and cache the plan
                let json_ops = self.value_to_json(operations);
                let plan = self.parse_plan(&json!({
                    "operations": json_ops
                }))?;

                let mut cache = self.plan_cache.write().await;
                cache.insert(plan_name.to_string(), plan);

                let response = json!({
                    "planName": plan_name,
                    "cached": true
                });
                self.json_to_value(&response)
            }

            "executeCachedPlan" => {
                let plan_name = if let Some(Value::String(s)) = args.first() {
                    s.as_str()
                } else {
                    return Err(RpcError::bad_request("executeCachedPlan requires plan name"));
                };

                let parameters = args.get(1)
                    .cloned()
                    .unwrap_or(Value::Object(std::collections::HashMap::new()));

                // Get cached plan
                let cache = self.plan_cache.read().await;
                let plan = cache.get(plan_name)
                    .ok_or_else(|| RpcError::not_found(&format!("Plan {} not found", plan_name)))?
                    .clone();

                // Execute it
                let result = self.plan_runner.execute_plan(&plan, parameters, vec![]).await
                    .map_err(|e| RpcError::internal(&format!("Plan execution failed: {:?}", e)))?;

                Ok(result)
            }

            // ============================================================================
            // BASIC CALCULATOR METHODS (for compatibility with existing tests)
            // ============================================================================

            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("add requires 2 arguments"));
                }

                let a = if let Value::Number(n) = &args[0] {
                    n.as_f64().ok_or_else(|| RpcError::bad_request("Invalid number"))?
                } else {
                    return Err(RpcError::bad_request("First argument must be a number"));
                };
                let b = if let Value::Number(n) = &args[1] {
                    n.as_f64().ok_or_else(|| RpcError::bad_request("Invalid number"))?
                } else {
                    return Err(RpcError::bad_request("Second argument must be a number"));
                };

                let result = Value::Number(serde_json::Number::from_f64(a + b)
                    .ok_or_else(|| RpcError::internal("Invalid number result"))?);

                // Store in session
                {
                    let mut sessions = self.sessions.write().await;
                    let session = sessions.entry("default".to_string()).or_insert_with(|| {
                        SessionState {
                            variables: HashMap::new(),
                            operations: Vec::new(),
                            last_result: None,
                            created_at: Utc::now().timestamp(),
                            last_accessed: Utc::now().timestamp(),
                        }
                    });
                    session.last_result = Some(result.clone());
                    session.operations.push("add".to_string());
                }

                Ok(result)
            }

            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("multiply requires 2 arguments"));
                }

                let a = if let Value::Number(n) = &args[0] {
                    n.as_f64().ok_or_else(|| RpcError::bad_request("Invalid number"))?
                } else {
                    return Err(RpcError::bad_request("First argument must be a number"));
                };
                let b = if let Value::Number(n) = &args[1] {
                    n.as_f64().ok_or_else(|| RpcError::bad_request("Invalid number"))?
                } else {
                    return Err(RpcError::bad_request("Second argument must be a number"));
                };

                let result = Value::Number(serde_json::Number::from_f64(a * b)
                    .ok_or_else(|| RpcError::internal("Invalid number result"))?);

                // Store in session
                {
                    let mut sessions = self.sessions.write().await;
                    let session = sessions.entry("default".to_string()).or_insert_with(|| {
                        SessionState {
                            variables: HashMap::new(),
                            operations: Vec::new(),
                            last_result: None,
                            created_at: Utc::now().timestamp(),
                            last_accessed: Utc::now().timestamp(),
                        }
                    });
                    session.last_result = Some(result.clone());
                    session.operations.push("multiply".to_string());
                }

                Ok(result)
            }

            "getStats" => {
                let count = *self.call_count.lock().await;
                let sessions = self.sessions.read().await;
                let capabilities = self.nested_capabilities.read().await;
                let plans = self.plan_cache.read().await;

                let response = json!({
                    "totalCalls": count,
                    "activeSessions": sessions.len(),
                    "nestedCapabilities": capabilities.len(),
                    "cachedPlans": plans.len()
                });
                self.json_to_value(&response)
            }

            _ => Err(RpcError::not_found(&format!("Method {} not found", method)))
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "total_calls" => {
                let count = *self.call_count.lock().await;
                Ok(Value::Number(serde_json::Number::from(count)))
            }
            "session_count" => {
                let sessions = self.sessions.read().await;
                Ok(Value::Number(serde_json::Number::from(sessions.len())))
            }
            "capability_count" => {
                let capabilities = self.nested_capabilities.read().await;
                Ok(Value::Number(serde_json::Number::from(capabilities.len())))
            }
            "cached_plans" => {
                let plans = self.plan_cache.read().await;
                Ok(Value::Number(serde_json::Number::from(plans.len())))
            }
            _ => Err(RpcError::not_found(&format!("Property {} not found", property)))
        }
    }
}

// Implement Clone for use in Arc
impl Clone for AdvancedCapability {
    fn clone(&self) -> Self {
        Self {
            resume_manager: self.resume_manager.clone(),
            persistent_manager: self.persistent_manager.clone(),
            sessions: self.sessions.clone(),
            capability_graph: self.capability_graph.clone(),
            capability_factory: self.capability_factory.clone(),
            capability_counter: self.capability_counter.clone(),
            plan_runner: self.plan_runner.clone(),
            plan_cache: self.plan_cache.clone(),
            id_allocator: self.id_allocator.clone(),
            import_table: self.import_table.clone(),
            export_table: self.export_table.clone(),
            call_count: self.call_count.clone(),
            nested_capabilities: self.nested_capabilities.clone(),
        }
    }
}

/// Builder for AdvancedCapability with fluent API
pub struct AdvancedCapabilityBuilder {
    config: AdvancedCapabilityConfig,
}

impl AdvancedCapabilityBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: AdvancedCapabilityConfig::default(),
        }
    }

    /// Set the secret key for token encryption
    pub fn with_secret_key(mut self, key: Vec<u8>) -> Self {
        self.config.secret_key = Some(key);
        self
    }

    /// Set token time-to-live in seconds
    pub fn with_token_ttl(mut self, ttl: u64) -> Self {
        self.config.token_ttl = ttl;
        self
    }

    /// Set maximum session age in seconds
    pub fn with_max_session_age(mut self, age: u64) -> Self {
        self.config.max_session_age = age;
        self
    }

    /// Set maximum number of capabilities
    pub fn with_max_capabilities(mut self, max: usize) -> Self {
        self.config.max_capabilities = max;
        self
    }

    /// Set maximum operations per plan
    pub fn with_max_plan_operations(mut self, max: usize) -> Self {
        self.config.max_plan_operations = max;
        self
    }

    /// Set plan execution timeout in milliseconds
    pub fn with_plan_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.plan_timeout_ms = timeout_ms;
        self
    }

    /// Build the AdvancedCapability instance
    pub fn build(self) -> AdvancedCapability {
        AdvancedCapability::with_config(self.config)
    }
}

impl Default for AdvancedCapabilityBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_advanced_capability_creation() {
        let cap = AdvancedCapability::new();

        // Test basic functionality
        let result = cap.call("getStats", vec![]).await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        if let Value::Object(obj) = stats {
            assert!(obj.contains_key("totalCalls"));
            assert!(obj.contains_key("activeSessions"));
        } else {
            panic!("Expected object result");
        }
    }

    #[tokio::test]
    async fn test_resume_token_methods() {
        let cap = AdvancedCapability::new();

        // Create resume token config
        let mut config_map = std::collections::HashMap::new();
        config_map.insert("sessionId".to_string(), Box::new(Value::String("test123".to_string())));
        config_map.insert("includeState".to_string(), Box::new(Value::Bool(true)));
        config_map.insert("expirationMinutes".to_string(), Box::new(Value::Number(serde_json::Number::from(60))));
        let config = Value::Object(config_map);

        let result = cap.call("createResumeToken", vec![config]).await;
        assert!(result.is_ok());

        let token_response = result.unwrap();
        if let Value::Object(obj) = token_response {
            assert!(obj.contains_key("token"));
            assert!(obj.contains_key("sessionId"));
            assert!(obj.contains_key("expiresAt"));
        } else {
            panic!("Expected object result");
        }
    }

    #[tokio::test]
    async fn test_nested_capability_methods() {
        let cap = AdvancedCapability::new();

        // Create sub-capability
        let mut config_map = std::collections::HashMap::new();
        config_map.insert("maxLength".to_string(), Box::new(Value::Number(serde_json::Number::from(100))));

        let result = cap.call("createSubCapability", vec![
            Value::String("validator".to_string()),
            Value::Object(config_map)
        ]).await;

        assert!(result.is_ok());

        let cap_response = result.unwrap();
        if let Value::Object(obj) = cap_response {
            assert!(obj.contains_key("capabilityId"));
            assert!(obj.contains_key("type"));
            assert!(obj.contains_key("name"));
        } else {
            panic!("Expected object result");
        }

        // List capabilities
        let list_result = cap.call("listSubCapabilities", vec![]).await;
        assert!(list_result.is_ok());

        if let Value::Array(arr) = list_result.unwrap() {
            assert!(!arr.is_empty());
        } else {
            panic!("Expected array result");
        }
    }

    #[tokio::test]
    async fn test_il_plan_execution() {
        let cap = AdvancedCapability::new();

        // Create a simple plan
        let plan = Value::Object(json!({
            "operations": [
                {
                    "type": "return",
                    "value": {
                        "type": "literal",
                        "value": "Hello from IL!"
                    }
                }
            ]
        }).as_object().unwrap().clone());

        let result = cap.call("executePlan", vec![plan, Value::Null]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("Hello from IL!".to_string()));
    }

    #[tokio::test]
    async fn test_calculator_compatibility() {
        let cap = AdvancedCapability::new();

        // Test add
        let result = cap.call("add", vec![Value::Number(10.0), Value::Number(20.0)]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(30.0));

        // Test multiply
        let result = cap.call("multiply", vec![Value::Number(5.0), Value::Number(6.0)]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(30.0));
    }
}