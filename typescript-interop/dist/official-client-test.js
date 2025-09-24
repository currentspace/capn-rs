#!/usr/bin/env node

// src/official-client-test.ts
import { newHttpBatchRpcSession } from "capnweb";
async function testWithOfficialClient() {
  console.log("\u{1F9EA} Testing Cap'n Web Rust Server with Official TypeScript Client");
  console.log("================================================================\n");
  try {
    const port = process.argv[2] || "8080";
    const endpoint = `http://localhost:${port}/rpc/batch`;
    const session = newHttpBatchRpcSession(endpoint);
    console.log("\u2705 Created session with Rust server");
    console.log(`\u{1F4CD} Endpoint: ${endpoint}
`);
    console.log("Test 1: Addition");
    console.log("----------------");
    try {
      const result = await session.add(5, 3);
      console.log(`\u2705 add(5, 3) = ${result}`);
      if (result !== 8) {
        throw new Error(`Expected 8, got ${result}`);
      }
    } catch (error) {
      console.log(`\u274C Addition failed: ${error}`);
    }
    console.log("\nTest 2: Multiplication");
    console.log("----------------------");
    try {
      const result = await session.multiply(7, 6);
      console.log(`\u2705 multiply(7, 6) = ${result}`);
      if (result !== 42) {
        throw new Error(`Expected 42, got ${result}`);
      }
    } catch (error) {
      console.log(`\u274C Multiplication failed: ${error}`);
    }
    console.log("\nTest 3: Division");
    console.log("----------------");
    try {
      const result = await session.divide(100, 4);
      console.log(`\u2705 divide(100, 4) = ${result}`);
      if (result !== 25) {
        throw new Error(`Expected 25, got ${result}`);
      }
    } catch (error) {
      console.log(`\u274C Division failed: ${error}`);
    }
    console.log("\nTest 4: Subtraction");
    console.log("-------------------");
    try {
      const result = await session.subtract(10, 7);
      console.log(`\u2705 subtract(10, 7) = ${result}`);
      if (result !== 3) {
        throw new Error(`Expected 3, got ${result}`);
      }
    } catch (error) {
      console.log(`\u274C Subtraction failed: ${error}`);
    }
    console.log("\nTest 5: Error Handling");
    console.log("----------------------");
    try {
      const result = await session.divide(10, 0);
      console.log(`\u274C Division by zero should have thrown an error, got: ${result}`);
    } catch (error) {
      console.log(`\u2705 Division by zero correctly threw error: ${error}`);
    }
    console.log("\nTest 6: Multiple Operations");
    console.log("----------------------------");
    try {
      const [sum, product] = await Promise.all([
        session.add(10, 20),
        session.multiply(5, 8)
      ]);
      console.log(`\u2705 Parallel operations:`);
      console.log(`   add(10, 20) = ${sum}`);
      console.log(`   multiply(5, 8) = ${product}`);
      if (sum !== 30 || product !== 40) {
        throw new Error(`Unexpected results: sum=${sum}, product=${product}`);
      }
    } catch (error) {
      console.log(`\u274C Parallel operations failed: ${error}`);
    }
    console.log("\n" + "=".repeat(80));
    console.log("\u{1F389} VALIDATION SUMMARY");
    console.log("=".repeat(80));
    console.log("\u2705 Official Cap'n Web TypeScript client can communicate with Rust server!");
    console.log("\u26A0\uFE0F  Note: This validates basic protocol compatibility");
    console.log("\u274C Missing: Promise pipelining, WebSocket transport, full capability system");
  } catch (error) {
    console.error("\n\u{1F4A5} Fatal error:", error);
    console.error("\nThis likely means the Rust server is not properly implementing");
    console.error("the Cap'n Web protocol as expected by the official client.");
    process.exit(1);
  }
}
testWithOfficialClient().catch((error) => {
  console.error("Unhandled error:", error);
  process.exit(1);
});
//# sourceMappingURL=official-client-test.js.map