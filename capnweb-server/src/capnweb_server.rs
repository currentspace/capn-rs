use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use capnweb_core::{
    protocol::{
        Message, Expression, ImportId, ExportId,
        RpcSession, IdAllocator, ImportTable, ExportTable,
        StubReference, Value,
    },
    RpcTarget,
};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

/// Cap'n Web server configuration
#[derive(Debug, Clone)]
pub struct CapnWebServerConfig {
    pub host: String,
    pub port: u16,
    pub max_batch_size: usize,
}

impl Default for CapnWebServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_batch_size: 100,
        }
    }
}

/// Cap'n Web server state
#[derive(Clone)]
struct ServerState {
    /// Main capability (ID 0)
    main_capability: Option<Arc<dyn RpcTarget>>,
    /// Registered capabilities by ID
    capabilities: Arc<RwLock<HashMap<i64, Arc<dyn RpcTarget>>>>,
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, Arc<RpcSession>>>>,
}

/// Cap'n Web server
pub struct CapnWebServer {
    config: CapnWebServerConfig,
    state: ServerState,
}

impl CapnWebServer {
    /// Create a new Cap'n Web server
    pub fn new(config: CapnWebServerConfig) -> Self {
        Self {
            config,
            state: ServerState {
                main_capability: None,
                capabilities: Arc::new(RwLock::new(HashMap::new())),
                sessions: Arc::new(RwLock::new(HashMap::new())),
            },
        }
    }

    /// Register the main capability (ID 0)
    pub fn register_main(&mut self, capability: Arc<dyn RpcTarget>) {
        self.state.main_capability = Some(capability);
    }

    /// Register a capability with a specific ID
    pub fn register_capability(&self, id: i64, capability: Arc<dyn RpcTarget>) {
        let state = self.state.clone();
        tokio::spawn(async move {
            state.capabilities.write().await.insert(id, capability);
        });
    }

    /// Build the router
    fn build_router(state: ServerState) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/rpc/batch", post(handle_batch))
            .route("/rpc/ws", get(handle_websocket))
            .layer(CorsLayer::permissive())
            .with_state(state)
    }

    /// Run the server
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let app = Self::build_router(self.state);

        println!("🚀 Cap'n Web server listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Batch RPC endpoint handler
async fn handle_batch(
    State(state): State<ServerState>,
    Json(messages): Json<Vec<serde_json::Value>>,
) -> impl IntoResponse {
    // Get or create session
    let session_id = uuid::Uuid::new_v4().to_string();
    let session = get_or_create_session(&state, &session_id).await;

    let mut responses = Vec::new();

    for msg_json in messages {
        // Parse message
        let message = match Message::from_json(&msg_json) {
            Ok(msg) => msg,
            Err(e) => {
                // Return error response
                responses.push(serde_json::json!([
                    "abort",
                    ["error", "ParseError", e.to_string()]
                ]));
                continue;
            }
        };

        // Process message
        match process_message(&state, &session, message).await {
            Ok(Some(response)) => {
                responses.push(response.to_json());
            }
            Ok(None) => {
                // No response needed (e.g., for Push without Pull)
            }
            Err(e) => {
                responses.push(serde_json::json!([
                    "abort",
                    ["error", "ProcessError", e.to_string()]
                ]));
            }
        }
    }

    Json(responses)
}

/// WebSocket endpoint handler
async fn handle_websocket(State(_state): State<ServerState>) -> impl IntoResponse {
    // TODO: Implement WebSocket support
    (StatusCode::NOT_IMPLEMENTED, "WebSocket not yet implemented")
}

/// Get or create a session
async fn get_or_create_session(
    state: &ServerState,
    session_id: &str,
) -> Arc<RpcSession> {
    let mut sessions = state.sessions.write().await;

    if let Some(session) = sessions.get(session_id) {
        session.clone()
    } else {
        let session = Arc::new(RpcSession::new());
        sessions.insert(session_id.to_string(), session.clone());
        session
    }
}

/// Process a Cap'n Web message
async fn process_message(
    state: &ServerState,
    session: &RpcSession,
    message: Message,
) -> Result<Option<Message>, Box<dyn std::error::Error>> {
    match message {
        Message::Push(expr) => {
            // For now, just evaluate locally
            // TODO: Proper push handling with import allocation
            match evaluate_expression(state, expr).await {
                Ok(_value) => Ok(None), // Push doesn't return immediate response
                Err(e) => Ok(Some(Message::Abort(Expression::Error(
                    capnweb_core::protocol::ErrorExpression {
                        error_type: "EvalError".to_string(),
                        message: e.to_string(),
                        stack: None,
                    }
                )))),
            }
        }

        Message::Pull(import_id) => {
            // TODO: Implement pull logic
            Ok(Some(Message::Resolve(
                import_id.to_export_id(),
                Expression::String("TODO: Implement pull".to_string()),
            )))
        }

        Message::Resolve(export_id, _expr) => {
            // Client is resolving an export
            // TODO: Handle resolution
            Ok(None)
        }

        Message::Reject(export_id, _expr) => {
            // Client is rejecting an export
            // TODO: Handle rejection
            Ok(None)
        }

        Message::Release(import_id, _refcount) => {
            // Client is releasing an import
            // TODO: Handle release
            Ok(None)
        }

        Message::Abort(_expr) => {
            // Session is being aborted
            // TODO: Clean up session
            Ok(None)
        }
    }
}

/// Simple expression evaluator for testing
async fn evaluate_expression(
    state: &ServerState,
    expr: Expression,
) -> Result<Value, Box<dyn std::error::Error>> {
    match expr {
        Expression::Null => Ok(Value::Null),
        Expression::Bool(b) => Ok(Value::Bool(b)),
        Expression::Number(n) => Ok(Value::Number(n)),
        Expression::String(s) => Ok(Value::String(s)),

        // For testing, handle simple import calls
        Expression::Import(import) => {
            // Check if this is the main capability (ID 0)
            if import.import_id.is_main() {
                if let Some(main) = &state.main_capability {
                    // Extract method name from property path
                    if let Some(path) = &import.property_path {
                        if let Some(capnweb_core::protocol::PropertyKey::String(method)) = path.first() {
                            // Extract call arguments
                            let args = if let Some(args_expr) = &import.call_arguments {
                                extract_args(&**args_expr)?
                            } else {
                                Vec::new()
                            };

                            // Call the method
                            return main.call(method, args).await
                                .map_err(|e| e.to_string().into());
                        }
                    }
                }
            }

            Ok(Value::String("Import not implemented".to_string()))
        }

        _ => Ok(Value::String("Expression type not implemented".to_string())),
    }
}

/// Extract arguments from an expression
fn extract_args(expr: &Expression) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    match expr {
        Expression::Array(elements) => {
            let mut args = Vec::new();
            for elem in elements {
                args.push(expr_to_value(elem)?);
            }
            Ok(args)
        }
        _ => Ok(vec![expr_to_value(expr)?]),
    }
}

/// Convert expression to value
fn expr_to_value(expr: &Expression) -> Result<Value, Box<dyn std::error::Error>> {
    match expr {
        Expression::Null => Ok(Value::Null),
        Expression::Bool(b) => Ok(Value::Bool(*b)),
        Expression::Number(n) => Ok(Value::Number(n.clone())),
        Expression::String(s) => Ok(Value::String(s.clone())),
        Expression::Array(elements) => {
            let mut values = Vec::new();
            for elem in elements {
                values.push(expr_to_value(elem)?);
            }
            Ok(Value::Array(values))
        }
        _ => Err("Unsupported expression type for conversion".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let config = CapnWebServerConfig::default();
        let server = CapnWebServer::new(config);
        assert!(server.state.main_capability.is_none());
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}