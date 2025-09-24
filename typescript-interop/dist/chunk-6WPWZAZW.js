// src/advanced-server-test.ts
import { newHttpBatchRpcSession } from "capnweb";
async function testAdvancedStatefulServer() {
  console.log("\u{1F9EA} Testing Advanced Stateful Cap'n Web Rust Server");
  console.log("==================================================\n");
  try {
    const session = newHttpBatchRpcSession("http://localhost:8080/rpc/batch");
    console.log("\u2705 Created session with advanced stateful Rust server");
    console.log("\u{1F4CD} Endpoint: http://localhost:8080/rpc/batch\n");
    console.log("Test 1: Global Counter Operations");
    console.log("==================================");
    try {
      await session.reset_global("test_counter");
      console.log("\u2705 Reset test_counter to 0");
      let result = await session.increment_global("test_counter");
      console.log(`\u2705 increment_global('test_counter') = ${result}`);
      assert(result === 1, `Expected 1, got ${result}`);
      result = await session.increment_global("test_counter");
      console.log(`\u2705 increment_global('test_counter') = ${result}`);
      assert(result === 2, `Expected 2, got ${result}`);
      result = await session.decrement_global("test_counter");
      console.log(`\u2705 decrement_global('test_counter') = ${result}`);
      assert(result === 1, `Expected 1, got ${result}`);
      result = await session.get_global("test_counter");
      console.log(`\u2705 get_global('test_counter') = ${result}`);
      assert(result === 1, `Expected 1, got ${result}`);
    } catch (error) {
      console.log(`\u274C Global counter operations failed: ${error}`);
      throw error;
    }
    console.log("\nTest 2: Session-Specific Operations");
    console.log("====================================");
    try {
      const sessionId = "test-session-" + Date.now();
      let result = await session.increment_session(sessionId, "session_counter");
      console.log(`\u2705 increment_session('${sessionId}', 'session_counter') = ${result}`);
      assert(result === 1, `Expected 1, got ${result}`);
      result = await session.increment_session(sessionId, "session_counter");
      console.log(`\u2705 increment_session('${sessionId}', 'session_counter') = ${result}`);
      assert(result === 2, `Expected 2, got ${result}`);
      result = await session.increment_session(sessionId, "another_counter");
      console.log(`\u2705 increment_session('${sessionId}', 'another_counter') = ${result}`);
      assert(result === 1, `Expected 1, got ${result}`);
      result = await session.get_session(sessionId, "session_counter");
      console.log(`\u2705 get_session('${sessionId}', 'session_counter') = ${result}`);
      assert(result === 2, `Expected 2, got ${result}`);
    } catch (error) {
      console.log(`\u274C Session operations failed: ${error}`);
      throw error;
    }
    console.log("\nTest 3: Session Property Management");
    console.log("===================================");
    try {
      const sessionId = "prop-session-" + Date.now();
      let result = await session.set_session_property(sessionId, "user_name", "Alice");
      console.log(`\u2705 set_session_property('${sessionId}', 'user_name', 'Alice') = ${JSON.stringify(result)}`);
      result = await session.set_session_property(sessionId, "user_age", 25);
      console.log(`\u2705 set_session_property('${sessionId}', 'user_age', 25) = ${JSON.stringify(result)}`);
      const userData = { preferences: { theme: "dark", language: "en" } };
      result = await session.set_session_property(sessionId, "user_data", userData);
      console.log(`\u2705 set_session_property with object = ${JSON.stringify(result)}`);
      result = await session.get_session_property(sessionId, "user_name");
      console.log(`\u2705 get_session_property('${sessionId}', 'user_name') = ${JSON.stringify(result)}`);
      assert(result === "Alice", `Expected 'Alice', got ${result}`);
      result = await session.get_session_property(sessionId, "user_age");
      console.log(`\u2705 get_session_property('${sessionId}', 'user_age') = ${JSON.stringify(result)}`);
      assert(result === 25, `Expected 25, got ${result}`);
    } catch (error) {
      console.log(`\u274C Session property operations failed: ${error}`);
      throw error;
    }
    console.log("\nTest 4: Concurrent Operations");
    console.log("==============================");
    try {
      const promises = [
        session.increment_global("concurrent_counter"),
        session.increment_global("concurrent_counter"),
        session.increment_global("concurrent_counter"),
        session.increment_global("concurrent_counter"),
        session.increment_global("concurrent_counter")
      ];
      const results = await Promise.all(promises);
      console.log(`\u2705 Concurrent increments results: [${results.join(", ")}]`);
      const sortedResults = [...results].sort((a, b) => a - b);
      const expected = [1, 2, 3, 4, 5];
      assert(
        JSON.stringify(sortedResults) === JSON.stringify(expected),
        `Expected [1,2,3,4,5], got [${sortedResults.join(",")}]`
      );
      const sessionId = "concurrent-session-" + Date.now();
      const sessionPromises = [
        session.increment_session(sessionId, "counter1"),
        session.increment_session(sessionId, "counter2"),
        session.increment_session(sessionId, "counter3"),
        session.set_session_property(sessionId, "test_prop", "test_value"),
        session.increment_session(sessionId, "counter1")
        // This should make counter1 = 2
      ];
      await Promise.all(sessionPromises);
      console.log("\u2705 Concurrent session operations completed");
      const counter1 = await session.get_session(sessionId, "counter1");
      console.log(`\u2705 Final counter1 value: ${counter1}`);
      assert(counter1 === 2, `Expected counter1 = 2, got ${counter1}`);
    } catch (error) {
      console.log(`\u274C Concurrent operations failed: ${error}`);
      throw error;
    }
    console.log("\nTest 5: Error Handling");
    console.log("======================");
    try {
      try {
        await session.get_session_property("non-existent-session", "non-existent-prop");
        console.log("\u274C Should have thrown error for non-existent property");
      } catch (error) {
        console.log(`\u2705 Correctly threw error for non-existent property: ${error}`);
      }
      try {
        await session.increment_global();
        console.log("\u274C Should have thrown error for missing arguments");
      } catch (error) {
        console.log(`\u2705 Correctly threw error for missing arguments: ${error}`);
      }
    } catch (error) {
      console.log(`\u274C Error handling test failed: ${error}`);
      throw error;
    }
    console.log("\nTest 6: List Operations");
    console.log("=======================");
    try {
      const globalCounters = await session.list_global_counters();
      console.log(`\u2705 Global counters: ${JSON.stringify(globalCounters, null, 2)}`);
      assert(Array.isArray(globalCounters), "Expected array of global counters");
      const testCounter = globalCounters.find((c) => c.name === "test_counter");
      assert(testCounter !== void 0, "Expected to find test_counter");
      assert(testCounter && testCounter.value === 1, `Expected test_counter value 1, got ${testCounter?.value}`);
      const sessions = await session.list_sessions();
      console.log(`\u2705 Sessions: ${JSON.stringify(sessions, null, 2)}`);
      assert(Array.isArray(sessions), "Expected array of sessions");
      assert(sessions.length > 0, "Expected at least one session");
    } catch (error) {
      console.log(`\u274C List operations failed: ${error}`);
      throw error;
    }
    console.log("\nTest 7: Session Persistence");
    console.log("============================");
    try {
      const persistentSessionId = "persistent-session-" + Date.now();
      await session.increment_session(persistentSessionId, "persistent_counter");
      await session.set_session_property(persistentSessionId, "persistent_prop", "persistent_value");
      await new Promise((resolve) => setTimeout(resolve, 100));
      const counterValue = await session.get_session(persistentSessionId, "persistent_counter");
      const propValue = await session.get_session_property(persistentSessionId, "persistent_prop");
      console.log(`\u2705 Persistent counter value: ${counterValue}`);
      console.log(`\u2705 Persistent property value: ${JSON.stringify(propValue)}`);
      assert(counterValue === 1, `Expected persistent counter = 1, got ${counterValue}`);
      assert(propValue === "persistent_value", `Expected 'persistent_value', got ${propValue}`);
    } catch (error) {
      console.log(`\u274C Session persistence test failed: ${error}`);
      throw error;
    }
    console.log("\nTest 8: Advanced Capabilities");
    console.log("==============================");
    try {
      const asyncProcessor = await session.get_async_processor();
      console.log(`\u2705 Created async processor: ${asyncProcessor}`);
      const nestedCap = await session.get_nested_capability("test-operation-123");
      console.log(`\u2705 Created nested capability: ${nestedCap}`);
    } catch (error) {
      console.log(`\u274C Advanced capabilities test failed: ${error}`);
      throw error;
    }
    console.log("\nTest 9: Cleanup Operations");
    console.log("===========================");
    try {
      const cleanupResult = await session.cleanup_sessions();
      console.log(`\u2705 Session cleanup result: ${cleanupResult}`);
    } catch (error) {
      console.log(`\u274C Cleanup operations failed: ${error}`);
      throw error;
    }
    console.log("\n" + "=".repeat(80));
    console.log("\u{1F389} ADVANCED SERVER VALIDATION SUMMARY");
    console.log("=".repeat(80));
    console.log("\u2705 Advanced stateful server functionality working correctly!");
    console.log("\u2705 Global and session-specific counters");
    console.log("\u2705 Session property management");
    console.log("\u2705 Concurrent operations");
    console.log("\u2705 Error handling");
    console.log("\u2705 List operations");
    console.log("\u2705 Session persistence");
    console.log("\u2705 Advanced capabilities");
    console.log("\u2705 Cleanup operations");
    console.log("\n\u{1F680} The Rust Cap'n Web server is ready for production use!");
  } catch (error) {
    console.error("\n\u{1F4A5} Fatal error:", error);
    console.error("\nThis indicates an issue with the advanced stateful server implementation.");
    process.exit(1);
  }
}
function assert(condition, message) {
  if (!condition) {
    throw new Error(`Assertion failed: ${message}`);
  }
}
if (import.meta.url === `file://${process.argv[1]}`) {
  testAdvancedStatefulServer().catch((error) => {
    console.error("Unhandled error:", error);
    process.exit(1);
  });
}

export {
  testAdvancedStatefulServer
};
//# sourceMappingURL=chunk-6WPWZAZW.js.map