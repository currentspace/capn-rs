use async_trait::async_trait;
use capnweb_core::{CapId, RpcError};
use capnweb_server::{RpcTarget, Server, ServerConfig};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// A stateful calculator capability with variable storage
struct Calculator {
    /// Storage for variables (name -> value)
    variables: Arc<RwLock<HashMap<String, f64>>>,
}

#[async_trait]
impl RpcTarget for Calculator {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match member {
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("add requires exactly 2 arguments"));
                }
                let a = args[0]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                Ok(json!(a + b))
            }
            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request(
                        "subtract requires exactly 2 arguments",
                    ));
                }
                let a = args[0]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                Ok(json!(a - b))
            }
            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request(
                        "multiply requires exactly 2 arguments",
                    ));
                }
                let a = args[0]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                Ok(json!(a * b))
            }
            "divide" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("divide requires exactly 2 arguments"));
                }
                let a = args[0]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                if b == 0.0 {
                    return Err(RpcError::bad_request("Division by zero"));
                }
                Ok(json!(a / b))
            }
            "setVariable" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request(
                        "setVariable requires exactly 2 arguments: name and value",
                    ));
                }
                let name = args[0]
                    .as_str()
                    .ok_or_else(|| RpcError::bad_request("Variable name must be a string"))?;
                let value = args[1]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Variable value must be a number"))?;

                let mut variables = self.variables.write().await;
                variables.insert(name.to_string(), value);
                Ok(json!(value))
            }
            "getVariable" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request(
                        "getVariable requires exactly 1 argument: name",
                    ));
                }
                let name = args[0]
                    .as_str()
                    .ok_or_else(|| RpcError::bad_request("Variable name must be a string"))?;

                let variables = self.variables.read().await;
                match variables.get(name) {
                    Some(&value) => Ok(json!(value)),
                    None => Err(RpcError::not_found(format!(
                        "Variable '{}' not found",
                        name
                    ))),
                }
            }
            "clearAllVariables" => {
                if !args.is_empty() {
                    return Err(RpcError::bad_request(
                        "clearAllVariables requires no arguments",
                    ));
                }

                let mut variables = self.variables.write().await;
                let count = variables.len();
                variables.clear();
                Ok(json!({
                    "cleared": count,
                    "message": format!("Cleared {} variables", count)
                }))
            }
            "getAsyncProcessor" => {
                if !args.is_empty() {
                    return Err(RpcError::bad_request(
                        "getAsyncProcessor requires no arguments",
                    ));
                }

                // Return a mock async processor capability reference
                // In a full implementation, this would return a capability reference
                Ok(json!({
                    "_type": "capability",
                    "id": "async_processor_1",
                    "methods": ["process_async", "get_operation_count"],
                    "description": "Mock Async Processor capability"
                }))
            }
            "getNested" => {
                let operation_id = if args.is_empty() {
                    "default_operation".to_string()
                } else {
                    args[0].as_str().unwrap_or("default_operation").to_string()
                };

                // Return a mock nested capability reference
                Ok(json!({
                    "_type": "capability",
                    "id": format!("nested_cap_{}", operation_id),
                    "operation_id": operation_id,
                    "methods": ["multiply_counter", "get_operation_id"],
                    "description": "Mock Nested Capability"
                }))
            }
            "createSubCalculator" => {
                let calc_id = if args.is_empty() {
                    "sub_calc_1".to_string()
                } else {
                    format!("sub_calc_{}", args[0].as_str().unwrap_or("1"))
                };

                // Return a mock sub-calculator capability reference
                Ok(json!({
                    "_type": "capability",
                    "id": calc_id,
                    "methods": ["add", "subtract", "multiply", "divide", "setVariable", "getVariable"],
                    "description": "Mock Sub-Calculator capability"
                }))
            }
            _ => Err(RpcError::not_found(format!(
                "Method '{}' not found on Calculator",
                member
            ))),
        }
    }
}

/// A simple echo service capability
struct EchoService;

#[async_trait]
impl RpcTarget for EchoService {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match member {
            "echo" => Ok(json!({
                "echoed": args,
                "count": args.len()
            })),
            "reverse" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("reverse requires exactly 1 argument"));
                }
                if let Some(s) = args[0].as_str() {
                    Ok(json!(s.chars().rev().collect::<String>()))
                } else {
                    Err(RpcError::bad_request("Argument must be a string"))
                }
            }
            _ => Err(RpcError::not_found(format!(
                "Method '{}' not found on EchoService",
                member
            ))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("capnweb_server=debug,basic_server=debug")
        .init();
    // Configure the server (use PORT env var or default to 9000)
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "9000".to_string())
        .parse::<u16>()
        .unwrap_or(9000);

    let config = ServerConfig {
        port,
        host: "127.0.0.1".to_string(),
        max_batch_size: 100,
    };

    // Create the server
    let server = Server::new(config);

    // Register capabilities
    let calculator = Arc::new(Calculator {
        variables: Arc::new(RwLock::new(HashMap::new())),
    });
    let echo = Arc::new(EchoService);

    server.register_capability(CapId::new(1), calculator);
    server.register_capability(CapId::new(2), echo);

    println!("Starting Cap'n Web server with the following capabilities:");
    println!("  - Calculator (ID: 1) - Methods: add, subtract, multiply, divide, setVariable, getVariable, clearAllVariables, getAsyncProcessor, getNested, createSubCalculator");
    println!("  - EchoService (ID: 2) - Methods: echo, reverse");
    println!();
    println!("Example request:");
    println!("  POST http://127.0.0.1:{}/rpc/batch", port);
    println!("  Content-Type: application/json");
    println!();
    println!(r#"  [{{"type":"call","id":1,"target":1,"member":"add","args":[5,3]}}]"#);
    println!();

    // Run the server
    server.run().await?;

    Ok(())
}
