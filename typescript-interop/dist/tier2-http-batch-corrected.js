#!/usr/bin/env node

// src/tier2-http-batch-corrected.ts
import { newHttpBatchRpcSession } from "capnweb";
var port = process.argv[2] || "9000";
var endpoint = `http://localhost:${port}/rpc/batch`;
var Tier2BatchTests = class {
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
  async batchOperations() {
    console.log("Testing batch operations (all in single request)...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Sending all operations in a single batch...");
      const results = await Promise.all([
        session.add(1, 2),
        session.multiply(3, 4),
        session.subtract(10, 5)
      ]);
      console.log(`Results: ${results.join(", ")}`);
      const expected = [3, 12, 5];
      const allCorrect = results.every((r, i) => r === expected[i]);
      if (allCorrect) {
        console.log("\u2713 All batch operations returned correct results");
        console.log("\u2713 Batch processing working correctly");
        return true;
      } else {
        console.log("\u2717 Batch operations returned incorrect results");
        console.log(`  Expected: [${expected.join(", ")}], Got: [${results.join(", ")}]`);
        return false;
      }
    } catch (error) {
      console.log(`Batch operations test failed: ${error.message}`);
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
      if (result1 === 10 && result2 === 36) {
        console.log("\u2713 Both sessions returned correct results");
        console.log("\u2713 Sessions are properly isolated");
        return true;
      } else {
        console.log("\u2717 Incorrect results from isolated sessions");
        console.log(`  Expected: [10, 36], Got: [${result1}, ${result2}]`);
        return false;
      }
    } catch (error) {
      console.log(`Session isolation test failed: ${error.message}`);
      return false;
    }
  }
  async concurrentBatchOperations() {
    console.log("Testing concurrent operations within a single batch...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Launching concurrent operations in single batch...");
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
        console.log("\u2713 Server handled batch request properly");
        return true;
      } else {
        console.log("\u2717 Batch operations returned incorrect results");
        console.log(`  Expected: [${expected.join(", ")}], Got: [${results.join(", ")}]`);
        return false;
      }
    } catch (error) {
      console.log(`Concurrent batch operations test failed: ${error.message}`);
      return false;
    }
  }
  async errorHandlingInBatch() {
    console.log("Testing error handling within batch...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Sending batch with valid and error-triggering operations...");
      const results = await Promise.allSettled([
        session.add(1, 1),
        session.divide(5, 0),
        // This should error
        session.multiply(3, 4)
      ]);
      console.log("Batch completed, analyzing results...");
      const successCount = results.filter((r) => r.status === "fulfilled").length;
      const errorCount = results.filter((r) => r.status === "rejected").length;
      console.log(`Success: ${successCount}, Errors: ${errorCount}`);
      if (results[0].status === "fulfilled" && results[0].value === 2) {
        console.log("\u2713 First operation succeeded as expected");
      }
      if (results[1].status === "rejected") {
        console.log("\u2713 Division by zero properly rejected");
      }
      if (results[2].status === "fulfilled" && results[2].value === 12) {
        console.log("\u2713 Third operation succeeded despite error in batch");
      }
      if (successCount === 2 && errorCount === 1) {
        console.log("\u2713 Batch properly handled mixed success/error cases");
        return true;
      } else {
        console.log("\u2717 Unexpected batch error handling behavior");
        return false;
      }
    } catch (error) {
      console.log(`Error handling test failed: ${error.message}`);
      return false;
    }
  }
  async multipleBatchRequests() {
    console.log("Testing multiple batch requests (new sessions)...");
    try {
      console.log("Creating new session for each batch...");
      const batch1 = newHttpBatchRpcSession(endpoint);
      const result1 = await Promise.all([
        batch1.add(1, 2),
        batch1.multiply(2, 3)
      ]);
      console.log(`Batch 1 results: ${result1.join(", ")}`);
      const batch2 = newHttpBatchRpcSession(endpoint);
      const result2 = await Promise.all([
        batch2.subtract(10, 4),
        batch2.divide(15, 3)
      ]);
      console.log(`Batch 2 results: ${result2.join(", ")}`);
      const batch3 = newHttpBatchRpcSession(endpoint);
      const result3 = await batch3.add(100, 200);
      console.log(`Batch 3 result: ${result3}`);
      const allCorrect = result1[0] === 3 && result1[1] === 6 && result2[0] === 6 && result2[1] === 5 && result3 === 300;
      if (allCorrect) {
        console.log("\u2713 All batches processed correctly");
        console.log("\u2713 Multiple sequential batches work with new sessions");
        return true;
      } else {
        console.log("\u2717 Some batch results incorrect");
        return false;
      }
    } catch (error) {
      console.log(`Multiple batch requests test failed: ${error.message}`);
      return false;
    }
  }
  async run() {
    console.log("\u{1F3C1} TIER 2: HTTP Batch Transport Tests (Corrected)");
    console.log("==================================================");
    console.log(`\u{1F4CD} Testing endpoint: ${endpoint}`);
    console.log("\u{1F3AF} Goal: Verify proper HTTP batch semantics");
    console.log("\u26A0\uFE0F  Note: HTTP batch sessions END after sending");
    console.log("");
    await this.test("Batch Operations", () => this.batchOperations());
    await this.test("Session Isolation", () => this.sessionIsolation());
    await this.test("Concurrent Batch Operations", () => this.concurrentBatchOperations());
    await this.test("Error Handling in Batch", () => this.errorHandlingInBatch());
    await this.test("Multiple Batch Requests", () => this.multipleBatchRequests());
    console.log("\n" + "=".repeat(60));
    console.log("\u{1F3C1} TIER 2 (HTTP BATCH) RESULTS");
    console.log("=".repeat(60));
    const passRate = Math.round(this.passed / this.total * 100);
    console.log(`\u2705 Passed: ${this.passed}/${this.total} (${passRate}%)`);
    if (this.passed === this.total) {
      console.log("\u{1F389} TIER 2 COMPLETE: HTTP batch semantics working correctly!");
      console.log("\u{1F4C8} For persistent sessions, use WebSocket transport");
      process.exit(0);
    } else if (this.passed >= this.total * 0.6) {
      console.log("\u26A0\uFE0F  TIER 2 PARTIAL: Some batch handling issues");
      process.exit(1);
    } else {
      console.log("\u{1F4A5} TIER 2 FAILED: Batch handling not working");
      process.exit(2);
    }
  }
};
process.on("unhandledRejection", (reason, promise) => {
  console.error("Unhandled Rejection at:", promise, "reason:", reason);
  process.exit(3);
});
var tier2 = new Tier2BatchTests();
tier2.run();
//# sourceMappingURL=tier2-http-batch-corrected.js.map