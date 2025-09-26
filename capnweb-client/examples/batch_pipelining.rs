// Rust Client Example: Batch Pipelining
// Demonstrates:
// - Batching + pipelining: multiple dependent calls, one round trip
// - Sequential calls: multiple round trips
// Mirrors the functionality of TypeScript's batch-pipelining/client.mjs

use anyhow::Result;
use capnweb_client::{Client, ClientConfig};
use capnweb_core::CapId;
use serde_json::{json, Value};
use std::time::Instant;
use tracing::{debug, info};

async fn run_pipelined() -> Result<(Value, Value, Value, u128, usize)> {
    let start = Instant::now();

    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:3000/rpc/batch".to_string());

    // Create client with configuration
    let config = ClientConfig {
        url: rpc_url.clone(),
        max_batch_size: 100,
        timeout_ms: 30000,
    };
    let client = Client::new(config)?;

    // Create batch of pipelined operations
    let mut batch = client.batch();

    // First call: authenticate (to capability 1, which is the Api)
    let user_result = batch.call(CapId::new(1), "authenticate", vec![json!("cookie-123")]);

    // Second call: get profile using the user ID from authentication (pipelined)
    let profile_result = batch.pipeline(
        &user_result,
        vec!["id"], // Path to the user ID in the result
        "getUserProfile",
        vec![], // The user ID will be extracted from the pipeline path
    );

    // Third call: get notifications using the user ID (pipelined)
    let notifications_result = batch.pipeline(
        &user_result,
        vec!["id"], // Path to the user ID in the result
        "getNotifications",
        vec![],
    );

    // Execute the batch - single HTTP POST
    let results = batch.execute().await?;

    // Extract the three result values
    let user = results.get(&user_result)?;
    let profile = results.get(&profile_result)?;
    let notifications = results.get(&notifications_result)?;

    let elapsed = start.elapsed().as_millis();
    let request_count = 1; // Single batch request

    Ok((user, profile, notifications, elapsed, request_count))
}

async fn run_sequential() -> Result<(Value, Value, Value, u128, usize)> {
    let start = Instant::now();
    let mut request_count = 0;

    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:3000/rpc/batch".to_string());

    // Create client
    let config = ClientConfig {
        url: rpc_url.clone(),
        max_batch_size: 100,
        timeout_ms: 30000,
    };
    let client = Client::new(config)?;

    // 1) Authenticate (1 round trip)
    request_count += 1;
    let user = client
        .call(CapId::new(1), "authenticate", vec![json!("cookie-123")])
        .await?;

    // Extract user ID
    let user_id = user
        .get("id")
        .ok_or_else(|| anyhow::anyhow!("No user id in response"))?
        .clone();

    // 2) Fetch profile (2nd round trip)
    request_count += 1;
    let profile = client
        .call(CapId::new(1), "getUserProfile", vec![user_id.clone()])
        .await?;

    // 3) Fetch notifications (3rd round trip)
    request_count += 1;
    let notifications = client
        .call(CapId::new(1), "getNotifications", vec![user_id])
        .await?;

    let elapsed = start.elapsed().as_millis();
    Ok((user, profile, notifications, elapsed, request_count))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("🚀 Cap'n Web Rust Client - Batch Pipelining Example");
    info!("================================================");

    // Run pipelined version
    info!("");
    info!("--- Running PIPELINED (batched, single round trip) ---");

    match run_pipelined().await {
        Ok((user_p, profile_p, notifs_p, elapsed_p, posts_p)) => {
            info!("✅ Success!");
            info!("  📬 HTTP POSTs: {}", posts_p);
            info!("  ⏱️  Time: {} ms", elapsed_p);
            info!("  👤 User: {}", serde_json::to_string_pretty(&user_p)?);
            info!(
                "  📝 Profile: {}",
                serde_json::to_string_pretty(&profile_p)?
            );
            info!(
                "  🔔 Notifications: {}",
                serde_json::to_string_pretty(&notifs_p)?
            );

            // Run sequential version
            info!("");
            info!("--- Running SEQUENTIAL (non-batched, multiple round trips) ---");

            match run_sequential().await {
                Ok((user_s, profile_s, notifs_s, elapsed_s, posts_s)) => {
                    info!("✅ Success!");
                    info!("  📬 HTTP POSTs: {}", posts_s);
                    info!("  ⏱️  Time: {} ms", elapsed_s);
                    info!("  👤 User: {}", serde_json::to_string_pretty(&user_s)?);
                    info!(
                        "  📝 Profile: {}",
                        serde_json::to_string_pretty(&profile_s)?
                    );
                    info!(
                        "  🔔 Notifications: {}",
                        serde_json::to_string_pretty(&notifs_s)?
                    );

                    // Summary and validation
                    info!("");
                    info!("📊 Summary:");
                    info!("  Pipelined: {} POST, {} ms", posts_p, elapsed_p);
                    info!("  Sequential: {} POSTs, {} ms", posts_s, elapsed_s);

                    let speedup = elapsed_s as f64 / elapsed_p.max(1) as f64;
                    info!(
                        "  🎯 Performance improvement: {:.1}x faster with pipelining",
                        speedup
                    );

                    // Validate results match
                    assert_eq!(user_p, user_s, "User results should match");
                    assert_eq!(profile_p, profile_s, "Profile results should match");
                    assert_eq!(notifs_p, notifs_s, "Notifications should match");
                    info!("");
                    info!("✅ All results validated successfully!");
                }
                Err(e) => {
                    info!("❌ Sequential execution failed: {}", e);
                }
            }
        }
        Err(e) => {
            info!("❌ Pipelined execution failed: {}", e);
            info!("Make sure the typescript_examples_server is running on port 3000");
        }
    }

    Ok(())
}
