#!/usr/bin/env node

import { newHttpBatchRpcSession } from './capnweb-github/dist/index.js';

const SERVER_URL = 'http://127.0.0.1:8080/rpc/batch';

async function testDebug() {
  console.log('Testing with single string argument:');
  const api = newHttpBatchRpcSession(SERVER_URL);

  try {
    // Try calling without any arguments at all
    console.log('Calling echo with no arguments...');
    const result1 = await api.echo();
    console.log('Result:', result1);
  } catch (error) {
    console.log('Error:', (error as Error).message);
  }
}

testDebug().catch(console.error);