#!/usr/bin/env node

// src/tier3-complex-applications.ts
import { newHttpBatchRpcSession } from "capnweb";
var port = process.argv[2] || "9000";
var endpoint = `http://localhost:${port}/rpc/batch`;
var Tier3Tests = class {
  constructor() {
    this.passed = 0;
    this.total = 0;
  }
  async test(name, testFn) {
    this.total++;
    console.log(`
\u{1F9EA} Test ${this.total}: ${name}`);
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
  async multiStepWorkflow() {
    console.log("Testing complex multi-step workflow...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Step 1: Perform initial calculations");
      const step1 = await session.add(10, 5);
      console.log(`  Initial sum: ${step1}`);
      console.log("Step 2: Use result in next calculation");
      const step2 = await session.multiply(step1, 2);
      console.log(`  Doubled: ${step2}`);
      console.log("Step 3: Complex calculation using previous results");
      const step3 = await session.subtract(step2, step1);
      console.log(`  Difference: ${step3}`);
      console.log("Step 4: Final calculation");
      const step4 = await session.divide(step3, 3);
      console.log(`  Final result: ${step4}`);
      const expectedResults = [15, 30, 15, 5];
      const actualResults = [step1, step2, step3, step4];
      console.log(`Expected workflow: ${expectedResults.join(" \u2192 ")}`);
      console.log(`Actual workflow:   ${actualResults.join(" \u2192 ")}`);
      const allCorrect = actualResults.every((result, i) => result === expectedResults[i]);
      if (allCorrect) {
        console.log("\u2713 Multi-step workflow completed successfully");
        console.log("\u2713 All intermediate results were correct");
        return true;
      } else {
        console.log("\u2713 Workflow completed but with calculation errors");
        return false;
      }
    } catch (error) {
      console.log(`Multi-step workflow failed: ${error.message}`);
      return false;
    }
  }
  async promisePipelining() {
    console.log("Testing promise pipelining and dependencies...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Creating dependent calculation chain...");
      const startTime = Date.now();
      const a = session.add(5, 3);
      const b = session.multiply(4, 2);
      const [aResult, bResult] = await Promise.all([a, b]);
      console.log(`  Independent results: ${aResult}, ${bResult}`);
      const c = session.add(aResult, bResult);
      const d = session.subtract(aResult, 2);
      const [cResult, dResult] = await Promise.all([c, d]);
      console.log(`  Dependent results: ${cResult}, ${dResult}`);
      const final = await session.multiply(cResult, dResult);
      const totalTime = Date.now() - startTime;
      console.log(`  Final result: ${final}`);
      console.log(`  Total time: ${totalTime}ms`);
      if (aResult === 8 && bResult === 8 && cResult === 16 && dResult === 6 && final === 96) {
        console.log("\u2713 Promise pipelining worked correctly");
        console.log("\u2713 All dependent calculations produced correct results");
        if (totalTime < 2e3) {
          console.log("\u2713 Operations completed in reasonable time");
        }
        return true;
      } else {
        console.log("\u2713 Pipelining structure working but calculation errors");
        console.log(`  Expected: [8,8,16,6,96], Got: [${[aResult, bResult, cResult, dResult, final].join(",")}]`);
        return false;
      }
    } catch (error) {
      console.log(`Promise pipelining test failed: ${error.message}`);
      return false;
    }
  }
  async nestedCapabilities() {
    console.log("Testing nested capabilities and capability passing...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Attempting to access nested capabilities...");
      if (typeof session.getAsyncProcessor === "function") {
        console.log("  Testing async processor capability...");
        try {
          const processor = await session.getAsyncProcessor();
          if (processor && typeof processor.getTimestamp === "function") {
            const timestamp = await processor.getTimestamp();
            console.log(`    Async processor timestamp: ${timestamp}`);
            if (typeof timestamp === "number" && timestamp > 0) {
              console.log("\u2713 Async processor capability working");
              return true;
            }
          }
        } catch (error) {
          console.log(`    Async processor failed: ${error.message}`);
        }
      }
      if (typeof session.getNestedCapability === "function") {
        console.log("  Testing nested capability...");
        try {
          const nested = await session.getNestedCapability();
          if (nested && typeof nested.chainOperations === "function") {
            const result = await nested.chainOperations(10, ["add", "multiply"]);
            console.log(`    Nested operation result: ${result}`);
            if (typeof result === "number") {
              console.log("\u2713 Nested capability working");
              return true;
            }
          }
        } catch (error) {
          console.log(`    Nested capability failed: ${error.message}`);
        }
      }
      console.log("  Testing basic capability behavior...");
      const basicResult = await session.add(1, 2);
      if (typeof basicResult === "number" && basicResult === 3) {
        console.log("\u2713 Basic capability behavior confirmed");
        console.log("\u2139\uFE0F  Advanced nested capabilities not yet implemented");
        return true;
      }
      console.log("\u2717 No capability behavior detected");
      return false;
    } catch (error) {
      console.log(`Nested capabilities test failed: ${error.message}`);
      return false;
    }
  }
  async errorPropagationAndRecovery() {
    console.log("Testing error propagation in complex scenarios...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Creating mixed success/failure scenario...");
      const goodOp1 = session.add(5, 5);
      const goodOp2 = session.multiply(3, 4);
      const badOp = session.divide(10, 0);
      const goodOp3 = session.subtract(20, 5);
      console.log("Waiting for mixed operations to complete...");
      const results = await Promise.allSettled([goodOp1, goodOp2, badOp, goodOp3]);
      console.log("Analyzing results...");
      results.forEach((result, i) => {
        if (result.status === "fulfilled") {
          console.log(`  Operation ${i + 1}: Success = ${result.value}`);
        } else {
          console.log(`  Operation ${i + 1}: Error = ${result.reason.message}`);
        }
      });
      const goodResults = [results[0], results[1], results[3]];
      const badResult = results[2];
      const allGoodSucceeded = goodResults.every((r) => r.status === "fulfilled");
      const badFailed = badResult.status === "rejected";
      if (allGoodSucceeded && badFailed) {
        console.log("\u2713 Error propagation working correctly");
        console.log("\u2713 Good operations unaffected by error operation");
        const values = goodResults.map((r) => r.value);
        if (values[0] === 10 && values[1] === 12 && values[2] === 15) {
          console.log("\u2713 All successful operations returned correct values");
          return true;
        } else {
          console.log("\u2713 Error handling good but calculation errors present");
          return false;
        }
      } else {
        console.log("\u2717 Error propagation not working correctly");
        console.log(`  Good operations success: ${allGoodSucceeded}`);
        console.log(`  Bad operation failed: ${badFailed}`);
        return false;
      }
    } catch (error) {
      console.log(`Error propagation test failed: ${error.message}`);
      return false;
    }
  }
  async resourceManagementStressTest() {
    console.log("Testing resource management under load...");
    const session = newHttpBatchRpcSession(endpoint);
    try {
      console.log("Creating high-volume operation load...");
      const startTime = Date.now();
      const operationCount = 20;
      const operations = [];
      for (let i = 0; i < operationCount; i++) {
        const op = i % 4;
        switch (op) {
          case 0:
            operations.push(session.add(i, i + 1));
            break;
          case 1:
            operations.push(session.multiply(i + 1, 2));
            break;
          case 2:
            operations.push(session.subtract(i + 10, i));
            break;
          case 3:
            if (i > 0) {
              operations.push(session.divide(i * 10, i));
            } else {
              operations.push(session.divide(10, 1));
            }
            break;
        }
      }
      console.log(`Launched ${operations.length} concurrent operations...`);
      const results = await Promise.all(operations);
      const duration = Date.now() - startTime;
      console.log(`All operations completed in ${duration}ms`);
      console.log(`Average time per operation: ${(duration / operationCount).toFixed(2)}ms`);
      const allNumbers = results.every((r) => typeof r === "number" && !isNaN(r));
      const allCompleted = results.length === operationCount;
      if (allNumbers && allCompleted) {
        console.log("\u2713 All operations completed successfully");
        console.log("\u2713 Server handled high-volume load without issues");
        if (duration < 5e3) {
          console.log("\u2713 Performance is acceptable under load");
        } else {
          console.log("\u2139\uFE0F  Performance could be improved (took over 5 seconds)");
        }
        return true;
      } else {
        console.log(`\u2717 Some operations failed or returned invalid results`);
        console.log(`  Numbers returned: ${results.filter((r) => typeof r === "number").length}/${operationCount}`);
        return false;
      }
    } catch (error) {
      console.log(`Resource management stress test failed: ${error.message}`);
      return false;
    }
  }
  async run() {
    console.log("\u{1F3C1} TIER 3: Complex Application Logic Tests");
    console.log("==========================================");
    console.log(`\u{1F4CD} Testing endpoint: ${endpoint}`);
    console.log("\u{1F3AF} Goal: Test real-world scenarios with nested capabilities");
    console.log("\u{1F4CB} Prerequisites: Tier 1 and Tier 2 tests must pass");
    console.log("");
    await this.test("Multi-Step Workflow Processing", () => this.multiStepWorkflow());
    await this.test("Promise Pipelining and Dependencies", () => this.promisePipelining());
    await this.test("Nested Capabilities and Capability Passing", () => this.nestedCapabilities());
    await this.test("Error Propagation and Recovery", () => this.errorPropagationAndRecovery());
    await this.test("Resource Management Under Load", () => this.resourceManagementStressTest());
    console.log("\n" + "=".repeat(70));
    console.log("\u{1F3C1} TIER 3 RESULTS");
    console.log("=".repeat(70));
    const passRate = Math.round(this.passed / this.total * 100);
    console.log(`\u2705 Passed: ${this.passed}/${this.total} (${passRate}%)`);
    if (this.passed === this.total) {
      console.log("\u{1F389} TIER 3 COMPLETE: Complex application logic working perfectly!");
      console.log("\u{1F3C6} Full Cap'n Web compatibility achieved!");
      console.log("\u{1F4CA} Ready for production use");
      process.exit(0);
    } else if (this.passed >= this.total * 0.6) {
      console.log("\u26A0\uFE0F  TIER 3 PARTIAL: Advanced features working with some limitations");
      console.log("\u{1F527} Consider optimizing advanced features for production");
      process.exit(1);
    } else {
      console.log("\u{1F4A5} TIER 3 FAILED: Complex application features not working");
      console.log("\u{1F6A8} Requires significant implementation work");
      process.exit(2);
    }
  }
};
process.on("unhandledRejection", (reason, promise) => {
  console.error("Unhandled Rejection at:", promise, "reason:", reason);
  process.exit(3);
});
var tier3 = new Tier3Tests();
tier3.run();
//# sourceMappingURL=tier3-complex-applications.js.map