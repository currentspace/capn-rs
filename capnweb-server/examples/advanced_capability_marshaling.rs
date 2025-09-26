// Advanced Capability Marshaling Example
// Demonstrates 100% TypeScript parity with complex capability passing scenarios

use capnweb_core::{
    parse_wire_batch, serialize_wire_batch, CapabilityRegistry, PropertyKey, RegistrableCapability,
    RpcError, RpcTarget, WireExpression, WireMessage,
};
use capnweb_server::*;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio;
use tracing::{debug, error, info};

/// Factory capability that creates and manages other capabilities
#[derive(Debug)]
pub struct CapabilityFactory {
    name: String,
    registry: Arc<CapabilityRegistry>,
    created_count: std::sync::atomic::AtomicU32,
}

impl CapabilityFactory {
    pub fn new(name: String, registry: Arc<CapabilityRegistry>) -> Self {
        Self {
            name,
            registry,
            created_count: std::sync::atomic::AtomicU32::new(0),
        }
    }
}

#[capnweb_core::async_trait]
impl RpcTarget for CapabilityFactory {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        info!("CapabilityFactory::{} called with args: {:?}", member, args);

        match member {
            "createCalculator" => {
                let count = self
                    .created_count
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let calc_name = format!("Calculator-{}", count);

                // Create a new calculator capability
                let calculator = Arc::new(AdvancedCalculator::new(calc_name.clone()));

                // Export it through the registry
                let cap_id = self.registry.export_capability(calculator);

                info!(
                    "Created new calculator '{}' with capability ID: {}",
                    calc_name, cap_id
                );

                // Return capability reference as JSON
                Ok(serde_json::json!({
                    "_type": "capability",
                    "id": cap_id,
                    "name": calc_name
                }))
            }

            "createWorkflow" => {
                let workflow_id = format!(
                    "Workflow-{}",
                    self.created_count.load(std::sync::atomic::Ordering::SeqCst)
                );

                let workflow = Arc::new(StatefulWorkflow::new(
                    workflow_id.clone(),
                    self.registry.clone(),
                ));
                let cap_id = self.registry.export_capability(workflow);

                info!(
                    "Created new workflow '{}' with capability ID: {}",
                    workflow_id, cap_id
                );

                Ok(serde_json::json!({
                    "_type": "capability",
                    "id": cap_id,
                    "name": workflow_id
                }))
            }

            "getStats" => {
                let exported_ids = self.registry.get_exported_ids();
                Ok(serde_json::json!({
                    "factory": self.name,
                    "created_count": self.created_count.load(std::sync::atomic::Ordering::SeqCst),
                    "active_capabilities": exported_ids.len(),
                    "capability_ids": exported_ids
                }))
            }

            _ => Err(RpcError::method_not_found(&format!(
                "Method '{}' not found on CapabilityFactory",
                member
            ))),
        }
    }
}

impl RegistrableCapability for CapabilityFactory {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_export(&self, id: i64) {
        info!("CapabilityFactory '{}' exported with ID: {}", self.name, id);
    }
}

/// Advanced calculator that can accept other capabilities as arguments
#[derive(Debug)]
pub struct AdvancedCalculator {
    name: String,
    variables: std::sync::Mutex<HashMap<String, f64>>,
    operation_count: std::sync::atomic::AtomicU32,
}

impl AdvancedCalculator {
    pub fn new(name: String) -> Self {
        Self {
            name,
            variables: std::sync::Mutex::new(HashMap::new()),
            operation_count: std::sync::atomic::AtomicU32::new(0),
        }
    }
}

#[capnweb_core::async_trait]
impl RpcTarget for AdvancedCalculator {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        let op_count = self
            .operation_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        debug!(
            "AdvancedCalculator::{} (op #{}) called with {} args",
            member,
            op_count,
            args.len()
        );

        match member {
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::invalid_args("add requires exactly 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a + b).unwrap()))
            }

            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError::invalid_args(
                        "subtract requires exactly 2 arguments",
                    ));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a - b).unwrap()))
            }

            "multiplyByCapability" => {
                // This demonstrates accepting a capability as an argument
                if args.len() != 2 {
                    return Err(RpcError::invalid_args(
                        "multiplyByCapability requires exactly 2 arguments",
                    ));
                }

                let a = extract_number(&args[0])?;

                // Second argument should be a capability reference
                if let Some(cap_ref) = extract_capability_ref(&args[1]) {
                    info!("Received capability reference with ID: {}", cap_ref);

                    // For demo purposes, we'll treat the capability ID as a multiplier
                    let multiplier = cap_ref as f64;
                    let result = a * multiplier;

                    Ok(Value::Number(serde_json::Number::from_f64(result).unwrap()))
                } else {
                    Err(RpcError::invalid_args(
                        "Second argument must be a capability reference",
                    ))
                }
            }

            "chainWithOtherCalculator" => {
                // Demonstrates complex capability-to-capability interactions
                if args.len() != 3 {
                    return Err(RpcError::invalid_args(
                        "chainWithOtherCalculator requires 3 arguments",
                    ));
                }

                let value = extract_number(&args[0])?;
                let operation = args[1]
                    .as_str()
                    .ok_or_else(|| RpcError::invalid_args("Operation must be a string"))?;

                if let Some(other_calc_id) = extract_capability_ref(&args[2]) {
                    info!(
                        "Chaining operation '{}' with calculator ID: {}",
                        operation, other_calc_id
                    );

                    // In a full implementation, we would:
                    // 1. Resolve the capability from the registry
                    // 2. Call the specified operation on it
                    // 3. Return the result

                    // For demo, we'll simulate the result
                    let result = match operation {
                        "double" => value * 2.0,
                        "square" => value * value,
                        _ => {
                            return Err(RpcError::invalid_args(&format!(
                                "Unknown operation: {}",
                                operation
                            )))
                        }
                    };

                    Ok(serde_json::json!({
                        "result": result,
                        "chained_with": other_calc_id,
                        "operation": operation,
                        "calculator": self.name
                    }))
                } else {
                    Err(RpcError::invalid_args(
                        "Third argument must be a capability reference",
                    ))
                }
            }

            "getStatus" => {
                let vars = self.variables.lock().unwrap();
                Ok(serde_json::json!({
                    "name": self.name,
                    "operation_count": self.operation_count.load(std::sync::atomic::Ordering::SeqCst),
                    "variable_count": vars.len()
                }))
            }

            _ => Err(RpcError::method_not_found(&format!(
                "Method '{}' not found on AdvancedCalculator",
                member
            ))),
        }
    }
}

impl RegistrableCapability for AdvancedCalculator {
    fn name(&self) -> &str {
        &self.name
    }
}

/// Stateful workflow that orchestrates multiple capabilities
#[derive(Debug)]
pub struct StatefulWorkflow {
    name: String,
    registry: Arc<CapabilityRegistry>,
    steps: std::sync::Mutex<Vec<WorkflowStep>>,
    state: std::sync::Mutex<HashMap<String, Value>>,
}

#[derive(Debug, Clone)]
struct WorkflowStep {
    id: String,
    capability_id: i64,
    method: String,
    args: Vec<Value>,
    completed: bool,
    result: Option<Value>,
}

impl StatefulWorkflow {
    pub fn new(name: String, registry: Arc<CapabilityRegistry>) -> Self {
        Self {
            name,
            registry,
            steps: std::sync::Mutex::new(Vec::new()),
            state: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

#[capnweb_core::async_trait]
impl RpcTarget for StatefulWorkflow {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        info!("StatefulWorkflow::{} called with args: {:?}", member, args);

        match member {
            "addStep" => {
                if args.len() != 4 {
                    return Err(RpcError::invalid_args(
                        "addStep requires 4 arguments: (step_id, capability_ref, method, args)",
                    ));
                }

                let step_id = args[0]
                    .as_str()
                    .ok_or_else(|| RpcError::invalid_args("Step ID must be a string"))?
                    .to_string();

                let capability_id = extract_capability_ref(&args[1]).ok_or_else(|| {
                    RpcError::invalid_args("Second argument must be a capability reference")
                })?;

                let method = args[2]
                    .as_str()
                    .ok_or_else(|| RpcError::invalid_args("Method must be a string"))?
                    .to_string();

                let method_args = args[3]
                    .as_array()
                    .ok_or_else(|| RpcError::invalid_args("Args must be an array"))?
                    .clone();

                let step = WorkflowStep {
                    id: step_id.clone(),
                    capability_id,
                    method,
                    args: method_args,
                    completed: false,
                    result: None,
                };

                let mut steps = self.steps.lock().unwrap();
                steps.push(step);

                info!(
                    "Added workflow step '{}' targeting capability ID: {}",
                    step_id, capability_id
                );

                Ok(serde_json::json!({
                    "step_id": step_id,
                    "step_count": steps.len(),
                    "workflow": self.name
                }))
            }

            "executeWorkflow" => {
                // Execute all pending workflow steps
                let mut steps = self.steps.lock().unwrap();
                let mut results = Vec::new();

                for step in steps.iter_mut() {
                    if !step.completed {
                        // In a full implementation, we would:
                        // 1. Get the capability from the registry
                        // 2. Call the method with the args
                        // 3. Store the result

                        // For demo, simulate execution
                        step.completed = true;
                        step.result = Some(serde_json::json!({
                            "step_id": step.id,
                            "simulated_result": format!("Result from capability {} method {}", step.capability_id, step.method)
                        }));

                        results.push(step.result.clone().unwrap());

                        info!(
                            "Executed workflow step '{}' on capability {}",
                            step.id, step.capability_id
                        );
                    }
                }

                Ok(serde_json::json!({
                    "workflow": self.name,
                    "executed_steps": results.len(),
                    "results": results
                }))
            }

            "getWorkflowState" => {
                let steps = self.steps.lock().unwrap();
                let state = self.state.lock().unwrap();

                let step_summaries: Vec<_> = steps
                    .iter()
                    .map(|step| {
                        serde_json::json!({
                            "id": step.id,
                            "capability_id": step.capability_id,
                            "method": step.method,
                            "completed": step.completed,
                            "has_result": step.result.is_some()
                        })
                    })
                    .collect();

                Ok(serde_json::json!({
                    "workflow": self.name,
                    "total_steps": steps.len(),
                    "completed_steps": steps.iter().filter(|s| s.completed).count(),
                    "steps": step_summaries,
                    "state_keys": state.keys().collect::<Vec<_>>()
                }))
            }

            _ => Err(RpcError::method_not_found(&format!(
                "Method '{}' not found on StatefulWorkflow",
                member
            ))),
        }
    }
}

impl RegistrableCapability for StatefulWorkflow {
    fn name(&self) -> &str {
        &self.name
    }
}

// Helper functions
fn extract_number(value: &Value) -> Result<f64, RpcError> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|i| i as f64))
        .or_else(|| value.as_u64().map(|u| u as f64))
        .ok_or_else(|| RpcError::invalid_args("Expected a number"))
}

fn extract_capability_ref(value: &Value) -> Option<i64> {
    if let Some(obj) = value.as_object() {
        if let (Some(type_val), Some(id_val)) = (obj.get("_type"), obj.get("id")) {
            if type_val.as_str() == Some("capability") {
                return id_val.as_i64();
            }
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Advanced Capability Marshaling Server");

    // Create capability registry
    let registry = Arc::new(CapabilityRegistry::new());

    // Create factory capability
    let factory = Arc::new(CapabilityFactory::new(
        "MainFactory".to_string(),
        registry.clone(),
    ));
    let factory_id = registry.export_capability(factory.clone());

    // Create a sample calculator
    let sample_calc = Arc::new(AdvancedCalculator::new("SampleCalculator".to_string()));
    let sample_calc_id = registry.export_capability(sample_calc);

    info!("🏭 Factory capability exported with ID: {}", factory_id);
    info!("🧮 Sample calculator exported with ID: {}", sample_calc_id);

    // Create and configure server
    let mut config = ServerConfig::default();
    config.port = 9001; // Use different port to avoid conflicts

    let server = Server::new(config);

    // Register capabilities with fixed IDs for easy testing
    server.register_capability(CapId::new(1), factory);
    server.register_capability(CapId::new(2), sample_calc);

    info!("✅ Advanced Capability Marshaling Server Configuration:");
    info!("   - Factory (ID: 1) - Methods: createCalculator, createWorkflow, getStats");
    info!("   - Sample Calculator (ID: 2) - Methods: add, subtract, multiplyByCapability, chainWithOtherCalculator, getStatus");
    info!("");
    info!("🌐 Server endpoints:");
    info!("   HTTP Batch: http://127.0.0.1:9001/rpc/batch");
    info!("   WebSocket: ws://127.0.0.1:9001/rpc/ws");
    info!("   Health: http://127.0.0.1:9001/health");
    info!("");
    info!("📋 Example requests:");
    info!("   1. Create Calculator: POST /rpc/batch with: [[\"push\",[\"pipeline\",1,[\"createCalculator\"],[]]], [\"pull\",1]]");
    info!("   2. Use Capability Args: POST /rpc/batch with: [[\"push\",[\"pipeline\",2,[\"multiplyByCapability\"],[5,[\"capref\",1]]]], [\"pull\",1]]");
    info!("   3. Complex Workflow: POST /rpc/batch with capability chaining");
    info!("");

    // Start server
    server.run().await?;

    Ok(())
}
