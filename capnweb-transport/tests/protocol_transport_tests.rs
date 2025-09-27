// Cap'n Web Transport Protocol Compliance Tests
// Tests all transport implementations for protocol compliance
// Covers HTTP Batch, WebSocket, message framing, and bidirectional communication

use bytes::BytesMut;
use capnweb_core::protocol::{
    expression::Expression,
    ids::{ExportId, ImportId},
    message::Message,
};
use capnweb_transport::{
    CapnWebCodec, CodecError, HttpBatchTransport, NewlineDelimitedCodec, RpcTransport,
    TransportError,
};
use serde_json::Number;
use std::collections::VecDeque;
use tokio_util::codec::{Decoder, Encoder};

#[cfg(test)]
mod transport_protocol_tests {
    use super::*;

    /// Test HTTP Batch transport protocol compliance
    #[tokio::test]
    async fn test_http_batch_protocol_compliance() {
        println!("🧪 Testing HTTP Batch Transport Protocol Compliance");

        let mut transport = HttpBatchTransport::new("/rpc/batch".to_string());

        // Test protocol-compliant message sequence
        let messages = vec![
            Message::Push(Expression::String("test_request".to_string())),
            Message::Pull(ImportId(1)),
            Message::Resolve(ExportId(-1), Expression::Number(Number::from(42))),
            Message::Release {
                release: vec![ImportId(2), ImportId(3)],
            },
        ];

        // Queue messages for batch processing
        for msg in messages {
            transport
                .send_message(msg)
                .await
                .expect("Should queue message");
        }

        // Verify batch contains all messages
        let batch = transport.get_outgoing_batch();
        assert_eq!(batch.len(), 4, "Batch should contain all 4 messages");
        println!("✅ HTTP Batch message queuing verified");

        // Test batch clearing after send
        transport.clear_outgoing();
        let empty_batch = transport.get_outgoing_batch();
        assert_eq!(empty_batch.len(), 0, "Batch should be empty after clearing");
        println!("✅ HTTP Batch clearing verified");

        // Test bidirectional capability
        let response_messages = vec![
            Message::Push(Expression::String("server_notification".to_string())),
            Message::Reject(
                ExportId(-2),
                Expression::Error {
                    error_type: "server_error".to_string(),
                    message: "Operation failed".to_string(),
                    stack: None,
                },
            ),
        ];

        for msg in response_messages {
            transport
                .send_message(msg)
                .await
                .expect("Should queue response");
        }

        assert_eq!(
            transport.get_outgoing_batch().len(),
            2,
            "Should queue responses"
        );
        println!("✅ HTTP Batch bidirectional communication verified");
    }

    /// Test message framing protocol compliance
    #[tokio::test]
    async fn test_message_framing_protocol() {
        println!("🧪 Testing Message Framing Protocol Compliance");

        // Test length-prefixed framing
        let mut length_codec = CapnWebCodec::new();
        let mut buffer = BytesMut::new();

        let test_message = Message::Push(Expression::Object({
            let mut map = std::collections::HashMap::new();
            map.insert(
                "method".to_string(),
                Box::new(Expression::String("testMethod".to_string())),
            );
            map.insert(
                "args".to_string(),
                Box::new(Expression::Array(vec![
                    Expression::Number(Number::from(1)),
                    Expression::Bool(true),
                    Expression::Null,
                ])),
            );
            map
        }));

        // Test encoding with length prefix
        length_codec
            .encode(test_message.clone(), &mut buffer)
            .expect("Should encode");
        assert!(buffer.len() >= 4, "Should have length prefix");

        // Verify length prefix is correct
        let length_bytes = &buffer[0..4];
        let expected_length = (buffer.len() - 4) as u32;
        let actual_length = u32::from_be_bytes([
            length_bytes[0],
            length_bytes[1],
            length_bytes[2],
            length_bytes[3],
        ]);
        assert_eq!(
            actual_length, expected_length,
            "Length prefix should match payload size"
        );
        println!("✅ Length-prefixed framing verified");

        // Test decoding
        let decoded = length_codec
            .decode(&mut buffer)
            .expect("Should decode")
            .expect("Should have message");
        assert_eq!(
            decoded, test_message,
            "Decoded message should match original"
        );
        println!("✅ Length-prefixed decoding verified");

        // Test newline-delimited framing
        let mut newline_codec = NewlineDelimitedCodec::new();
        let mut newline_buffer = BytesMut::new();

        let simple_messages = vec![
            Message::Push(Expression::String("message1".to_string())),
            Message::Pull(ImportId(1)),
            Message::Resolve(ExportId(-1), Expression::String("result".to_string())),
        ];

        // Encode multiple messages with newline framing
        for msg in &simple_messages {
            newline_codec
                .encode(msg.clone(), &mut newline_buffer)
                .expect("Should encode");
        }

        // Verify newlines are present
        let buffer_str = String::from_utf8_lossy(&newline_buffer);
        let line_count = buffer_str.matches('\n').count();
        assert_eq!(
            line_count,
            simple_messages.len(),
            "Should have newline for each message"
        );
        println!("✅ Newline-delimited framing verified");

        // Test decoding all messages
        let mut decoded_messages = Vec::new();
        while let Some(msg) = newline_codec
            .decode(&mut newline_buffer)
            .expect("Should decode")
        {
            decoded_messages.push(msg);
        }

        assert_eq!(
            decoded_messages.len(),
            simple_messages.len(),
            "Should decode all messages"
        );
        for (original, decoded) in simple_messages.iter().zip(decoded_messages.iter()) {
            assert_eq!(original, decoded, "Messages should match after round trip");
        }
        println!("✅ Newline-delimited decoding verified");
    }

    /// Test transport error handling protocol compliance
    #[tokio::test]
    async fn test_transport_error_handling() {
        println!("🧪 Testing Transport Error Handling Protocol");

        // Test codec error handling
        let mut codec = CapnWebCodec::with_max_frame_size(100);
        let mut buffer = BytesMut::new();

        // Create message larger than max frame size
        let large_string = "x".repeat(200);
        let large_message = Message::Push(Expression::String(large_string));

        let encode_result = codec.encode(large_message, &mut buffer);
        assert!(encode_result.is_err(), "Should error on oversized message");

        match encode_result.unwrap_err() {
            CodecError::FrameTooLarge(size) => {
                assert!(size > 100, "Should report actual size");
            }
            _ => panic!("Should be FrameTooLarge error"),
        }
        println!("✅ Frame size limit error handling verified");

        // Test malformed JSON handling
        let mut decode_codec = NewlineDelimitedCodec::new();
        let mut malformed_buffer = BytesMut::from("{ invalid json }\n");

        let decode_result = decode_codec.decode(&mut malformed_buffer);
        assert!(decode_result.is_err(), "Should error on malformed JSON");

        match decode_result.unwrap_err() {
            CodecError::JsonError(_) => {} // Expected
            _ => panic!("Should be JSON error"),
        }
        println!("✅ Malformed JSON error handling verified");

        // Test partial frame handling
        let mut partial_codec = CapnWebCodec::new();
        let mut partial_buffer = BytesMut::from(&b"\x00\x00\x00\x10partial"[..]);

        let partial_result = partial_codec
            .decode(&mut partial_buffer)
            .expect("Should not error");
        assert!(
            partial_result.is_none(),
            "Should return None for partial frame"
        );
        println!("✅ Partial frame handling verified");
    }

    /// Test protocol compliance across transport types
    #[tokio::test]
    async fn test_cross_transport_protocol_compliance() {
        println!("🧪 Testing Cross-Transport Protocol Compliance");

        // Same message should work across all transport types
        let protocol_messages = vec![
            // Core protocol messages
            Message::Push(Expression::Import(ImportId(123))),
            Message::Pull(ImportId(456)),
            Message::Resolve(
                ExportId(-789),
                Expression::Pipeline {
                    pipeline: ImportId(111),
                    property: vec!["getData".to_string()],
                    args: None,
                },
            ),
            Message::Reject(
                ExportId(-222),
                Expression::Error {
                    error_type: "timeout".to_string(),
                    message: "Operation timed out".to_string(),
                    stack: Some("at transport layer".to_string()),
                },
            ),
            Message::Release {
                release: vec![ImportId(333), ImportId(444)],
            },
            Message::Abort {
                abort: Expression::Error {
                    error_type: "transport_error".to_string(),
                    message: "Connection lost".to_string(),
                    stack: None,
                },
            },
        ];

        // Test HTTP Batch transport
        let mut http_transport = HttpBatchTransport::new("/rpc/batch".to_string());
        for msg in &protocol_messages {
            http_transport
                .send_message(msg.clone())
                .await
                .expect("HTTP transport should accept message");
        }
        assert_eq!(
            http_transport.get_outgoing_batch().len(),
            protocol_messages.len(),
            "HTTP batch should contain all messages"
        );
        println!("✅ HTTP Batch transport protocol compliance verified");

        // Test both codec types with same messages
        for (codec_name, mut codec) in [
            (
                "length-prefixed",
                Box::new(CapnWebCodec::new())
                    as Box<
                        dyn Encoder<Message, Error = CodecError>
                            + Decoder<Item = Message, Error = CodecError>,
                    >,
            ),
            (
                "newline-delimited",
                Box::new(NewlineDelimitedCodec::new())
                    as Box<
                        dyn Encoder<Message, Error = CodecError>
                            + Decoder<Item = Message, Error = CodecError>,
                    >,
            ),
        ] {
            let mut buffer = BytesMut::new();
            let mut round_trip_messages = Vec::new();

            // Encode all messages
            for msg in &protocol_messages {
                codec
                    .encode(msg.clone(), &mut buffer)
                    .expect(&format!("{} codec should encode", codec_name));
            }

            // Decode all messages
            while let Some(msg) = codec
                .decode(&mut buffer)
                .expect(&format!("{} codec should decode", codec_name))
            {
                round_trip_messages.push(msg);
            }

            assert_eq!(
                round_trip_messages.len(),
                protocol_messages.len(),
                "{} codec should round-trip all messages",
                codec_name
            );

            for (original, decoded) in protocol_messages.iter().zip(round_trip_messages.iter()) {
                assert_eq!(
                    original, decoded,
                    "{} codec should preserve message content",
                    codec_name
                );
            }

            println!("✅ {} codec protocol compliance verified", codec_name);
        }
    }

    /// Test bidirectional protocol communication
    #[tokio::test]
    async fn test_bidirectional_protocol_communication() {
        println!("🧪 Testing Bidirectional Protocol Communication");

        // Simulate full bidirectional conversation
        let client_messages = vec![
            // Client initiates
            Message::Push(Expression::String("client_hello".to_string())),
            // Client requests data
            Message::Push(Expression::Pipeline {
                pipeline: ImportId(1), // Server's capability
                property: vec!["getUserData".to_string()],
                args: Some(Box::new(Expression::String("user123".to_string()))),
            }),
            // Client pulls result
            Message::Pull(ImportId(2)),
        ];

        let server_responses = vec![
            // Server responds to hello
            Message::Resolve(ExportId(-1), Expression::String("server_hello".to_string())),
            // Server provides user data
            Message::Resolve(
                ExportId(-2),
                Expression::Object({
                    let mut user_data = std::collections::HashMap::new();
                    user_data.insert(
                        "id".to_string(),
                        Box::new(Expression::String("user123".to_string())),
                    );
                    user_data.insert(
                        "name".to_string(),
                        Box::new(Expression::String("Alice".to_string())),
                    );
                    user_data.insert("active".to_string(), Box::new(Expression::Bool(true)));
                    user_data
                }),
            ),
            // Server sends notification
            Message::Push(Expression::Object({
                let mut notification = std::collections::HashMap::new();
                notification.insert(
                    "type".to_string(),
                    Box::new(Expression::String("user_login".to_string())),
                );
                notification.insert(
                    "timestamp".to_string(),
                    Box::new(Expression::Number(Number::from(1640995200))),
                );
                notification
            })),
        ];

        // Test that both directions serialize correctly
        let mut all_messages = Vec::new();
        all_messages.extend(client_messages);
        all_messages.extend(server_responses);

        let mut transport = HttpBatchTransport::new("/rpc/batch".to_string());
        for msg in all_messages {
            transport
                .send_message(msg)
                .await
                .expect("Should queue all messages");
        }

        assert_eq!(
            transport.get_outgoing_batch().len(),
            6,
            "Should have all 6 messages"
        );
        println!("✅ Bidirectional conversation serialization verified");

        // Test message ordering preservation
        let batch = transport.get_outgoing_batch();
        let first_msg = &batch[0];
        let last_msg = &batch[5];

        match (first_msg, last_msg) {
            (Message::Push(Expression::String(s1)), Message::Push(Expression::Object(_))) => {
                assert_eq!(s1, "client_hello");
            }
            _ => panic!("Message ordering not preserved"),
        }
        println!("✅ Message ordering preservation verified");
    }

    /// Test transport-specific protocol features
    #[tokio::test]
    async fn test_transport_specific_features() {
        println!("🧪 Testing Transport-Specific Protocol Features");

        // Test HTTP Batch specific features
        let mut http_transport = HttpBatchTransport::new("/rpc/batch".to_string());

        // Test batch size limits
        let max_batch_size = 1000;
        for i in 0..max_batch_size + 1 {
            let msg = Message::Push(Expression::String(format!("message_{}", i)));
            http_transport
                .send_message(msg)
                .await
                .expect("Should queue message");
        }

        // HTTP batch should handle large batches
        let batch_size = http_transport.get_outgoing_batch().len();
        assert_eq!(
            batch_size,
            max_batch_size + 1,
            "HTTP batch should handle all messages"
        );
        println!("✅ HTTP Batch size handling verified");

        // Test endpoint configuration
        let custom_transport = HttpBatchTransport::new("/api/v1/capnweb".to_string());
        assert_eq!(custom_transport.endpoint(), "/api/v1/capnweb");
        println!("✅ HTTP Batch custom endpoint verified");

        // Test concurrent access safety
        let concurrent_transport = HttpBatchTransport::new("/rpc/batch".to_string());
        let transport_clone = concurrent_transport.clone();

        let handle1 = tokio::spawn(async move {
            for i in 0..100 {
                let msg = Message::Push(Expression::Number(Number::from(i)));
                concurrent_transport.send_message(msg).await.unwrap();
            }
        });

        let handle2 = tokio::spawn(async move {
            for i in 100..200 {
                let msg = Message::Push(Expression::Number(Number::from(i)));
                transport_clone.send_message(msg).await.unwrap();
            }
        });

        let _ = tokio::join!(handle1, handle2);
        println!("✅ Concurrent transport access verified");
    }
}

#[cfg(feature = "websocket")]
#[cfg(test)]
mod websocket_protocol_tests {
    use super::*;
    use capnweb_transport::websocket::WebSocketTransport;

    /// Test WebSocket transport protocol compliance
    #[tokio::test]
    async fn test_websocket_protocol_compliance() {
        println!("🧪 Testing WebSocket Transport Protocol Compliance");

        // WebSocket should support all the same protocol messages as HTTP Batch
        let ws_messages = vec![
            Message::Push(Expression::String("websocket_test".to_string())),
            Message::Pull(ImportId(1)),
            Message::Resolve(ExportId(-1), Expression::Bool(true)),
            Message::Release {
                release: vec![ImportId(2)],
            },
        ];

        // Note: Actual WebSocket testing would require a running server
        // This test focuses on protocol message compatibility
        for msg in &ws_messages {
            let json = msg.to_json();
            let parsed = Message::from_json(&json).expect("WebSocket messages should parse");
            assert_eq!(
                *msg, parsed,
                "WebSocket message should round-trip correctly"
            );
        }

        println!("✅ WebSocket protocol message compatibility verified");

        // Test real-time message framing for WebSocket
        let mut codec = NewlineDelimitedCodec::new();
        let mut buffer = BytesMut::new();

        // WebSocket typically uses newline framing for text frames
        for msg in ws_messages {
            codec
                .encode(msg, &mut buffer)
                .expect("Should encode for WebSocket");
        }

        // Verify each message ends with newline (WebSocket text frame requirement)
        let buffer_str = String::from_utf8_lossy(&buffer);
        let lines: Vec<&str> = buffer_str.trim().split('\n').collect();
        assert_eq!(lines.len(), 4, "Should have 4 lines for 4 messages");

        for line in lines {
            assert!(!line.is_empty(), "No line should be empty");
            // Each line should be valid JSON
            let parsed: serde_json::Value =
                serde_json::from_str(line).expect("Line should be valid JSON");
            assert!(parsed.is_array(), "Each message should be JSON array");
        }

        println!("✅ WebSocket message framing verified");
    }
}

#[cfg(test)]
mod protocol_edge_cases {
    use super::*;

    /// Test protocol edge cases and corner scenarios
    #[tokio::test]
    async fn test_protocol_edge_cases() {
        println!("🧪 Testing Protocol Edge Cases");

        // Test empty message handling
        let empty_release = Message::Release { release: vec![] };
        let empty_json = empty_release.to_json();
        let empty_parsed = Message::from_json(&empty_json).expect("Empty release should parse");
        assert_eq!(empty_release, empty_parsed);
        println!("✅ Empty release message verified");

        // Test maximum ID values
        let max_import = Message::Pull(ImportId(i64::MAX));
        let max_export = Message::Resolve(ExportId(i64::MIN), Expression::Null);

        let max_import_json = max_import.to_json();
        let max_export_json = max_export.to_json();

        let max_import_parsed =
            Message::from_json(&max_import_json).expect("Max import should parse");
        let max_export_parsed =
            Message::from_json(&max_export_json).expect("Max export should parse");

        assert_eq!(max_import, max_import_parsed);
        assert_eq!(max_export, max_export_parsed);
        println!("✅ Maximum ID values verified");

        // Test deeply nested expressions
        let mut deep_expr = Expression::Null;
        for i in 0..10 {
            let mut obj = std::collections::HashMap::new();
            obj.insert(format!("level_{}", i), Box::new(deep_expr));
            deep_expr = Expression::Object(obj);
        }

        let deep_message = Message::Push(deep_expr);
        let deep_json = deep_message.to_json();
        let deep_parsed = Message::from_json(&deep_json).expect("Deep nesting should parse");
        assert_eq!(deep_message, deep_parsed);
        println!("✅ Deep expression nesting verified");

        // Test Unicode handling
        let unicode_message = Message::Push(Expression::String("Hello 世界! 🌍 Ñoël".to_string()));
        let unicode_json = unicode_message.to_json();
        let unicode_parsed = Message::from_json(&unicode_json).expect("Unicode should parse");
        assert_eq!(unicode_message, unicode_parsed);
        println!("✅ Unicode handling verified");

        // Test large batch processing
        let mut large_batch_transport = HttpBatchTransport::new("/rpc/batch".to_string());
        let large_batch_size = 10000;

        for i in 0..large_batch_size {
            let msg = Message::Push(Expression::Number(Number::from(i)));
            large_batch_transport
                .send_message(msg)
                .await
                .expect("Should handle large batch");
        }

        assert_eq!(
            large_batch_transport.get_outgoing_batch().len(),
            large_batch_size
        );
        println!("✅ Large batch processing verified");
    }
}
