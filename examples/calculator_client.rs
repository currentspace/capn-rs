//! Example Calculator Client
//!
//! Demonstrates advanced client features:
//! - Multiple transport types
//! - Promise pipelining
//! - Complex plan construction
//! - Error handling and recovery

use capnweb_client::{Client, ClientConfig, Recorder, params, record_object, record_array};
use capnweb_core::{CapId, Plan};
use capnweb_transport::{HttpBatchTransport, WebSocketTransport};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn, Level};
use tracing_subscriber;

async fn run_calculator_examples(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Basic Calculator Operations ===");

    // Simple calculations using the recorder API
    let recorder = Recorder::new();
    let calc = recorder.capture("calculator", CapId::new(1));

    // Basic arithmetic operations
    let sum = calc.call("add", params![15.5, 24.3]);
    let product = calc.call("multiply", params![7, 8]);
    let power = calc.call("power", params![2, 10]);

    // Build and execute plan
    let plan = recorder.build(record_array!(recorder; [sum, product, power]).as_source());
    let results = client.execute_plan(&plan, None).await?;

    info!("Calculation results: {}", results);

    info!("=== Advanced Mathematical Operations ===");

    // Advanced operations with error handling
    let recorder = Recorder::new();
    let calc = recorder.capture("calculator", CapId::new(2)); // Scientific calculator

    let sqrt_result = calc.call("sqrt", params![144]);
    let factorial_result = calc.call("factorial", params![5]);

    // This will demonstrate error handling
    let error_test = calc.call("divide", params![10, 0]);

    let advanced_plan = recorder.build(record_object!(recorder; {
        "sqrt_144" => sqrt_result,
        "factorial_5" => factorial_result,
        "error_test" => error_test,
    }).as_source());

    match client.execute_plan(&advanced_plan, None).await {
        Ok(results) => info!("Advanced results: {}", results),
        Err(e) => warn!("Expected error occurred: {}", e),
    }

    Ok(())
}

async fn run_user_management_examples(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== User Management Operations ===");

    let recorder = Recorder::new();
    let user_mgr = recorder.capture("userManager", CapId::new(100));

    // Fetch multiple users
    let user1 = user_mgr.call("getUser", params![1]);
    let user2 = user_mgr.call("getUser", params![2]);
    let user3 = user_mgr.call("getUser", params![3]);

    // Create a new user
    let new_user_data = json!({
        "name": "David Smith",
        "email": "david@example.com"
    });
    let new_user = user_mgr.call("createUser", params![new_user_data]);

    // Build comprehensive user plan
    let user_plan = recorder.build(record_object!(recorder; {
        "existing_users" => record_array!(recorder; [user1, user2, user3]),
        "new_user" => new_user,
    }).as_source());

    let user_results = client.execute_plan(&user_plan, None).await?;
    info!("User management results: {}", serde_json::to_string_pretty(&user_results)?);

    Ok(())
}

async fn run_complex_pipelining_example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Complex Promise Pipelining Example ===");

    let recorder = Recorder::new();
    let calc = recorder.capture("calculator", CapId::new(1));
    let user_mgr = recorder.capture("userManager", CapId::new(100));

    // Create a complex computation pipeline
    // Step 1: Get user data
    let user = user_mgr.call("getUser", params![1]);

    // Step 2: Perform calculations based on user ID
    // Note: In a real scenario, you'd extract the user ID from the user object
    // For this example, we'll use known values
    let base_calc = calc.call("add", params![10, 5]); // 15
    let advanced_calc = calc.call("multiply", base_calc.as_source(), params![2]); // 30
    let final_calc = calc.call("power", advanced_calc.as_source(), params![2]); // 900

    // Step 3: Create summary object
    let summary = record_object!(recorder; {
        "user_info" => user,
        "calculation_chain" => record_array!(recorder; [base_calc, advanced_calc, final_calc]),
        "final_result" => final_calc,
        "metadata" => record_object!(recorder; {
            "computed_at" => params![chrono::Utc::now().to_rfc3339()][0].clone(),
            "steps" => params![3][0].clone(),
        }),
    });

    let complex_plan = recorder.build(summary.as_source());
    let complex_results = client.execute_plan(&complex_plan, None).await?;

    info!("Complex pipelining results:");
    info!("{}", serde_json::to_string_pretty(&complex_results)?);

    Ok(())
}

async fn run_performance_test(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Performance Test ===");

    let start = std::time::Instant::now();
    let mut tasks = Vec::new();

    // Run multiple concurrent operations
    for i in 0..10 {
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            let recorder = Recorder::new();
            let calc = recorder.capture("calculator", CapId::new(1));

            let result = calc.call("add", params![i, i * 2]);
            let plan = recorder.build(result.as_source());

            client.execute_plan(&plan, None).await
        }));
    }

    let results = futures::future::join_all(tasks).await;
    let duration = start.elapsed();

    let successful = results.iter().filter(|r| r.is_ok()).count();
    info!("Performance test completed:");
    info!("  {} operations in {:?}", successful, duration);
    info!("  Average: {:?} per operation", duration / 10);

    Ok(())
}

async fn run_websocket_client() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== WebSocket Client Example ===");

    let transport = WebSocketTransport::connect("ws://localhost:8080/ws").await?;
    let config = ClientConfig::default();
    let client = Client::new(transport, config);

    run_calculator_examples(&client).await?;
    run_user_management_examples(&client).await?;
    run_complex_pipelining_example(&client).await?;
    run_performance_test(&client).await?;

    Ok(())
}

async fn run_http_client() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== HTTP Batch Client Example ===");

    let transport = HttpBatchTransport::new("http://localhost:8080/batch".to_string());
    let config = ClientConfig::default();
    let client = Client::new(transport, config);

    run_calculator_examples(&client).await?;
    run_user_management_examples(&client).await?;

    Ok(())
}

async fn demonstrate_error_recovery() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Error Recovery Example ===");

    // Try WebSocket first, fall back to HTTP if it fails
    match run_websocket_client().await {
        Ok(_) => info!("WebSocket client completed successfully"),
        Err(e) => {
            warn!("WebSocket client failed: {}, trying HTTP...", e);

            match run_http_client().await {
                Ok(_) => info!("HTTP client completed successfully"),
                Err(e) => error!("Both WebSocket and HTTP clients failed: {}", e),
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Cap'n Web Calculator Client Example");
    info!("Make sure the calculator server is running on localhost:8080");
    info!("");

    // Wait a moment for server to be ready
    sleep(Duration::from_millis(100)).await;

    // Demonstrate different transport types and error recovery
    demonstrate_error_recovery().await?;

    info!("");
    info!("Client examples completed!");
    info!("Check the server logs to see the capability calls being processed.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use capnweb_transport::MockTransport;

    #[tokio::test]
    async fn test_plan_construction() {
        let recorder = Recorder::new();
        let calc = recorder.capture("calculator", CapId::new(1));

        let sum = calc.call("add", params![5, 3]);
        let product = calc.call("multiply", params![2, 4]);

        let result = record_object!(recorder; {
            "sum" => sum,
            "product" => product,
        });

        let plan = recorder.build(result.as_source());

        // Verify plan structure
        assert_eq!(plan.captures.len(), 1);
        assert_eq!(plan.ops.len(), 3); // 2 calls + 1 object
    }

    #[tokio::test]
    async fn test_complex_pipelining() {
        let recorder = Recorder::new();
        let calc = recorder.capture("calculator", CapId::new(1));

        // Create a chain of dependent calculations
        let step1 = calc.call("add", params![10, 5]);
        let step2 = calc.call("multiply", step1.as_source(), params![2]);
        let step3 = calc.call("power", step2.as_source(), params![2]);

        let plan = recorder.build(step3.as_source());

        // Verify dependency chain
        assert_eq!(plan.ops.len(), 3);

        // Each operation should depend on the previous one
        if let capnweb_core::Op::Call { result, .. } = &plan.ops[0] {
            assert_eq!(*result, 0);
        }
        if let capnweb_core::Op::Call { result, .. } = &plan.ops[1] {
            assert_eq!(*result, 1);
        }
        if let capnweb_core::Op::Call { result, .. } = &plan.ops[2] {
            assert_eq!(*result, 2);
        }
    }
}