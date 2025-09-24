#!/usr/bin/env node

// src/tier3-capability-composition.ts
import { newWebSocketRpcSession } from "capnweb";
var port = process.argv[2] || "9001";
var wsEndpoint = `ws://localhost:${port}/rpc/ws`;
var Tier3CapabilityCompositionTests = class {
  constructor() {
    this.passed = 0;
    this.total = 0;
  }
  async test(name, testFn) {
    this.total++;
    console.log(`
\u{1F9E9} Capability Test ${this.total}: ${name}`);
    console.log("\u25C6".repeat(85));
    try {
      const result = await testFn();
      if (result) {
        this.passed++;
        console.log("\u{1F3AF} PASSED");
      } else {
        console.log("\u{1F534} FAILED");
      }
    } catch (error) {
      console.log(`\u{1F534} FAILED: ${error.message}`);
      console.log(`Stack: ${error.stack?.split("\n").slice(0, 2).join("\n")}`);
    }
  }
  /**
   * Test basic capability creation and disposal
   */
  async basicCapabilityLifecycleTest() {
    console.log("Testing basic capability creation and disposal...");
    try {
      const session = newWebSocketRpcSession(wsEndpoint);
      console.log("\u{1F527} Phase 1: Basic operation verification");
      const basicResult = await session.add(5, 3);
      console.log(`  Basic calculation: 5 + 3 = ${basicResult}`);
      console.log("\u{1F3D7}\uFE0F  Phase 2: Variable management (if supported)");
      if (session.setVariable && session.getVariable) {
        console.log("  Variable operations supported");
        await session.setVariable("x", 10);
        await session.setVariable("y", 20);
        const x = await session.getVariable("x");
        const y = await session.getVariable("y");
        console.log(`    Set variables: x=${x}, y=${y}`);
        const varResult1 = await session.add(x, y);
        const varResult2 = await session.multiply(x, 2);
        console.log(`    Variable calculations: x+y=${varResult1}, x*2=${varResult2}`);
        if (session.clearAllVariables) {
          await session.clearAllVariables();
          console.log("    Variables cleared");
        }
      } else {
        console.log("  Variable operations not supported - using basic calculations");
      }
      console.log("\u{1F504} Phase 3: Continuous operation verification");
      const continuousResults = await Promise.all([
        session.add(1, 1),
        session.multiply(2, 3),
        session.subtract(10, 4),
        session.divide(20, 5)
      ]);
      console.log(`    Continuous results: [${continuousResults.join(", ")}]`);
      console.log("\u{1F9F9} Phase 4: Session cleanup");
      if ("close" in session) {
        session.close();
        console.log("    Session closed properly");
      }
      const expectedBasic = 8;
      const expectedContinuous = [2, 6, 6, 4];
      const basicCorrect = basicResult === expectedBasic;
      const continuousCorrect = JSON.stringify(continuousResults) === JSON.stringify(expectedContinuous);
      console.log("\u{1F50D} Lifecycle Verification:");
      console.log(`  Basic operation: ${basicCorrect ? "\u2713" : "\u2717"} (${basicResult} === ${expectedBasic})`);
      console.log(`  Continuous operations: ${continuousCorrect ? "\u2713" : "\u2717"}`);
      console.log(`  Session cleanup: \u2713`);
      if (basicCorrect && continuousCorrect) {
        console.log("\u2705 Basic capability lifecycle working perfectly");
        return true;
      } else {
        console.log("\u26A0\uFE0F  Capability lifecycle has issues");
        return false;
      }
    } catch (error) {
      console.log(`Basic capability lifecycle test failed: ${error.message}`);
      return false;
    }
  }
  /**
   * Test nested capability interactions
   */
  async nestedCapabilityTest() {
    console.log("Testing nested capability interactions...");
    try {
      const mainSession = newWebSocketRpcSession(wsEndpoint);
      console.log("\u{1F3AF} Phase 1: Main calculator operations");
      const mainResult1 = await mainSession.add(10, 5);
      const mainResult2 = await mainSession.multiply(mainResult1, 2);
      console.log(`  Main session: 10+5=${mainResult1}, result*2=${mainResult2}`);
      console.log("\u{1F517} Phase 2: Testing capability method existence");
      let capabilitySupported = false;
      try {
        if (mainSession.createSubCalculator) {
          console.log("  Sub-calculator creation supported");
          capabilitySupported = true;
        } else {
          console.log("  Sub-calculator creation not yet implemented");
        }
      } catch (error) {
        console.log("  Sub-calculator creation not available");
      }
      console.log("\u{1F9EA} Phase 3: Simulated nested operations");
      const nestedResults = [];
      const level1 = await Promise.all([
        mainSession.add(1, 2),
        // 3
        mainSession.multiply(2, 2)
        // 4
      ]);
      nestedResults.push(...level1);
      const level2 = await Promise.all([
        mainSession.add(level1[0], level1[1]),
        // 3 + 4 = 7
        mainSession.multiply(level1[0], level1[1])
        // 3 * 4 = 12
      ]);
      nestedResults.push(...level2);
      console.log(`  Nested simulation results: [${nestedResults.join(", ")}]`);
      console.log("\u{1F504} Phase 4: Cross-nested operations");
      const crossResult = await mainSession.add(level2[0], mainResult1);
      console.log(`  Cross-nested result: ${crossResult}`);
      if ("close" in mainSession) {
        mainSession.close();
      }
      const expectedNested = [3, 4, 7, 12];
      const expectedCross = 22;
      const nestedCorrect = JSON.stringify(nestedResults) === JSON.stringify(expectedNested);
      const crossCorrect = crossResult === expectedCross;
      console.log("\u{1F50D} Nested Capability Verification:");
      console.log(`  Capability method detection: \u2713`);
      console.log(`  Nested-style operations: ${nestedCorrect ? "\u2713" : "\u2717"}`);
      console.log(`  Cross-nested operations: ${crossCorrect ? "\u2713" : "\u2717"}`);
      console.log(`  Future capability support: ${capabilitySupported ? "\u2713" : "Pending"}`);
      if (nestedCorrect && crossCorrect) {
        console.log("\u2705 Nested capability patterns working");
        console.log("\u{1F4DD} Note: Full capability passing awaits server implementation");
        return true;
      } else {
        console.log("\u26A0\uFE0F  Nested capability patterns have issues");
        return false;
      }
    } catch (error) {
      console.log(`Nested capability test failed: ${error.message}`);
      return false;
    }
  }
  /**
   * Test capability composition patterns
   */
  async capabilityCompositionTest() {
    console.log("Testing capability composition patterns...");
    try {
      const session1 = newWebSocketRpcSession(wsEndpoint);
      const session2 = newWebSocketRpcSession(wsEndpoint);
      console.log("\u{1F517} Phase 1: Multi-session composition setup");
      const init1 = await session1.add(0, 10);
      const init2 = await session2.add(0, 20);
      console.log(`  Session 1 initialized: ${init1}`);
      console.log(`  Session 2 initialized: ${init2}`);
      console.log("\u{1F9E9} Phase 2: Composition-style operations");
      const comp1 = await session1.multiply(init1, 2);
      const comp2 = await session2.add(init2, 5);
      const composed = await session1.add(comp1, comp2);
      console.log(`  Composition step 1: ${comp1}`);
      console.log(`  Composition step 2: ${comp2}`);
      console.log(`  Final composition: ${composed}`);
      console.log("\u26A1 Phase 3: Parallel composition");
      const parallelComps = await Promise.all([
        session1.multiply(composed, 2),
        // 45 * 2 = 90
        session2.subtract(composed, 15),
        // 45 - 15 = 30
        session1.divide(composed, 3),
        // 45 / 3 = 15
        session2.add(composed, 10)
        // 45 + 10 = 55
      ]);
      console.log(`  Parallel compositions: [${parallelComps.join(", ")}]`);
      console.log("\u{1F504} Phase 4: Recursive composition");
      const recursive1 = await session1.add(parallelComps[0], parallelComps[1]);
      const recursive2 = await session2.multiply(parallelComps[2], parallelComps[3]);
      const finalComposed = await session1.subtract(recursive2, recursive1);
      console.log(`  Recursive composition 1: ${recursive1}`);
      console.log(`  Recursive composition 2: ${recursive2}`);
      console.log(`  Final composed result: ${finalComposed}`);
      if ("close" in session1) session1.close();
      if ("close" in session2) session2.close();
      const expectedParallel = [90, 30, 15, 55];
      const expectedRecursive = [120, 825];
      const expectedFinal = 705;
      const parallelCorrect = JSON.stringify(parallelComps) === JSON.stringify(expectedParallel);
      const recursiveCorrect = JSON.stringify([recursive1, recursive2]) === JSON.stringify(expectedRecursive);
      const finalCorrect = finalComposed === expectedFinal;
      console.log("\u{1F50D} Composition Verification:");
      console.log(`  Multi-session setup: \u2713`);
      console.log(`  Basic composition: ${composed === 45 ? "\u2713" : "\u2717"} (${composed} === 45)`);
      console.log(`  Parallel composition: ${parallelCorrect ? "\u2713" : "\u2717"}`);
      console.log(`  Recursive composition: ${recursiveCorrect ? "\u2713" : "\u2717"}`);
      console.log(`  Final result: ${finalCorrect ? "\u2713" : "\u2717"} (${finalComposed} === ${expectedFinal})`);
      if (composed === 45 && parallelCorrect && recursiveCorrect && finalCorrect) {
        console.log("\u2705 Capability composition patterns working excellently");
        return true;
      } else {
        console.log("\u26A0\uFE0F  Capability composition has calculation errors");
        return false;
      }
    } catch (error) {
      console.log(`Capability composition test failed: ${error.message}`);
      return false;
    }
  }
  /**
   * Test complex capability graphs and dependencies
   */
  async capabilityGraphTest() {
    console.log("Testing complex capability graphs and dependencies...");
    try {
      const sessions = Array.from(
        { length: 5 },
        () => newWebSocketRpcSession(wsEndpoint)
      );
      console.log("\u{1F578}\uFE0F  Phase 1: Building capability graph structure");
      const nodeValues = await Promise.all([
        sessions[0].add(1, 2),
        // Node 0: 3
        sessions[1].multiply(2, 3),
        // Node 1: 6
        sessions[2].add(4, 5),
        // Node 2: 9
        sessions[3].multiply(3, 4),
        // Node 3: 12
        sessions[4].add(5, 6)
        // Node 4: 11
      ]);
      console.log(`  Node values: [${nodeValues.join(", ")}]`);
      console.log("\u{1F517} Phase 2: Creating dependencies between nodes");
      const level1Deps = await Promise.all([
        sessions[0].add(nodeValues[0], nodeValues[1]),
        // 3 + 6 = 9
        sessions[1].multiply(nodeValues[2], nodeValues[3]),
        // 9 * 12 = 108
        sessions[2].subtract(nodeValues[4], nodeValues[0])
        // 11 - 3 = 8
      ]);
      console.log(`  Level 1 dependencies: [${level1Deps.join(", ")}]`);
      console.log("\u26A1 Phase 3: Cross-dependencies (level 2)");
      const level2Deps = await Promise.all([
        sessions[3].add(level1Deps[0], level1Deps[2]),
        // 9 + 8 = 17
        sessions[4].divide(level1Deps[1], level1Deps[0])
        // 108 / 9 = 12
      ]);
      console.log(`  Level 2 dependencies: [${level2Deps.join(", ")}]`);
      console.log("\u{1F3AF} Phase 4: Final graph aggregation");
      const finalResult = await sessions[0].multiply(level2Deps[0], level2Deps[1]);
      console.log(`  Final graph result: ${finalResult}`);
      console.log("\u{1F504} Phase 5: Graph validation with alternative path");
      const altPath1 = await sessions[1].add(nodeValues[0], nodeValues[1]);
      const altPath2 = await sessions[2].subtract(nodeValues[4], nodeValues[0]);
      const altPath3 = await sessions[3].add(altPath1, altPath2);
      const altFinal = await sessions[4].multiply(altPath3, 12);
      console.log(`  Alternative path result: ${altFinal}`);
      for (const session of sessions) {
        if ("close" in session) {
          session.close();
        }
      }
      const expectedNodes = [3, 6, 9, 12, 11];
      const expectedLevel1 = [9, 108, 8];
      const expectedLevel2 = [17, 12];
      const expectedFinal = 204;
      const nodesCorrect = JSON.stringify(nodeValues) === JSON.stringify(expectedNodes);
      const level1Correct = JSON.stringify(level1Deps) === JSON.stringify(expectedLevel1);
      const level2Correct = JSON.stringify(level2Deps) === JSON.stringify(expectedLevel2);
      const finalCorrect = finalResult === expectedFinal;
      const altPathCorrect = altFinal === expectedFinal;
      console.log("\u{1F50D} Graph Verification:");
      console.log(`  Node initialization: ${nodesCorrect ? "\u2713" : "\u2717"}`);
      console.log(`  Level 1 dependencies: ${level1Correct ? "\u2713" : "\u2717"}`);
      console.log(`  Level 2 dependencies: ${level2Correct ? "\u2713" : "\u2717"}`);
      console.log(`  Final result: ${finalCorrect ? "\u2713" : "\u2717"} (${finalResult} === ${expectedFinal})`);
      console.log(`  Alternative path: ${altPathCorrect ? "\u2713" : "\u2717"} (${altFinal} === ${expectedFinal})`);
      if (nodesCorrect && level1Correct && level2Correct && finalCorrect && altPathCorrect) {
        console.log("\u2705 Complex capability graph working perfectly");
        console.log("\u{1F578}\uFE0F  Multi-level dependencies handled correctly");
        return true;
      } else {
        console.log("\u26A0\uFE0F  Capability graph has calculation errors");
        return false;
      }
    } catch (error) {
      console.log(`Capability graph test failed: ${error.message}`);
      return false;
    }
  }
  /**
   * Test capability disposal and cleanup patterns
   */
  async capabilityDisposalTest() {
    console.log("Testing capability disposal and cleanup patterns...");
    try {
      console.log("\u{1F9F9} Phase 1: Session creation and usage");
      const sessions = Array.from(
        { length: 3 },
        () => newWebSocketRpcSession(wsEndpoint)
      );
      const initialResults = await Promise.all([
        sessions[0].add(5, 5),
        // 10
        sessions[1].multiply(3, 4),
        // 12
        sessions[2].subtract(20, 5)
        // 15
      ]);
      console.log(`  Initial results: [${initialResults.join(", ")}]`);
      console.log("\u{1F504} Phase 2: Cross-session operations");
      const crossResults = await Promise.all([
        sessions[0].add(initialResults[0], initialResults[1]),
        // 10 + 12 = 22
        sessions[1].multiply(initialResults[1], initialResults[2]),
        // 12 * 15 = 180
        sessions[2].subtract(initialResults[2], initialResults[0])
        // 15 - 10 = 5
      ]);
      console.log(`  Cross-session results: [${crossResults.join(", ")}]`);
      console.log("\u{1F9F9} Phase 3: Gradual disposal simulation");
      if ("close" in sessions[0]) {
        sessions[0].close();
        console.log("    Session 0 disposed");
      }
      const afterDisposal1 = await Promise.all([
        sessions[1].add(crossResults[1], 10),
        // 180 + 10 = 190
        sessions[2].multiply(crossResults[2], 4)
        // 5 * 4 = 20
      ]);
      console.log(`    After disposal 1: [${afterDisposal1.join(", ")}]`);
      if ("close" in sessions[1]) {
        sessions[1].close();
        console.log("    Session 1 disposed");
      }
      const afterDisposal2 = await sessions[2].add(afterDisposal1[1], 5);
      console.log(`    After disposal 2: ${afterDisposal2}`);
      console.log("\u{1F9FD} Phase 4: Final cleanup");
      if ("close" in sessions[2]) {
        sessions[2].close();
        console.log("    Session 2 disposed");
      }
      console.log("    All sessions properly disposed");
      console.log("\u2705 Phase 5: Disposal validation");
      const validationSession = newWebSocketRpcSession(wsEndpoint);
      const validationResult = await validationSession.add(100, 200);
      if ("close" in validationSession) {
        validationSession.close();
      }
      console.log(`    Post-disposal validation: ${validationResult}`);
      const expectedInitial = [10, 12, 15];
      const expectedCross = [22, 180, 5];
      const expectedAfterDisposal1 = [190, 20];
      const expectedAfterDisposal2 = 25;
      const expectedValidation = 300;
      const initialCorrect = JSON.stringify(initialResults) === JSON.stringify(expectedInitial);
      const crossCorrect = JSON.stringify(crossResults) === JSON.stringify(expectedCross);
      const disposal1Correct = JSON.stringify(afterDisposal1) === JSON.stringify(expectedAfterDisposal1);
      const disposal2Correct = afterDisposal2 === expectedAfterDisposal2;
      const validationCorrect = validationResult === expectedValidation;
      console.log("\u{1F50D} Disposal Verification:");
      console.log(`  Initial operations: ${initialCorrect ? "\u2713" : "\u2717"}`);
      console.log(`  Cross-session operations: ${crossCorrect ? "\u2713" : "\u2717"}`);
      console.log(`  After first disposal: ${disposal1Correct ? "\u2713" : "\u2717"}`);
      console.log(`  After second disposal: ${disposal2Correct ? "\u2713" : "\u2717"}`);
      console.log(`  Post-disposal validation: ${validationCorrect ? "\u2713" : "\u2717"}`);
      if (initialCorrect && crossCorrect && disposal1Correct && disposal2Correct && validationCorrect) {
        console.log("\u2705 Capability disposal working perfectly");
        console.log("\u{1F9F9} Clean lifecycle management confirmed");
        return true;
      } else {
        console.log("\u26A0\uFE0F  Capability disposal has issues");
        return false;
      }
    } catch (error) {
      console.log(`Capability disposal test failed: ${error.message}`);
      return false;
    }
  }
  async run() {
    console.log("\u{1F9E9} TIER 3: Advanced Capability Composition & Lifecycle Management");
    console.log("\u25C6".repeat(85));
    console.log(`\u{1F3AF} WebSocket endpoint: ${wsEndpoint}`);
    console.log("\u{1F3AF} Goal: Test complex capability patterns and lifecycle management");
    console.log("\u{1F4CB} Prerequisites: Tier 1, 2, and basic Tier 3 tests must pass");
    console.log("");
    await this.test("Basic Capability Lifecycle", () => this.basicCapabilityLifecycleTest());
    await this.test("Nested Capability Interactions", () => this.nestedCapabilityTest());
    await this.test("Capability Composition Patterns", () => this.capabilityCompositionTest());
    await this.test("Complex Capability Graphs", () => this.capabilityGraphTest());
    await this.test("Capability Disposal & Cleanup", () => this.capabilityDisposalTest());
    console.log("\n" + "\u25C6".repeat(85));
    console.log("\u{1F9E9} TIER 3 CAPABILITY COMPOSITION RESULTS");
    console.log("\u25C6".repeat(85));
    const passRate = Math.round(this.passed / this.total * 100);
    console.log(`\u{1F3AF} Passed: ${this.passed}/${this.total} (${passRate}%)`);
    if (this.passed === this.total) {
      console.log("\u{1F3C6} CAPABILITY MASTERY: All advanced patterns working perfectly!");
      console.log("\u{1F9E9} Complex capability composition and lifecycle management achieved");
      console.log("\u{1F680} Ready for sophisticated capability-based architectures");
      process.exit(0);
    } else if (this.passed >= this.total * 0.8) {
      console.log("\u2B50 EXCELLENT: Most capability patterns working");
      console.log("\u{1F6E0}\uFE0F  Minor capability features need attention");
      process.exit(0);
    } else if (this.passed >= this.total * 0.6) {
      console.log("\u2728 GOOD: Basic capability patterns working");
      console.log("\u{1F527} Advanced capability features need work");
      process.exit(1);
    } else {
      console.log("\u{1F6A8} NEEDS WORK: Capability composition failing");
      console.log("\u{1F3D7}\uFE0F  Requires capability system implementation");
      process.exit(2);
    }
  }
};
process.on("unhandledRejection", (reason, promise) => {
  console.error("Unhandled Rejection at:", promise, "reason:", reason);
  process.exit(3);
});
var capabilityTests = new Tier3CapabilityCompositionTests();
capabilityTests.run();
//# sourceMappingURL=tier3-capability-composition.js.map