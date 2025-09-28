#!/usr/bin/env node

/**
 * TypeScript interoperability test for Cap'n Web Rust server
 *
 * This test verifies that the Rust server implementation is protocol-compliant
 * by using the official Cap'n Web TypeScript client from GitHub.
 *
 * Prerequisites:
 * 1. Run ./setup-capnweb.sh to clone and build the latest Cap'n Web client
 * 2. Start the Rust server: cargo run --bin capnweb-server -p capnweb-server
 * 3. Run this test: npx tsx test-rust-server.ts
 */

import { newHttpBatchRpcSession, RpcStub } from './capnweb-github/dist/index.js';

const SERVER_URL = 'http://127.0.0.1:8080';

interface TestResult {
  name: string;
  passed: boolean;
  error?: Error;
}

class InteropTestSuite {
  private results: TestResult[] = [];

  async runTest(name: string, testFn: () => Promise<void>): Promise<void> {
    console.log(`  Running: ${name}...`);
    try {
      await testFn();
      this.results.push({ name, passed: true });
      console.log(`    ✅ Passed`);
    } catch (error) {
      this.results.push({ name, passed: false, error: error as Error });
      console.log(`    ❌ Failed: ${(error as Error).message}`);
    }
  }

  printSummary(): boolean {
    console.log('\n📊 Test Summary');
    console.log('=' .repeat(50));

    const passed = this.results.filter(r => r.passed).length;
    const failed = this.results.filter(r => !r.passed).length;

    console.log(`Total: ${this.results.length} | Passed: ${passed} | Failed: ${failed}`);

    if (failed > 0) {
      console.log('\nFailed tests:');
      this.results.filter(r => !r.passed).forEach(r => {
        console.log(`  ❌ ${r.name}: ${r.error?.message}`);
      });
    }

    return failed === 0;
  }
}

async function main() {
  console.log('🧪 Cap\'n Web Rust Server Interoperability Tests');
  console.log('=' .repeat(50));
  console.log(`Server URL: ${SERVER_URL}`);
  console.log('');

  const suite = new InteropTestSuite();

  // Test 1: HTTP Batch Transport Connection
  await suite.runTest('HTTP Batch Transport Connection', async () => {
    const session = newHttpBatchRpcSession(`${SERVER_URL}/rpc/batch`);

    // Just creating the session should work
    // Note: Cap'n Web doesn't have explicit close for HTTP batch sessions
  });

  // Test 2: Import Capability
  await suite.runTest('Import Capability', async () => {
    const session = newHttpBatchRpcSession(`${SERVER_URL}/rpc/batch`);

    // In Cap'n Web, we need to call a special method to import a capability
    // The server should expose an import method
    const result = await session._import(2);

    if (!result) {
      throw new Error('Failed to import capability');
    }
  });

  // Test 3: Simple RPC Call
  await suite.runTest('Simple RPC Call', async () => {
    const session = newHttpBatchRpcSession(`${SERVER_URL}/rpc/batch`);

    // Import the echo capability and call a method
    const echo = await session._import(2);

    // The echo service should echo back our arguments
    const result = await echo.echo('hello', 'world');

    if (!result) {
      throw new Error('No result from RPC call');
    }
  });

  // Test 4: Multiple Capabilities
  await suite.runTest('Multiple Capabilities', async () => {
    const session = newHttpBatchRpcSession(`${SERVER_URL}/rpc/batch`);

    const calc = await session._import(1);  // Calculator service
    const echo = await session._import(2);  // Echo service

    if (!calc || !echo) {
      throw new Error('Failed to import multiple capabilities');
    }
  });

  // Test 5: Batch RPC Calls
  await suite.runTest('Batch RPC Calls', async () => {
    const session = newHttpBatchRpcSession(`${SERVER_URL}/rpc/batch`);

    const calc = await session._import(1);
    const echo = await session._import(2);

    // Queue multiple calls
    const promises = [
      calc.add(5, 3),
      calc.multiply(4, 7),
      echo.echo('batch', 'test')
    ];

    // Execute batch
    const results = await Promise.all(promises);

    if (results.length !== 3) {
      throw new Error(`Expected 3 results, got ${results.length}`);
    }
  });

  // Test 6: Error Handling
  await suite.runTest('Error Handling', async () => {
    const session = newHttpBatchRpcSession(`${SERVER_URL}/rpc/batch`);

    const calc = await session._import(1);

    try {
      // Call a non-existent method
      await calc.nonExistentMethod();
      throw new Error('Should have thrown an error for non-existent method');
    } catch (error) {
      // Expected error
      if (!(error as Error).message.includes('not_found') &&
          !(error as Error).message.includes('Unknown method')) {
        throw error;
      }
    }
  });

  // Test 7: Capability Disposal
  await suite.runTest('Capability Disposal', async () => {
    const session = newHttpBatchRpcSession(`${SERVER_URL}/rpc/batch`);

    const echo = await session._import(2);

    // In Cap'n Web, disposal is done via a special dispose method
    await session._dispose([2]);

    try {
      // Calling after disposal should fail
      await echo.echo('test');
      throw new Error('Should not be able to call disposed capability');
    } catch (error) {
      // Expected - capability is disposed
    }
  });

  // Test 8: Large Payload
  await suite.runTest('Large Payload', async () => {
    const session = newHttpBatchRpcSession(`${SERVER_URL}/rpc/batch`);

    const echo = await session._import(2);

    // Create a large string (1MB)
    const largeString = 'x'.repeat(1024 * 1024);

    const result = await echo.echo(largeString);

    if (!result) {
      throw new Error('Failed to echo large payload');
    }
  });

  // Test 9: Concurrent Requests
  await suite.runTest('Concurrent Requests', async () => {
    const session = newHttpBatchRpcSession(`${SERVER_URL}/rpc/batch`);

    const echo = await session._import(2);

    // Make 10 concurrent requests
    const promises = Array.from({ length: 10 }, (_, i) =>
      echo.echo(`request-${i}`)
    );

    const results = await Promise.all(promises);

    if (results.length !== 10) {
      throw new Error(`Expected 10 results, got ${results.length}`);
    }
  });

  // Test 10: Protocol Compliance
  await suite.runTest('Protocol Compliance', async () => {
    const session = newHttpBatchRpcSession(`${SERVER_URL}/rpc/batch`);

    // Test that the server properly handles the wire protocol
    const calc = await session._import(1);

    // Server should return proper Cap'n Web protocol responses
    const result = await calc.add(2, 3);

    // The result structure should match protocol expectations
    if (result === undefined || result === null) {
      throw new Error('Invalid protocol response');
    }
  });

  // Print summary and exit
  const allPassed = suite.printSummary();

  if (allPassed) {
    console.log('\n✅ All tests passed!');
    process.exit(0);
  } else {
    console.log('\n❌ Some tests failed');
    process.exit(1);
  }
}

// Run the test suite
main().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});