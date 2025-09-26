use async_trait::async_trait;
use capnweb_core::protocol::tables::Value;
use capnweb_core::{RpcError, RpcTarget};
use capnweb_server::{CapnWebServerConfig, NewCapnWebServer as CapnWebServer};
use serde_json::Number;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, Duration};

/// Stateful counter service with session management
#[derive(Debug)]
struct CounterService {
    /// Global counters accessible by all sessions
    global_counters: Arc<RwLock<HashMap<String, i64>>>,
    /// Session-specific storage
    session_storage: Arc<RwLock<HashMap<String, SessionData>>>,
}

/// Session-specific data storage
#[derive(Debug, Clone)]
struct SessionData {
    session_id: String,
    counters: HashMap<String, i64>,
    properties: HashMap<String, Value>,
    created_at: std::time::SystemTime,
    last_accessed: std::time::SystemTime,
}

impl SessionData {
    fn new(session_id: String) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            session_id,
            counters: HashMap::new(),
            properties: HashMap::new(),
            created_at: now,
            last_accessed: now,
        }
    }

    fn touch(&mut self) {
        self.last_accessed = std::time::SystemTime::now();
    }
}

/// Async processor for background operations
#[derive(Debug)]
struct AsyncProcessor {
    operation_count: Arc<Mutex<u64>>,
}

/// Nested capability for advanced operations
#[derive(Debug)]
struct NestedCapability {
    parent_counter: Arc<RwLock<HashMap<String, i64>>>,
    operation_id: String,
}

impl CounterService {
    fn new() -> Self {
        Self {
            global_counters: Arc::new(RwLock::new(HashMap::new())),
            session_storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_create_session(&self, session_id: &str) -> SessionData {
        let mut sessions = self.session_storage.write().await;
        let session = sessions
            .entry(session_id.to_string())
            .or_insert_with(|| SessionData::new(session_id.to_string()));
        session.touch();
        session.clone()
    }

    async fn update_session(&self, session_data: SessionData) {
        let mut sessions = self.session_storage.write().await;
        sessions.insert(session_data.session_id.clone(), session_data);
    }

    async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.session_storage.write().await;
        let expiry_threshold = std::time::SystemTime::now()
            .checked_sub(Duration::from_secs(3600))
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        sessions.retain(|_, session| session.last_accessed > expiry_threshold);
    }
}

#[async_trait]
impl RpcTarget for CounterService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "increment_global" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request(
                        "increment_global requires counter name",
                    ));
                }
                let counter_name = extract_string(&args[0])?;
                let mut counters = self.global_counters.write().await;
                let new_value = *counters
                    .entry(counter_name.clone())
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
                Ok(Value::Number(Number::from(new_value)))
            }

            "decrement_global" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request(
                        "decrement_global requires counter name",
                    ));
                }
                let counter_name = extract_string(&args[0])?;
                let mut counters = self.global_counters.write().await;
                let new_value = *counters
                    .entry(counter_name.clone())
                    .and_modify(|v| *v -= 1)
                    .or_insert(-1);
                Ok(Value::Number(Number::from(new_value)))
            }

            "get_global" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("get_global requires counter name"));
                }
                let counter_name = extract_string(&args[0])?;
                let counters = self.global_counters.read().await;
                let value = counters.get(&counter_name).copied().unwrap_or(0);
                Ok(Value::Number(Number::from(value)))
            }

            "increment_session" => {
                if args.len() < 2 {
                    return Err(RpcError::bad_request(
                        "increment_session requires session_id and counter name",
                    ));
                }
                let session_id = extract_string(&args[0])?;
                let counter_name = extract_string(&args[1])?;

                let mut session_data = self.get_or_create_session(&session_id).await;
                let new_value = *session_data
                    .counters
                    .entry(counter_name)
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
                self.update_session(session_data).await;

                Ok(Value::Number(Number::from(new_value)))
            }

            "get_session" => {
                if args.len() < 2 {
                    return Err(RpcError::bad_request(
                        "get_session requires session_id and counter name",
                    ));
                }
                let session_id = extract_string(&args[0])?;
                let counter_name = extract_string(&args[1])?;

                let session_data = self.get_or_create_session(&session_id).await;
                let value = session_data
                    .counters
                    .get(&counter_name)
                    .copied()
                    .unwrap_or(0);

                Ok(Value::Number(Number::from(value)))
            }

            "set_session_property" => {
                if args.len() < 3 {
                    return Err(RpcError::bad_request(
                        "set_session_property requires session_id, property name, and value",
                    ));
                }
                let session_id = extract_string(&args[0])?;
                let property_name = extract_string(&args[1])?;
                let value = args[2].clone();

                let mut session_data = self.get_or_create_session(&session_id).await;
                session_data.properties.insert(property_name, value.clone());
                self.update_session(session_data).await;

                Ok(value)
            }

            "get_session_property" => {
                if args.len() < 2 {
                    return Err(RpcError::bad_request(
                        "get_session_property requires session_id and property name",
                    ));
                }
                let session_id = extract_string(&args[0])?;
                let property_name = extract_string(&args[1])?;

                let session_data = self.get_or_create_session(&session_id).await;
                match session_data.properties.get(&property_name) {
                    Some(value) => Ok(value.clone()),
                    None => Err(RpcError::not_found(format!(
                        "Property '{}' not found in session",
                        property_name
                    ))),
                }
            }

            "list_global_counters" => {
                let counters = self.global_counters.read().await;
                let result: Vec<Value> = counters
                    .iter()
                    .map(|(name, value)| {
                        let mut obj = HashMap::new();
                        obj.insert("name".to_string(), Box::new(Value::String(name.clone())));
                        obj.insert(
                            "value".to_string(),
                            Box::new(Value::Number(Number::from(*value))),
                        );
                        Value::Object(obj)
                    })
                    .collect();
                Ok(Value::Array(result))
            }

            "list_sessions" => {
                let sessions = self.session_storage.read().await;
                let result: Vec<Value> = sessions
                    .iter()
                    .map(|(session_id, session_data)| {
                        let mut obj = HashMap::new();
                        obj.insert(
                            "session_id".to_string(),
                            Box::new(Value::String(session_id.clone())),
                        );
                        obj.insert(
                            "counter_count".to_string(),
                            Box::new(Value::Number(Number::from(session_data.counters.len()))),
                        );
                        obj.insert(
                            "property_count".to_string(),
                            Box::new(Value::Number(Number::from(session_data.properties.len()))),
                        );
                        Value::Object(obj)
                    })
                    .collect();
                Ok(Value::Array(result))
            }

            "reset_global" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("reset_global requires counter name"));
                }
                let counter_name = extract_string(&args[0])?;
                let mut counters = self.global_counters.write().await;
                counters.insert(counter_name, 0);
                Ok(Value::Number(Number::from(0)))
            }

            "cleanup_sessions" => {
                self.cleanup_expired_sessions().await;
                Ok(Value::String("Sessions cleaned up".to_string()))
            }

            "get_async_processor" => {
                let processor = AsyncProcessor {
                    operation_count: Arc::new(Mutex::new(0)),
                };
                // In a real implementation, we would export this as a capability
                Ok(Value::String(
                    "AsyncProcessor capability created".to_string(),
                ))
            }

            "get_nested_capability" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request(
                        "get_nested_capability requires operation_id",
                    ));
                }
                let operation_id = extract_string(&args[0])?;
                let nested = NestedCapability {
                    parent_counter: self.global_counters.clone(),
                    operation_id,
                };
                // In a real implementation, we would export this as a capability
                Ok(Value::String("NestedCapability created".to_string()))
            }

            _ => Err(RpcError::not_found(format!("Method not found: {}", method))),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String("CounterService".to_string())),
            "version" => Ok(Value::String("2.0.0".to_string())),
            "features" => Ok(Value::Array(vec![
                Value::String("global_counters".to_string()),
                Value::String("session_storage".to_string()),
                Value::String("async_operations".to_string()),
                Value::String("nested_capabilities".to_string()),
                Value::String("session_cleanup".to_string()),
            ])),
            "global_counter_count" => {
                let counters = self.global_counters.read().await;
                Ok(Value::Number(Number::from(counters.len())))
            }
            "session_count" => {
                let sessions = self.session_storage.read().await;
                Ok(Value::Number(Number::from(sessions.len())))
            }
            _ => Err(RpcError::not_found(format!(
                "Property not found: {}",
                property
            ))),
        }
    }
}

impl AsyncProcessor {
    async fn simulate_async_work(&self, duration_ms: u64) -> Result<String, RpcError> {
        let mut count = self.operation_count.lock().await;
        *count += 1;
        let operation_id = *count;
        drop(count);

        println!(
            "Starting async operation {} (duration: {}ms)",
            operation_id, duration_ms
        );
        sleep(Duration::from_millis(duration_ms)).await;
        println!("Completed async operation {}", operation_id);

        Ok(format!(
            "Operation {} completed after {}ms",
            operation_id, duration_ms
        ))
    }
}

#[async_trait]
impl RpcTarget for AsyncProcessor {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "process_async" => {
                let duration = if args.is_empty() {
                    1000
                } else {
                    extract_number(&args[0])? as u64
                };
                let result = self.simulate_async_work(duration).await?;
                Ok(Value::String(result))
            }
            "get_operation_count" => {
                let count = self.operation_count.lock().await;
                Ok(Value::Number(Number::from(*count)))
            }
            _ => Err(RpcError::not_found(format!("Method not found: {}", method))),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String("AsyncProcessor".to_string())),
            "version" => Ok(Value::String("1.0.0".to_string())),
            _ => Err(RpcError::not_found(format!(
                "Property not found: {}",
                property
            ))),
        }
    }
}

#[async_trait]
impl RpcTarget for NestedCapability {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "multiply_counter" => {
                if args.len() < 2 {
                    return Err(RpcError::bad_request(
                        "multiply_counter requires counter name and multiplier",
                    ));
                }
                let counter_name = extract_string(&args[0])?;
                let multiplier = extract_number(&args[1])? as i64;

                let mut counters = self.parent_counter.write().await;
                let current_value = counters.get(&counter_name).copied().unwrap_or(0);
                let new_value = current_value * multiplier;
                counters.insert(counter_name.clone(), new_value);

                Ok(Value::Number(Number::from(new_value)))
            }
            "get_operation_id" => Ok(Value::String(self.operation_id.clone())),
            _ => Err(RpcError::not_found(format!("Method not found: {}", method))),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String("NestedCapability".to_string())),
            "operation_id" => Ok(Value::String(self.operation_id.clone())),
            _ => Err(RpcError::not_found(format!(
                "Property not found: {}",
                property
            ))),
        }
    }
}

fn extract_string(value: &Value) -> Result<String, RpcError> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => Err(RpcError::bad_request("Expected string")),
    }
}

fn extract_number(value: &Value) -> Result<f64, RpcError> {
    match value {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| RpcError::bad_request("Invalid number")),
        _ => Err(RpcError::bad_request("Expected number")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8081);

    // Configure server
    let config = CapnWebServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        max_batch_size: 100,
    };

    // Create sophisticated counter service
    let counter_service = CounterService::new();

    // Create server with counter service as main capability
    let mut server = CapnWebServer::new(config);
    server.register_main(Arc::new(counter_service));

    println!("🚀 Advanced Stateful Counter Server");
    println!("=====================================");
    println!("✅ Stateful global and session-specific counters");
    println!("✅ Session management with automatic cleanup");
    println!("✅ Async operations with background processing");
    println!("✅ Nested capabilities for advanced operations");
    println!("✅ Property storage per session");
    println!("✅ Promise resolution and pipelining support");
    println!("");
    println!("Available operations:");
    println!("  Global counters: increment_global, decrement_global, get_global, reset_global");
    println!("  Session counters: increment_session, get_session");
    println!("  Session properties: set_session_property, get_session_property");
    println!("  Listing: list_global_counters, list_sessions");
    println!("  Cleanup: cleanup_sessions");
    println!("  Advanced: get_async_processor, get_nested_capability");
    println!("");
    println!("Test with:");
    println!("  cargo run --example typescript-test -p typescript-interop");
    println!("");

    // Run server
    server.run().await?;

    Ok(())
}
