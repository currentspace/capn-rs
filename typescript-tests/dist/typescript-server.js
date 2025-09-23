import { CapnWebServer, MockCalculator, MockUserManager } from './capnweb/server.js';

var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });
async function main() {
  console.log("\u{1F525} Starting TypeScript Cap'n Web Server for Interop Testing");
  console.log("=".repeat(60));
  const server = new CapnWebServer({
    port: 8081,
    host: "localhost",
    path: "/ws"
  });
  server.registerCapability(1, new MockCalculator());
  server.registerCapability(2, new MockCalculator());
  server.registerCapability(100, new MockUserManager());
  console.log("\u{1F4CB} Registered capabilities:");
  console.log("   \u2022 Calculator (ID: 1) - Basic arithmetic operations");
  console.log("   \u2022 Scientific Calculator (ID: 2) - Advanced math functions");
  console.log("   \u2022 User Manager (ID: 100) - User management operations");
  console.log();
  try {
    await server.start();
    console.log("\u2705 TypeScript server started successfully!");
    console.log("\u{1F310} Server Details:");
    console.log("   \u2022 Host: localhost");
    console.log("   \u2022 Port: 8081");
    console.log("   \u2022 WebSocket Path: /ws");
    console.log("   \u2022 Full URL: ws://localhost:8081/ws");
    console.log();
    console.log("\u{1F9EA} Available Test Capabilities:");
    console.log();
    console.log("Calculator (ID: 1, 2):");
    console.log("   \u2022 add(a, b) \u2192 a + b");
    console.log("   \u2022 subtract(a, b) \u2192 a - b");
    console.log("   \u2022 multiply(a, b) \u2192 a * b");
    console.log("   \u2022 divide(a, b) \u2192 a / b (throws on division by zero)");
    console.log("   \u2022 power(base, exp) \u2192 base^exp");
    console.log("   \u2022 sqrt(n) \u2192 \u221An (throws on negative numbers)");
    console.log("   \u2022 factorial(n) \u2192 n! (throws on negative, max 20)");
    console.log();
    console.log("User Manager (ID: 100):");
    console.log("   \u2022 getUser(id) \u2192 User object");
    console.log("   \u2022 createUser(userData) \u2192 Created user object");
    console.log();
    console.log("\u{1F50C} Ready for Rust client connections!");
    console.log("\u{1F4A1} Test with Rust client:");
    console.log("   cd .. && cargo run --example calculator_client --features typescript-server");
    console.log();
    console.log("Press Ctrl+C to stop the server...");
    await new Promise((resolve) => {
      process.on("SIGINT", () => {
        console.log("\n\u{1F6D1} Received SIGINT, shutting down server...");
        resolve(void 0);
      });
      process.on("SIGTERM", () => {
        console.log("\n\u{1F6D1} Received SIGTERM, shutting down server...");
        resolve(void 0);
      });
    });
  } catch (error) {
    console.error("\u{1F4A5} Failed to start server:", error);
    process.exit(1);
  } finally {
    console.log("\u{1F504} Stopping TypeScript server...");
    await server.stop();
    console.log("\u2705 Server stopped successfully");
    process.exit(0);
  }
}
__name(main, "main");
main().catch((error) => {
  console.error("\u{1F4A5} Unhandled error:", error);
  process.exit(1);
});
//# sourceMappingURL=typescript-server.js.map
//# sourceMappingURL=typescript-server.js.map