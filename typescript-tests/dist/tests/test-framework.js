import 'node:test';
import assert from 'node:assert';

var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });
class InteropTestFramework {
  static {
    __name(this, "InteropTestFramework");
  }
  logger;
  suites = [];
  constructor(logger) {
    this.logger = logger || console;
  }
  async runTestSuite(name, testFunctions) {
    this.logger.info(`
\u{1F680} Running test suite: ${name}`);
    this.logger.info("=".repeat(50));
    const suite = {
      name,
      results: [],
      totalPassed: 0,
      totalFailed: 0,
      totalDuration: 0
    };
    for (let i = 0; i < testFunctions.length; i++) {
      const testFunc = testFunctions[i];
      const testName = testFunc.name || `Test ${i + 1}`;
      const result = await this.runSingleTest(testName, testFunc);
      suite.results.push(result);
      if (result.passed) {
        suite.totalPassed++;
      } else {
        suite.totalFailed++;
      }
      suite.totalDuration += result.duration;
    }
    this.suites.push(suite);
    this.logSuiteResults(suite);
    return suite;
  }
  async runSingleTest(name, testFunc) {
    const startTime = Date.now();
    try {
      this.logger.info(`\u{1F4CB} Running: ${name}`);
      await testFunc();
      const duration = Date.now() - startTime;
      this.logger.info(`\u2705 PASSED: ${name} (${duration}ms)`);
      return {
        name,
        passed: true,
        duration
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      const testError = error instanceof Error ? error : new Error(String(error));
      this.logger.error(`\u274C FAILED: ${name} (${duration}ms)`);
      this.logger.error(`   Error: ${testError.message}`);
      if (testError.stack) {
        this.logger.error(`   Stack: ${testError.stack}`);
      }
      return {
        name,
        passed: false,
        error: testError,
        duration
      };
    }
  }
  logSuiteResults(suite) {
    this.logger.info("\n\u{1F4CA} Test Suite Results:");
    this.logger.info(`   Total: ${suite.results.length}`);
    this.logger.info(`   Passed: ${suite.totalPassed}`);
    this.logger.info(`   Failed: ${suite.totalFailed}`);
    this.logger.info(`   Duration: ${suite.totalDuration}ms`);
    if (suite.totalFailed > 0) {
      this.logger.error("\n\u{1F525} Failed Tests:");
      suite.results.filter((r) => !r.passed).forEach((r) => {
        this.logger.error(`   - ${r.name}: ${r.error?.message}`);
      });
    }
  }
  generateReport() {
    this.logger.info("\n\u{1F3AF} INTEROPERABILITY TEST REPORT");
    this.logger.info("=".repeat(60));
    let totalTests = 0;
    let totalPassed = 0;
    let totalFailed = 0;
    let totalDuration = 0;
    for (const suite of this.suites) {
      totalTests += suite.results.length;
      totalPassed += suite.totalPassed;
      totalFailed += suite.totalFailed;
      totalDuration += suite.totalDuration;
      this.logger.info(`
\u{1F4E6} ${suite.name}:`);
      this.logger.info(`   Tests: ${suite.results.length}`);
      this.logger.info(`   Passed: ${suite.totalPassed}`);
      this.logger.info(`   Failed: ${suite.totalFailed}`);
      this.logger.info(`   Duration: ${suite.totalDuration}ms`);
    }
    this.logger.info("\n\u{1F3C6} OVERALL RESULTS:");
    this.logger.info(`   Total Tests: ${totalTests}`);
    this.logger.info(`   Passed: ${totalPassed}`);
    this.logger.info(`   Failed: ${totalFailed}`);
    this.logger.info(`   Success Rate: ${(totalPassed / totalTests * 100).toFixed(1)}%`);
    this.logger.info(`   Total Duration: ${totalDuration}ms`);
    if (totalFailed === 0) {
      this.logger.info("\n\u{1F389} ALL TESTS PASSED! TypeScript \u2194 Rust interoperability verified!");
    } else {
      this.logger.error(`
\u{1F4A5} ${totalFailed} tests failed. Interoperability issues detected.`);
    }
  }
  getAllResults() {
    return this.suites;
  }
  getOverallStats() {
    const totalTests = this.suites.reduce((sum, suite) => sum + suite.results.length, 0);
    const totalPassed = this.suites.reduce((sum, suite) => sum + suite.totalPassed, 0);
    const totalFailed = this.suites.reduce((sum, suite) => sum + suite.totalFailed, 0);
    const totalDuration = this.suites.reduce((sum, suite) => sum + suite.totalDuration, 0);
    return {
      totalTests,
      totalPassed,
      totalFailed,
      totalDuration,
      successRate: totalTests > 0 ? totalPassed / totalTests * 100 : 0
    };
  }
}
class InteropAssert {
  static {
    __name(this, "InteropAssert");
  }
  static equal(actual, expected, message) {
    assert.strictEqual(actual, expected, message);
  }
  static deepEqual(actual, expected, message) {
    assert.deepStrictEqual(actual, expected, message);
  }
  static ok(value, message) {
    assert.ok(value, message);
  }
  static throws(fn, message) {
    assert.throws(fn, message);
  }
  static async rejects(promise, message) {
    await assert.rejects(promise, message);
  }
  static async doesNotReject(promise, message) {
    await assert.doesNotReject(promise, message);
  }
  static isNumber(value, message) {
    assert.ok(typeof value === "number", message || `Expected number, got ${typeof value}`);
  }
  static isString(value, message) {
    assert.ok(typeof value === "string", message || `Expected string, got ${typeof value}`);
  }
  static isObject(value, message) {
    assert.ok(typeof value === "object" && value !== null, message || `Expected object, got ${typeof value}`);
  }
  static isArray(value, message) {
    assert.ok(Array.isArray(value), message || `Expected array, got ${typeof value}`);
  }
  static hasProperty(obj, property, message) {
    assert.ok(property in obj, message || `Expected object to have property '${property}'`);
  }
  static approximatelyEqual(actual, expected, tolerance = 1e-4, message) {
    const diff = Math.abs(actual - expected);
    assert.ok(
      diff <= tolerance,
      message || `Expected ${actual} to be approximately ${expected} (tolerance: ${tolerance}), diff: ${diff}`
    );
  }
}
class Timer {
  static {
    __name(this, "Timer");
  }
  startTime = 0;
  start() {
    this.startTime = Date.now();
  }
  elapsed() {
    return Date.now() - this.startTime;
  }
  static async measure(operation) {
    const start = Date.now();
    const result = await operation();
    const duration = Date.now() - start;
    return { result, duration };
  }
}
const wait = /* @__PURE__ */ __name((ms) => new Promise((resolve) => setTimeout(resolve, ms)), "wait");

export { InteropAssert, InteropTestFramework, Timer, wait };
//# sourceMappingURL=test-framework.js.map
//# sourceMappingURL=test-framework.js.map