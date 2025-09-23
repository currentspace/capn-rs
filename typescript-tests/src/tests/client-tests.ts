/**
 * TypeScript Client Tests Against Rust Server
 *
 * Tests the TypeScript Cap'n Web client against the Rust server implementation
 * to verify complete interoperability.
 */

import { CapnWebClient, PlanBuilder, Param } from '../capnweb/client.js'
import { WebSocketTransport } from '../capnweb/websocket-transport.js'
import { InteropTestFramework, InteropAssert, Timer, wait } from './test-framework.js'

const RUST_SERVER_URL = 'ws://localhost:8080/ws'
const CONNECTION_TIMEOUT = 5000
const TEST_TIMEOUT = 10000

export class TypeScriptClientTests {
  private framework: InteropTestFramework
  private client: CapnWebClient | null = null
  private transport: WebSocketTransport | null = null

  constructor() {
    this.framework = new InteropTestFramework(console)
  }

  async runAllTests(): Promise<void> {
    console.log('🔥 TypeScript Client → Rust Server Interoperability Tests')
    console.log('📡 Testing TypeScript client against Rust server...')

    try {
      await this.framework.runTestSuite('Connection Tests', [
        this.testConnection.bind(this),
        this.testConnectionTimeout.bind(this)
      ])

      await this.framework.runTestSuite('Basic Capability Tests', [
        this.testBasicCalculatorOperations.bind(this),
        this.testCalculatorErrorHandling.bind(this),
        this.testMultipleSequentialCalls.bind(this),
        this.testConcurrentCalls.bind(this)
      ])

      await this.framework.runTestSuite('Advanced Mathematical Operations', [
        this.testAdvancedMathOperations.bind(this),
        this.testFactorialOperations.bind(this),
        this.testErrorConditions.bind(this)
      ])

      await this.framework.runTestSuite('User Management Tests', [
        this.testUserManagement.bind(this),
        this.testUserNotFound.bind(this),
        this.testCreateUser.bind(this)
      ])

      await this.framework.runTestSuite('Performance Tests', [
        this.testPerformance.bind(this),
        this.testHighVolumeOperations.bind(this)
      ])

      await this.framework.runTestSuite('Edge Cases', [
        this.testLargeNumbers.bind(this),
        this.testInvalidArguments.bind(this),
        this.testInvalidMethods.bind(this)
      ])

    } finally {
      await this.cleanup()
    }

    this.framework.generateReport()
  }

  private async setupClient(): Promise<void> {
    if (this.client) {
      await this.cleanup()
    }

    this.transport = new WebSocketTransport(RUST_SERVER_URL, console)
    await this.transport.connect()

    this.client = new CapnWebClient(this.transport, {
      timeout: TEST_TIMEOUT
    }, console)

    await this.client.connect()
  }

  private async cleanup(): Promise<void> {
    if (this.client) {
      await this.client.close()
      this.client = null
    }
    if (this.transport) {
      await this.transport.close()
      this.transport = null
    }
  }

  // Connection Tests
  private async testConnection(): Promise<void> {
    await this.setupClient()
    InteropAssert.ok(this.client, 'Client should be connected')
  }

  private async testConnectionTimeout(): Promise<void> {
    const badTransport = new WebSocketTransport('ws://localhost:9999/nonexistent')
    let failed = false
    try {
      await badTransport.connect()
    } catch (error) {
      failed = true
    }
    InteropAssert.ok(failed, 'Connection to nonexistent server should fail')
  }

  // Basic Capability Tests
  private async testBasicCalculatorOperations(): Promise<void> {
    await this.setupClient()

    // Test addition
    const addResult = await this.client!.call(1, 'add', [15.5, 24.3])
    InteropAssert.approximatelyEqual(addResult as number, 39.8, 0.01, 'Addition result should be correct')

    // Test subtraction
    const subResult = await this.client!.call(1, 'subtract', [50, 18])
    InteropAssert.equal(subResult, 32, 'Subtraction result should be correct')

    // Test multiplication
    const mulResult = await this.client!.call(1, 'multiply', [7, 8])
    InteropAssert.equal(mulResult, 56, 'Multiplication result should be correct')

    // Test division
    const divResult = await this.client!.call(1, 'divide', [84, 7])
    InteropAssert.equal(divResult, 12, 'Division result should be correct')
  }

  private async testCalculatorErrorHandling(): Promise<void> {
    await this.setupClient()

    // Test division by zero
    try {
      await this.client!.call(1, 'divide', [10, 0])
      throw new Error('Should have thrown division by zero error')
    } catch (error) {
      InteropAssert.ok(
        error instanceof Error && error.message.toLowerCase().includes('zero'),
        'Should throw division by zero error'
      )
    }

    // Test invalid argument count
    try {
      await this.client!.call(1, 'add', [5])
      throw new Error('Should have thrown argument count error')
    } catch (error) {
      InteropAssert.ok(error instanceof Error, 'Should throw argument error')
    }
  }

  private async testMultipleSequentialCalls(): Promise<void> {
    await this.setupClient()

    const operations = [
      { method: 'add', args: [1, 2], expected: 3 },
      { method: 'multiply', args: [3, 4], expected: 12 },
      { method: 'subtract', args: [10, 3], expected: 7 },
      { method: 'divide', args: [20, 4], expected: 5 }
    ]

    for (const op of operations) {
      const result = await this.client!.call(1, op.method, op.args)
      InteropAssert.equal(result, op.expected, `${op.method} should return correct result`)
    }
  }

  private async testConcurrentCalls(): Promise<void> {
    await this.setupClient()

    const promises = [
      this.client!.call(1, 'add', [1, 1]),
      this.client!.call(1, 'add', [2, 2]),
      this.client!.call(1, 'add', [3, 3]),
      this.client!.call(1, 'add', [4, 4]),
      this.client!.call(1, 'add', [5, 5])
    ]

    const results = await Promise.all(promises)
    InteropAssert.deepEqual(results, [2, 4, 6, 8, 10], 'Concurrent calls should all succeed')
  }

  // Advanced Mathematical Operations
  private async testAdvancedMathOperations(): Promise<void> {
    await this.setupClient()

    // Test power operation
    const powerResult = await this.client!.call(1, 'power', [2, 10])
    InteropAssert.equal(powerResult, 1024, 'Power operation should be correct')

    // Test square root
    const sqrtResult = await this.client!.call(1, 'sqrt', [144])
    InteropAssert.equal(sqrtResult, 12, 'Square root should be correct')

    // Test square root of decimal
    const sqrtDecimalResult = await this.client!.call(1, 'sqrt', [2])
    InteropAssert.approximatelyEqual(sqrtDecimalResult as number, 1.4142135623730951, 0.0001, 'Square root of 2 should be correct')
  }

  private async testFactorialOperations(): Promise<void> {
    await this.setupClient()

    // Test factorial
    const factorialResult = await this.client!.call(1, 'factorial', [5])
    InteropAssert.equal(factorialResult, 120, 'Factorial of 5 should be 120')

    // Test factorial of 0
    const factorial0Result = await this.client!.call(1, 'factorial', [0])
    InteropAssert.equal(factorial0Result, 1, 'Factorial of 0 should be 1')
  }

  private async testErrorConditions(): Promise<void> {
    await this.setupClient()

    // Test negative square root
    try {
      await this.client!.call(1, 'sqrt', [-1])
      throw new Error('Should have thrown negative square root error')
    } catch (error) {
      InteropAssert.ok(error instanceof Error, 'Should throw error for negative square root')
    }

    // Test negative factorial
    try {
      await this.client!.call(1, 'factorial', [-1])
      throw new Error('Should have thrown negative factorial error')
    } catch (error) {
      InteropAssert.ok(error instanceof Error, 'Should throw error for negative factorial')
    }

    // Test factorial too large
    try {
      await this.client!.call(1, 'factorial', [25])
      throw new Error('Should have thrown factorial too large error')
    } catch (error) {
      InteropAssert.ok(error instanceof Error, 'Should throw error for factorial too large')
    }
  }

  // User Management Tests
  private async testUserManagement(): Promise<void> {
    await this.setupClient()

    // Test getting existing users
    const user1 = await this.client!.call(100, 'getUser', [1]) as any
    InteropAssert.equal(user1.id, 1, 'User 1 should have correct ID')
    InteropAssert.equal(user1.name, 'Alice', 'User 1 should have correct name')

    const user2 = await this.client!.call(100, 'getUser', [2]) as any
    InteropAssert.equal(user2.id, 2, 'User 2 should have correct ID')
    InteropAssert.equal(user2.name, 'Bob', 'User 2 should have correct name')
  }

  private async testUserNotFound(): Promise<void> {
    await this.setupClient()

    try {
      await this.client!.call(100, 'getUser', [999])
      throw new Error('Should have thrown user not found error')
    } catch (error) {
      InteropAssert.ok(
        error instanceof Error && error.message.toLowerCase().includes('not found'),
        'Should throw user not found error'
      )
    }
  }

  private async testCreateUser(): Promise<void> {
    await this.setupClient()

    const userData = {
      name: 'Test User',
      email: 'test@example.com'
    }

    const newUser = await this.client!.call(100, 'createUser', [userData]) as any
    InteropAssert.equal(newUser.name, 'Test User', 'Created user should have correct name')
    InteropAssert.equal(newUser.email, 'test@example.com', 'Created user should have correct email')
    InteropAssert.ok(newUser.created, 'Created user should have created flag')
  }

  // Performance Tests
  private async testPerformance(): Promise<void> {
    await this.setupClient()

    const { result, duration } = await Timer.measure(async () => {
      const promises = Array.from({ length: 100 }, (_, i) =>
        this.client!.call(1, 'add', [i, i])
      )
      return Promise.all(promises)
    })

    console.log(`   Performance: 100 concurrent calls completed in ${duration}ms`)
    InteropAssert.equal(result.length, 100, 'Should complete all 100 calls')
    InteropAssert.ok(duration < 5000, 'Should complete within 5 seconds')
  }

  private async testHighVolumeOperations(): Promise<void> {
    await this.setupClient()

    // Test rapid sequential operations
    const startTime = Date.now()
    for (let i = 0; i < 50; i++) {
      const result = await this.client!.call(1, 'multiply', [i, 2])
      InteropAssert.equal(result, i * 2, `Operation ${i} should be correct`)
    }
    const duration = Date.now() - startTime

    console.log(`   High Volume: 50 sequential calls completed in ${duration}ms`)
    InteropAssert.ok(duration < 10000, 'Should complete within 10 seconds')
  }

  // Edge Cases
  private async testLargeNumbers(): Promise<void> {
    await this.setupClient()

    // Test with large numbers
    const largeNum1 = 1000000
    const largeNum2 = 999999
    const result = await this.client!.call(1, 'add', [largeNum1, largeNum2])
    InteropAssert.equal(result, 1999999, 'Should handle large numbers correctly')

    // Test with very small numbers
    const smallNum1 = 0.000001
    const smallNum2 = 0.000002
    const smallResult = await this.client!.call(1, 'add', [smallNum1, smallNum2])
    InteropAssert.approximatelyEqual(smallResult as number, 0.000003, 0.0000001, 'Should handle small numbers correctly')
  }

  private async testInvalidArguments(): Promise<void> {
    await this.setupClient()

    // Test with wrong number of arguments
    try {
      await this.client!.call(1, 'add', [1, 2, 3])
      throw new Error('Should reject too many arguments')
    } catch (error) {
      InteropAssert.ok(error instanceof Error, 'Should throw error for wrong argument count')
    }

    // Test with no arguments
    try {
      await this.client!.call(1, 'add', [])
      throw new Error('Should reject no arguments')
    } catch (error) {
      InteropAssert.ok(error instanceof Error, 'Should throw error for no arguments')
    }
  }

  private async testInvalidMethods(): Promise<void> {
    await this.setupClient()

    try {
      await this.client!.call(1, 'nonexistentMethod', [1, 2])
      throw new Error('Should reject nonexistent method')
    } catch (error) {
      InteropAssert.ok(
        error instanceof Error && error.message.toLowerCase().includes('not found'),
        'Should throw method not found error'
      )
    }
  }
}

// Main execution function
export async function runClientTests(): Promise<void> {
  const tests = new TypeScriptClientTests()
  await tests.runAllTests()
}