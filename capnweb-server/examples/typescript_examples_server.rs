// TypeScript Examples Compatible Server
// Implements the exact API required by the official Cap'n Web TypeScript examples
// (batch-pipelining and worker-react)

use std::{sync::Arc, collections::HashMap, time::Duration};
use capnweb_core::{RpcError, CapId};
use capnweb_server::{Server, ServerConfig, RpcTarget};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{info, debug};
use async_trait::async_trait;

// User data structure matching TypeScript examples
#[derive(Debug, Clone)]
struct User {
    id: String,
    name: String,
}

#[derive(Debug, Clone)]
struct Profile {
    id: String,
    bio: String,
}

/// API implementation matching the TypeScript examples exactly
#[derive(Debug)]
pub struct Api {
    // Simulated session storage
    sessions: Arc<RwLock<HashMap<String, User>>>,
    // User profiles
    profiles: Arc<RwLock<HashMap<String, Profile>>>,
    // User notifications
    notifications: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl Api {
    pub fn new() -> Self {
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let profiles = Arc::new(RwLock::new(HashMap::new()));
        let notifications = Arc::new(RwLock::new(HashMap::new()));

        // Initialize with sample data matching TypeScript examples
        let api = Self {
            sessions: sessions.clone(),
            profiles: profiles.clone(),
            notifications: notifications.clone(),
        };

        // Set up initial data
        tokio::spawn(async move {
            let mut sess = sessions.write().await;
            sess.insert("cookie-123".to_string(), User {
                id: "u_1".to_string(),
                name: "Ada Lovelace".to_string(),
            });
            sess.insert("cookie-456".to_string(), User {
                id: "u_2".to_string(),
                name: "Alan Turing".to_string(),
            });

            let mut profs = profiles.write().await;
            profs.insert("u_1".to_string(), Profile {
                id: "u_1".to_string(),
                bio: "Mathematician & first programmer".to_string(),
            });
            profs.insert("u_2".to_string(), Profile {
                id: "u_2".to_string(),
                bio: "Mathematician & computer science pioneer".to_string(),
            });

            let mut notifs = notifications.write().await;
            notifs.insert("u_1".to_string(), vec![
                "Welcome to jsrpc!".to_string(),
                "You have 2 new followers".to_string(),
            ]);
            notifs.insert("u_2".to_string(), vec![
                "New feature: pipelining!".to_string(),
                "Security tips for your account".to_string(),
            ]);
        });

        api
    }
}

#[async_trait]
impl RpcTarget for Api {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        debug!("Api::{} called with args: {:?}", member, args);

        match member {
            // Authenticate user with session token
            "authenticate" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("authenticate requires exactly 1 argument"));
                }

                let session_token = args[0].as_str()
                    .ok_or_else(|| RpcError::bad_request("sessionToken must be a string"))?;

                // Simulate processing delay
                let delay = std::env::var("DELAY_AUTH_MS")
                    .unwrap_or_else(|_| "80".to_string())
                    .parse::<u64>()
                    .unwrap_or(80);
                sleep(Duration::from_millis(delay)).await;

                let sessions = self.sessions.read().await;
                let user = sessions.get(session_token)
                    .ok_or_else(|| RpcError::bad_request("Invalid session"))?;

                Ok(json!({
                    "id": user.id,
                    "name": user.name
                }))
            }

            // Get user profile by user ID
            "getUserProfile" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("getUserProfile requires exactly 1 argument"));
                }

                let user_id = args[0].as_str()
                    .ok_or_else(|| RpcError::bad_request("userId must be a string"))?;

                // Simulate processing delay
                let delay = std::env::var("DELAY_PROFILE_MS")
                    .unwrap_or_else(|_| "120".to_string())
                    .parse::<u64>()
                    .unwrap_or(120);
                sleep(Duration::from_millis(delay)).await;

                let profiles = self.profiles.read().await;
                let profile = profiles.get(user_id)
                    .ok_or_else(|| RpcError::not_found("No such user"))?;

                Ok(json!({
                    "id": profile.id,
                    "bio": profile.bio
                }))
            }

            // Get notifications for a user
            "getNotifications" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("getNotifications requires exactly 1 argument"));
                }

                let user_id = args[0].as_str()
                    .ok_or_else(|| RpcError::bad_request("userId must be a string"))?;

                // Simulate processing delay
                let delay = std::env::var("DELAY_NOTIFS_MS")
                    .unwrap_or_else(|_| "120".to_string())
                    .parse::<u64>()
                    .unwrap_or(120);
                sleep(Duration::from_millis(delay)).await;

                let notifications = self.notifications.read().await;
                let notifs = notifications.get(user_id)
                    .cloned()
                    .unwrap_or_else(Vec::new);

                Ok(json!(notifs))
            }

            _ => {
                Err(RpcError::not_found(&format!("Method '{}' not found", member)))
            }
        }
    }
}

/// Calculator capability for tests that need basic arithmetic
#[derive(Debug)]
pub struct Calculator;

#[async_trait]
impl RpcTarget for Calculator {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        debug!("Calculator::{} called with args: {:?}", member, args);

        match member {
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("add requires exactly 2 arguments"));
                }
                let a = args[0].as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1].as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                Ok(json!(a + b))
            }
            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("multiply requires exactly 2 arguments"));
                }
                let a = args[0].as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1].as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                Ok(json!(a * b))
            }
            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("subtract requires exactly 2 arguments"));
                }
                let a = args[0].as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1].as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                Ok(json!(a - b))
            }
            "divide" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("divide requires exactly 2 arguments"));
                }
                let a = args[0].as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1].as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                if b == 0.0 {
                    return Err(RpcError::bad_request("Cannot divide by zero"));
                }
                Ok(json!(a / b))
            }
            _ => {
                Err(RpcError::not_found(&format!("Method '{}' not found on Calculator", member)))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,hyper=info".into()),
        )
        .init();

    // Server configuration
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        max_batch_size: 100,
    };

    info!("🚀 Starting TypeScript Examples Compatible Server on http://{}:{}", config.host, config.port);
    info!("📍 Endpoints:");
    info!("   - HTTP Batch: http://{}:{}/rpc/batch", config.host, config.port);
    info!("   - WebSocket: ws://{}:{}/rpc/ws", config.host, config.port);
    info!("   - Health: http://{}:{}/health", config.host, config.port);
    info!("");
    info!("🔧 API Capabilities:");
    info!("   - Api (default): authenticate, getUserProfile, getNotifications");
    info!("   - Calculator (cap 1): add, multiply, subtract, divide");
    info!("");
    info!("📝 Example data:");
    info!("   - Session tokens: 'cookie-123', 'cookie-456'");
    info!("   - User IDs: 'u_1' (Ada Lovelace), 'u_2' (Alan Turing)");
    info!("");

    // Create server
    let mut server = Server::new(config);

    // Register capabilities
    // IMPORTANT: The server currently hardcodes import_id=0 to map to CapId(1)
    // So we register the Api at CapId(1) to be the default capability
    server.register_capability(CapId::new(1), Arc::new(Api::new()));

    // Register Calculator at cap 0 (not used by TypeScript examples)
    server.register_capability(CapId::new(0), Arc::new(Calculator));

    // Run server
    server.run().await?;

    Ok(())
}