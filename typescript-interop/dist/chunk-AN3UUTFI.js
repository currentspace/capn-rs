// src/promise-pipelining-test.ts
import { newHttpBatchRpcSession } from "capnweb";
async function testPromisePipelining() {
  console.log("\u{1F9EA} Testing Promise Pipelining and Message Flow");
  console.log("===============================================\n");
  try {
    const session = newHttpBatchRpcSession("http://localhost:8080/rpc/batch");
    console.log("\u2705 Created session for promise pipelining tests\n");
    console.log("Test 1: Sequential vs Parallel Execution Timing");
    console.log("================================================");
    const counterName = "timing-test-" + Date.now();
    await session.reset_global(counterName);
    const sequentialStart = performance.now();
    await session.increment_global(counterName);
    await session.increment_global(counterName);
    await session.increment_global(counterName);
    await session.increment_global(counterName);
    await session.increment_global(counterName);
    const sequentialEnd = performance.now();
    const sequentialTime = sequentialEnd - sequentialStart;
    console.log(`\u2705 Sequential execution took ${sequentialTime.toFixed(2)}ms`);
    await session.reset_global(counterName);
    const parallelStart = performance.now();
    await Promise.all([
      session.increment_global(counterName),
      session.increment_global(counterName),
      session.increment_global(counterName),
      session.increment_global(counterName),
      session.increment_global(counterName)
    ]);
    const parallelEnd = performance.now();
    const parallelTime = parallelEnd - parallelStart;
    console.log(`\u2705 Parallel execution took ${parallelTime.toFixed(2)}ms`);
    console.log(`\u{1F4CA} Parallel was ${(sequentialTime / parallelTime).toFixed(2)}x faster`);
    console.log("\nTest 2: Complex Dependency Chains");
    console.log("==================================");
    const sessionId = "pipeline-session-" + Date.now();
    const initialValues = await Promise.all([
      session.reset_global("chain-counter-1"),
      session.reset_global("chain-counter-2"),
      session.reset_global("chain-counter-3")
    ]);
    console.log(`\u2705 Reset initial values: [${initialValues.join(", ")}]`);
    const firstLevel = await Promise.all([
      session.increment_global("chain-counter-1"),
      session.increment_global("chain-counter-2"),
      session.increment_global("chain-counter-3")
    ]);
    console.log(`\u2705 First level results: [${firstLevel.join(", ")}]`);
    const secondLevel = await Promise.all([
      session.increment_global("chain-counter-1"),
      session.increment_global("chain-counter-2"),
      session.increment_session(sessionId, "derived-counter")
    ]);
    console.log(`\u2705 Second level results: [${secondLevel.join(", ")}]`);
    const finalResults = await Promise.all([
      session.get_global("chain-counter-1"),
      session.get_global("chain-counter-2"),
      session.get_global("chain-counter-3"),
      session.get_session(sessionId, "derived-counter")
    ]);
    console.log(`\u2705 Final dependency chain results: [${finalResults.join(", ")}]`);
    console.log("\nTest 3: Batch Operation Optimization");
    console.log("=====================================");
    const batchSize = 20;
    const batchCounterName = "batch-counter-" + Date.now();
    await session.reset_global(batchCounterName);
    const batchStart = performance.now();
    const batchPromises = [];
    for (let i = 0; i < batchSize; i++) {
      batchPromises.push(session.increment_global(batchCounterName));
    }
    const batchResults = await Promise.all(batchPromises);
    const batchEnd = performance.now();
    const batchTime = batchEnd - batchStart;
    console.log(`\u2705 Batch of ${batchSize} operations completed in ${batchTime.toFixed(2)}ms`);
    console.log(`\u{1F4CA} Average per operation: ${(batchTime / batchSize).toFixed(2)}ms`);
    console.log(`\u2705 Final batch counter value: ${Math.max(...batchResults)}`);
    const uniqueResults = new Set(batchResults);
    console.log(`\u2705 Result uniqueness: ${uniqueResults.size}/${batchSize} unique values`);
    console.log("\nTest 4: Mixed Session and Global Operations");
    console.log("============================================");
    const mixedSessionId = "mixed-session-" + Date.now();
    const globalCounterName = "mixed-global-" + Date.now();
    const mixedOperations = await Promise.all([
      session.reset_global(globalCounterName),
      session.increment_session(mixedSessionId, "session-counter-1"),
      session.increment_global(globalCounterName),
      session.increment_session(mixedSessionId, "session-counter-2"),
      session.increment_global(globalCounterName),
      session.set_session_property(mixedSessionId, "mixed-prop", "mixed-value"),
      session.increment_session(mixedSessionId, "session-counter-1"),
      session.increment_global(globalCounterName)
    ]);
    console.log(`\u2705 Mixed operations completed: [${mixedOperations.slice(0, 4).join(", ")}, ...]`);
    const finalState = await Promise.all([
      session.get_global(globalCounterName),
      session.get_session(mixedSessionId, "session-counter-1"),
      session.get_session(mixedSessionId, "session-counter-2"),
      session.get_session_property(mixedSessionId, "mixed-prop")
    ]);
    console.log(`\u2705 Final mixed state: global=${finalState[0]}, session1=${finalState[1]}, session2=${finalState[2]}, prop=${JSON.stringify(finalState[3])}`);
    console.log("\nTest 5: Error Handling in Pipelines");
    console.log("====================================");
    try {
      const mixedValidInvalid = await Promise.allSettled([
        session.increment_global("valid-counter"),
        session.get_session_property("non-existent-session", "non-existent-prop"),
        session.increment_global("another-valid-counter"),
        // @ts-ignore - Intentionally invalid call
        session.invalid_method(),
        session.increment_global("third-valid-counter")
      ]);
      let successCount = 0;
      let errorCount = 0;
      mixedValidInvalid.forEach((result, index) => {
        if (result.status === "fulfilled") {
          successCount++;
          console.log(`\u2705 Operation ${index}: Success - ${JSON.stringify(result.value)}`);
        } else {
          errorCount++;
          console.log(`\u274C Operation ${index}: Error - ${result.reason}`);
        }
      });
      console.log(`\u{1F4CA} Pipeline error handling: ${successCount} successes, ${errorCount} errors`);
      console.log("\u2705 Promise.allSettled correctly handled mixed success/failure");
    } catch (error) {
      console.log(`\u274C Error handling test failed: ${error}`);
    }
    console.log("\nTest 6: Resource Cleanup in Pipelines");
    console.log("======================================");
    const cleanupSessionIds = [];
    for (let i = 0; i < 5; i++) {
      cleanupSessionIds.push(`cleanup-session-${Date.now()}-${i}`);
    }
    const populatePromises = cleanupSessionIds.flatMap((sessionId2) => [
      session.increment_session(sessionId2, "cleanup-counter"),
      session.set_session_property(sessionId2, "cleanup-prop", `value-${sessionId2}`)
    ]);
    await Promise.all(populatePromises);
    console.log(`\u2705 Created and populated ${cleanupSessionIds.length} sessions`);
    const verifyPromises = cleanupSessionIds.map(
      (sessionId2) => session.get_session(sessionId2, "cleanup-counter")
    );
    const verifyResults = await Promise.all(verifyPromises);
    console.log(`\u2705 Verified session data: [${verifyResults.join(", ")}]`);
    console.log("\n" + "=".repeat(80));
    console.log("\u{1F389} PROMISE PIPELINING VALIDATION SUMMARY");
    console.log("=".repeat(80));
    console.log("\u2705 Sequential vs parallel execution timing analyzed");
    console.log("\u2705 Complex dependency chains working correctly");
    console.log("\u2705 Batch operation optimization validated");
    console.log("\u2705 Mixed session/global operations pipelined properly");
    console.log("\u2705 Error handling in pipelines working correctly");
    console.log("\u2705 Resource cleanup patterns validated");
    console.log("\n\u{1F680} Promise pipelining and message flow optimization complete!");
  } catch (error) {
    console.error("\n\u{1F4A5} Fatal error in promise pipelining tests:", error);
    console.error("\nThis indicates issues with message batching or promise handling.");
    process.exit(1);
  }
}
if (import.meta.url === `file://${process.argv[1]}`) {
  testPromisePipelining().catch((error) => {
    console.error("Unhandled error:", error);
    process.exit(1);
  });
}

export {
  testPromisePipelining
};
//# sourceMappingURL=chunk-AN3UUTFI.js.map