use crate::Server;
use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message as WsMessage}},
    response::Response,
};
use capnweb_core::Message;
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

async fn handle_socket(socket: WebSocket, server: Arc<Server>) {
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

                        // Parse JSON message
                        match serde_json::from_str::<Message>(&text) {
                            Ok(message) => {
                                debug!("Parsed message: {:?}", message);

                                // Process the message using server's logic
                                let response = server.process_message(message).await;
                                debug!("Response: {:?}", response);

                                // Serialize and send response as JSON text
                                match serde_json::to_string(&response) {
                                    Ok(response_json) => {
                                        if let Err(e) = sender.send(WsMessage::Text(response_json)).await {
                                            error!("Failed to send response: {}", e);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to serialize response: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse JSON message: {}", e);
                                // Send error response
                                let error_response = Message::result(
                                    capnweb_core::CallId::new(0),
                                    capnweb_core::Outcome::Error {
                                        error: capnweb_core::RpcError::bad_request(
                                            format!("Invalid JSON: {}", e)
                                        )
                                    }
                                );
                                if let Ok(error_json) = serde_json::to_string(&error_response) {
                                    let _ = sender.send(WsMessage::Text(error_json)).await;
                                }
                            }
                        }
                    }
                    WsMessage::Binary(data) => {
                        warn!("Received binary message, Cap'n Web over WebSocket expects text/JSON");
                        // Try to decode as UTF-8 and process as text
                        if let Ok(text) = String::from_utf8(data) {
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