use async_trait::async_trait;
use serde_json::Value;
use capnweb_core::{RpcError, Message, CallId, CapId, Target, Outcome};
use std::sync::Arc;
use axum::{
    Router,
    routing::{post, get},
    extract::State,
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use crate::CapTable;
use tokio::net::TcpListener;

#[async_trait]
pub trait RpcTarget: Send + Sync {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError>;
}

#[derive(Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub max_batch_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            max_batch_size: 100,
        }
    }
}

#[derive(Clone)]
pub struct Server {
    config: ServerConfig,
    cap_table: Arc<CapTable>,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Server {
            config,
            cap_table: Arc::new(CapTable::new()),
        }
    }

    pub fn cap_table(&self) -> &Arc<CapTable> {
        &self.cap_table
    }

    pub fn register_capability(&self, id: CapId, target: Arc<dyn RpcTarget>) {
        self.cap_table.insert(id, target);
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let app = Router::new()
            .route("/rpc/batch", post(handle_batch))
            .route("/health", get(handle_health))
            .with_state(Arc::new(self.clone()));

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        println!("Server listening on {}", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }

    pub async fn process_message(&self, msg: Message) -> Message {
        match msg {
            Message::Call { call } => {
                let result = match &call.target {
                    Target::Cap { cap } => {
                        match self.cap_table.lookup(&cap.id) {
                            Some(cap_target) => {
                                match cap_target.call(&call.member, call.args.clone()).await {
                                    Ok(value) => Outcome::Success { value },
                                    Err(error) => Outcome::Error { error },
                                }
                            }
                            None => Outcome::Error {
                                error: RpcError::not_found(format!("Capability {} not found", cap.id))
                            },
                        }
                    }
                    Target::Special { special } => {
                        Outcome::Error {
                            error: RpcError::not_found(format!("Special target '{}' not implemented", special))
                        }
                    }
                };

                Message::result(call.id, result)
            }
            Message::Dispose { dispose } => {
                for cap_id in &dispose.caps {
                    self.cap_table.remove(cap_id);
                }
                // Dispose doesn't return a response
                Message::dispose(vec![])
            }
            msg => msg, // CapRef and Result just pass through
        }
    }
}

async fn handle_batch(
    State(server): State<Arc<Server>>,
    Json(messages): Json<Vec<Message>>,
) -> impl IntoResponse {
    // Check batch size
    if messages.len() > server.config.max_batch_size {
        return (
            StatusCode::BAD_REQUEST,
            Json(vec![Message::result(
                CallId::new(0),
                Outcome::Error {
                    error: RpcError::bad_request(format!(
                        "Batch size {} exceeds maximum {}",
                        messages.len(),
                        server.config.max_batch_size
                    ))
                }
            )])
        );
    }

    // Process each message
    let mut responses = Vec::new();
    for msg in messages {
        let response = server.process_message(msg).await;
        // Only include actual responses (not empty dispose confirmations)
        match &response {
            Message::Dispose { dispose } if dispose.caps.is_empty() => continue,
            _ => responses.push(response),
        }
    }

    (StatusCode::OK, Json(responses))
}

async fn handle_health(State(server): State<Arc<Server>>) -> impl IntoResponse {
    let capability_count = server.cap_table.len();

    let health_response = serde_json::json!({
        "status": "healthy",
        "server": "capnweb-rust",
        "capabilities": capability_count,
        "max_batch_size": server.config.max_batch_size,
        "endpoints": {
            "batch": "/rpc/batch",
            "health": "/health"
        }
    });

    (StatusCode::OK, Json(health_response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct TestTarget;

    #[async_trait]
    impl RpcTarget for TestTarget {
        async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
            match member {
                "echo" => Ok(args.first().cloned().unwrap_or(Value::Null)),
                "add" => {
                    if args.len() != 2 {
                        return Err(RpcError::bad_request("add requires 2 arguments"));
                    }
                    let a = args[0].as_f64().ok_or_else(|| RpcError::bad_request("First arg must be number"))?;
                    let b = args[1].as_f64().ok_or_else(|| RpcError::bad_request("Second arg must be number"))?;
                    Ok(json!(a + b))
                }
                _ => Err(RpcError::not_found(format!("Method '{}' not found", member)))
            }
        }
    }

    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig::default();
        let server = Server::new(config);
        assert_eq!(server.config.port, 8080);
    }

    #[tokio::test]
    async fn test_register_capability() {
        let server = Server::new(ServerConfig::default());
        let cap_id = CapId::new(42);
        let target = Arc::new(TestTarget);

        server.register_capability(cap_id, target);
        assert!(server.cap_table.lookup(&cap_id).is_some());
    }

    #[tokio::test]
    async fn test_process_call_message() {
        let server = Server::new(ServerConfig::default());
        let cap_id = CapId::new(1);
        server.register_capability(cap_id, Arc::new(TestTarget));

        let msg = Message::call(
            CallId::new(1),
            Target::cap(cap_id),
            "echo".to_string(),
            vec![json!("hello")],
        );

        let response = server.process_message(msg).await;

        match response {
            Message::Result { result } => {
                assert_eq!(result.id, CallId::new(1));
                match &result.outcome {
                    Outcome::Success { value } => assert_eq!(*value, json!("hello")),
                    _ => panic!("Expected success outcome"),
                }
            }
            _ => panic!("Expected Result message"),
        }
    }

    #[tokio::test]
    async fn test_process_dispose_message() {
        let server = Server::new(ServerConfig::default());
        let cap_id = CapId::new(1);
        server.register_capability(cap_id, Arc::new(TestTarget));

        assert!(server.cap_table.lookup(&cap_id).is_some());

        let msg = Message::dispose(vec![cap_id]);
        let _ = server.process_message(msg).await;

        assert!(server.cap_table.lookup(&cap_id).is_none());
    }

    #[tokio::test]
    async fn test_unknown_capability() {
        let server = Server::new(ServerConfig::default());

        let msg = Message::call(
            CallId::new(1),
            Target::cap(CapId::new(999)),
            "test".to_string(),
            vec![],
        );

        let response = server.process_message(msg).await;

        match response {
            Message::Result { result } => {
                match &result.outcome {
                    Outcome::Error { error } => {
                        assert!(error.message.contains("not found"));
                    }
                    _ => panic!("Expected error outcome"),
                }
            }
            _ => panic!("Expected Result message"),
        }
    }
}