// Rust Client Example: Calculator Operations
// Demonstrates arithmetic operations using Cap'n Web client
// - Basic arithmetic calls
// - Batched calculations
// - Error handling for division by zero

use anyhow::Result;
use currentspace_capnweb_client::{Client, ClientConfig};
use currentspace_capnweb_core::CapId;
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

    info!("🧮 Cap'n Web Rust Client - Calculator Example");
    info!("==========================================");

    // Configure client for calculator server (typically on port 9000)
    let config = ClientConfig {
        url: std::env::var("RPC_URL")
            .unwrap_or_else(|_| "http://localhost:9000/rpc/batch".to_string()),
        max_batch_size: 100,
        timeout_ms: 10000,
    };

    // Create client
    let client = Client::new(config)?;
    info!("✅ Client created and connected");

    // Calculator capability is typically at CapId 2
    let calc_cap = CapId::new(2);

    // Test 1: Simple addition
    info!("");
    info!("📝 Test 1: Addition (5 + 3)");
    match client.call(calc_cap, "add", vec![json!(5), json!(3)]).await {
        Ok(result) => {
            info!("✅ Result: {}", result);
        }
        Err(e) => {
            info!("❌ Failed: {}", e);
        }
    }

    // Test 2: Multiplication
    info!("");
    info!("📝 Test 2: Multiplication (7 * 6)");
    match client
        .call(calc_cap, "multiply", vec![json!(7), json!(6)])
        .await
    {
        Ok(result) => {
            info!("✅ Result: {}", result);
        }
        Err(e) => {
            info!("❌ Failed: {}", e);
        }
    }

    // Test 3: Subtraction
    info!("");
    info!("📝 Test 3: Subtraction (10 - 4)");
    match client
        .call(calc_cap, "subtract", vec![json!(10), json!(4)])
        .await
    {
        Ok(result) => {
            info!("✅ Result: {}", result);
        }
        Err(e) => {
            info!("❌ Failed: {}", e);
        }
    }

    // Test 4: Division
    info!("");
    info!("📝 Test 4: Division (20 / 5)");
    match client
        .call(calc_cap, "divide", vec![json!(20), json!(5)])
        .await
    {
        Ok(result) => {
            info!("✅ Result: {}", result);
        }
        Err(e) => {
            info!("❌ Failed: {}", e);
        }
    }

    // Test 5: Division by zero (should error)
    info!("");
    info!("📝 Test 5: Division by zero (10 / 0)");
    match client
        .call(calc_cap, "divide", vec![json!(10), json!(0)])
        .await
    {
        Ok(result) => {
            info!("⚠️ Unexpected success: {}", result);
        }
        Err(e) => {
            info!("✅ Correctly rejected: {}", e);
        }
    }

    // Test 6: Batched calculations
    info!("");
    info!("📝 Test 6: Batched calculations");
    info!("   Computing: (10 + 5) * 2 - 8 / 4");

    let mut batch = client.batch();

    // Step 1: 10 + 5 = 15
    let sum = batch.call(calc_cap, "add", vec![json!(10), json!(5)]);

    // Step 2: 15 * 2 = 30
    let product = batch.pipeline(&sum, vec![], "multiply", vec![json!(2)]);

    // Step 3: 8 / 4 = 2
    let quotient = batch.call(calc_cap, "divide", vec![json!(8), json!(4)]);

    // Step 4: 30 - 2 = 28
    let result = batch.pipeline(&product, vec![], "subtract", vec![json!(2)]);

    match batch.execute().await {
        Ok(results) => {
            info!("   Sum (10 + 5): {}", results.get(&sum)?);
            info!("   Product (15 * 2): {}", results.get(&product)?);
            info!("   Quotient (8 / 4): {}", results.get(&quotient)?);
            info!("   Final result: {}", results.get(&result)?);
            info!("✅ Batch calculation completed!");
        }
        Err(e) => {
            info!("❌ Batch failed: {}", e);
        }
    }

    // Test 7: Complex expression with floating point
    info!("");
    info!("📝 Test 7: Floating point operations");
    match client
        .call(calc_cap, "divide", vec![json!(22.0), json!(7.0)])
        .await
    {
        Ok(result) => {
            info!("✅ 22.0 / 7.0 = {}", result);
        }
        Err(e) => {
            info!("❌ Failed: {}", e);
        }
    }

    info!("");
    info!("🎉 All calculator tests completed!");

    Ok(())
}
