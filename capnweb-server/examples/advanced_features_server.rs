// Advanced Features Server
// Exposes Resume Tokens, Nested Capabilities, and IL Plan Runner via RPC

use capnweb_server::{
    AdvancedCapability, AdvancedCapabilityBuilder, CapnWebServerConfig, NewCapnWebServer,
};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 Starting Cap'n Web Advanced Features Server");
    info!("==================================================");
    info!("");
    info!("This server exposes all advanced protocol features:");
    info!("  ✅ Resume Tokens (Session persistence & recovery)");
    info!("  ✅ Nested Capabilities (Dynamic capability creation)");
    info!("  ✅ IL Plan Runner (Complex instruction execution)");
    info!("");

    // Create server configuration
    let config = CapnWebServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        max_batch_size: 100,
    };

    // Create the advanced capability using the builder pattern
    let advanced_cap = Arc::new(
        AdvancedCapabilityBuilder::new()
            .with_token_ttl(7200) // 2 hour token TTL
            .with_max_session_age(86400) // 24 hour max session age
            .with_max_capabilities(500) // Support up to 500 nested capabilities
            .with_max_plan_operations(2000) // Allow complex plans with many ops
            .with_plan_timeout(60000) // 60 second timeout for plan execution
            .build(),
    );

    // Create and configure server
    let mut server = NewCapnWebServer::new(config);
    server.register_main(advanced_cap.clone());

    // Also register it as capability ID 1 for additional access patterns
    server.register_capability(1, advanced_cap.clone());

    info!("📡 Server Configuration:");
    info!("  - HTTP Batch endpoint: http://127.0.0.1:8080/rpc/batch");
    info!("  - WebSocket endpoint: ws://127.0.0.1:8080/rpc/ws");
    info!("  - Max batch size: 100");
    info!("");
    info!("🔧 Available RPC Methods:");
    info!("");
    info!("  Resume Tokens:");
    info!("    • createResumeToken(config) - Create session snapshot");
    info!("    • restoreSession(config) - Restore from token");
    info!("    • setVariable(name, value) - Store session variable");
    info!("    • getVariable(name) - Retrieve session variable");
    info!("");
    info!("  Nested Capabilities:");
    info!("    • createSubCapability(type, config) - Create nested capability");
    info!("    • callSubCapability(name, method, ...args) - Call nested capability");
    info!("    • disposeSubCapability(name) - Dispose nested capability");
    info!("    • listSubCapabilities() - List all nested capabilities");
    info!("");
    info!("  IL Plan Runner:");
    info!("    • executePlan(plan, parameters, captures) - Execute IL plan");
    info!("    • createPlan(name, operations) - Create and cache plan");
    info!("    • executeCachedPlan(name, parameters) - Execute cached plan");
    info!("");
    info!("  Basic Operations (for compatibility):");
    info!("    • add(a, b) - Add two numbers");
    info!("    • multiply(a, b) - Multiply two numbers");
    info!("    • getStats() - Get server statistics");
    info!("");

    // Build the server router and application
    let router = server.build_router();
    let addr = format!("{}:{}", server.config().host, server.config().port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("✨ Server is ready and listening on {}", addr);
    info!("");
    info!("📝 Example TypeScript client usage:");
    info!("");
    info!("  const client = new Client({{");
    info!("    endpoint: 'http://127.0.0.1:8080/rpc/batch',");
    info!("    transport: 'http-batch'");
    info!("  }});");
    info!("");
    info!("  const cap = await client.import(0);");
    info!("");
    info!("  // Resume Tokens");
    info!("  const token = await cap.createResumeToken({{");
    info!("    sessionId: 'test123',");
    info!("    includeState: true,");
    info!("    expirationMinutes: 60");
    info!("  }});");
    info!("");
    info!("  // Nested Capabilities");
    info!("  const subCap = await cap.createSubCapability('validator', {{");
    info!("    maxLength: 100");
    info!("  }});");
    info!("");
    info!("  // IL Plan Runner");
    info!("  const plan = {{");
    info!("    operations: [{{");
    info!("      type: 'return',");
    info!("      value: {{ type: 'literal', value: 'Hello!' }}");
    info!("    }}]");
    info!("  }};");
    info!("  const result = await cap.executePlan(plan, {{}});");
    info!("");
    info!("==================================================");
    info!("Server running... Press Ctrl+C to stop");

    // Run the server
    axum::serve(listener, router).await?;

    Ok(())
}
