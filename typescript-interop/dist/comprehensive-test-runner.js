#!/usr/bin/env node

// src/comprehensive-test-runner.ts
import { spawn } from "child_process";
var testTiers = [
  {
    name: "TIER 1: Protocol Compliance",
    script: "./dist/tier1-protocol-compliance.js",
    port: 9e3,
    critical: true,
    transport: "http"
  },
  {
    name: "TIER 2: HTTP Batch (Corrected)",
    script: "./dist/tier2-http-batch-corrected.js",
    port: 9e3,
    critical: true,
    transport: "http"
  },
  {
    name: "TIER 2: WebSocket Sessions",
    script: "./dist/tier2-websocket-tests.js",
    port: 9e3,
    critical: false,
    // WebSocket might not be implemented yet
    transport: "websocket"
  },
  {
    name: "TIER 3: Capability Composition",
    script: "./dist/tier3-capability-composition.js",
    port: 9e3,
    critical: false,
    transport: "http"
  },
  {
    name: "TIER 3: Complex Applications",
    script: "./dist/tier3-complex-applications.js",
    port: 9e3,
    critical: false,
    transport: "http"
  }
];
var ComprehensiveTestRunner = class {
  constructor() {
    this.totalTests = 0;
    this.passedTests = 0;
    this.failedTests = 0;
    this.results = /* @__PURE__ */ new Map();
  }
  async runTest(tier) {
    return new Promise((resolve) => {
      console.log("\n" + "=".repeat(60));
      console.log(`\u{1F680} Running ${tier.name}`);
      console.log(`\u{1F4CD} Port: ${tier.port}, Transport: ${tier.transport || "default"}`);
      console.log("=".repeat(60));
      const child = spawn("node", [tier.script, String(tier.port)], {
        cwd: process.cwd(),
        stdio: "inherit",
        env: { ...process.env }
      });
      child.on("exit", (code) => {
        const success = code === 0;
        if (success) {
          console.log(`
\u2705 ${tier.name}: PASSED`);
        } else if (code === 1 && !tier.critical) {
          console.log(`
\u26A0\uFE0F  ${tier.name}: PARTIAL PASS (non-critical)`);
        } else {
          console.log(`
\u274C ${tier.name}: FAILED with exit code ${code}`);
        }
        this.results.set(tier.name, {
          passed: success ? 1 : 0,
          failed: success ? 0 : 1,
          exitCode: code || 0
        });
        resolve(success || !tier.critical);
      });
      child.on("error", (err) => {
        console.error(`
\u{1F4A5} Failed to run ${tier.name}:`, err);
        this.results.set(tier.name, {
          passed: 0,
          failed: 1,
          exitCode: -1
        });
        resolve(!tier.critical);
      });
    });
  }
  async runAllTests() {
    console.log("\u{1F3C1} CAP'N WEB RUST IMPLEMENTATION - COMPREHENSIVE TEST SUITE");
    console.log("============================================================");
    console.log("\u{1F4CB} Protocol Compliance Testing with TypeScript Reference Client");
    console.log("\u{1F3AF} Testing official Cap'n Web wire protocol (newline-delimited)");
    console.log("");
    let shouldContinue = true;
    for (const tier of testTiers) {
      if (!shouldContinue) {
        console.log(`
\u23E9 Skipping ${tier.name} due to critical failure`);
        this.results.set(tier.name, {
          passed: 0,
          failed: 0,
          exitCode: -2
          // Skipped
        });
        continue;
      }
      const success = await this.runTest(tier);
      if (!success && tier.critical) {
        shouldContinue = false;
        console.log("\n\u{1F6D1} Critical test failed - stopping test execution");
      }
    }
    this.printSummary();
  }
  printSummary() {
    console.log("\n" + "=".repeat(60));
    console.log("\u{1F4CA} COMPREHENSIVE TEST RESULTS SUMMARY");
    console.log("=".repeat(60));
    let totalPassed = 0;
    let totalFailed = 0;
    let skipped = 0;
    console.log("\n\u{1F4CB} Individual Tier Results:");
    console.log("-".repeat(60));
    for (const [name, result] of this.results) {
      const icon = result.exitCode === 0 ? "\u2705" : result.exitCode === -2 ? "\u23E9" : result.exitCode === 1 ? "\u26A0\uFE0F" : "\u274C";
      const status = result.exitCode === 0 ? "PASSED" : result.exitCode === -2 ? "SKIPPED" : result.exitCode === 1 ? "PARTIAL" : "FAILED";
      console.log(`${icon} ${name.padEnd(40)} ${status}`);
      if (result.exitCode === -2) {
        skipped++;
      } else {
        totalPassed += result.passed;
        totalFailed += result.failed;
      }
    }
    const completionRate = totalPassed + totalFailed > 0 ? (totalPassed / (totalPassed + totalFailed) * 100).toFixed(1) : "0.0";
    console.log("\n\u{1F4C8} Overall Statistics:");
    console.log("-".repeat(60));
    console.log(`   Tests Run: ${totalPassed + totalFailed}`);
    console.log(`   Passed: ${totalPassed} \u2705`);
    console.log(`   Failed: ${totalFailed} \u274C`);
    console.log(`   Skipped: ${skipped} \u23E9`);
    console.log(`   Success Rate: ${completionRate}%`);
    console.log("\n\u{1F3AF} Protocol Compliance Status:");
    console.log("-".repeat(60));
    const tier1Result = this.results.get("TIER 1: Protocol Compliance");
    const tier2HttpResult = this.results.get("TIER 2: HTTP Batch (Corrected)");
    const tier2WsResult = this.results.get("TIER 2: WebSocket Sessions");
    if (tier1Result?.exitCode === 0) {
      console.log("\u2705 Basic Wire Protocol: COMPLIANT");
    } else {
      console.log("\u274C Basic Wire Protocol: NON-COMPLIANT");
    }
    if (tier2HttpResult?.exitCode === 0) {
      console.log("\u2705 HTTP Batch Transport: COMPLIANT");
    } else {
      console.log("\u26A0\uFE0F  HTTP Batch Transport: PARTIAL/NON-COMPLIANT");
    }
    if (tier2WsResult?.exitCode === 0) {
      console.log("\u2705 WebSocket Transport: COMPLIANT");
    } else if (tier2WsResult?.exitCode === -2) {
      console.log("\u23E9 WebSocket Transport: NOT TESTED");
    } else {
      console.log("\u26A0\uFE0F  WebSocket Transport: NOT IMPLEMENTED/NON-COMPLIANT");
    }
    console.log("\n" + "=".repeat(60));
    const allCriticalPassed = tier1Result?.exitCode === 0 && tier2HttpResult?.exitCode === 0;
    if (allCriticalPassed) {
      console.log("\u{1F389} IMPLEMENTATION STATUS: PROTOCOL COMPLIANT");
      console.log("=".repeat(60));
      console.log("\u2705 The Rust server correctly implements the Cap'n Web protocol");
      console.log("\u2705 Compatible with official TypeScript reference client");
      console.log("\u2705 HTTP batch transport working correctly");
      if (tier2WsResult?.exitCode === 0) {
        console.log("\u2705 WebSocket transport also working");
      }
      process.exit(0);
    } else {
      console.log("\u274C IMPLEMENTATION STATUS: NON-COMPLIANT");
      console.log("=".repeat(60));
      console.log("\u26A0\uFE0F  Critical protocol compliance issues detected");
      console.log("\u{1F527} Review failed tests and fix protocol implementation");
      process.exit(1);
    }
  }
};
process.on("SIGINT", () => {
  console.log("\n\n\u26A0\uFE0F  Test suite interrupted by user");
  process.exit(130);
});
if (import.meta.url === `file://${process.argv[1]}`) {
  const runner = new ComprehensiveTestRunner();
  runner.runAllTests().catch((error) => {
    console.error("\n\u{1F4A5} Fatal error in test runner:", error);
    process.exit(1);
  });
}
export {
  ComprehensiveTestRunner
};
//# sourceMappingURL=comprehensive-test-runner.js.map