use async_trait::async_trait;
use serde_json::Value;
use capnweb_core::{RpcError, Message, CallId, CapId, Target, Outcome};
use std::sync::Arc;
use axum::{
    Router,
    routing::post,
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
            .with_state(Arc::new(self.clone()));

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        println!("Server listening on {}", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }

    pub async fn process_message(&self, msg: Message) -> Message {
        match msg {
            Message::Call { id, target, member, args } => {
                let result = match target {
                    Target::Cap(cap_id) => {
                        match self.cap_table.lookup(&cap_id) {
                            Some(cap) => {
                                match cap.call(&member, args).await {
                                    Ok(value) => Outcome::Success { value },
                                    Err(error) => Outcome::Error { error },
                                }
                            }
                            None => Outcome::Error {
                                error: RpcError::not_found(format!("Capability {} not found", cap_id))
                            },
                        }
                    }
                    Target::Special(name) => {
                        Outcome::Error {
                            error: RpcError::not_found(format!("Special target '{}' not implemented", name))
                        }
                    }
                };

                Message::Result { id, outcome: result }
            }
            Message::Dispose { caps } => {
                for cap_id in caps {
                    self.cap_table.remove(&cap_id);
                }
                // Dispose doesn't return a response
                Message::Dispose { caps: vec![] }
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
            Json(vec![Message::Result {
                id: CallId::new(0),
                outcome: Outcome::Error {
                    error: RpcError::bad_request(format!(
                        "Batch size {} exceeds maximum {}",
                        messages.len(),
                        server.config.max_batch_size
                    ))
                }
            }])
        );
    }

    // Process each message
    let mut responses = Vec::new();
    for msg in messages {
        let response = server.process_message(msg).await;
        // Only include actual responses (not empty dispose confirmations)
        match &response {
            Message::Dispose { caps } if caps.is_empty() => continue,
            _ => responses.push(response),
        }
    }

    (StatusCode::OK, Json(responses))
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

        let msg = Message::Call {
            id: CallId::new(1),
            target: Target::Cap(cap_id),
            member: "echo".to_string(),
            args: vec![json!("hello")],
        };

        let response = server.process_message(msg).await;

        match response {
            Message::Result { id, outcome } => {
                assert_eq!(id, CallId::new(1));
                match outcome {
                    Outcome::Success { value } => assert_eq!(value, json!("hello")),
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

        let msg = Message::Dispose { caps: vec![cap_id] };
        let _ = server.process_message(msg).await;

        assert!(server.cap_table.lookup(&cap_id).is_none());
    }

    #[tokio::test]
    async fn test_unknown_capability() {
        let server = Server::new(ServerConfig::default());

        let msg = Message::Call {
            id: CallId::new(1),
            target: Target::Cap(CapId::new(999)),
            member: "test".to_string(),
            args: vec![],
        };

        let response = server.process_message(msg).await;

        match response {
            Message::Result { outcome: Outcome::Error { error }, .. } => {
                assert!(error.message.contains("not found"));
            }
            _ => panic!("Expected error response"),
        }
    }
}