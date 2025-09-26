// Rust Client Example: Basic RPC Calls
// Demonstrates simple Cap'n Web client operations
// - Authentication
// - Fetching user data
// - Basic error handling

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
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("🚀 Cap'n Web Rust Client - Basic Example");
    info!("========================================");

    // Configure client
    let config = ClientConfig {
        url: "http://localhost:3000/rpc/batch".to_string(),
        max_batch_size: 100,
        timeout_ms: 10000,
    };

    // Create client
    let client = Client::new(config)?;
    info!("✅ Client created and connected");

    // Test 1: Authenticate with valid token
    info!("");
    info!("📝 Test 1: Authenticate with valid token");
    match client.call(CapId::new(1), "authenticate", vec![json!("cookie-123")]).await {
        Ok(result) => {
            info!("✅ Authentication successful!");
            info!("   User: {}", serde_json::to_string_pretty(&result)?);

            // Extract user ID for next call
            if let Some(user_id) = result.get("id") {
                info!("");
                info!("📝 Test 2: Get user profile");
                match client.call(CapId::new(1), "getUserProfile", vec![user_id.clone()]).await {
                    Ok(profile) => {
                        info!("✅ Profile retrieved!");
                        info!("   Profile: {}", serde_json::to_string_pretty(&profile)?);
                    }
                    Err(e) => {
                        info!("❌ Failed to get profile: {}", e);
                    }
                }

                info!("");
                info!("📝 Test 3: Get notifications");
                match client.call(CapId::new(1), "getNotifications", vec![user_id.clone()]).await {
                    Ok(notifications) => {
                        info!("✅ Notifications retrieved!");
                        info!("   Notifications: {}", serde_json::to_string_pretty(&notifications)?);
                    }
                    Err(e) => {
                        info!("❌ Failed to get notifications: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            info!("❌ Authentication failed: {}", e);
        }
    }

    // Test 4: Try invalid authentication
    info!("");
    info!("📝 Test 4: Authenticate with invalid token");
    match client.call(CapId::new(1), "authenticate", vec![json!("invalid-token")]).await {
        Ok(result) => {
            info!("⚠️  Unexpected success: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            info!("✅ Authentication correctly rejected: {}", e);
        }
    }

    // Test 5: Call non-existent method
    info!("");
    info!("📝 Test 5: Call non-existent method");
    match client.call(CapId::new(1), "nonExistentMethod", vec![]).await {
        Ok(result) => {
            info!("⚠️  Unexpected success: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            info!("✅ Method correctly not found: {}", e);
        }
    }

    info!("");
    info!("🎉 All tests completed!");

    Ok(())
}