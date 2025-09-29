#!/usr/bin/env node

/**
 * Comprehensive TypeScript interoperability test for Cap'n Web Rust server
 *
 * This test verifies full protocol compliance including:
 * - Bootstrap service and getCapability method
 * - Capability references
 * - Method calls with various argument types
 */

import { newHttpBatchRpcSession } from './capnweb-github/dist/index.js';

const SERVER_URL = 'http://127.0.0.1:8080/rpc/batch';

async function testComprehensive() {
  console.log('🧪 Testing Cap\'n Web Rust Server - Comprehensive Test');
  console.log('=' .repeat(60));
  console.log(`Server URL: ${SERVER_URL}`);
  console.log('');

  let allTestsPassed = true;

  try {
    console.log('1. Creating HTTP batch session...');
    const api = newHttpBatchRpcSession(SERVER_URL);
    console.log('   ✅ Session created');

    // Test 1: Bootstrap echo method
    console.log('\n2. Testing bootstrap echo method...');
    try {
      // Pass a single argument - Cap'n Web doesn't support multiple arguments
      const result = await api.echo('hello world');
      console.log('   ✅ Bootstrap echo succeeded');
      console.log('   Result:', JSON.stringify(result, null, 2));
    } catch (error) {
      console.log('   ❌ Bootstrap echo failed:', (error as Error).message);
      allTestsPassed = false;
    }

    // Test 2: Get capability using getCapability
    console.log('\n3. Testing getCapability method...');
    try {
      // Get capability 1 (Calculator Service)
      const calculator = await api.getCapability(1);
      console.log('   ✅ Got capability 1 (Calculator)');

      // Test the calculator's echo method
      console.log('\n4. Testing Calculator.echo method...');
      const echoResult = await calculator.echo('test message');
      console.log('   ✅ Calculator.echo succeeded');
      console.log('   Result:', JSON.stringify(echoResult, null, 2));
    } catch (error) {
      console.log('   ❌ getCapability or method call failed:', (error as Error).message);
      allTestsPassed = false;
    }

    // Test 3: Get and use Echo Service
    console.log('\n5. Testing Echo Service (capability 2)...');
    try {
      const echoService = await api.getCapability(2);
      console.log('   ✅ Got capability 2 (Echo Service)');

      // Echo service handles any method
      const result = await echoService.customMethod('test argument');
      console.log('   ✅ Echo.customMethod succeeded');
      console.log('   Result:', JSON.stringify(result, null, 2));
    } catch (error) {
      console.log('   ❌ Echo service test failed:', (error as Error).message);
      allTestsPassed = false;
    }

    // Test 4: Invalid capability
    console.log('\n6. Testing invalid capability request...');
    try {
      await api.getCapability(999);
      console.log('   ❌ Should have failed but didn\'t');
      allTestsPassed = false;
    } catch (error) {
      console.log('   ✅ Correctly rejected invalid capability:', (error as Error).message);
    }

    console.log('\n📊 Summary');
    console.log('=' .repeat(60));
    if (allTestsPassed) {
      console.log('✅ ALL TESTS PASSED!');
      console.log('The Rust server is fully compatible with the TypeScript client.');
      process.exit(0);
    } else {
      console.log('❌ Some tests failed.');
      console.log('The server needs adjustments for full protocol compliance.');
      process.exit(1);
    }

  } catch (error) {
    console.error('Fatal error:', error);
    process.exit(1);
  }
}

// Run the test
testComprehensive().catch(error => {
  console.error('Unhandled error:', error);
  process.exit(1);
});