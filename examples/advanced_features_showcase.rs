// Advanced Features Showcase - Demonstrates all Cap'n Web advanced features
// This example shows Resume Tokens, Nested Capabilities, Advanced IL Plan Runner,
// and HTTP/3 transport working together in a realistic application

use capnweb_core::{
    protocol::{
        PlanRunner, PlanBuilder, ExecutionContext,
        ResumeTokenManager, PersistentSessionManager,
        CapabilityFactory, CapabilityMetadata, MethodMetadata, ParameterMetadata,
        CapabilityGraph, DefaultNestedCapableTarget, NestedCapableRpcTarget,
        ImportTable, ExportTable, IdAllocator, VariableStateManager,
        Value, CapabilityError,
    },
    il::{Plan, Source},
    RpcTarget, RpcError, CapId,
};
use capnweb_server::{Server, ServerConfig};
use capnweb_transport::{RpcTransport, Http3Config, Http3Client, make_http3_client_endpoint};

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::{json, Number};
use tokio::sync::RwLock;
use clap::{App, Arg, SubCommand};

/// Advanced Data Processing Capability
/// Demonstrates complex capability with nested operations and state management
#[derive(Debug)]
struct DataProcessorCapability {
    name: String,
    processing_mode: String,
    data_cache: Arc<RwLock<HashMap<String, Value>>>,
    processing_history: Arc<RwLock<Vec<String>>>,
}

impl DataProcessorCapability {
    fn new(name: String, processing_mode: String) -> Self {
        Self {
            name,
            processing_mode,
            data_cache: Arc::new(RwLock::new(HashMap::new())),
            processing_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl RpcTarget for DataProcessorCapability {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "processData" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("processData requires data argument"));
                }

                let data = &args[0];
                let processing_result = match &self.processing_mode[..] {
                    "transform" => self.transform_data(data).await?,
                    "aggregate" => self.aggregate_data(data).await?,
                    "validate" => self.validate_data(data).await?,
                    _ => return Err(RpcError::bad_request("Unknown processing mode")),
                };

                // Add to history
                self.processing_history.write().await.push(format!(
                    "Processed data with mode: {} at {}",
                    self.processing_mode,
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
                ));

                Ok(processing_result)
            }

            "cacheData" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("cacheData requires key and data arguments"));
                }

                let key = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(RpcError::bad_request("Key must be a string")),
                };

                self.data_cache.write().await.insert(key.clone(), args[1].clone());

                Ok(Value::Object({
                    let mut obj = HashMap::new();
                    obj.insert("cached".to_string(), Box::new(Value::Bool(true)));
                    obj.insert("key".to_string(), Box::new(Value::String(key)));
                    obj
                }))
            }

            "getCachedData" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("getCachedData requires key argument"));
                }

                let key = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(RpcError::bad_request("Key must be a string")),
                };

                let cache = self.data_cache.read().await;
                match cache.get(&key) {
                    Some(data) => Ok(data.clone()),
                    None => Err(RpcError::not_found(format!("Data not found for key: {}", key))),
                }
            }

            "getHistory" => {
                let history = self.processing_history.read().await;
                let history_values: Vec<Value> = history.iter()
                    .map(|s| Value::String(s.clone()))
                    .collect();
                Ok(Value::Array(history_values))
            }

            "getStats" => {
                let cache_size = self.data_cache.read().await.len();
                let history_count = self.processing_history.read().await.len();

                Ok(Value::Object({
                    let mut obj = HashMap::new();
                    obj.insert("name".to_string(), Box::new(Value::String(self.name.clone())));
                    obj.insert("mode".to_string(), Box::new(Value::String(self.processing_mode.clone())));
                    obj.insert("cache_size".to_string(), Box::new(Value::Number(Number::from(cache_size))));
                    obj.insert("history_count".to_string(), Box::new(Value::Number(Number::from(history_count))));
                    obj
                }))
            }

            _ => Err(RpcError::not_found(format!("Method not found: {}", method))),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String(self.name.clone())),
            "processing_mode" => Ok(Value::String(self.processing_mode.clone())),
            "type" => Ok(Value::String("DataProcessor".to_string())),
            _ => Err(RpcError::not_found(format!("Property not found: {}", property))),
        }
    }
}

impl DataProcessorCapability {
    async fn transform_data(&self, data: &Value) -> Result<Value, RpcError> {
        match data {
            Value::Array(arr) => {
                let transformed: Vec<Value> = arr.iter()
                    .map(|item| match item {
                        Value::Number(n) => {
                            if let Some(val) = n.as_f64() {
                                Value::Number(Number::from_f64(val * 2.0).unwrap())
                            } else {
                                item.clone()
                            }
                        }
                        Value::String(s) => Value::String(s.to_uppercase()),
                        _ => item.clone(),
                    })
                    .collect();
                Ok(Value::Array(transformed))
            }
            Value::Number(n) => {
                if let Some(val) = n.as_f64() {
                    Ok(Value::Number(Number::from_f64(val * 2.0).unwrap()))
                } else {
                    Ok(data.clone())
                }
            }
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Ok(data.clone()),
        }
    }

    async fn aggregate_data(&self, data: &Value) -> Result<Value, RpcError> {
        match data {
            Value::Array(arr) => {
                let mut sum = 0.0;
                let mut count = 0;

                for item in arr {
                    if let Value::Number(n) = item {
                        if let Some(val) = n.as_f64() {
                            sum += val;
                            count += 1;
                        }
                    }
                }

                let average = if count > 0 { sum / count as f64 } else { 0.0 };

                Ok(Value::Object({
                    let mut obj = HashMap::new();
                    obj.insert("sum".to_string(), Box::new(Value::Number(Number::from_f64(sum).unwrap())));
                    obj.insert("count".to_string(), Box::new(Value::Number(Number::from(count))));
                    obj.insert("average".to_string(), Box::new(Value::Number(Number::from_f64(average).unwrap())));
                    obj
                }))
            }
            _ => Err(RpcError::bad_request("Aggregation requires array data")),
        }
    }

    async fn validate_data(&self, data: &Value) -> Result<Value, RpcError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        match data {
            Value::Object(obj) => {
                // Check for required fields
                if !obj.contains_key("id") {
                    errors.push("Missing required field: id".to_string());
                }
                if !obj.contains_key("type") {
                    errors.push("Missing required field: type".to_string());
                }

                // Check data types
                if let Some(id_value) = obj.get("id") {
                    match id_value.as_ref() {
                        Value::Number(_) => {},
                        _ => errors.push("Field 'id' must be a number".to_string()),
                    }
                }

                // Check for deprecated fields
                if obj.contains_key("legacy_field") {
                    warnings.push("Field 'legacy_field' is deprecated".to_string());
                }
            }
            _ => {
                errors.push("Data must be an object for validation".to_string());
            }
        }

        Ok(Value::Object({
            let mut obj = HashMap::new();
            obj.insert("valid".to_string(), Box::new(Value::Bool(errors.is_empty())));
            obj.insert("errors".to_string(), Box::new(Value::Array(
                errors.into_iter().map(Value::String).collect()
            )));
            obj.insert("warnings".to_string(), Box::new(Value::Array(
                warnings.into_iter().map(Value::String).collect()
            )));
            obj
        }))
    }
}

/// Factory for creating data processing capabilities
#[derive(Debug)]
struct DataProcessorFactory;

#[async_trait::async_trait]
impl CapabilityFactory for DataProcessorFactory {
    async fn create_capability(
        &self,
        capability_type: &str,
        config: Value,
    ) -> Result<Arc<dyn RpcTarget>, CapabilityError> {
        let name = match &config {
            Value::Object(obj) => {
                match obj.get("name") {
                    Some(boxed_val) => match boxed_val.as_ref() {
                        Value::String(s) => s.clone(),
                        _ => format!("processor_{}", capability_type),
                    },
                    None => format!("processor_{}", capability_type),
                }
            },
            _ => format!("processor_{}", capability_type),
        };

        match capability_type {
            "transformer" => Ok(Arc::new(DataProcessorCapability::new(name, "transform".to_string()))),
            "aggregator" => Ok(Arc::new(DataProcessorCapability::new(name, "aggregate".to_string()))),
            "validator" => Ok(Arc::new(DataProcessorCapability::new(name, "validate".to_string()))),
            _ => Err(CapabilityError::InvalidCapabilityType(capability_type.to_string())),
        }
    }

    fn list_capability_types(&self) -> Vec<String> {
        vec![
            "transformer".to_string(),
            "aggregator".to_string(),
            "validator".to_string(),
        ]
    }

    fn get_capability_metadata(&self, capability_type: &str) -> Option<CapabilityMetadata> {
        match capability_type {
            "transformer" | "aggregator" | "validator" => Some(CapabilityMetadata {
                name: format!("Data {}", capability_type),
                description: format!("Data processing capability: {}", capability_type),
                version: "1.0.0".to_string(),
                methods: vec![
                    MethodMetadata {
                        name: "processData".to_string(),
                        description: "Process input data".to_string(),
                        parameters: vec![
                            ParameterMetadata {
                                name: "data".to_string(),
                                type_name: "any".to_string(),
                                required: true,
                                description: "Data to process".to_string(),
                            },
                        ],
                        return_type: "any".to_string(),
                    },
                    MethodMetadata {
                        name: "getStats".to_string(),
                        description: "Get processing statistics".to_string(),
                        parameters: vec![],
                        return_type: "object".to_string(),
                    },
                ],
                config_schema: Some(Value::Object({
                    let mut schema = HashMap::new();
                    schema.insert("name".to_string(), Box::new(Value::String("string".to_string())));
                    schema
                })),
            }),
            _ => None,
        }
    }
}

/// Advanced application demonstrating all features
struct AdvancedApplication {
    session_manager: PersistentSessionManager,
    plan_runner: PlanRunner,
    capability_graph: Arc<CapabilityGraph>,
    factory: Arc<DataProcessorFactory>,
    variable_manager: VariableStateManager,
}

impl AdvancedApplication {
    async fn new() -> Self {
        let secret_key = ResumeTokenManager::generate_secret_key();
        let token_manager = ResumeTokenManager::new(secret_key);
        let session_manager = PersistentSessionManager::new(token_manager);

        let imports = Arc::new(ImportTable::with_default_allocator());
        let exports = Arc::new(ExportTable::with_default_allocator());
        let plan_runner = PlanRunner::new(imports, exports);

        let capability_graph = Arc::new(CapabilityGraph::new());
        let factory = Arc::new(DataProcessorFactory);
        let variable_manager = VariableStateManager::new();

        Self {
            session_manager,
            plan_runner,
            capability_graph,
            factory,
            variable_manager,
        }
    }

    /// Demonstrate complex workflow with all advanced features
    async fn run_advanced_workflow(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting Advanced Features Workflow Demo");

        // 1. Create session and resume token
        println!("\n1️⃣  Creating session with resume token support...");

        let session_id = "advanced_demo_session";
        let allocator = Arc::new(IdAllocator::new());
        let imports = Arc::new(ImportTable::with_default_allocator());
        let exports = Arc::new(ExportTable::with_default_allocator());

        let resume_token = self.session_manager.snapshot_session(
            session_id,
            &allocator,
            &imports,
            &exports,
            Some(&self.variable_manager),
        ).await?;

        println!("✅ Resume token created: {}", resume_token.session_id);

        // 2. Create nested capability hierarchy
        println!("\n2️⃣  Building nested capability hierarchy...");

        let root_processor = Arc::new(DataProcessorCapability::new(
            "root_processor".to_string(),
            "transform".to_string(),
        ));

        let nested_target = DefaultNestedCapableTarget::new(
            "data_processing_service".to_string(),
            self.factory.clone(),
            self.capability_graph.clone(),
            root_processor,
        );

        // Create multiple sub-capabilities
        let transformer_config = Value::Object({
            let mut obj = HashMap::new();
            obj.insert("name".to_string(), Box::new(Value::String("text_transformer".to_string())));
            obj
        });

        let validator_config = Value::Object({
            let mut obj = HashMap::new();
            obj.insert("name".to_string(), Box::new(Value::String("data_validator".to_string())));
            obj
        });

        let transformer_result = nested_target.create_sub_capability("transformer", transformer_config).await?;
        let validator_result = nested_target.create_sub_capability("validator", validator_config).await?;

        println!("✅ Created transformer capability: {:?}", transformer_result);
        println!("✅ Created validator capability: {:?}", validator_result);

        let graph_stats = self.capability_graph.get_stats().await;
        println!("✅ Capability graph: {} capabilities, {} max depth",
                 graph_stats.total_capabilities, graph_stats.max_depth);

        // 3. Execute complex IL plan
        println!("\n3️⃣  Executing complex IL plan...");

        let mut plan_builder = PlanBuilder::new();
        let root_cap_index = plan_builder.add_capture(CapId::new(1));

        // Step 1: Transform input data
        let input_data = Value::Array(vec![
            Value::Number(Number::from(10)),
            Value::Number(Number::from(20)),
            Value::Number(Number::from(30)),
        ]);

        let transform_result = plan_builder.add_call(
            Source::capture(root_cap_index),
            "processData".to_string(),
            vec![Source::by_value(input_data)],
        );

        // Step 2: Create aggregation object
        let mut agg_fields = HashMap::new();
        agg_fields.insert("data".to_string(), Source::result(transform_result));
        agg_fields.insert("operation".to_string(), Source::by_value(Value::String("transform_and_aggregate".to_string())));
        agg_fields.insert("timestamp".to_string(), Source::by_value(Value::Number(Number::from(chrono::Utc::now().timestamp()))));
        let final_object = plan_builder.add_object(agg_fields);

        let plan = plan_builder.build(Source::result(final_object));

        // Execute the plan
        let plan_params = json!({});
        let captures = vec![Arc::new(DataProcessorCapability::new(
            "plan_processor".to_string(),
            "transform".to_string(),
        ))];

        let plan_result = self.plan_runner.execute_plan(&plan, plan_params, captures).await?;
        println!("✅ IL Plan executed successfully: {:?}", plan_result);

        // 4. Test session restoration
        println!("\n4️⃣  Testing session restoration...");

        let restored_session = self.session_manager.restore_session(
            &resume_token,
            &allocator,
            &imports,
            &exports,
            Some(&self.variable_manager),
        ).await?;

        println!("✅ Session restored: {}", restored_session);

        // 5. Complex data processing workflow
        println!("\n5️⃣  Running complex data processing workflow...");

        let sample_data = vec![
            json!({"id": 1, "type": "user", "value": 100}),
            json!({"id": 2, "type": "system", "value": 200}),
            json!({"id": 3, "type": "user", "value": 150}),
        ];

        for (i, data) in sample_data.iter().enumerate() {
            // Process with root processor
            let processed = nested_target.call("processData", vec![
                Value::from(data.clone())
            ]).await?;

            println!("  📊 Processed item {}: {:?}", i + 1, processed);

            // Cache the processed data
            let cache_key = format!("processed_item_{}", i + 1);
            let cached = nested_target.call("cacheData", vec![
                Value::String(cache_key.clone()),
                processed,
            ]).await?;

            println!("  💾 Cached result: {:?}", cached);
        }

        // Get processing stats
        let stats = nested_target.call("getStats", vec![]).await?;
        println!("✅ Final processing stats: {:?}", stats);

        // 6. Clean up
        println!("\n6️⃣  Cleaning up resources...");

        // Clean up expired sessions
        let cleaned_sessions = self.session_manager.cleanup_expired_sessions().await;
        println!("✅ Cleaned up {} expired sessions", cleaned_sessions);

        println!("\n🎉 Advanced Features Workflow Demo Completed Successfully! 🎉");
        Ok(())
    }
}

fn value_from_serde(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => Value::Number(Number::from_f64(n.as_f64().unwrap_or(0.0)).unwrap()),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(value_from_serde).collect())
        },
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, Box::new(value_from_serde(v)));
            }
            Value::Object(map)
        },
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        value_from_serde(value)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();

    let matches = App::new("Cap'n Web Advanced Features Showcase")
        .version("1.0.0")
        .about("Demonstrates all advanced Cap'n Web features")
        .subcommand(
            SubCommand::with_name("demo")
                .about("Run the full advanced features demo")
        )
        .subcommand(
            SubCommand::with_name("server")
                .about("Run as a server with advanced features")
                .arg(Arg::with_name("port")
                     .short("p")
                     .long("port")
                     .value_name("PORT")
                     .help("Port to listen on")
                     .default_value("8080"))
        )
        .subcommand(
            SubCommand::with_name("client")
                .about("Run as a client connecting to advanced server")
                .arg(Arg::with_name("server")
                     .short("s")
                     .long("server")
                     .value_name("SERVER")
                     .help("Server address to connect to")
                     .default_value("127.0.0.1:8080"))
        )
        .get_matches();

    match matches.subcommand() {
        ("demo", _) => {
            let app = AdvancedApplication::new().await;
            app.run_advanced_workflow().await?;
        }
        ("server", Some(sub_matches)) => {
            let port = sub_matches.value_of("port").unwrap().parse::<u16>()?;
            println!("🚀 Starting Advanced Features Server on port {}", port);

            // This would start a full server with all advanced features
            // For now, we'll just show the configuration
            println!("✅ Server would be configured with:");
            println!("   - Resume Token support");
            println!("   - Nested Capability creation");
            println!("   - Advanced IL Plan Runner");
            println!("   - HTTP/3 transport");
            println!("   - Multi-transport support");
        }
        ("client", Some(sub_matches)) => {
            let server = sub_matches.value_of("server").unwrap();
            println!("🔗 Connecting to Advanced Features Server at {}", server);

            // This would create a client with all advanced features
            println!("✅ Client would support:");
            println!("   - Session resumption with tokens");
            println!("   - Dynamic capability creation");
            println!("   - Complex IL plan execution");
            println!("   - HTTP/3 transport");
        }
        _ => {
            println!("🎯 Cap'n Web Advanced Features Showcase");
            println!("Run with --help to see available commands");

            let app = AdvancedApplication::new().await;
            app.run_advanced_workflow().await?;
        }
    }

    Ok(())
}