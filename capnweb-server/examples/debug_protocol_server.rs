use axum::{
    extract::{State, Json},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

/// Debug server state
#[derive(Clone)]
struct DebugState {
    request_count: Arc<RwLock<usize>>,
}

/// Debug server that logs all incoming requests
pub struct DebugProtocolServer {
    port: u16,
    state: DebugState,
}

impl DebugProtocolServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            state: DebugState {
                request_count: Arc::new(RwLock::new(0)),
            },
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("127.0.0.1:{}", self.port);
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/rpc/batch", post(handle_batch))
            .layer(CorsLayer::permissive())
            .with_state(self.state);

        println!("🔍 Debug Protocol Server");
        println!("======================");
        println!("📍 Listening on: http://{}", addr);
        println!("📝 Logging all incoming requests");
        println!("");

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Batch RPC endpoint handler that logs everything
async fn handle_batch(
    State(state): State<DebugState>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    // Increment request count
    let mut count = state.request_count.write().await;
    *count += 1;
    let request_num = *count;
    drop(count);

    println!("📨 REQUEST #{}", request_num);
    println!("=============");

    // Log headers
    println!("📋 Headers:");
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            println!("  {}: {}", name, value_str);
        }
    }

    // Log body
    println!("📄 Body ({} bytes):", body.len());
    println!("  Raw: {}", body);

    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        println!("  JSON: {}", serde_json::to_string_pretty(&json).unwrap_or("parse error".to_string()));
    } else {
        println!("  JSON: Failed to parse");
    }

    // Try to parse line by line
    println!("  Lines:");
    for (i, line) in body.lines().enumerate() {
        if !line.trim().is_empty() {
            println!("    {}: {}", i, line);
            if let Ok(line_json) = serde_json::from_str::<serde_json::Value>(line) {
                println!("      -> JSON: {}", serde_json::to_string(&line_json).unwrap_or("parse error".to_string()));
            }
        }
    }

    println!("");

    // For now, return empty array (which causes "Batch RPC request ended" in the client)
    // This helps us understand what the client expects vs what we're sending
    let response = serde_json::json!([]);

    println!("📤 RESPONSE #{}", request_num);
    println!("==============");
    println!("  Sending: {}", serde_json::to_string(&response).unwrap());
    println!("");

    Json(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let server = DebugProtocolServer::new(port);
    server.run().await?;

    Ok(())
}