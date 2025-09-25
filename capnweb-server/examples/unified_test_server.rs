use async_trait::async_trait;
use capnweb_core::{CapId, RpcError};
use capnweb_server::{RpcTarget, Server, ServerConfig};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, debug, error};

/// A calculator capability for basic arithmetic
struct Calculator;

#[async_trait]
impl RpcTarget for Calculator {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        debug!("Calculator.{} called with args: {:?}", member, args);

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
                    return Err(RpcError::bad_request("subtract requires exactly 2 arguments"));
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
                    return Err(RpcError::bad_request("multiply requires exactly 2 arguments"));
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
            "echo" => {
                // Echo back the first argument
                Ok(args.first().cloned().unwrap_or(Value::Null))
            }
            "concat" => {
                // Concatenate all string arguments
                let mut result = String::new();
                for arg in args {
                    if let Some(s) = arg.as_str() {
                        result.push_str(s);
                    } else {
                        result.push_str(&arg.to_string());
                    }
                }
                Ok(json!(result))
            }
            _ => Err(RpcError::not_found(format!("Method '{}' not found", member))),
        }
    }
}

/// A stateful calculator with memory and session state
struct StatefulCalculator {
    memory: Arc<Mutex<f64>>,
    history: Arc<Mutex<Vec<String>>>,
}

impl StatefulCalculator {
    fn new() -> Self {
        StatefulCalculator {
            memory: Arc::new(Mutex::new(0.0)),
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl RpcTarget for StatefulCalculator {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        debug!("StatefulCalculator.{} called with args: {:?}", member, args);

        match member {
            "add" | "subtract" | "multiply" | "divide" => {
                // Delegate to basic calculator operations
                let calc = Calculator;
                let result = calc.call(member, args.clone()).await?;

                // Record in history
                let mut history = self.history.lock().unwrap();
                history.push(format!("{}: {:?} = {}", member, args, result));

                Ok(result)
            }
            "store" => {
                // Store a value in memory
                if args.is_empty() {
                    return Err(RpcError::bad_request("store requires an argument"));
                }
                let value = args[0]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Argument must be a number"))?;
                *self.memory.lock().unwrap() = value;
                Ok(json!(value))
            }
            "recall" => {
                // Recall value from memory
                let value = *self.memory.lock().unwrap();
                Ok(json!(value))
            }
            "clear" => {
                // Clear memory
                *self.memory.lock().unwrap() = 0.0;
                Ok(json!(0))
            }
            "history" => {
                // Return operation history
                let history = self.history.lock().unwrap();
                Ok(json!(history.clone()))
            }
            "clearHistory" => {
                // Clear history
                self.history.lock().unwrap().clear();
                Ok(json!("History cleared"))
            }
            _ => Err(RpcError::not_found(format!("Method '{}' not found", member))),
        }
    }
}

/// A global counter shared across all sessions
struct GlobalCounter {
    count: Arc<Mutex<i64>>,
}

impl GlobalCounter {
    fn new() -> Self {
        GlobalCounter {
            count: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl RpcTarget for GlobalCounter {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        debug!("GlobalCounter.{} called with args: {:?}", member, args);

        match member {
            "increment" => {
                let mut count = self.count.lock().unwrap();
                *count += 1;
                Ok(json!(*count))
            }
            "decrement" => {
                let mut count = self.count.lock().unwrap();
                *count -= 1;
                Ok(json!(*count))
            }
            "get" => {
                let count = self.count.lock().unwrap();
                Ok(json!(*count))
            }
            "set" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("set requires an argument"));
                }
                let value = args[0]
                    .as_i64()
                    .ok_or_else(|| RpcError::bad_request("Argument must be an integer"))?;
                *self.count.lock().unwrap() = value;
                Ok(json!(value))
            }
            "reset" => {
                *self.count.lock().unwrap() = 0;
                Ok(json!(0))
            }
            _ => Err(RpcError::not_found(format!("Method '{}' not found", member))),
        }
    }
}

/// Key-value store capability
struct KeyValueStore {
    store: Arc<Mutex<HashMap<String, Value>>>,
}

impl KeyValueStore {
    fn new() -> Self {
        KeyValueStore {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl RpcTarget for KeyValueStore {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        debug!("KeyValueStore.{} called with args: {:?}", member, args);

        match member {
            "set" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("set requires exactly 2 arguments"));
                }
                let key = args[0]
                    .as_str()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a string"))?;
                let value = args[1].clone();
                self.store.lock().unwrap().insert(key.to_string(), value.clone());
                Ok(value)
            }
            "get" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("get requires a key"));
                }
                let key = args[0]
                    .as_str()
                    .ok_or_else(|| RpcError::bad_request("Key must be a string"))?;
                let store = self.store.lock().unwrap();
                Ok(store.get(key).cloned().unwrap_or(Value::Null))
            }
            "delete" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("delete requires a key"));
                }
                let key = args[0]
                    .as_str()
                    .ok_or_else(|| RpcError::bad_request("Key must be a string"))?;
                let removed = self.store.lock().unwrap().remove(key);
                Ok(json!(removed.is_some()))
            }
            "keys" => {
                let store = self.store.lock().unwrap();
                let keys: Vec<String> = store.keys().cloned().collect();
                Ok(json!(keys))
            }
            "values" => {
                let store = self.store.lock().unwrap();
                let values: Vec<Value> = store.values().cloned().collect();
                Ok(json!(values))
            }
            "clear" => {
                self.store.lock().unwrap().clear();
                Ok(json!("Store cleared"))
            }
            "size" => {
                let store = self.store.lock().unwrap();
                Ok(json!(store.len()))
            }
            _ => Err(RpcError::not_found(format!("Method '{}' not found", member))),
        }
    }
}

/// Test capability for error handling
struct ErrorTest;

#[async_trait]
impl RpcTarget for ErrorTest {
    async fn call(&self, member: &str, _args: Vec<Value>) -> Result<Value, RpcError> {
        debug!("ErrorTest.{} called", member);

        match member {
            "throwError" => {
                Err(RpcError::internal("Intentional test error"))
            }
            "throwBadRequest" => {
                Err(RpcError::bad_request("Intentional bad request"))
            }
            "throwNotFound" => {
                Err(RpcError::not_found("Intentional not found"))
            }
            "throwCustom" => {
                // Use internal error with custom message for custom errors
                Err(RpcError::internal("Custom error for testing"))
            }
            "success" => {
                Ok(json!("Success"))
            }
            _ => Err(RpcError::not_found(format!("Method '{}' not found", member))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,capnweb_server=debug,capnweb_core=debug,unified_test_server=debug".into()),
        )
        .init();

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "9000".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    let host = std::env::var("HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());

    // Create server configuration
    let config = ServerConfig {
        port,
        host: host.clone(),
        max_batch_size: 1000,
    };

    info!("🚀 Starting Unified Cap'n Web Test Server");
    info!("📍 Address: {}:{}", host, port);

    // Create server
    let server = Server::new(config);

    // Register all capabilities
    info!("📦 Registering capabilities:");

    // Basic Calculator (ID: 1, main capability)
    server.register_capability(CapId::new(1), Arc::new(Calculator));
    info!("  ✅ Calculator (ID: 1) - Basic arithmetic operations");

    // Stateful Calculator (ID: 2)
    server.register_capability(CapId::new(2), Arc::new(StatefulCalculator::new()));
    info!("  ✅ StatefulCalculator (ID: 2) - Calculator with memory");

    // Global Counter (ID: 3)
    server.register_capability(CapId::new(3), Arc::new(GlobalCounter::new()));
    info!("  ✅ GlobalCounter (ID: 3) - Shared counter");

    // Key-Value Store (ID: 4)
    server.register_capability(CapId::new(4), Arc::new(KeyValueStore::new()));
    info!("  ✅ KeyValueStore (ID: 4) - Persistent storage");

    // Error Test (ID: 5)
    server.register_capability(CapId::new(5), Arc::new(ErrorTest));
    info!("  ✅ ErrorTest (ID: 5) - Error handling tests");

    info!("");
    info!("🌐 Available endpoints:");
    info!("  HTTP Batch: http://{}:{}/rpc/batch", host, port);

    #[cfg(feature = "all-transports")]
    info!("  WebSocket: ws://{}:{}/rpc/ws", host, port);

    info!("  Health: http://{}:{}/health", host, port);
    info!("");
    info!("✨ Server ready for testing!");
    info!("📝 Protocol: Official Cap'n Web wire protocol (newline-delimited)");

    // Run the server
    server.run().await?;

    Ok(())
}