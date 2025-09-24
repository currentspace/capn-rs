#!/usr/bin/env node

// src/tier2-websocket-tests.ts
import { newWebSocketRpcSession } from "capnweb";
var port = process.argv[2] || "9000";
var wsEndpoint = `ws://localhost:${port}/rpc/ws`;
var Tier2WebSocketTests = class {
  constructor() {
    this.passed = 0;
    this.total = 0;
  }
  async test(name, testFn) {
    this.total++;
    console.log(`
\u{1F9EA} Test ${this.total}: ${name} (WebSocket)`);
    console.log("\u2500".repeat(70));
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
    console.log("Testing WebSocket session persistence across multiple messages...");
    try {
      const session = newWebSocketRpcSession(wsEndpoint);
      const results = [];
      console.log("Making sequential requests over WebSocket...");
      results.push(await session.add(1, 2));
      results.push(await session.multiply(3, 4));
      results.push(await session.subtract(10, 5));
      console.log(`Results: ${results.join(", ")}`);
      if ("close" in session) {
        session.close();
      }
      const allNumbers = results.every((r) => typeof r === "number" && !isNaN(r));
      const correctValues = results[0] === 3 && results[1] === 12 && results[2] === 5;
      if (allNumbers && correctValues) {
        console.log("\u2713 All operations returned correct results");
        console.log("\u2713 WebSocket session maintained state across multiple messages");
        return true;
      } else if (allNumbers) {
        console.log("\u2713 WebSocket session persistent (wrong values may indicate calculation issues)");
        console.log(`  Expected: [3, 12, 5], Got: [${results.join(", ")}]`);
        return false;
      } else {
        console.log("\u2717 Inconsistent response types or WebSocket session issues");
        return false;
      }
    } catch (error) {
      console.log(`WebSocket session persistence test failed: ${error.message}`);
      return false;
    }
  }
  async sessionIsolation() {
    console.log("Testing WebSocket session isolation between different connections...");
    try {
      const session1 = newWebSocketRpcSession(wsEndpoint);
      const session2 = newWebSocketRpcSession(wsEndpoint);
      console.log("Creating two separate WebSocket client sessions...");
      const [result1, result2] = await Promise.all([
        session1.add(5, 5),
        session2.multiply(6, 6)
      ]);
      console.log(`WebSocket Session 1 result: ${result1}`);
      console.log(`WebSocket Session 2 result: ${result2}`);
      if ("close" in session1) session1.close();
      if ("close" in session2) session2.close();
      if (typeof result1 === "number" && typeof result2 === "number") {
        if (result1 === 10 && result2 === 36) {
          console.log("\u2713 Both WebSocket sessions returned correct results");
          console.log("\u2713 WebSocket sessions are properly isolated");
          return true;
        } else {
          console.log("\u2713 WebSocket sessions isolated but calculation errors");
          console.log(`  Expected: [10, 36], Got: [${result1}, ${result2}]`);
          return false;
        }
      } else {
        console.log("\u2717 One or both WebSocket sessions failed to respond properly");
        return false;
      }
    } catch (error) {
      console.log(`WebSocket session isolation test failed: ${error.message}`);
      return false;
    }
  }
  async concurrentOperations() {
    console.log("Testing concurrent operations within a single WebSocket session...");
    try {
      const session = newWebSocketRpcSession(wsEndpoint);
      console.log("Launching concurrent operations over WebSocket...");
      const startTime = Date.now();
      const results = await Promise.all([
        session.add(2, 3),
        session.multiply(4, 5),
        session.divide(20, 4),
        session.subtract(15, 7)
      ]);
      const duration = Date.now() - startTime;
      console.log(`All WebSocket operations completed in ${duration}ms`);
      console.log(`Results: ${results.join(", ")}`);
      if ("close" in session) session.close();
      const expected = [5, 20, 5, 8];
      const allCorrect = results.every((r, i) => r === expected[i]);
      if (allCorrect) {
        console.log("\u2713 All concurrent WebSocket operations returned correct results");
        console.log("\u2713 Server handled concurrent WebSocket requests properly");
        if (duration < 1e3) {
          console.log("\u2713 WebSocket operations appear to be processed concurrently");
        }
        return true;
      } else {
        console.log("\u2713 Concurrent WebSocket operations completed but with incorrect results");
        console.log(`  Expected: [${expected.join(", ")}], Got: [${results.join(", ")}]`);
        return false;
      }
    } catch (error) {
      console.log(`Concurrent WebSocket operations test failed: ${error.message}`);
      return false;
    }
  }
  async errorRecovery() {
    console.log("Testing error recovery and WebSocket session stability...");
    try {
      const session = newWebSocketRpcSession(wsEndpoint);
      console.log("Performing initial successful operation over WebSocket...");
      const initial = await session.add(1, 1);
      console.log(`Initial result: ${initial}`);
      if (typeof initial !== "number" || initial !== 2) {
        console.log("\u2717 Initial WebSocket operation failed - cannot test error recovery");
        if ("close" in session) session.close();
        return false;
      }
      console.log("Triggering an error (division by zero) over WebSocket...");
      let errorOccurred = false;
      try {
        await session.divide(5, 0);
        console.log("\u2139\uFE0F  Division by zero did not throw error (unexpected)");
      } catch (error) {
        console.log(`\u2713 Error properly thrown over WebSocket: ${error.message}`);
        errorOccurred = true;
      }
      console.log("Testing WebSocket session recovery with another operation...");
      const recovery = await session.multiply(3, 4);
      console.log(`Recovery result: ${recovery}`);
      if ("close" in session) session.close();
      if (typeof recovery === "number" && recovery === 12) {
        console.log("\u2713 WebSocket session recovered after error");
        console.log("\u2713 Error handling did not corrupt WebSocket session state");
        return true;
      } else {
        console.log("\u2717 WebSocket session corrupted after error");
        return false;
      }
    } catch (error) {
      console.log(`WebSocket error recovery test failed: ${error.message}`);
      return false;
    }
  }
  async realTimeMessaging() {
    console.log("Testing real-time bidirectional messaging over WebSocket...");
    try {
      const session = newWebSocketRpcSession(wsEndpoint);
      console.log("Testing rapid-fire operations over persistent WebSocket connection...");
      const startTime = Date.now();
      const operations = [];
      for (let i = 0; i < 10; i++) {
        operations.push(session.add(i, i + 1));
      }
      const results = await Promise.all(operations);
      const duration = Date.now() - startTime;
      console.log(`10 rapid WebSocket operations completed in ${duration}ms`);
      console.log(`Average per operation: ${(duration / 10).toFixed(1)}ms`);
      if ("close" in session) session.close();
      const expected = Array.from({ length: 10 }, (_, i) => i + (i + 1));
      const allCorrect = results.every((r, i) => r === expected[i]);
      if (allCorrect) {
        console.log("\u2713 All rapid-fire WebSocket operations returned correct results");
        console.log("\u2713 WebSocket demonstrates real-time messaging capability");
        if (duration < 500) {
          console.log("\u2713 Excellent WebSocket performance for real-time use");
        }
        return true;
      } else {
        console.log("\u2717 Some rapid-fire WebSocket operations returned incorrect results");
        console.log(`  Expected: [${expected.join(", ")}]`);
        console.log(`  Got: [${results.join(", ")}]`);
        return false;
      }
    } catch (error) {
      console.log(`Real-time WebSocket messaging test failed: ${error.message}`);
      return false;
    }
  }
  async run() {
    console.log("\u{1F310} TIER 2 WebSocket: Stateful Session Management Tests");
    console.log("====================================================");
    console.log(`\u{1F4CD} Testing WebSocket endpoint: ${wsEndpoint}`);
    console.log("\u{1F3AF} Goal: Verify session persistence over WebSocket transport");
    console.log("\u{1F4CB} Prerequisites: Tier 1 tests must pass + WebSocket support");
    console.log("");
    await this.test("WebSocket Session Persistence", () => this.sessionPersistence());
    await this.test("WebSocket Session Isolation", () => this.sessionIsolation());
    await this.test("Concurrent WebSocket Operations", () => this.concurrentOperations());
    await this.test("WebSocket Error Recovery", () => this.errorRecovery());
    await this.test("Real-time Bidirectional Messaging", () => this.realTimeMessaging());
    console.log("\n" + "=".repeat(70));
    console.log("\u{1F310} TIER 2 WebSocket RESULTS");
    console.log("=".repeat(70));
    const passRate = Math.round(this.passed / this.total * 100);
    console.log(`\u2705 Passed: ${this.passed}/${this.total} (${passRate}%)`);
    if (this.passed === this.total) {
      console.log("\u{1F389} TIER 2 WebSocket COMPLETE: WebSocket stateful session management working!");
      console.log("\u{1F680} WebSocket transport provides real-time capabilities");
      console.log("\u{1F4C8} Ready for Tier 3: Complex Application Logic over WebSocket");
      process.exit(0);
    } else if (this.passed >= this.total * 0.6) {
      console.log("\u26A0\uFE0F  TIER 2 WebSocket PARTIAL: Some WebSocket session management issues remain");
      console.log("\u{1F527} Address WebSocket state issues before Tier 3");
      process.exit(1);
    } else {
      console.log("\u{1F4A5} TIER 2 WebSocket FAILED: WebSocket session management not working");
      console.log("\u{1F6A8} Fix WebSocket state tracking before proceeding");
      process.exit(2);
    }
  }
};
process.on("unhandledRejection", (reason, promise) => {
  console.error("Unhandled Rejection at:", promise, "reason:", reason);
  process.exit(3);
});
var tier2WebSocket = new Tier2WebSocketTests();
tier2WebSocket.run();
//# sourceMappingURL=tier2-websocket-tests.js.map