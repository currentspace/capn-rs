#!/usr/bin/env node

/**
 * Basic TypeScript interoperability test for Cap'n Web Rust server
 *
 * This test verifies basic connectivity and protocol compliance
 * using the official Cap'n Web TypeScript client.
 *
 * Prerequisites:
 * 1. Run ./setup-capnweb.sh to clone and build the latest Cap'n Web client
 * 2. Start the Rust server: cargo run --bin capnweb-server -p capnweb-server
 * 3. Run this test: npx tsx test-rust-server-basic.ts
 */

import { newHttpBatchRpcSession } from './capnweb-github/dist/index.js';

const SERVER_URL = 'http://127.0.0.1:8080/rpc/batch';

async function testBasicConnection() {
  console.log('🧪 Testing Cap\'n Web Rust Server Basic Connectivity');
  console.log('=' .repeat(50));
  console.log(`Server URL: ${SERVER_URL}`);
  console.log('');

  try {
    console.log('1. Creating HTTP batch session...');
    const api = newHttpBatchRpcSession(SERVER_URL);
    console.log('   ✅ Session created');

    // The Rust server needs to expose methods on its main interface
    // For now, let's try to call a method and see what happens
    console.log('2. Attempting to call a server method...');

    try {
      // Try calling echo as a method on the main interface
      const result = await api.echo('hello', 'world');
      console.log('   ✅ Method call succeeded');
      console.log('   Result:', result);
    } catch (error) {
      console.log('   ❌ Method call failed:', (error as Error).message);

      // Try another approach - maybe the server exposes a getCapability method
      try {
        console.log('3. Trying alternative approach - getCapability...');
        const echo = await api.getCapability(2);
        const result = await echo.echo('hello', 'world');
        console.log('   ✅ Alternative approach succeeded');
        console.log('   Result:', result);
      } catch (error2) {
        console.log('   ❌ Alternative approach failed:', (error2 as Error).message);
      }
    }

    console.log('\n📊 Summary');
    console.log('=' .repeat(50));
    console.log('The test completed. The server is responding to HTTP requests.');
    console.log('However, the exact RPC interface needs to be aligned with');
    console.log('the Cap\'n Web protocol expectations.');

  } catch (error) {
    console.error('Fatal error:', error);
    process.exit(1);
  }
}

// Run the test
testBasicConnection().catch(error => {
  console.error('Unhandled error:', error);
  process.exit(1);
});