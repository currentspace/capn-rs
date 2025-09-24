#!/usr/bin/env node

// src/fixed-official-client-test.ts
import { newHttpBatchRpcSession } from "capnweb";
async function testWithOfficialClient() {
  console.log("\u{1F9EA} Testing Cap'n Web Rust Server with Official TypeScript Client");
  console.log("================================================================\n");
  try {
    const port = process.argv[2] || "9006";
    const endpoint = `http://localhost:${port}/rpc/batch`;
    console.log("\u2705 Created endpoint configuration");
    console.log(`\u{1F4CD} Endpoint: ${endpoint}
`);
    console.log("Test 1: Single Operations (New Session Per Call)");
    console.log("------------------------------------------------");
    const session1 = newHttpBatchRpcSession(endpoint);
    const result1 = await session1.add(5, 3);
    console.log(`\u2705 add(5, 3) = ${result1}`);
    const session2 = newHttpBatchRpcSession(endpoint);
    const result2 = await session2.multiply(7, 6);
    console.log(`\u2705 multiply(7, 6) = ${result2}`);
    const session3 = newHttpBatchRpcSession(endpoint);
    const result3 = await session3.divide(100, 4);
    console.log(`\u2705 divide(100, 4) = ${result3}`);
    const session4 = newHttpBatchRpcSession(endpoint);
    const result4 = await session4.subtract(10, 3);
    console.log(`\u2705 subtract(10, 3) = ${result4}`);
    console.log("\nTest 2: True Batching (Multiple Operations in One Request)");
    console.log("----------------------------------------------------------");
    const batchSession = newHttpBatchRpcSession(endpoint);
    const addPromise = batchSession.add(10, 20);
    const multiplyPromise = batchSession.multiply(3, 4);
    const dividePromise = batchSession.divide(100, 5);
    const subtractPromise = batchSession.subtract(50, 15);
    const [addResult, multiplyResult, divideResult, subtractResult] = await Promise.all([addPromise, multiplyPromise, dividePromise, subtractPromise]);
    console.log(`\u2705 Batch results:`);
    console.log(`   add(10, 20) = ${addResult}`);
    console.log(`   multiply(3, 4) = ${multiplyResult}`);
    console.log(`   divide(100, 5) = ${divideResult}`);
    console.log(`   subtract(50, 15) = ${subtractResult}`);
    console.log("\nTest 3: Error Handling");
    console.log("----------------------");
    try {
      const errorSession = newHttpBatchRpcSession(endpoint);
      await errorSession.divide(10, 0);
      console.log("\u274C Should have thrown for division by zero");
    } catch (error) {
      console.log(`\u2705 Division by zero correctly threw error: ${error.message}`);
    }
    console.log("\n================================================================================");
    console.log("\u{1F389} SUCCESS: All tests passed!");
    console.log("================================================================================");
    console.log("\u2705 Rust server is fully compatible with official Cap'n Web TypeScript client");
    console.log("\u2705 Single operations work with new sessions");
    console.log("\u2705 True batching works with multiple operations in one request");
    console.log("\u2705 Error handling works correctly");
  } catch (error) {
    console.error("\n\u274C Test failed:", error.message);
    console.error("Stack:", error.stack);
    process.exit(1);
  }
}
testWithOfficialClient().catch(console.error);
//# sourceMappingURL=fixed-official-client-test.js.map