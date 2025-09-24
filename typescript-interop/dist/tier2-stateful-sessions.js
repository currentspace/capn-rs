#!/usr/bin/env node

// src/tier2-stateful-sessions.ts
import { newHttpBatchRpcSession } from "capnweb";
var port = process.argv[2] || "9000";
var endpoint = `http://localhost:${port}/rpc/batch`;
var Tier2Tests = class {
  constructor() {
    this.passed = 0;
    this.total = 0;
  }
  async test(name, testFn) {
    this.total++;
    console.log(`
\u{1F9EA} Test ${this.total}: ${name}`);
    console.log("\u2500".repeat(60));
    try {
      const result = await testFn();
      if (result) {
        this.passed++;
        console.log("\u2705 PASSED");
      } else {
        console.log("\u274C FAILED");
      }
    } catch (error) {
      console.log(`\u274C FAILED: ${error.message}`);
    }
  }
  async sessionPersistence() {
    console.log("Testing session persistence across multiple requests...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      const results = [];
      console.log("Making sequential requests...");
      results.push(await session.add(1, 2));
      results.push(await session.multiply(3, 4));
      results.push(await session.subtract(10, 5));
      console.log(`Results: ${results.join(", ")}`);
      const allNumbers = results.every((r) => typeof r === "number" && !isNaN(r));
      const correctValues = results[0] === 3 && results[1] === 12 && results[2] === 5;
      if (allNumbers && correctValues) {
        console.log("\u2713 All operations returned correct results");
        console.log("\u2713 Session maintained state across multiple requests");
        return true;
      } else if (allNumbers) {
        console.log("\u2713 Session persistent (wrong values may indicate calculation issues)");
        console.log(`  Expected: [3, 12, 5], Got: [${results.join(", ")}]`);
        return false;
      } else {
        console.log("\u2717 Inconsistent response types or session issues");
        return false;
      }
    } catch (error) {
      console.log(`Session persistence test failed: ${error.message}`);
      return false;
    }
  }
  async sessionIsolation() {
    console.log("Testing session isolation between different clients...");
    try {
      const session1 = newHttpBatchRpcSession(endpoint);
      const session2 = newHttpBatchRpcSession(endpoint);
      console.log("Creating two separate client sessions...");
      const [result1, result2] = await Promise.all([
        session1.add(5, 5),
        session2.multiply(6, 6)
      ]);
      console.log(`Session 1 result: ${result1}`);
      console.log(`Session 2 result: ${result2}`);
      if (typeof result1 === "number" && typeof result2 === "number") {
        if (result1 === 10 && result2 === 36) {
          console.log("\u2713 Both sessions returned correct results");
          console.log("\u2713 Sessions are properly isolated");
          return true;
        } else {
          console.log("\u2713 Sessions isolated but calculation errors");
          console.log(`  Expected: [10, 36], Got: [${result1}, ${result2}]`);
          return false;
        }
      } else {
        console.log("\u2717 One or both sessions failed to respond properly");
        return false;
      }
    } catch (error) {
      console.log(`Session isolation test failed: ${error.message}`);
      return false;
    }
  }
  async concurrentOperations() {
    console.log("Testing concurrent operations within a single session...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Launching concurrent operations...");
      const startTime = Date.now();
      const results = await Promise.all([
        session.add(2, 3),
        session.multiply(4, 5),
        session.divide(20, 4),
        session.subtract(15, 7)
      ]);
      const duration = Date.now() - startTime;
      console.log(`All operations completed in ${duration}ms`);
      console.log(`Results: ${results.join(", ")}`);
      const expected = [5, 20, 5, 8];
      const allCorrect = results.every((r, i) => r === expected[i]);
      if (allCorrect) {
        console.log("\u2713 All concurrent operations returned correct results");
        console.log("\u2713 Server handled concurrent requests properly");
        if (duration < 1e3) {
          console.log("\u2713 Operations appear to be processed concurrently");
        }
        return true;
      } else {
        console.log("\u2713 Concurrent operations completed but with incorrect results");
        console.log(`  Expected: [${expected.join(", ")}], Got: [${results.join(", ")}]`);
        return false;
      }
    } catch (error) {
      console.log(`Concurrent operations test failed: ${error.message}`);
      return false;
    }
  }
  async errorRecovery() {
    console.log("Testing error recovery and session stability...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Performing initial successful operation...");
      const initial = await session.add(1, 1);
      console.log(`Initial result: ${initial}`);
      if (typeof initial !== "number" || initial !== 2) {
        console.log("\u2717 Initial operation failed - cannot test error recovery");
        return false;
      }
      console.log("Triggering an error (division by zero)...");
      let errorOccurred = false;
      try {
        await session.divide(5, 0);
        console.log("\u2139\uFE0F  Division by zero did not throw error (unexpected)");
      } catch (error) {
        console.log(`\u2713 Error properly thrown: ${error.message}`);
        errorOccurred = true;
      }
      console.log("Testing session recovery with another operation...");
      const recovery = await session.multiply(3, 4);
      console.log(`Recovery result: ${recovery}`);
      if (typeof recovery === "number" && recovery === 12) {
        console.log("\u2713 Session recovered after error");
        console.log("\u2713 Error handling did not corrupt session state");
        return true;
      } else {
        console.log("\u2717 Session corrupted after error");
        return false;
      }
    } catch (error) {
      console.log(`Error recovery test failed: ${error.message}`);
      return false;
    }
  }
  async importExportLifecycle() {
    console.log("Testing import/export lifecycle management...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Testing multiple operations to check import/export handling...");
      const operations = [
        { op: "add", args: [1, 2], expected: 3 },
        { op: "multiply", args: [2, 3], expected: 6 },
        { op: "subtract", args: [10, 4], expected: 6 },
        { op: "divide", args: [15, 3], expected: 5 }
      ];
      const results = [];
      for (const { op, args, expected } of operations) {
        console.log(`  ${op}(${args.join(", ")}) = ?`);
        const result = await session[op](...args);
        results.push(result);
        console.log(`    -> ${result} (expected ${expected})`);
        if (typeof result !== "number") {
          console.log("\u2717 Non-numeric result indicates import/export issues");
          return false;
        }
      }
      const allCompleted = results.length === operations.length;
      const allNumbers = results.every((r) => typeof r === "number");
      if (allCompleted && allNumbers) {
        console.log("\u2713 All operations completed with numeric results");
        console.log("\u2713 Import/export lifecycle appears functional");
        const allCorrect = operations.every((op, i) => results[i] === op.expected);
        if (allCorrect) {
          console.log("\u2713 All calculations correct");
          return true;
        } else {
          console.log("\u2139\uFE0F  Import/export working but calculation errors present");
          return false;
        }
      } else {
        console.log("\u2717 Import/export lifecycle has issues");
        return false;
      }
    } catch (error) {
      console.log(`Import/export lifecycle test failed: ${error.message}`);
      return false;
    }
  }
  async run() {
    console.log("\u{1F3C1} TIER 2: Stateful Session Management Tests");
    console.log("============================================");
    console.log(`\u{1F4CD} Testing endpoint: ${endpoint}`);
    console.log("\u{1F3AF} Goal: Verify session persistence and state tracking");
    console.log("\u{1F4CB} Prerequisites: Tier 1 tests must pass");
    console.log("");
    await this.test("Session Persistence Across Requests", () => this.sessionPersistence());
    await this.test("Session Isolation Between Clients", () => this.sessionIsolation());
    await this.test("Concurrent Operations Within Session", () => this.concurrentOperations());
    await this.test("Error Recovery and Session Stability", () => this.errorRecovery());
    await this.test("Import/Export Lifecycle Management", () => this.importExportLifecycle());
    console.log("\n" + "=".repeat(60));
    console.log("\u{1F3C1} TIER 2 RESULTS");
    console.log("=".repeat(60));
    const passRate = Math.round(this.passed / this.total * 100);
    console.log(`\u2705 Passed: ${this.passed}/${this.total} (${passRate}%)`);
    if (this.passed === this.total) {
      console.log("\u{1F389} TIER 2 COMPLETE: Stateful session management working!");
      console.log("\u{1F4C8} Ready for Tier 3: Complex Application Logic");
      process.exit(0);
    } else if (this.passed >= this.total * 0.6) {
      console.log("\u26A0\uFE0F  TIER 2 PARTIAL: Some session management issues remain");
      console.log("\u{1F527} Address session state issues before Tier 3");
      process.exit(1);
    } else {
      console.log("\u{1F4A5} TIER 2 FAILED: Session management not working");
      console.log("\u{1F6A8} Fix state tracking before proceeding");
      process.exit(2);
    }
  }
};
process.on("unhandledRejection", (reason, promise) => {
  console.error("Unhandled Rejection at:", promise, "reason:", reason);
  process.exit(3);
});
var tier2 = new Tier2Tests();
tier2.run();
//# sourceMappingURL=tier2-stateful-sessions.js.map