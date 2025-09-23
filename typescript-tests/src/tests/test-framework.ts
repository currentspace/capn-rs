/**
 * TypeScript Test Framework for Cap'n Web Interoperability
 *
 * A comprehensive test framework for verifying interoperability between
 * TypeScript and Rust Cap'n Web implementations.
 */

import { test, describe } from 'node:test'
import assert from 'node:assert'
import type { Logger } from '../capnweb/types.js'

export interface TestResult {
  name: string
  passed: boolean
  error?: Error
  duration: number
  details?: string
}

export interface TestSuite {
  name: string
  results: TestResult[]
  totalPassed: number
  totalFailed: number
  totalDuration: number
}

export class InteropTestFramework {
  private logger: Logger
  private suites: TestSuite[] = []

  constructor(logger?: Logger) {
    this.logger = logger || console
  }

  async runTestSuite(
    name: string,
    testFunctions: Array<() => Promise<void>>
  ): Promise<TestSuite> {
    this.logger.info(`\n🚀 Running test suite: ${name}`)
    this.logger.info('='.repeat(50))

    const suite: TestSuite = {
      name,
      results: [],
      totalPassed: 0,
      totalFailed: 0,
      totalDuration: 0
    }

    for (let i = 0; i < testFunctions.length; i++) {
      const testFunc = testFunctions[i]
      const testName = testFunc.name || `Test ${i + 1}`

      const result = await this.runSingleTest(testName, testFunc)
      suite.results.push(result)

      if (result.passed) {
        suite.totalPassed++
      } else {
        suite.totalFailed++
      }

      suite.totalDuration += result.duration
    }

    this.suites.push(suite)
    this.logSuiteResults(suite)

    return suite
  }

  private async runSingleTest(
    name: string,
    testFunc: () => Promise<void>
  ): Promise<TestResult> {
    const startTime = Date.now()

    try {
      this.logger.info(`📋 Running: ${name}`)
      await testFunc()
      const duration = Date.now() - startTime

      this.logger.info(`✅ PASSED: ${name} (${duration}ms)`)
      return {
        name,
        passed: true,
        duration
      }
    } catch (error) {
      const duration = Date.now() - startTime
      const testError = error instanceof Error ? error : new Error(String(error))

      this.logger.error(`❌ FAILED: ${name} (${duration}ms)`)
      this.logger.error(`   Error: ${testError.message}`)
      if (testError.stack) {
        this.logger.error(`   Stack: ${testError.stack}`)
      }

      return {
        name,
        passed: false,
        error: testError,
        duration
      }
    }
  }

  private logSuiteResults(suite: TestSuite): void {
    this.logger.info('\n📊 Test Suite Results:')
    this.logger.info(`   Total: ${suite.results.length}`)
    this.logger.info(`   Passed: ${suite.totalPassed}`)
    this.logger.info(`   Failed: ${suite.totalFailed}`)
    this.logger.info(`   Duration: ${suite.totalDuration}ms`)

    if (suite.totalFailed > 0) {
      this.logger.error('\n🔥 Failed Tests:')
      suite.results
        .filter(r => !r.passed)
        .forEach(r => {
          this.logger.error(`   - ${r.name}: ${r.error?.message}`)
        })
    }
  }

  generateReport(): void {
    this.logger.info('\n🎯 INTEROPERABILITY TEST REPORT')
    this.logger.info('='.repeat(60))

    let totalTests = 0
    let totalPassed = 0
    let totalFailed = 0
    let totalDuration = 0

    for (const suite of this.suites) {
      totalTests += suite.results.length
      totalPassed += suite.totalPassed
      totalFailed += suite.totalFailed
      totalDuration += suite.totalDuration

      this.logger.info(`\n📦 ${suite.name}:`)
      this.logger.info(`   Tests: ${suite.results.length}`)
      this.logger.info(`   Passed: ${suite.totalPassed}`)
      this.logger.info(`   Failed: ${suite.totalFailed}`)
      this.logger.info(`   Duration: ${suite.totalDuration}ms`)
    }

    this.logger.info('\n🏆 OVERALL RESULTS:')
    this.logger.info(`   Total Tests: ${totalTests}`)
    this.logger.info(`   Passed: ${totalPassed}`)
    this.logger.info(`   Failed: ${totalFailed}`)
    this.logger.info(`   Success Rate: ${((totalPassed / totalTests) * 100).toFixed(1)}%`)
    this.logger.info(`   Total Duration: ${totalDuration}ms`)

    if (totalFailed === 0) {
      this.logger.info('\n🎉 ALL TESTS PASSED! TypeScript ↔ Rust interoperability verified!')
    } else {
      this.logger.error(`\n💥 ${totalFailed} tests failed. Interoperability issues detected.`)
    }
  }

  getAllResults(): TestSuite[] {
    return this.suites
  }

  getOverallStats() {
    const totalTests = this.suites.reduce((sum, suite) => sum + suite.results.length, 0)
    const totalPassed = this.suites.reduce((sum, suite) => sum + suite.totalPassed, 0)
    const totalFailed = this.suites.reduce((sum, suite) => sum + suite.totalFailed, 0)
    const totalDuration = this.suites.reduce((sum, suite) => sum + suite.totalDuration, 0)

    return {
      totalTests,
      totalPassed,
      totalFailed,
      totalDuration,
      successRate: totalTests > 0 ? (totalPassed / totalTests) * 100 : 0
    }
  }
}

// Utility functions for assertions
export class InteropAssert {
  static equal(actual: unknown, expected: unknown, message?: string): void {
    assert.strictEqual(actual, expected, message)
  }

  static deepEqual(actual: unknown, expected: unknown, message?: string): void {
    assert.deepStrictEqual(actual, expected, message)
  }

  static ok(value: unknown, message?: string): void {
    assert.ok(value, message)
  }

  static throws(fn: () => void, message?: string): void {
    assert.throws(fn, message)
  }

  static async rejects(promise: Promise<unknown>, message?: string): Promise<void> {
    await assert.rejects(promise, message)
  }

  static async doesNotReject(promise: Promise<unknown>, message?: string): Promise<void> {
    await assert.doesNotReject(promise, message)
  }

  static isNumber(value: unknown, message?: string): void {
    assert.ok(typeof value === 'number', message || `Expected number, got ${typeof value}`)
  }

  static isString(value: unknown, message?: string): void {
    assert.ok(typeof value === 'string', message || `Expected string, got ${typeof value}`)
  }

  static isObject(value: unknown, message?: string): void {
    assert.ok(typeof value === 'object' && value !== null, message || `Expected object, got ${typeof value}`)
  }

  static isArray(value: unknown, message?: string): void {
    assert.ok(Array.isArray(value), message || `Expected array, got ${typeof value}`)
  }

  static hasProperty(obj: object, property: string, message?: string): void {
    assert.ok(property in obj, message || `Expected object to have property '${property}'`)
  }

  static approximatelyEqual(actual: number, expected: number, tolerance = 0.0001, message?: string): void {
    const diff = Math.abs(actual - expected)
    assert.ok(
      diff <= tolerance,
      message || `Expected ${actual} to be approximately ${expected} (tolerance: ${tolerance}), diff: ${diff}`
    )
  }
}

// Utility for timing operations
export class Timer {
  private startTime: number = 0

  start(): void {
    this.startTime = Date.now()
  }

  elapsed(): number {
    return Date.now() - this.startTime
  }

  static async measure<T>(operation: () => Promise<T>): Promise<{ result: T; duration: number }> {
    const start = Date.now()
    const result = await operation()
    const duration = Date.now() - start
    return { result, duration }
  }
}

// Utility for waiting
export const wait = (ms: number): Promise<void> => new Promise(resolve => setTimeout(resolve, ms))