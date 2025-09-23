import { runClientTests } from './tests/client-tests.js';
import { runServerTests } from './tests/server-tests.js';
import { wait } from './tests/test-framework.js';

var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });
async function parseArgs() {
  const args = process.argv.slice(2);
  const config = {
    runClientTests: true,
    runServerTests: true,
    waitForRustServer: 3e3,
    verbose: false
  };
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    switch (arg) {
      case "--client-only":
        config.runClientTests = true;
        config.runServerTests = false;
        break;
      case "--server-only":
        config.runClientTests = false;
        config.runServerTests = true;
        break;
      case "--wait":
        config.waitForRustServer = parseInt(args[++i] || "3000", 10);
        break;
      case "--verbose":
        config.verbose = true;
        break;
      case "--help":
        console.log(`
Cap'n Web TypeScript Interoperability Test Suite

Usage: node dist/index.js [options]

Options:
  --client-only     Run only TypeScript client \u2192 Rust server tests
  --server-only     Run only TypeScript server \u2190 Rust client tests
  --wait <ms>       Wait time for Rust server startup (default: 3000ms)
  --verbose         Enable verbose logging
  --help           Show this help message

Examples:
  node dist/index.js                    # Run all tests
  node dist/index.js --client-only      # Test TS client \u2192 Rust server
  node dist/index.js --server-only      # Test TS server \u2190 Rust client
  node dist/index.js --wait 5000        # Wait 5 seconds for Rust server
`);
        process.exit(0);
        break;
      default:
        console.warn(`Unknown argument: ${arg}`);
        break;
    }
  }
  return config;
}
__name(parseArgs, "parseArgs");
async function checkRustServerAvailability() {
  try {
    const response = await fetch("http://localhost:8080/health");
    return response.ok;
  } catch {
    try {
      const ws = new (await import('ws')).default("ws://localhost:8080/ws");
      return new Promise((resolve) => {
        const timeout = setTimeout(() => {
          ws.close();
          resolve(false);
        }, 2e3);
        ws.on("open", () => {
          clearTimeout(timeout);
          ws.close();
          resolve(true);
        });
        ws.on("error", () => {
          clearTimeout(timeout);
          resolve(false);
        });
      });
    } catch {
      return false;
    }
  }
}
__name(checkRustServerAvailability, "checkRustServerAvailability");
async function main() {
  console.log("\u{1F31F} Cap'n Web TypeScript \u2194 Rust Interoperability Test Suite");
  console.log("=".repeat(70));
  console.log();
  const config = await parseArgs();
  if (config.verbose) {
    console.log("\u{1F4CB} Test Configuration:");
    console.log(`   Client Tests: ${config.runClientTests ? "\u2705" : "\u274C"}`);
    console.log(`   Server Tests: ${config.runServerTests ? "\u2705" : "\u274C"}`);
    console.log(`   Rust Server Wait: ${config.waitForRustServer}ms`);
    console.log();
  }
  let totalDuration = 0;
  const overallStart = Date.now();
  try {
    if (config.runClientTests) {
      console.log("\u{1F680} PHASE 1: TypeScript Client \u2192 Rust Server Tests");
      console.log("-".repeat(50));
      console.log("\u{1F50D} Checking for Rust server availability...");
      let serverAvailable = await checkRustServerAvailability();
      if (!serverAvailable) {
        console.log(`\u23F3 Rust server not ready, waiting ${config.waitForRustServer}ms...`);
        console.log("\u{1F4A1} Make sure to start the Rust server first:");
        console.log("   cd .. && cargo run --example calculator_server");
        console.log();
        await wait(config.waitForRustServer);
        serverAvailable = await checkRustServerAvailability();
      }
      if (serverAvailable) {
        console.log("\u2705 Rust server is available, proceeding with client tests...");
        const clientStart = Date.now();
        await runClientTests();
        const clientDuration = Date.now() - clientStart;
        console.log(`\u23F1\uFE0F  Client tests completed in ${clientDuration}ms`);
        totalDuration += clientDuration;
      } else {
        console.error("\u274C Rust server is not available. Skipping client tests.");
        console.error("   Start the Rust server with: cargo run --example calculator_server");
      }
      console.log();
    }
    if (config.runServerTests) {
      console.log("\u{1F3AF} PHASE 2: TypeScript Server \u2190 Rust Client Tests");
      console.log("-".repeat(50));
      const serverStart = Date.now();
      await runServerTests();
      const serverDuration = Date.now() - serverStart;
      console.log(`\u23F1\uFE0F  Server tests completed in ${serverDuration}ms`);
      totalDuration += serverDuration;
      console.log();
    }
  } catch (error) {
    console.error("\u{1F4A5} Fatal error during test execution:");
    console.error(error);
    process.exit(1);
  }
  const overallDuration = Date.now() - overallStart;
  console.log("\u{1F3C1} FINAL INTEROPERABILITY REPORT");
  console.log("=".repeat(70));
  console.log(`\u23F1\uFE0F  Total Test Duration: ${overallDuration}ms`);
  {
    console.log("\u26A0\uFE0F  No tests were executed");
    console.log("   Check server availability and configuration");
  }
  console.log();
  console.log("\u{1F4DA} For more information:");
  console.log("   \u2022 Cap'n Web Specification: https://capnproto.org/capnweb");
  console.log("   \u2022 Rust Implementation: ../README.md");
  console.log("   \u2022 TypeScript Implementation: ./README.md");
  console.log();
}
__name(main, "main");
process.on("SIGINT", () => {
  console.log("\n\u{1F6D1} Test suite interrupted by user");
  process.exit(130);
});
process.on("SIGTERM", () => {
  console.log("\n\u{1F6D1} Test suite terminated");
  process.exit(143);
});
main().catch((error) => {
  console.error("\u{1F4A5} Unhandled error in main:", error);
  process.exit(1);
});
//# sourceMappingURL=index.js.map
//# sourceMappingURL=index.js.map