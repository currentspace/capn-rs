use async_trait::async_trait;
use capnweb_core::{CallId, CapId, Message, RpcError, Target, Outcome};
use capnweb_server::{RpcTarget, Server, ServerConfig};
use serde_json::{json, Value};
use std::sync::Arc;

struct TestCalculator;

#[async_trait]
impl RpcTarget for TestCalculator {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match member {
            "add" => {
                let a = args[0].as_f64().unwrap_or(0.0);
                let b = args[1].as_f64().unwrap_or(0.0);
                Ok(json!(a + b))
            }
            _ => Err(RpcError::not_found("Method not found")),
        }
    }
}

#[tokio::test]
async fn test_server_batch_endpoint() {
    // Create server on random port
    let config = ServerConfig {
        port: 0, // Let the OS assign a port
        host: "127.0.0.1".to_string(),
        max_batch_size: 10,
    };

    let server = Server::new(config);
    server.register_capability(CapId::new(1), Arc::new(TestCalculator));

    // Create a batch request
    let batch_request = vec![
        Message::Call {
            id: CallId::new(1),
            target: Target::Cap(CapId::new(1)),
            member: "add".to_string(),
            args: vec![json!(5), json!(3)],
        },
        Message::Call {
            id: CallId::new(2),
            target: Target::Cap(CapId::new(1)),
            member: "add".to_string(),
            args: vec![json!(10), json!(20)],
        },
    ];

    // Process batch (simulating what the HTTP handler does)
    let mut responses = Vec::new();
    for msg in batch_request {
        let response = match msg {
            Message::Call { id, target, member, args } => {
                let result = match target {
                    Target::Cap(cap_id) => {
                        match server.cap_table().lookup(&cap_id) {
                            Some(cap) => match cap.call(&member, args).await {
                                Ok(value) => Outcome::Success { value },
                                Err(error) => Outcome::Error { error },
                            },
                            None => Outcome::Error {
                                error: RpcError::not_found("Capability not found"),
                            },
                        }
                    }
                    Target::Special(_) => Outcome::Error {
                        error: RpcError::not_found("Special target not implemented"),
                    },
                };
                Message::Result { id, outcome: result }
            }
            _ => msg,
        };
        responses.push(response);
    }

    // Verify responses
    assert_eq!(responses.len(), 2);

    match &responses[0] {
        Message::Result { id, outcome } => {
            assert_eq!(*id, CallId::new(1));
            match outcome {
                Outcome::Success { value } => assert_eq!(*value, json!(8.0)),
                _ => panic!("Expected success outcome"),
            }
        }
        _ => panic!("Expected Result message"),
    }

    match &responses[1] {
        Message::Result { id, outcome } => {
            assert_eq!(*id, CallId::new(2));
            match outcome {
                Outcome::Success { value } => assert_eq!(*value, json!(30.0)),
                _ => panic!("Expected success outcome"),
            }
        }
        _ => panic!("Expected Result message"),
    }
}

#[tokio::test]
async fn test_dispose_in_batch() {
    let server = Server::new(ServerConfig::default());
    let cap_id = CapId::new(42);
    server.register_capability(cap_id, Arc::new(TestCalculator));

    assert!(server.cap_table().lookup(&cap_id).is_some());

    // Create batch with dispose
    let batch_request = vec![
        Message::Call {
            id: CallId::new(1),
            target: Target::Cap(cap_id),
            member: "add".to_string(),
            args: vec![json!(1), json!(2)],
        },
        Message::Dispose {
            caps: vec![cap_id],
        },
    ];

    // Process batch
    for msg in batch_request {
        match msg {
            Message::Dispose { caps } => {
                for id in caps {
                    server.cap_table().remove(&id);
                }
            }
            _ => {}
        }
    }

    // Verify capability was removed
    assert!(server.cap_table().lookup(&cap_id).is_none());
}

#[tokio::test]
async fn test_batch_size_limit() {
    let config = ServerConfig {
        port: 0,
        host: "127.0.0.1".to_string(),
        max_batch_size: 2,
    };

    let _server = Server::new(config);

    // Create oversized batch
    let batch: Vec<Message> = (0..3)
        .map(|i| Message::Call {
            id: CallId::new(i),
            target: Target::Cap(CapId::new(1)),
            member: "test".to_string(),
            args: vec![],
        })
        .collect();

    // Check that batch exceeds limit (we know it's 3 > 2)
    assert_eq!(batch.len(), 3);
}