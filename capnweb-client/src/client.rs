// Cap'n Web Client Library
// Implements the client side of the Cap'n Web protocol with full support for:
// - Batching multiple operations in a single request
// - Promise pipelining for dependent operations
// - Capability passing and lifecycle management
// - Error handling and validation

use anyhow::{Context, Result};
use capnweb_core::CapId;
use reqwest::Client as HttpClient;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, trace};

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL for the RPC endpoint
    pub url: String,
    /// Maximum number of operations in a single batch
    pub max_batch_size: usize,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:3000/rpc/batch".to_string(),
            max_batch_size: 100,
            timeout_ms: 30000,
        }
    }
}

/// Main client struct for Cap'n Web protocol communication
pub struct Client {
    config: ClientConfig,
    http_client: HttpClient,
    next_call_id: AtomicU64,
    capabilities: Arc<RwLock<HashMap<CapId, Value>>>,
}

impl Client {
    /// Create a new client with the given configuration
    pub fn new(config: ClientConfig) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            config,
            http_client,
            next_call_id: AtomicU64::new(1), // Start from 1 for Cap'n Web protocol
            capabilities: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new client with default configuration
    pub fn new_with_url(url: &str) -> Result<Self> {
        let config = ClientConfig {
            url: url.to_string(),
            ..Default::default()
        };
        Self::new(config)
    }

    /// Create a new batch builder for batching multiple operations
    pub fn batch(&self) -> BatchBuilder<'_> {
        BatchBuilder::new(self)
    }

    /// Perform a single RPC call
    pub async fn call(&self, cap_id: CapId, method: &str, args: Vec<Value>) -> Result<Value> {
        // Cap'n Web protocol uses import IDs starting from 1
        let import_id = self.next_call_id.fetch_add(1, Ordering::SeqCst);

        // Create batch with single operation
        let messages = vec![
            json!(["push", ["call", cap_id.as_u64(), [method], args]]),
            json!(["pull", import_id]),
        ];

        let results = self.send_batch(messages).await?;

        // Extract result
        for result in results {
            if let Some(arr) = result.as_array() {
                if arr.len() >= 3
                    && (arr[0] == "result" || arr[0] == "resolve")
                    && arr[1] == import_id
                {
                    return Ok(arr[2].clone());
                } else if arr.len() >= 3 && arr[0] == "error" && arr[1] == import_id {
                    let error = arr[2]
                        .as_object()
                        .and_then(|o| o.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    return Err(anyhow::anyhow!("RPC error: {}", error));
                }
            }
        }

        Err(anyhow::anyhow!("No result received for call"))
    }

    /// Send a batch of messages to the server
    async fn send_batch(&self, messages: Vec<Value>) -> Result<Vec<Value>> {
        // Convert messages to newline-delimited JSON
        let body_parts: Result<Vec<String>> = messages
            .iter()
            .map(|m| serde_json::to_string(m).context("Failed to serialize message"))
            .collect();
        let body = body_parts?.join("\n");

        debug!(
            "Sending batch request to {}: {} messages",
            self.config.url,
            messages.len()
        );
        trace!("Request body:\n{}", body);

        let response = self
            .http_client
            .post(&self.config.url)
            .header("Content-Type", "text/plain")
            .body(body)
            .send()
            .await
            .context("Failed to send HTTP request")?;

        let status = response.status();
        let text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!("HTTP error {}: {}", status, text));
        }

        trace!("Response body:\n{}", text);

        // Parse each line as a separate message
        let mut results = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let value: Value = serde_json::from_str(line)
                    .with_context(|| format!("Failed to parse response line: {}", line))?;
                results.push(value);
            }
        }

        debug!("Received {} response messages", results.len());
        Ok(results)
    }

    /// Register a capability for use in future calls
    pub async fn register_capability(&self, id: CapId, cap: Value) {
        let mut caps = self.capabilities.write().await;
        caps.insert(id, cap);
    }

    /// Get a registered capability
    pub async fn get_capability(&self, id: CapId) -> Option<Value> {
        let caps = self.capabilities.read().await;
        caps.get(&id).cloned()
    }

    /// Dispose of a capability
    pub async fn dispose_capability(&self, id: CapId) -> Result<()> {
        let messages = vec![json!(["dispose", id.as_u64()])];

        self.send_batch(messages).await?;

        let mut caps = self.capabilities.write().await;
        caps.remove(&id);

        Ok(())
    }
}

/// Builder for creating batched operations
pub struct BatchBuilder<'a> {
    client: &'a Client,
    operations: Vec<BatchOperation>,
    next_result_id: u64,
}

/// A single operation in a batch
#[derive(Debug, Clone)]
pub struct BatchOperation {
    pub id: u64,
    pub message: Value,
    pub is_pipeline: bool,
    pub depends_on: Option<u64>,
}

/// Handle for a pending operation result
#[derive(Debug, Clone)]
pub struct PendingResult {
    pub id: u64,
    pub path: Vec<String>,
}

impl<'a> BatchBuilder<'a> {
    fn new(client: &'a Client) -> Self {
        Self {
            client,
            operations: Vec::new(),
            next_result_id: 1, // Start from 1 to match server's import_id assignment
        }
    }

    /// Add a call operation to the batch
    pub fn call(&mut self, cap_id: CapId, method: &str, args: Vec<Value>) -> PendingResult {
        let result_id = self.next_result_id;
        self.next_result_id += 1;

        let message = json!(["push", ["call", cap_id.as_u64(), [method], args]]);

        self.operations.push(BatchOperation {
            id: result_id,
            message,
            is_pipeline: false,
            depends_on: None,
        });

        PendingResult {
            id: result_id,
            path: vec![],
        }
    }

    /// Add a pipeline operation that depends on a previous result
    pub fn pipeline(
        &mut self,
        base: &PendingResult,
        path: Vec<&str>,
        method: &str,
        args: Vec<Value>,
    ) -> PendingResult {
        let result_id = self.next_result_id;
        self.next_result_id += 1;

        // Build the pipeline arguments, replacing references to the base result
        let mut pipeline_args = Vec::new();

        // If path is provided, create a pipeline expression to extract the value
        if !path.is_empty() {
            // Create a pipeline expression to extract the value from the base result
            let path_strings: Vec<Value> = path.iter().map(|s| json!(s)).collect();
            pipeline_args.push(json!(["pipeline", base.id, path_strings]));
        }

        // Add any additional arguments
        pipeline_args.extend(args);

        // Create the pipeline message: ["pipeline", import_id, [method], [args]]
        let message = json!([
            "push",
            ["pipeline", base.id, vec![json!(method)], pipeline_args]
        ]);

        self.operations.push(BatchOperation {
            id: result_id,
            message,
            is_pipeline: true,
            depends_on: Some(base.id),
        });

        PendingResult {
            id: result_id,
            path: vec![],
        }
    }

    /// Create a reference to a result field for use in arguments
    pub fn reference(&self, result: &PendingResult, field: &str) -> PendingResult {
        PendingResult {
            id: result.id,
            path: {
                let mut path = result.path.clone();
                path.push(field.to_string());
                path
            },
        }
    }

    /// Execute the batch and return results
    pub async fn execute(self) -> Result<BatchResults> {
        if self.operations.is_empty() {
            return Ok(BatchResults {
                results: HashMap::new(),
            });
        }

        // Build messages: all pushes, then all pulls
        let mut messages = Vec::new();

        // Add all push operations
        for op in &self.operations {
            messages.push(op.message.clone());
        }

        // Add pull operations for all results
        for op in &self.operations {
            messages.push(json!(["pull", op.id]));
        }

        // Send batch
        let responses = self.client.send_batch(messages).await?;

        // Parse results
        let mut results = HashMap::new();

        for response in responses {
            if let Some(arr) = response.as_array() {
                if arr.len() >= 2 {
                    let msg_type = arr[0].as_str().unwrap_or("");

                    match msg_type {
                        "result" | "resolve" => {
                            if let Some(id) = arr[1].as_u64() {
                                let value = arr.get(2).cloned().unwrap_or(json!(null));
                                results.insert(id, Ok(value));
                            }
                        }
                        "error" => {
                            if let Some(id) = arr[1].as_u64() {
                                let error_obj = arr.get(2).cloned().unwrap_or(json!({}));
                                let error_msg = error_obj
                                    .get("message")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Unknown error");
                                results
                                    .insert(id, Err(anyhow::anyhow!("RPC error: {}", error_msg)));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(BatchResults { results })
    }
}

/// Results from a batch execution
pub struct BatchResults {
    results: HashMap<u64, Result<Value>>,
}

impl BatchResults {
    /// Get a result by its pending handle
    pub fn get(&self, pending: &PendingResult) -> Result<Value> {
        let value = self
            .results
            .get(&pending.id)
            .ok_or_else(|| anyhow::anyhow!("No result for operation {}", pending.id))?
            .as_ref()
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .clone();

        // Navigate path if specified
        if pending.path.is_empty() {
            Ok(value)
        } else {
            let mut current = value;
            for segment in &pending.path {
                current = current
                    .get(segment)
                    .ok_or_else(|| anyhow::anyhow!("Field '{}' not found in result", segment))?
                    .clone();
            }
            Ok(current)
        }
    }

    /// Check if a result exists
    pub fn contains(&self, pending: &PendingResult) -> bool {
        self.results.contains_key(&pending.id)
    }

    /// Get all results
    pub fn all(&self) -> &HashMap<u64, Result<Value>> {
        &self.results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = ClientConfig::default();
        let client = Client::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_batch_builder() {
        let config = ClientConfig::default();
        let client = Client::new(config).unwrap();
        let mut batch = client.batch();

        let result = batch.call(CapId::new(1), "test", vec![json!("arg")]);
        assert_eq!(result.id, 1); // IDs start from 1 per Cap'n Web protocol

        let pipelined = batch.pipeline(&result, vec!["field"], "method", vec![]);
        assert_eq!(pipelined.id, 2);
    }
}
