#!/usr/bin/env node

// src/comprehensive-stateful-test.ts
import { newHttpBatchRpcSession } from "capnweb";
var port = process.argv[2] || "9001";
var endpoint = `http://localhost:${port}/rpc/batch`;
async function testBasicCalculator() {
  console.log("\u{1F9EE} Testing Basic Calculator Operations");
  console.log("=====================================");
  const session = newHttpBatchRpcSession(endpoint);
  let passed = 0;
  let total = 0;
  total++;
  try {
    const result = await session.add(5, 3);
    if (result === 8) {
      console.log("\u2705 Addition test passed");
      passed++;
    } else {
      console.log(`\u274C Addition test failed: expected 8, got ${result}`);
    }
  } catch (error) {
    console.log(`\u274C Addition test failed with error: ${error}`);
  }
  total++;
  try {
    const result = await session.multiply(7, 6);
    if (result === 42) {
      console.log("\u2705 Multiplication test passed");
      passed++;
    } else {
      console.log(`\u274C Multiplication test failed: expected 42, got ${result}`);
    }
  } catch (error) {
    console.log(`\u274C Multiplication test failed with error: ${error}`);
  }
  total++;
  try {
    const result = await session.divide(100, 4);
    if (result === 25) {
      console.log("\u2705 Division test passed");
      passed++;
    } else {
      console.log(`\u274C Division test failed: expected 25, got ${result}`);
    }
  } catch (error) {
    console.log(`\u274C Division test failed with error: ${error}`);
  }
  total++;
  try {
    const result = await session.subtract(10, 7);
    if (result === 3) {
      console.log("\u2705 Subtraction test passed");
      passed++;
    } else {
      console.log(`\u274C Subtraction test failed: expected 3, got ${result}`);
    }
  } catch (error) {
    console.log(`\u274C Subtraction test failed with error: ${error}`);
  }
  total++;
  try {
    const result = await session.divide(10, 0);
    console.log(`\u274C Division by zero should have thrown an error, got: ${result}`);
  } catch (error) {
    console.log("\u2705 Division by zero correctly threw error");
    passed++;
  }
  console.log(`
\u{1F4CA} Basic Calculator: ${passed}/${total} tests passed
`);
  return { passed, total };
}
async function testConcurrentOperations() {
  console.log("\u{1F504} Testing Concurrent Operations");
  console.log("================================");
  const session = newHttpBatchRpcSession(endpoint);
  let passed = 0;
  let total = 0;
  total++;
  try {
    const start = Date.now();
    const [sum, product, quotient, difference] = await Promise.all([
      session.add(10, 20),
      session.multiply(5, 8),
      session.divide(100, 5),
      session.subtract(50, 15)
    ]);
    const duration = Date.now() - start;
    const expectedResults = [30, 40, 20, 35];
    const actualResults = [sum, product, quotient, difference];
    if (JSON.stringify(actualResults) === JSON.stringify(expectedResults)) {
      console.log(`\u2705 Concurrent operations passed (${duration}ms)`);
      console.log(`   Results: ${actualResults.join(", ")}`);
      passed++;
    } else {
      console.log(`\u274C Concurrent operations failed:`);
      console.log(`   Expected: ${expectedResults.join(", ")}`);
      console.log(`   Actual: ${actualResults.join(", ")}`);
    }
  } catch (error) {
    console.log(`\u274C Concurrent operations failed with error: ${error}`);
  }
  console.log(`
\u{1F4CA} Concurrent Operations: ${passed}/${total} tests passed
`);
  return { passed, total };
}
async function testSessionPersistence() {
  console.log("\u{1F4BE} Testing Session Persistence");
  console.log("==============================");
  let passed = 0;
  let total = 0;
  total++;
  try {
    const session1 = newHttpBatchRpcSession(endpoint);
    const session2 = newHttpBatchRpcSession(endpoint);
    const [result1, result2] = await Promise.all([
      session1.add(1, 2),
      session2.multiply(3, 4)
    ]);
    if (result1 === 3 && result2 === 12) {
      console.log("\u2705 Multiple sessions work independently");
      passed++;
    } else {
      console.log(`\u274C Session independence failed: ${result1}, ${result2}`);
    }
  } catch (error) {
    console.log(`\u274C Session persistence test failed: ${error}`);
  }
  console.log(`
\u{1F4CA} Session Persistence: ${passed}/${total} tests passed
`);
  return { passed, total };
}
async function testErrorScenarios() {
  console.log("\u26A0\uFE0F  Testing Error Scenarios");
  console.log("===========================");
  const session = newHttpBatchRpcSession(endpoint);
  let passed = 0;
  let total = 0;
  total++;
  try {
    await session.invalidMethod(1, 2);
    console.log("\u274C Invalid method should have failed");
  } catch (error) {
    console.log("\u2705 Invalid method correctly failed");
    passed++;
  }
  total++;
  try {
    await session.divide(1, 0);
    console.log("\u274C Division by zero should have failed");
  } catch (error) {
    console.log("\u2705 Division by zero correctly failed");
    passed++;
  }
  console.log(`
\u{1F4CA} Error Scenarios: ${passed}/${total} tests passed
`);
  return { passed, total };
}
async function testPerformance() {
  console.log("\u26A1 Testing Performance");
  console.log("=====================");
  const session = newHttpBatchRpcSession(endpoint);
  let passed = 0;
  let total = 0;
  total++;
  try {
    const startSeq = Date.now();
    await session.add(1, 2);
    await session.add(3, 4);
    await session.add(5, 6);
    await session.add(7, 8);
    const sequentialTime = Date.now() - startSeq;
    const startPar = Date.now();
    await Promise.all([
      session.add(1, 2),
      session.add(3, 4),
      session.add(5, 6),
      session.add(7, 8)
    ]);
    const parallelTime = Date.now() - startPar;
    console.log(`\u{1F4C8} Sequential: ${sequentialTime}ms, Parallel: ${parallelTime}ms`);
    if (parallelTime <= sequentialTime * 1.5) {
      console.log("\u2705 Parallel operations perform well");
      passed++;
    } else {
      console.log("\u26A0\uFE0F  Parallel operations may need optimization");
      passed++;
    }
  } catch (error) {
    console.log(`\u274C Performance test failed: ${error}`);
  }
  console.log(`
\u{1F4CA} Performance: ${passed}/${total} tests passed
`);
  return { passed, total };
}
async function main() {
  console.log("\u{1F680} Comprehensive Stateful Server Test Suite");
  console.log("===========================================");
  console.log(`\u{1F4CD} Testing endpoint: ${endpoint}
`);
  try {
    const results = await Promise.all([
      testBasicCalculator(),
      testConcurrentOperations(),
      testSessionPersistence(),
      testErrorScenarios(),
      testPerformance()
    ]);
    const totalPassed = results.reduce((sum, result) => sum + result.passed, 0);
    const totalTests = results.reduce((sum, result) => sum + result.total, 0);
    const passRate = Math.round(totalPassed / totalTests * 100);
    console.log("\u{1F3C1} Final Results");
    console.log("================");
    console.log(`\u2705 Passed: ${totalPassed}/${totalTests} (${passRate}%)`);
    if (totalPassed === totalTests) {
      console.log("\u{1F389} All tests passed! The stateful server is working correctly.");
      process.exit(0);
    } else {
      console.log("\u{1F4A5} Some tests failed. Check the server implementation.");
      process.exit(1);
    }
  } catch (error) {
    console.error("\u{1F4A5} Test suite failed with error:", error);
    process.exit(1);
  }
}
process.on("unhandledRejection", (reason, promise) => {
  console.error("Unhandled Rejection at:", promise, "reason:", reason);
  process.exit(1);
});
main();
//# sourceMappingURL=comprehensive-stateful-test.js.map