//! Example Cap'n Web Server
//!
//! A demonstration server showing various Cap'n Web features and patterns.
//! This server implements several example services that can be used for
//! learning and testing the Cap'n Web protocol.

use anyhow::Result;
use async_trait::async_trait;
use capnweb_core::{CapId, RpcError};
use capnweb_server::{RpcTarget, Server, ServerConfig};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// Counter service - demonstrates stateful services
#[derive(Debug)]
struct CounterService {
    count: Arc<Mutex<i64>>,
}

impl CounterService {
    fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl RpcTarget for CounterService {
    async fn call(&self, method: &str, _args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "increment" => {
                let mut count = self.count.lock().unwrap();
                *count += 1;
                Ok(json!({ "count": *count }))
            }
            "decrement" => {
                let mut count = self.count.lock().unwrap();
                *count -= 1;
                Ok(json!({ "count": *count }))
            }
            "get" => {
                let count = self.count.lock().unwrap();
                Ok(json!({ "count": *count }))
            }
            "reset" => {
                let mut count = self.count.lock().unwrap();
                *count = 0;
                Ok(json!({ "count": 0 }))
            }
            _ => Err(RpcError::not_found(format!("Unknown method: {}", method)))
        }
    }
}

/// Key-Value store service - demonstrates CRUD operations
#[derive(Debug)]
struct KeyValueStore {
    store: Arc<Mutex<HashMap<String, Value>>>,
}

impl KeyValueStore {
    fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl RpcTarget for KeyValueStore {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "get" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("get requires a key"));
                }
                let key = args[0].as_str()
                    .ok_or_else(|| RpcError::bad_request("Key must be a string"))?;

                let store = self.store.lock().unwrap();
                match store.get(key) {
                    Some(value) => Ok(json!({ "value": value })),
                    None => Ok(json!({ "value": null })),
                }
            }
            "set" => {
                if args.len() < 2 {
                    return Err(RpcError::bad_request("set requires key and value"));
                }
                let key = args[0].as_str()
                    .ok_or_else(|| RpcError::bad_request("Key must be a string"))?;

                let mut store = self.store.lock().unwrap();
                store.insert(key.to_string(), args[1].clone());
                Ok(json!({ "success": true }))
            }
            "delete" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("delete requires a key"));
                }
                let key = args[0].as_str()
                    .ok_or_else(|| RpcError::bad_request("Key must be a string"))?;

                let mut store = self.store.lock().unwrap();
                let existed = store.remove(key).is_some();
                Ok(json!({ "deleted": existed }))
            }
            "list" => {
                let store = self.store.lock().unwrap();
                let keys: Vec<String> = store.keys().cloned().collect();
                Ok(json!({ "keys": keys }))
            }
            "clear" => {
                let mut store = self.store.lock().unwrap();
                let count = store.len();
                store.clear();
                Ok(json!({ "cleared": count }))
            }
            _ => Err(RpcError::not_found(format!("Unknown method: {}", method)))
        }
    }
}

/// Time service - demonstrates async operations
#[derive(Debug)]
struct TimeService;

#[async_trait]
impl RpcTarget for TimeService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "now" => {
                Ok(json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "unix": chrono::Utc::now().timestamp(),
                }))
            }
            "delay" => {
                // Simulate async delay
                let delay_ms = args.first()
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000);

                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

                Ok(json!({
                    "delayed": delay_ms,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }))
            }
            "format" => {
                let timestamp = args.first()
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| RpcError::bad_request("format requires a unix timestamp"))?;

                use chrono::{DateTime, Utc};
                let dt = DateTime::from_timestamp(timestamp, 0)
                    .ok_or_else(|| RpcError::bad_request("Invalid timestamp"))?;

                Ok(json!({
                    "formatted": dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    "iso": dt.to_rfc3339(),
                }))
            }
            _ => Err(RpcError::not_found(format!("Unknown method: {}", method)))
        }
    }
}

/// Math service - demonstrates computational operations
#[derive(Debug)]
struct MathService;

#[async_trait]
impl RpcTarget for MathService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "fibonacci" => {
                let n = args.first()
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| RpcError::bad_request("fibonacci requires a number"))?;

                if n > 93 {
                    return Err(RpcError::bad_request("fibonacci input too large (max 93)"));
                }

                let result = fibonacci(n);
                Ok(json!({ "result": result, "n": n }))
            }
            "factorial" => {
                let n = args.first()
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| RpcError::bad_request("factorial requires a number"))?;

                if n > 20 {
                    return Err(RpcError::bad_request("factorial input too large (max 20)"));
                }

                let result = factorial(n);
                Ok(json!({ "result": result, "n": n }))
            }
            "isPrime" => {
                let n = args.first()
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| RpcError::bad_request("isPrime requires a number"))?;

                let result = is_prime(n);
                Ok(json!({ "isPrime": result, "n": n }))
            }
            "sqrt" => {
                let n = args.first()
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| RpcError::bad_request("sqrt requires a number"))?;

                if n < 0.0 {
                    return Err(RpcError::bad_request("sqrt requires non-negative number"));
                }

                Ok(json!({ "result": n.sqrt() }))
            }
            _ => Err(RpcError::not_found(format!("Unknown method: {}", method)))
        }
    }
}

// Helper functions for math operations
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for i in 2..=((n as f64).sqrt() as u64) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

/// Main/Bootstrap service
#[derive(Debug)]
struct MainService;

#[async_trait]
impl RpcTarget for MainService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "getCapability" => {
                let id = args.first()
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| RpcError::bad_request("getCapability requires an ID"))?;

                // Return capability reference
                Ok(json!({
                    "$capnweb": {
                        "import_id": id
                    }
                }))
            }
            "listServices" => {
                Ok(json!({
                    "services": [
                        { "id": 1, "name": "counter", "description": "Stateful counter service" },
                        { "id": 2, "name": "keyvalue", "description": "Key-value store" },
                        { "id": 3, "name": "time", "description": "Time and delay operations" },
                        { "id": 4, "name": "math", "description": "Mathematical operations" },
                    ]
                }))
            }
            "health" => {
                Ok(json!({
                    "status": "healthy",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }))
            }
            _ => Err(RpcError::not_found(format!("Unknown method: {}", method)))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Cap'n Web Example Server");

    // Configure server
    let config = ServerConfig {
        port: 8080,
        host: "127.0.0.1".to_string(),
        max_batch_size: 100,
    };

    // Create server
    let server = Server::new(config);

    // Register services
    server.register_capability(CapId::new(0), Arc::new(MainService));
    server.register_capability(CapId::new(1), Arc::new(CounterService::new()));
    server.register_capability(CapId::new(2), Arc::new(KeyValueStore::new()));
    server.register_capability(CapId::new(3), Arc::new(TimeService));
    server.register_capability(CapId::new(4), Arc::new(MathService));

    info!("Server configured with example services:");
    info!("  - CapId(0): Main Service (bootstrap)");
    info!("  - CapId(1): Counter Service");
    info!("  - CapId(2): Key-Value Store");
    info!("  - CapId(3): Time Service");
    info!("  - CapId(4): Math Service");

    // Start server
    info!("Starting server on http://127.0.0.1:8080");
    info!("Endpoints:");
    info!("  - HTTP Batch: http://127.0.0.1:8080/rpc/batch");
    info!("  - WebSocket:  ws://127.0.0.1:8080/rpc/ws");

    if let Err(e) = server.run().await {
        warn!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}