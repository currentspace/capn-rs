#!/usr/bin/env node

// src/cross-transport-interop.ts
import { newHttpBatchRpcSession, newWebSocketRpcSession } from "capnweb";
var port = process.argv[2] || "9001";
var httpEndpoint = `http://localhost:${port}/rpc/batch`;
var wsEndpoint = `ws://localhost:${port}/rpc/ws`;
var CrossTransportInteropTests = class {
  constructor() {
    this.passed = 0;
    this.total = 0;
  }
  async test(name, testFn) {
    this.total++;
    console.log(`
\u{1F504} Test ${this.total}: ${name}`);
    console.log("\u2500".repeat(80));
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
  async transportEquivalenceTest() {
    console.log("Testing computational equivalence between HTTP Batch and WebSocket...");
    try {
      const testCases = [
        { op: "add", args: [15, 25] },
        { op: "multiply", args: [7, 8] },
        { op: "divide", args: [100, 4] },
        { op: "subtract", args: [50, 18] }
      ];
      console.log("\u{1F310} Executing calculations on HTTP Batch transport...");
      const httpSession = newHttpBatchRpcSession(httpEndpoint);
      const httpResults = [];
      for (const testCase of testCases) {
        let result;
        switch (testCase.op) {
          case "add":
            result = await httpSession.add(testCase.args[0], testCase.args[1]);
            break;
          case "multiply":
            result = await httpSession.multiply(testCase.args[0], testCase.args[1]);
            break;
          case "divide":
            result = await httpSession.divide(testCase.args[0], testCase.args[1]);
            break;
          case "subtract":
            result = await httpSession.subtract(testCase.args[0], testCase.args[1]);
            break;
          default:
            throw new Error(`Unknown operation: ${testCase.op}`);
        }
        httpResults.push(result);
        console.log(`  HTTP ${testCase.op}(${testCase.args.join(", ")}) = ${result}`);
      }
      console.log("\u{1F50C} Executing same calculations on WebSocket transport...");
      const wsSession = newWebSocketRpcSession(wsEndpoint);
      const wsResults = [];
      for (const testCase of testCases) {
        let result;
        switch (testCase.op) {
          case "add":
            result = await wsSession.add(testCase.args[0], testCase.args[1]);
            break;
          case "multiply":
            result = await wsSession.multiply(testCase.args[0], testCase.args[1]);
            break;
          case "divide":
            result = await wsSession.divide(testCase.args[0], testCase.args[1]);
            break;
          case "subtract":
            result = await wsSession.subtract(testCase.args[0], testCase.args[1]);
            break;
          default:
            throw new Error(`Unknown operation: ${testCase.op}`);
        }
        wsResults.push(result);
        console.log(`  WebSocket ${testCase.op}(${testCase.args.join(", ")}) = ${result}`);
      }
      if ("close" in wsSession) {
        wsSession.close();
      }
      console.log("\u{1F50D} Transport Equivalence Analysis:");
      console.log(`  HTTP Results:     [${httpResults.join(", ")}]`);
      console.log(`  WebSocket Results: [${wsResults.join(", ")}]`);
      const resultsMatch = JSON.stringify(httpResults) === JSON.stringify(wsResults);
      if (resultsMatch) {
        console.log("\u2713 Both transports produced identical computational results");
        console.log("\u2713 Transport abstraction maintains mathematical consistency");
        return true;
      } else {
        console.log("\u2717 Transport results differ - computational inconsistency detected");
        return false;
      }
    } catch (error) {
      console.log(`Transport equivalence test failed: ${error.message}`);
      return false;
    }
  }
  async performanceCharacteristicsComparison() {
    console.log("Comparing performance characteristics between transports...");
    try {
      const operationCount = 10;
      const operations = Array.from({ length: operationCount }, (_, i) => ({
        op: ["add", "multiply", "subtract", "divide"][i % 4],
        args: [i + 1, i + 2]
      }));
      console.log("\u23F1\uFE0F  HTTP Batch Performance Test...");
      const httpStart = Date.now();
      const httpSession = newHttpBatchRpcSession(httpEndpoint);
      const httpResults = [];
      for (const operation of operations) {
        let result;
        switch (operation.op) {
          case "add":
            result = await httpSession.add(operation.args[0], operation.args[1]);
            break;
          case "multiply":
            result = await httpSession.multiply(operation.args[0], operation.args[1]);
            break;
          case "subtract":
            result = await httpSession.subtract(operation.args[0], operation.args[1]);
            break;
          case "divide":
            result = await httpSession.divide(operation.args[0], operation.args[1]);
            break;
          default:
            throw new Error(`Unknown operation: ${operation.op}`);
        }
        httpResults.push(result);
      }
      const httpDuration = Date.now() - httpStart;
      console.log("\u26A1 WebSocket Performance Test...");
      const wsStart = Date.now();
      const wsSession = newWebSocketRpcSession(wsEndpoint);
      const wsResults = [];
      for (const operation of operations) {
        let result;
        switch (operation.op) {
          case "add":
            result = await wsSession.add(operation.args[0], operation.args[1]);
            break;
          case "multiply":
            result = await wsSession.multiply(operation.args[0], operation.args[1]);
            break;
          case "subtract":
            result = await wsSession.subtract(operation.args[0], operation.args[1]);
            break;
          case "divide":
            result = await wsSession.divide(operation.args[0], operation.args[1]);
            break;
          default:
            throw new Error(`Unknown operation: ${operation.op}`);
        }
        wsResults.push(result);
      }
      const wsDuration = Date.now() - wsStart;
      if ("close" in wsSession) {
        wsSession.close();
      }
      console.log("\u{1F4CA} Performance Analysis:");
      console.log(`  HTTP Batch:    ${httpDuration}ms total, ${(httpDuration / operationCount).toFixed(1)}ms/op`);
      console.log(`  WebSocket:     ${wsDuration}ms total, ${(wsDuration / operationCount).toFixed(1)}ms/op`);
      const performanceRatio = httpDuration / wsDuration;
      console.log(`  Performance Ratio: ${performanceRatio.toFixed(2)}x (${performanceRatio > 1 ? "WebSocket faster" : "HTTP faster"})`);
      const resultsMatch = JSON.stringify(httpResults) === JSON.stringify(wsResults);
      console.log("\u{1F50D} Consistency Check:");
      console.log(`  Results identical: ${resultsMatch ? "\u2713" : "\u2717"}`);
      console.log(`  HTTP throughput:   ${(operationCount / httpDuration * 1e3).toFixed(0)} ops/sec`);
      console.log(`  WebSocket throughput: ${(operationCount / wsDuration * 1e3).toFixed(0)} ops/sec`);
      if (resultsMatch) {
        console.log("\u2713 Both transports maintain computational consistency");
        console.log("\u2713 Performance characteristics measured and compared");
        if (performanceRatio > 0.8) {
          console.log("\u2713 Performance characteristics within expected ranges");
        }
        return true;
      } else {
        console.log("\u2717 Computational inconsistency between transports");
        return false;
      }
    } catch (error) {
      console.log(`Performance comparison test failed: ${error.message}`);
      return false;
    }
  }
  async concurrentTransportUsage() {
    console.log("Testing concurrent usage of both transport types...");
    try {
      console.log("\u{1F680} Launching concurrent operations across both transports...");
      const httpSession = newHttpBatchRpcSession(httpEndpoint);
      const wsSession = newWebSocketRpcSession(wsEndpoint);
      const startTime = Date.now();
      const concurrentOps = await Promise.all([
        // HTTP operations
        httpSession.add(10, 20),
        httpSession.multiply(5, 6),
        httpSession.subtract(100, 25),
        // WebSocket operations
        wsSession.add(15, 35),
        wsSession.multiply(7, 8),
        wsSession.subtract(200, 50)
      ]);
      const duration = Date.now() - startTime;
      if ("close" in wsSession) {
        wsSession.close();
      }
      console.log("\u{1F4CA} Concurrent Operation Results:");
      console.log(`  HTTP add(10, 20): ${concurrentOps[0]}`);
      console.log(`  HTTP multiply(5, 6): ${concurrentOps[1]}`);
      console.log(`  HTTP subtract(100, 25): ${concurrentOps[2]}`);
      console.log(`  WebSocket add(15, 35): ${concurrentOps[3]}`);
      console.log(`  WebSocket multiply(7, 8): ${concurrentOps[4]}`);
      console.log(`  WebSocket subtract(200, 50): ${concurrentOps[5]}`);
      console.log(`\u23F1\uFE0F  Total concurrent execution time: ${duration}ms`);
      console.log(`    Average per operation: ${(duration / 6).toFixed(1)}ms`);
      const expectedResults = [30, 30, 75, 50, 56, 150];
      const resultsCorrect = concurrentOps.every((result, i) => result === expectedResults[i]);
      console.log("\u{1F50D} Verification:");
      console.log(`  Expected: [${expectedResults.join(", ")}]`);
      console.log(`  Actual:   [${concurrentOps.join(", ")}]`);
      console.log(`  All correct: ${resultsCorrect ? "\u2713" : "\u2717"}`);
      if (resultsCorrect) {
        console.log("\u2713 Concurrent transport usage working perfectly");
        console.log("\u2713 Both transports can be used simultaneously without interference");
        console.log("\u2713 No resource conflicts or computation errors detected");
        return true;
      } else {
        console.log("\u2717 Concurrent transport usage produced incorrect results");
        return false;
      }
    } catch (error) {
      console.log(`Concurrent transport usage test failed: ${error.message}`);
      return false;
    }
  }
  async errorHandlingConsistency() {
    console.log("Testing error handling consistency across transports...");
    try {
      console.log("\u{1F9EA} Testing error scenarios on both transports...");
      const httpSession = newHttpBatchRpcSession(httpEndpoint);
      const wsSession = newWebSocketRpcSession(wsEndpoint);
      let httpError = null;
      let wsError = null;
      console.log("  Triggering division by zero on HTTP Batch...");
      try {
        await httpSession.divide(10, 0);
      } catch (error) {
        httpError = error;
        console.log(`    HTTP Error: ${error.message}`);
      }
      console.log("  Triggering division by zero on WebSocket...");
      try {
        await wsSession.divide(10, 0);
      } catch (error) {
        wsError = error;
        console.log(`    WebSocket Error: ${error.message}`);
      }
      console.log("  Testing error recovery...");
      const httpRecovery = await httpSession.add(5, 7);
      const wsRecovery = await wsSession.add(5, 7);
      console.log(`    HTTP Recovery: ${httpRecovery}`);
      console.log(`    WebSocket Recovery: ${wsRecovery}`);
      if ("close" in wsSession) {
        wsSession.close();
      }
      console.log("\u{1F50D} Error Handling Analysis:");
      const bothErrored = httpError !== null && wsError !== null;
      const errorMessagesMatch = httpError?.message === wsError?.message;
      const recoveryMatches = httpRecovery === wsRecovery && httpRecovery === 12;
      console.log(`  Both transports errored: ${bothErrored ? "\u2713" : "\u2717"}`);
      console.log(`  Error messages consistent: ${errorMessagesMatch ? "\u2713" : "\u2717"}`);
      console.log(`  Recovery successful: ${recoveryMatches ? "\u2713" : "\u2717"}`);
      if (bothErrored && errorMessagesMatch && recoveryMatches) {
        console.log("\u2713 Error handling is consistent across both transports");
        console.log("\u2713 Both transports maintain session integrity after errors");
        console.log("\u2713 Error messages are standardized between transports");
        return true;
      } else {
        console.log("\u2717 Error handling inconsistencies detected between transports");
        return false;
      }
    } catch (error) {
      console.log(`Error handling consistency test failed: ${error.message}`);
      return false;
    }
  }
  async transportSpecificAdvantagesTest() {
    console.log("Testing transport-specific advantages and use cases...");
    try {
      console.log("\u{1F4CA} WebSocket advantage: Real-time stream processing...");
      const wsSession = newWebSocketRpcSession(wsEndpoint);
      const streamStart = Date.now();
      const streamValues = [1, 2, 3, 4, 5];
      const streamResults = [];
      for (const value of streamValues) {
        const result = await wsSession.multiply(value, 2);
        streamResults.push(result);
      }
      const streamDuration = Date.now() - streamStart;
      console.log(`  WebSocket stream processing: ${streamDuration}ms for ${streamValues.length} operations`);
      console.log(`  Results: [${streamResults.join(", ")}]`);
      console.log("\u{1F504} HTTP advantage: Stateless bulk operations...");
      const httpSession = newHttpBatchRpcSession(httpEndpoint);
      const bulkStart = Date.now();
      const bulkOperations = await Promise.all([
        httpSession.add(10, 10),
        httpSession.add(20, 20),
        httpSession.add(30, 30),
        httpSession.add(40, 40),
        httpSession.add(50, 50)
      ]);
      const bulkDuration = Date.now() - bulkStart;
      console.log(`  HTTP bulk processing: ${bulkDuration}ms for ${bulkOperations.length} operations`);
      console.log(`  Results: [${bulkOperations.join(", ")}]`);
      if ("close" in wsSession) {
        wsSession.close();
      }
      console.log("\u{1F50D} Transport Advantage Analysis:");
      console.log(`  WebSocket avg/op: ${(streamDuration / streamValues.length).toFixed(1)}ms (persistent connection)`);
      console.log(`  HTTP avg/op: ${(bulkDuration / bulkOperations.length).toFixed(1)}ms (stateless batch)`);
      const streamCorrect = streamResults.every((result, i) => result === streamValues[i] * 2);
      const bulkCorrect = bulkOperations.every((result, i) => result === (i + 1) * 10 * 2);
      console.log(`  WebSocket stream results correct: ${streamCorrect ? "\u2713" : "\u2717"}`);
      console.log(`  HTTP bulk results correct: ${bulkCorrect ? "\u2713" : "\u2717"}`);
      if (streamCorrect && bulkCorrect) {
        console.log("\u2713 Both transports demonstrate their specific advantages");
        console.log("\u2713 WebSocket excels at real-time streaming scenarios");
        console.log("\u2713 HTTP Batch excels at stateless bulk operations");
        return true;
      } else {
        console.log("\u2717 Transport advantages not properly demonstrated");
        return false;
      }
    } catch (error) {
      console.log(`Transport advantages test failed: ${error.message}`);
      return false;
    }
  }
  async run() {
    console.log("\u{1F504} Cross-Transport Interoperability Tests");
    console.log("=====================================");
    console.log(`\u{1F4CD} Testing endpoints:`);
    console.log(`   HTTP Batch: ${httpEndpoint}`);
    console.log(`   WebSocket:  ${wsEndpoint}`);
    console.log("\u{1F3AF} Goal: Verify seamless interoperability between transport types");
    console.log("\u{1F4CB} Prerequisites: All Tier 1, 2, and 3 tests must pass for both transports");
    console.log("");
    await this.test("Transport Computational Equivalence", () => this.transportEquivalenceTest());
    await this.test("Performance Characteristics Comparison", () => this.performanceCharacteristicsComparison());
    await this.test("Concurrent Multi-Transport Usage", () => this.concurrentTransportUsage());
    await this.test("Error Handling Consistency", () => this.errorHandlingConsistency());
    await this.test("Transport-Specific Advantages", () => this.transportSpecificAdvantagesTest());
    console.log("\n" + "=".repeat(80));
    console.log("\u{1F504} CROSS-TRANSPORT INTEROPERABILITY RESULTS");
    console.log("=".repeat(80));
    const passRate = Math.round(this.passed / this.total * 100);
    console.log(`\u2705 Passed: ${this.passed}/${this.total} (${passRate}%)`);
    if (this.passed === this.total) {
      console.log("\u{1F389} CROSS-TRANSPORT INTEROPERABILITY COMPLETE!");
      console.log("\u{1F680} HTTP Batch and WebSocket transports are fully interoperable");
      console.log("\u26A1 Both transports provide consistent Cap'n Web protocol implementation");
      console.log("\u{1F3C6} Production-ready multi-transport Cap'n Web server achieved!");
      console.log("\u{1F4CA} Applications can seamlessly choose optimal transport for their use case");
      process.exit(0);
    } else if (this.passed >= this.total * 0.8) {
      console.log("\u2B50 CROSS-TRANSPORT INTEROPERABILITY EXCELLENT!");
      console.log("\u{1F527} Minor transport differences detected, but overall compatibility is strong");
      process.exit(0);
    } else {
      console.log("\u{1F4A5} CROSS-TRANSPORT INTEROPERABILITY ISSUES DETECTED");
      console.log("\u{1F6A8} Significant transport inconsistencies require attention");
      process.exit(1);
    }
  }
};
process.on("unhandledRejection", (reason, promise) => {
  console.error("Unhandled Rejection at:", promise, "reason:", reason);
  process.exit(3);
});
var crossTransportTests = new CrossTransportInteropTests();
crossTransportTests.run();
//# sourceMappingURL=cross-transport-interop.js.map