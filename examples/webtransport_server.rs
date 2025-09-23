//! WebTransport HTTP/3 Server Example
//!
//! Demonstrates modern QUIC-based transport with HTTP/3
//! Features:
//! - HTTP/3 server with self-signed certificates
//! - WebTransport bidirectional streams
//! - Modern async/await patterns

use capnweb_core::{CapId, RpcError};
use capnweb_server::{Server, ServerConfig, RpcTarget, H3Server};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, Level};
use tracing_subscriber;

/// High-performance data processor capability
#[derive(Debug)]
struct DataProcessor {
    processor_id: String,
}

impl DataProcessor {
    fn new(processor_id: String) -> Self {
        Self { processor_id }
    }
}

impl RpcTarget for DataProcessor {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        info!("DataProcessor '{}' called method: {} with {} args",
              self.processor_id, member, args.len());

        match member {
            "process_batch" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("process_batch requires data array"));
                }

                let data = &args[0];
                if let Some(array) = data.as_array() {
                    // Simulate data processing
                    let processed_count = array.len();
                    let sum: f64 = array
                        .iter()
                        .filter_map(|v| v.as_f64())
                        .sum();

                    Ok(json!({
                        "processor_id": self.processor_id,
                        "processed_count": processed_count,
                        "sum": sum,
                        "average": if processed_count > 0 { sum / processed_count as f64 } else { 0.0 },
                        "status": "completed"
                    }))
                } else {
                    Err(RpcError::bad_request("data must be an array"))
                }
            }
            "get_stats" => {
                Ok(json!({
                    "processor_id": self.processor_id,
                    "uptime": "running",
                    "memory_usage": "12.5MB",
                    "processed_today": 1456,
                    "queue_size": 0
                }))
            }
            "benchmark" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("benchmark requires operation count"));
                }

                let count = args[0].as_i64().unwrap_or(1000);
                let start = std::time::Instant::now();

                // Simulate intensive computation
                let mut result = 0.0;
                for i in 0..count {
                    result += (i as f64).sqrt();
                }

                let duration = start.elapsed();

                Ok(json!({
                    "processor_id": self.processor_id,
                    "operations": count,
                    "duration_ms": duration.as_millis(),
                    "ops_per_second": count as f64 / duration.as_secs_f64(),
                    "result": result
                }))
            }
            _ => Err(RpcError::not_found(format!("method '{}' not found", member))),
        }
    }
}

/// Real-time streaming capability
#[derive(Debug)]
struct StreamProcessor;

impl RpcTarget for StreamProcessor {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        info!("StreamProcessor called method: {} with {} args", member, args.len());

        match member {
            "start_stream" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("start_stream requires stream config"));
                }

                let config = &args[0];
                let stream_type = config.get("type").and_then(|v| v.as_str()).unwrap_or("default");
                let rate = config.get("rate").and_then(|v| v.as_i64()).unwrap_or(10);

                Ok(json!({
                    "stream_id": "stream_001",
                    "type": stream_type,
                    "rate": rate,
                    "status": "started",
                    "endpoint": "wss://localhost:8443/stream/001"
                }))
            }
            "get_stream_stats" => {
                let stream_id = args.get(0)
                    .and_then(|v| v.as_str())
                    .unwrap_or("stream_001");

                Ok(json!({
                    "stream_id": stream_id,
                    "active": true,
                    "messages_sent": 15420,
                    "bytes_transferred": 2048576,
                    "connected_clients": 3,
                    "uptime_seconds": 3600
                }))
            }
            _ => Err(RpcError::not_found(format!("method '{}' not found", member))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Cap'n Web WebTransport/HTTP3 Server...");

    // Create server configuration optimized for HTTP/3
    let config = ServerConfig {
        http_bind_addr: "0.0.0.0:8443".to_string(),
        enable_h3: true,
        max_connections: 10000,
        max_batch_size: 1000,
        rate_limit_requests_per_second: Some(1000),
        ..Default::default()
    };

    // Create server
    let server = Arc::new(Server::new(config));

    // Register high-performance capabilities
    info!("Registering capabilities...");

    // Multiple data processors for load balancing
    for i in 1..=4 {
        let processor = Arc::new(DataProcessor::new(format!("processor_{}", i)));
        server.register_capability(CapId::new(i), processor)?;
    }

    // Stream processor
    let stream_processor = Arc::new(StreamProcessor);
    server.register_capability(CapId::new(100), stream_processor)?;

    info!("Capabilities registered:");
    info!("  - Data Processors (ID: 1-4) - High-performance batch processing");
    info!("  - Stream Processor (ID: 100) - Real-time data streaming");

    // Create HTTP/3 server
    let mut h3_server = H3Server::new(server.clone());

    info!("Starting WebTransport/HTTP3 server...");
    info!("Server listening on: https://0.0.0.0:8443");
    info!("WebTransport endpoint: https://0.0.0.0:8443/.well-known/webtransport");
    info!("");
    info!("Note: This server uses self-signed certificates for testing.");
    info!("In production, use proper TLS certificates.");
    info!("");
    info!("Example WebTransport client connection:");
    info!("  const transport = new WebTransport('https://localhost:8443');");
    info!("  await transport.ready;");

    // Start server in background
    let server_handle = {
        let addr = "0.0.0.0:8443".parse()?;
        tokio::spawn(async move {
            if let Err(e) = h3_server.listen(addr).await {
                error!("WebTransport server error: {}", e);
            }
        })
    };

    // Wait for shutdown signal
    info!("Server started! Press Ctrl+C to shutdown.");
    info!("");
    info!("Available capabilities:");
    info!("  DataProcessor (ID: 1-4):");
    info!("    - process_batch(data: number[]): ProcessResult");
    info!("    - get_stats(): ProcessorStats");
    info!("    - benchmark(count: number): BenchmarkResult");
    info!("");
    info!("  StreamProcessor (ID: 100):");
    info!("    - start_stream(config: StreamConfig): StreamInfo");
    info!("    - get_stream_stats(stream_id: string): StreamStats");

    // Wait for Ctrl+C
    signal::ctrl_c().await?;

    info!("Shutdown signal received, stopping server...");

    // Shutdown server gracefully
    server.shutdown().await;
    server_handle.abort();

    info!("WebTransport server stopped.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_data_processor() {
        let processor = DataProcessor::new("test_processor".to_string());

        // Test batch processing
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = processor.call("process_batch", vec![json!(data)]).await.unwrap();

        assert_eq!(result["processed_count"], json!(5));
        assert_eq!(result["sum"], json!(15.0));
        assert_eq!(result["average"], json!(3.0));

        // Test stats
        let stats = processor.call("get_stats", vec![]).await.unwrap();
        assert!(stats["processor_id"].is_string());

        // Test benchmark
        let benchmark = processor.call("benchmark", vec![json!(100)]).await.unwrap();
        assert_eq!(benchmark["operations"], json!(100));
        assert!(benchmark["duration_ms"].is_number());
    }

    #[tokio::test]
    async fn test_stream_processor() {
        let processor = StreamProcessor;

        // Test stream start
        let config = json!({
            "type": "realtime",
            "rate": 50
        });
        let result = processor.call("start_stream", vec![config]).await.unwrap();
        assert_eq!(result["status"], json!("started"));

        // Test stream stats
        let stats = processor.call("get_stream_stats", vec![json!("stream_001")]).await.unwrap();
        assert_eq!(stats["stream_id"], json!("stream_001"));
        assert!(stats["active"].is_boolean());
    }
}