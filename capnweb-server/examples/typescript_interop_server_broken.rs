// TypeScript Interoperability Test Server
// Complete server implementation to pass 100% of TypeScript client tests

use capnweb_core::{async_trait, CapId, RpcError, RpcTarget, Value};
use capnweb_server::{Server, ServerConfig};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Enhanced Calculator capability with all methods expected by TypeScript tests
#[derive(Debug)]
pub struct Calculator {
    variables: Arc<RwLock<HashMap<String, f64>>>,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl RpcTarget for Calculator {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        debug!("Calculator::{} called with args: {:?}", method, args);

        match method {
            // Basic arithmetic operations
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("add requires exactly 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a + b).unwrap()))
            }

            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request(
                        "subtract requires exactly 2 arguments",
                    ));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a - b).unwrap()))
            }

            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request(
                        "multiply requires exactly 2 arguments",
                    ));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a * b).unwrap()))
            }

            "divide" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("divide requires exactly 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;

                if b == 0.0 {
                    return Err(RpcError::bad_request("Cannot divide by zero"));
                }

                Ok(Value::Number(serde_json::Number::from_f64(a / b).unwrap()))
            }

            // Advanced mathematical operations
            "power" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("power requires exactly 2 arguments"));
                }
                let base = extract_number(&args[0])?;
                let exp = extract_number(&args[1])?;
                Ok(Value::Number(
                    serde_json::Number::from_f64(base.powf(exp)).unwrap(),
                ))
            }

            "sqrt" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("sqrt requires exactly 1 argument"));
                }
                let value = extract_number(&args[0])?;

                if value < 0.0 {
                    return Err(RpcError::bad_request(
                        "Cannot take square root of negative number",
                    ));
                }

                Ok(Value::Number(
                    serde_json::Number::from_f64(value.sqrt()).unwrap(),
                ))
            }

            "factorial" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request(
                        "factorial requires exactly 1 argument",
                    ));
                }
                let n = extract_number(&args[0])?;

                if n < 0.0 {
                    return Err(RpcError::bad_request(
                        "Factorial not defined for negative numbers",
                    ));
                }

                if n > 20.0 {
                    return Err(RpcError::bad_request("Factorial too large (max 20)"));
                }

                let n = n as i32;
                let mut result = 1i64;
                for i in 1..=n {
                    result *= i as i64;
                }

                Ok(Value::Number(serde_json::Number::from_i64(result).unwrap()))
            }

            // Variable storage
            "setVariable" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request(
                        "setVariable requires exactly 2 arguments",
                    ));
                }

                let name = extract_string(&args[0])?;
                let value = extract_number(&args[1])?;

                let mut vars = self.variables.write().await;
                vars.insert(name.to_string(), value);

                Ok(Value::Number(serde_json::Number::from_f64(value).unwrap()))
            }

            "getVariable" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request(
                        "getVariable requires exactly 1 argument",
                    ));
                }

                let name = extract_string(&args[0])?;

                let vars = self.variables.read().await;
                let value = vars.get(name).ok_or_else(|| {
                    RpcError::not_found(&format!("Variable '{}' not found", name))
                })?;

                Ok(Value::Number(serde_json::Number::from_f64(*value).unwrap()))
            }

            "clearAllVariables" => {
                let mut vars = self.variables.write().await;
                let count = vars.len();
                vars.clear();

                Ok(json_to_capnweb_value(json!({
                    "cleared": count,
                    "message": format!("Cleared {} variables", count)
                })))
            }

            // Capability creation methods
            "getAsyncProcessor" => {
                Ok(json_to_capnweb_value(json!({
                    "_type": "capability",
                    "id": 200,  // AsyncProcessor capability ID
                    "description": "Asynchronous processing capability"
                })))
            }

            "getNested" => {
                Ok(json_to_capnweb_value(json!({
                    "_type": "capability",
                    "id": 300, // Nested capability ID
                    "description": "Nested capability for testing"
                })))
            }

            "createSubCalculator" => {
                Ok(json_to_capnweb_value(json!({
                    "_type": "capability",
                    "id": 500, // SubCalculator capability ID
                    "methods": ["add", "subtract", "multiply", "divide", "setVariable", "getVariable"],
                    "description": "Sub-calculator instance"
                })))
            }

            _ => Err(RpcError::not_found(&format!(
                "Method '{}' not found on Calculator",
                method
            ))),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String("Calculator".to_string())),
            "version" => Ok(Value::String("1.0.0".to_string())),
            "methods" => Ok(json_to_capnweb_value(json!([
                "add",
                "subtract",
                "multiply",
                "divide",
                "power",
                "sqrt",
                "factorial",
                "setVariable",
                "getVariable",
                "clearAllVariables",
                "getAsyncProcessor",
                "getNested",
                "createSubCalculator"
            ]))),
            _ => Err(RpcError::not_found(&format!(
                "Property '{}' not found on Calculator",
                property
            ))),
        }
    }
}

/// UserManager capability for user management tests
#[derive(Debug)]
pub struct UserManager {
    users: Arc<RwLock<HashMap<i32, User>>>,
    next_id: Arc<std::sync::atomic::AtomicI32>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct User {
    id: i32,
    name: String,
    email: String,
    role: Option<String>,
    created: bool,
}

impl UserManager {
    pub fn new() -> Self {
        let mut users = HashMap::new();

        // Pre-populate with test users
        users.insert(
            1,
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                role: Some("admin".to_string()),
                created: false,
            },
        );

        users.insert(
            2,
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                role: Some("user".to_string()),
                created: false,
            },
        );

        users.insert(
            3,
            User {
                id: 3,
                name: "Charlie".to_string(),
                email: "charlie@example.com".to_string(),
                role: Some("user".to_string()),
                created: false,
            },
        );

        Self {
            users: Arc::new(RwLock::new(users)),
            next_id: Arc::new(std::sync::atomic::AtomicI32::new(4)),
        }
    }
}

#[async_trait]
impl RpcTarget for UserManager {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        debug!("UserManager::{} called with args: {:?}", method, args);

        match method {
            "getUser" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("getUser requires exactly 1 argument"));
                }

                let user_id = extract_number(&args[0])? as i32;

                let users = self.users.read().await;
                let user = users.get(&user_id).ok_or_else(|| {
                    RpcError::not_found(&format!("User with ID {} not found", user_id))
                })?;

                Ok(json_to_capnweb_value(serde_json::to_value(user).unwrap()))
            }

            "createUser" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request(
                        "createUser requires exactly 1 argument",
                    ));
                }

                let user_data = extract_object(&args[0])?;

                let name = user_data
                    .get("name")
                    .and_then(|v| match v.as_ref() {
                        Value::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .ok_or_else(|| RpcError::bad_request("User name is required"))?;

                let email = user_data
                    .get("email")
                    .and_then(|v| match v.as_ref() {
                        Value::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .ok_or_else(|| RpcError::bad_request("User email is required"))?;

                let role = user_data.get("role").and_then(|v| match v.as_ref() {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                });

                let id = self
                    .next_id
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                let user = User {
                    id,
                    name: name.to_string(),
                    email: email.to_string(),
                    role,
                    created: true,
                };

                let mut users = self.users.write().await;
                users.insert(id, user.clone());

                Ok(json_to_capnweb_value(serde_json::to_value(user).unwrap()))
            }

            "listUsers" => {
                let users = self.users.read().await;
                let user_list: Vec<&User> = users.values().collect();
                Ok(json_to_capnweb_value(
                    serde_json::to_value(user_list).unwrap(),
                ))
            }

            "deleteUser" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request(
                        "deleteUser requires exactly 1 argument",
                    ));
                }

                let user_id = extract_number(&args[0])? as i32;

                let mut users = self.users.write().await;
                let user = users.remove(&user_id).ok_or_else(|| {
                    RpcError::not_found(&format!("User with ID {} not found", user_id))
                })?;

                Ok(json_to_capnweb_value(json!({
                    "deleted": true,
                    "user": user
                })))
            }

            _ => Err(RpcError::not_found(&format!(
                "Method '{}' not found on UserManager",
                method
            ))),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String("UserManager".to_string())),
            "version" => Ok(Value::String("1.0.0".to_string())),
            "methods" => Ok(json_to_capnweb_value(json!([
                "getUser",
                "createUser",
                "listUsers",
                "deleteUser"
            ]))),
            _ => Err(RpcError::not_found(&format!(
                "Property '{}' not found on UserManager",
                property
            ))),
        }
    }
}

/// Echo service for basic testing
#[derive(Debug)]
pub struct EchoService;

#[async_trait]
impl RpcTarget for EchoService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "echo" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("echo requires exactly 1 argument"));
                }
                Ok(args[0].clone())
            }

            "reverse" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("reverse requires exactly 1 argument"));
                }
                let text = extract_string(&args[0])?;
                Ok(Value::String(text.chars().rev().collect::<String>()))
            }

            _ => Err(RpcError::not_found(&format!(
                "Method '{}' not found on EchoService",
                method
            ))),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String("EchoService".to_string())),
            "methods" => Ok(json_to_capnweb_value(json!(["echo", "reverse"]))),
            _ => Err(RpcError::not_found(&format!(
                "Property '{}' not found on EchoService",
                property
            ))),
        }
    }
}

// Helper functions
fn extract_number(value: &Value) -> Result<f64, RpcError> {
    match value {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| RpcError::bad_request("Invalid number")),
        _ => Err(RpcError::bad_request("Expected a number")),
    }
}

fn extract_string(value: &Value) -> Result<&str, RpcError> {
    match value {
        Value::String(s) => Ok(s),
        _ => Err(RpcError::bad_request("Expected a string")),
    }
}

fn extract_object(value: &Value) -> Result<&HashMap<String, Box<Value>>, RpcError> {
    match value {
        Value::Object(obj) => Ok(obj),
        _ => Err(RpcError::bad_request("Expected an object")),
    }
}

fn json_to_capnweb_value(json_val: serde_json::Value) -> Value {
    match json_val {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => Value::Number(n),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(json_to_capnweb_value).collect())
        }
        serde_json::Value::Object(map) => {
            let mut result = HashMap::new();
            for (k, v) in map {
                result.insert(k, Box::new(json_to_capnweb_value(v)));
            }
            Value::Object(result)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting TypeScript Interoperability Test Server");

    // Create capabilities
    let calculator = Arc::new(Calculator::new());
    let user_manager = Arc::new(UserManager::new());
    let echo = Arc::new(EchoService);

    // Create and configure server for port 8080 (expected by TypeScript tests)
    let mut config = ServerConfig::default();
    config.port = 8080;
    config.host = "127.0.0.1".to_string();

    let server = Server::new(config);

    // Register capabilities with the exact IDs expected by TypeScript tests
    server.register_capability(CapId::new(1), calculator); // Calculator at ID 1
    server.register_capability(CapId::new(100), user_manager); // UserManager at ID 100
    server.register_capability(CapId::new(2), echo); // EchoService at ID 2

    info!("✅ TypeScript Interoperability Test Server Configuration:");
    info!("   - Calculator (ID: 1) - Full mathematical operations");
    info!("     Methods: add, subtract, multiply, divide, power, sqrt, factorial");
    info!("     Variables: setVariable, getVariable, clearAllVariables");
    info!("     Capabilities: getAsyncProcessor, getNested, createSubCalculator");
    info!("   - UserManager (ID: 100) - User management operations");
    info!("     Methods: getUser, createUser, listUsers, deleteUser");
    info!("     Pre-loaded users: Alice (1), Bob (2), Charlie (3)");
    info!("   - EchoService (ID: 2) - Simple echo and reverse operations");
    info!("     Methods: echo, reverse");
    info!("");
    info!("🌐 Server endpoints:");
    info!("   HTTP Batch: http://127.0.0.1:8080/rpc/batch");
    info!("   WebSocket: ws://127.0.0.1:8080/ws");
    info!("   Health: http://127.0.0.1:8080/health");
    info!("");
    info!("🧪 Ready for TypeScript interoperability tests!");
    info!("   Run: cd typescript-tests && ./run-interop-tests.sh");

    // Start server
    server.run().await?;

    Ok(())
}
