// Rust Client Example: Error Handling
// Demonstrates various error scenarios and handling strategies
// - Network errors
// - Method not found
// - Invalid arguments
// - Capability not found
// - Timeout handling

use anyhow::Result;
use capnweb_client::{Client, ClientConfig};
use capnweb_core::CapId;
use serde_json::json;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("⚠️ Cap'n Web Rust Client - Error Handling Example");
    info!("================================================");

    let base_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:3000/rpc/batch".to_string());

    // Test 1: Connection to non-existent server
    info!("");
    info!("📝 Test 1: Connection to non-existent server");
    let bad_config = ClientConfig {
        url: "http://localhost:99999/rpc/batch".to_string(),
        max_batch_size: 100,
        timeout_ms: 3000,
    };

    match Client::new(bad_config) {
        Ok(client) => match client.call(CapId::new(1), "test", vec![]).await {
            Ok(_) => info!("⚠️ Unexpected success"),
            Err(e) => info!("✅ Expected error: {}", e),
        },
        Err(e) => {
            info!("✅ Expected error during client creation: {}", e);
        }
    }

    // Create a working client for remaining tests
    let config = ClientConfig {
        url: base_url.clone(),
        max_batch_size: 100,
        timeout_ms: 10000,
    };
    let client = Client::new(config)?;
    info!("✅ Client connected to server");

    // Test 2: Call to non-existent method
    info!("");
    info!("📝 Test 2: Call to non-existent method");
    match client
        .call(CapId::new(1), "nonExistentMethod", vec![])
        .await
    {
        Ok(result) => {
            info!("⚠️ Unexpected success: {}", result);
        }
        Err(e) => {
            info!("✅ Expected error: {}", e);
        }
    }

    // Test 3: Invalid arguments
    info!("");
    info!("📝 Test 3: Invalid arguments (wrong type)");
    match client
        .call(CapId::new(1), "authenticate", vec![json!(12345)])
        .await
    {
        Ok(result) => {
            info!("⚠️ Unexpected success: {}", result);
        }
        Err(e) => {
            info!("✅ Expected error: {}", e);
        }
    }

    // Test 4: Missing required arguments
    info!("");
    info!("📝 Test 4: Missing required arguments");
    match client.call(CapId::new(1), "authenticate", vec![]).await {
        Ok(result) => {
            info!("⚠️ Unexpected success: {}", result);
        }
        Err(e) => {
            info!("✅ Expected error: {}", e);
        }
    }

    // Test 5: Capability not found
    info!("");
    info!("📝 Test 5: Invalid capability ID");
    match client.call(CapId::new(999), "someMethod", vec![]).await {
        Ok(result) => {
            info!("⚠️ Unexpected success: {}", result);
        }
        Err(e) => {
            info!("✅ Expected error: {}", e);
        }
    }

    // Test 6: Batch with mixed success and failure
    info!("");
    info!("📝 Test 6: Batch with mixed operations");
    let mut batch = client.batch();

    // This should succeed
    let valid_call = batch.call(CapId::new(1), "authenticate", vec![json!("cookie-123")]);

    // This should fail (invalid method)
    let invalid_call = batch.call(CapId::new(1), "invalidMethod", vec![]);

    // This should fail (wrong arguments)
    let bad_args = batch.call(CapId::new(1), "authenticate", vec![json!(123), json!(456)]);

    match batch.execute().await {
        Ok(results) => {
            info!("Batch executed with partial results:");

            match results.get(&valid_call) {
                Ok(value) => info!("  ✅ Valid call succeeded: {}", value),
                Err(e) => info!("  ❌ Valid call failed: {}", e),
            }

            match results.get(&invalid_call) {
                Ok(value) => info!("  ⚠️ Invalid method unexpectedly succeeded: {}", value),
                Err(e) => info!("  ✅ Invalid method correctly failed: {}", e),
            }

            match results.get(&bad_args) {
                Ok(value) => info!("  ⚠️ Bad args unexpectedly succeeded: {}", value),
                Err(e) => info!("  ✅ Bad args correctly failed: {}", e),
            }
        }
        Err(e) => {
            info!("❌ Entire batch failed: {}", e);
        }
    }

    // Test 7: Timeout simulation (if server supports slow operations)
    info!("");
    info!("📝 Test 7: Timeout handling");
    let timeout_config = ClientConfig {
        url: base_url.clone(),
        max_batch_size: 100,
        timeout_ms: 100, // Very short timeout
    };

    match Client::new(timeout_config) {
        Ok(timeout_client) => {
            match timeout_client
                .call(CapId::new(1), "authenticate", vec![json!("cookie-123")])
                .await
            {
                Ok(_) => info!("  Call completed within timeout"),
                Err(e) => {
                    if e.to_string().contains("timeout") || e.to_string().contains("elapsed") {
                        info!("✅ Timeout correctly triggered: {}", e);
                    } else {
                        info!("  Other error occurred: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            info!("❌ Failed to create timeout client: {}", e);
        }
    }

    // Test 8: Pipeline with invalid reference
    info!("");
    info!("📝 Test 8: Pipeline with invalid reference");
    let mut batch = client.batch();

    // This call will fail
    let failing_call = batch.call(CapId::new(1), "authenticate", vec![json!("invalid-token")]);

    // This pipeline depends on the failing call
    let dependent = batch.pipeline(&failing_call, vec!["id"], "getUserProfile", vec![]);

    match batch.execute().await {
        Ok(results) => {
            match results.get(&failing_call) {
                Ok(_) => info!("  ⚠️ First call unexpectedly succeeded"),
                Err(e) => info!("  ✅ First call failed as expected: {}", e),
            }

            match results.get(&dependent) {
                Ok(_) => info!("  ⚠️ Dependent call unexpectedly succeeded"),
                Err(e) => info!("  ✅ Dependent call correctly failed: {}", e),
            }
        }
        Err(e) => {
            info!("❌ Batch execution failed: {}", e);
        }
    }

    // Test 9: Dispose non-existent capability
    info!("");
    info!("📝 Test 9: Dispose non-existent capability");
    match client.dispose_capability(CapId::new(999)).await {
        Ok(_) => info!("  ⚠️ Dispose unexpectedly succeeded"),
        Err(e) => info!("✅ Dispose correctly failed: {}", e),
    }

    // Test 10: Large batch exceeding max size
    info!("");
    info!("📝 Test 10: Batch size limits");
    let limited_config = ClientConfig {
        url: base_url.clone(),
        max_batch_size: 5,
        timeout_ms: 10000,
    };

    match Client::new(limited_config) {
        Ok(limited_client) => {
            let mut large_batch = limited_client.batch();

            // Try to add more operations than allowed
            for i in 0..10 {
                large_batch.call(
                    CapId::new(1),
                    "authenticate",
                    vec![json!(format!("token-{}", i))],
                );
            }

            match large_batch.execute().await {
                Ok(results) => {
                    info!("  Batch executed with {} results", results.all().len());
                    if results.all().len() <= 5 {
                        info!("✅ Batch size was correctly limited");
                    }
                }
                Err(e) => {
                    info!("  Batch failed (might be due to size limit): {}", e);
                }
            }
        }
        Err(e) => {
            info!("❌ Failed to create limited client: {}", e);
        }
    }

    info!("");
    info!("🎉 All error handling tests completed!");
    info!("Note: Some tests depend on server implementation details");

    Ok(())
}
