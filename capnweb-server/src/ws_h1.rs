use crate::Server;
use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// WebSocket handler for Cap'n Web protocol
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(server): State<Arc<Server>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, server))
}

async fn handle_socket(socket: WebSocket, _server: Arc<Server>) {
    let session_id = uuid::Uuid::new_v4().to_string();
    info!("WebSocket connection established: {}", session_id);

    let (mut sender, mut receiver) = socket.split();

    // Handle incoming messages
    while let Some(result) = receiver.next().await {
        match result {
            Ok(msg) => {
                match msg {
                    WsMessage::Text(text) => {
                        debug!("Received text message: {}", text);

                        // LEGACY MESSAGE FORMAT - NOT SUPPORTED IN WIRE PROTOCOL
                        // The official Cap'n Web protocol uses newline-delimited arrays only
                        // WebSocket support will need to be reimplemented with wire protocol
                        error!("WebSocket handler uses legacy Message format which is no longer supported");
                        error!("Only the official Cap'n Web wire protocol (newline-delimited arrays) is supported");

                        // Send error response
                        let error_msg = "WebSocket support requires wire protocol implementation";
                        if let Err(e) = sender
                            .send(WsMessage::Text(error_msg.to_string().into()))
                            .await
                        {
                            error!("Failed to send error response: {}", e);
                            break;
                        }
                    }
                    WsMessage::Binary(data) => {
                        warn!(
                            "Received binary message, Cap'n Web over WebSocket expects text/JSON"
                        );
                        // Try to decode as UTF-8 and process as text
                        if let Ok(text) = String::from_utf8(data.to_vec()) {
                            debug!("Converted binary to text: {}", text);
                            // Recursively handle as text message
                            continue;
                        } else {
                            error!("Binary message is not valid UTF-8");
                        }
                    }
                    WsMessage::Ping(_) => {
                        debug!("Received ping, WebSocket will auto-respond with pong");
                    }
                    WsMessage::Pong(_) => {
                        debug!("Received pong");
                    }
                    WsMessage::Close(frame) => {
                        info!("WebSocket closing: {} (reason: {:?})", session_id, frame);
                        break;
                    }
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Clean up session
    // TODO: Add lifecycle management for cleaning up capabilities when available

    info!("WebSocket disconnected: {}", session_id);
}
