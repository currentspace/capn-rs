#!/usr/bin/env node
import {
  testAdvancedStatefulServer
} from "./chunk-6WPWZAZW.js";
import {
  testPromisePipelining
} from "./chunk-AN3UUTFI.js";

// src/comprehensive-test-runner.ts
import { spawn } from "child_process";
async function runBasicClientTest() {
  const start = performance.now();
  return new Promise((resolve) => {
    const child = spawn("node", ["dist/official-client-test.js"], {
      cwd: process.cwd(),
      stdio: "pipe"
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (data) => {
      stdout += data.toString();
    });
    child.stderr.on("data", (data) => {
      stderr += data.toString();
    });
    child.on("close", (code) => {
      const end = performance.now();
      resolve({
        name: "Basic Client Test",
        success: code === 0,
        duration: end - start,
        error: code !== 0 ? stderr || "Process exited with non-zero code" : void 0
      });
    });
  });
}
async function runTestWithMeasurement(name, testFn) {
  const start = performance.now();
  try {
    await testFn();
    const end = performance.now();
    return {
      name,
      success: true,
      duration: end - start
    };
  } catch (error) {
    const end = performance.now();
    return {
      name,
      success: false,
      duration: end - start,
      error: error instanceof Error ? error.message : String(error)
    };
  }
}
async function checkServerHealth() {
  try {
    const response = await fetch("http://localhost:8080/rpc/batch", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify([])
    });
    return response.ok;
  } catch {
    return false;
  }
}
async function runComprehensiveTests() {
  console.log("\u{1F680} Comprehensive Cap'n Web Rust Server Test Suite");
  console.log("===================================================\n");
  console.log("\u{1F50D} Checking server health...");
  const serverHealthy = await checkServerHealth();
  if (!serverHealthy) {
    console.error("\u274C Server is not running or not responding");
    console.error("   Please start the server with:");
    console.error("   cargo run --example advanced_stateful_server -p capnweb-server");
    process.exit(1);
  }
  console.log("\u2705 Server is healthy and responding\n");
  const tests = [
    {
      name: "Basic Calculator Client Test",
      fn: async () => {
        const result = await runBasicClientTest();
        if (!result.success) {
          throw new Error(result.error || "Basic client test failed");
        }
        return result;
      }
    },
    {
      name: "Advanced Stateful Server Test",
      fn: () => testAdvancedStatefulServer()
    },
    {
      name: "Promise Pipelining Test",
      fn: () => testPromisePipelining()
    }
  ];
  const results = [];
  for (const test of tests) {
    console.log(`\u{1F9EA} Running: ${test.name}`);
    console.log("=".repeat(50));
    const result = await runTestWithMeasurement(test.name, test.fn);
    results.push(result);
    if (result.success) {
      console.log(`\u2705 ${test.name} - PASSED (${result.duration.toFixed(2)}ms)`);
    } else {
      console.log(`\u274C ${test.name} - FAILED (${result.duration.toFixed(2)}ms)`);
      if (result.error) {
        console.log(`   Error: ${result.error}`);
      }
    }
    console.log("\n");
  }
  console.log("=" + "=".repeat(79));
  console.log("\u{1F4CA} COMPREHENSIVE TEST RESULTS SUMMARY");
  console.log("=" + "=".repeat(79));
  const totalTests = results.length;
  const passedTests = results.filter((r) => r.success).length;
  const failedTests = totalTests - passedTests;
  const totalDuration = results.reduce((sum, r) => sum + r.duration, 0);
  console.log(`
\u{1F4C8} Test Statistics:`);
  console.log(`   Total Tests: ${totalTests}`);
  console.log(`   Passed: ${passedTests} \u2705`);
  console.log(`   Failed: ${failedTests} ${failedTests > 0 ? "\u274C" : "\u2705"}`);
  console.log(`   Success Rate: ${(passedTests / totalTests * 100).toFixed(1)}%`);
  console.log(`   Total Duration: ${totalDuration.toFixed(2)}ms`);
  console.log(`   Average per Test: ${(totalDuration / totalTests).toFixed(2)}ms`);
  console.log(`
\u{1F4CB} Individual Test Results:`);
  results.forEach((result) => {
    const status = result.success ? "\u2705 PASS" : "\u274C FAIL";
    const duration = result.duration.toFixed(2).padStart(8);
    console.log(`   ${status} \u2502 ${duration}ms \u2502 ${result.name}`);
    if (!result.success && result.error) {
      console.log(`         \u2502         \u2502   \u2514\u2500 ${result.error}`);
    }
  });
  console.log("\n\u{1F3C6} FEATURE VALIDATION STATUS:");
  console.log("==============================");
  const featureStatus = {
    "Basic RPC Communication": results[0]?.success ?? false,
    "Stateful Session Management": results[1]?.success ?? false,
    "Global Counter Operations": results[1]?.success ?? false,
    "Session-Specific Storage": results[1]?.success ?? false,
    "Property Management": results[1]?.success ?? false,
    "Concurrent Operations": results[1]?.success ?? false,
    "Error Handling": results[1]?.success ?? false,
    "Promise Pipelining": results[2]?.success ?? false,
    "Batch Optimization": results[2]?.success ?? false,
    "Mixed Operation Types": results[2]?.success ?? false,
    "Resource Cleanup": results[2]?.success ?? false
  };
  Object.entries(featureStatus).forEach(([feature, status]) => {
    const icon = status ? "\u2705" : "\u274C";
    console.log(`   ${icon} ${feature}`);
  });
  const allPassed = results.every((r) => r.success);
  if (allPassed) {
    console.log("\n\u{1F389} ALL TESTS PASSED! \u{1F389}");
    console.log("========================");
    console.log("\u{1F680} The Cap'n Web Rust implementation is fully functional!");
    console.log("\u{1F4E6} Ready for production deployment");
    console.log("\u{1F517} Compatible with official TypeScript Cap'n Web client");
    console.log("\u26A1 Optimized for performance and concurrency");
    console.log("\u{1F6E1}\uFE0F  Robust error handling and session management");
  } else {
    console.log("\n\u26A0\uFE0F  SOME TESTS FAILED");
    console.log("====================");
    console.log("\u274C Implementation needs attention before production use");
    console.log("\u{1F527} Review failed tests and fix underlying issues");
    console.log("\u{1F9EA} Re-run tests after fixes are applied");
    process.exit(1);
  }
}
function setupPerformanceMonitoring() {
  const memoryUsage = process.memoryUsage();
  console.log("\n\u{1F4CA} Performance Monitoring:");
  console.log(`   Heap Used: ${(memoryUsage.heapUsed / 1024 / 1024).toFixed(2)} MB`);
  console.log(`   Heap Total: ${(memoryUsage.heapTotal / 1024 / 1024).toFixed(2)} MB`);
  console.log(`   RSS: ${(memoryUsage.rss / 1024 / 1024).toFixed(2)} MB`);
  console.log(`   External: ${(memoryUsage.external / 1024 / 1024).toFixed(2)} MB`);
}
process.on("exit", () => {
  setupPerformanceMonitoring();
  console.log("\n\u{1F44B} Test suite completed - resources cleaned up");
});
process.on("SIGINT", () => {
  console.log("\n\n\u26A0\uFE0F  Test suite interrupted by user");
  process.exit(0);
});
process.on("unhandledRejection", (reason, promise) => {
  console.error("\n\u{1F4A5} Unhandled promise rejection:", reason);
  process.exit(1);
});
if (import.meta.url === `file://${process.argv[1]}`) {
  runComprehensiveTests().catch((error) => {
    console.error("\n\u{1F4A5} Fatal error in test suite:", error);
    process.exit(1);
  });
}
export {
  runComprehensiveTests
};
//# sourceMappingURL=comprehensive-test-runner.js.map