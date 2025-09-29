#!/usr/bin/env node

// Direct test to understand serialization issue

import { serialize, deserialize } from './capnweb-github/dist/index.js';

console.log('Testing Cap\'n Web serialization:');
console.log('');

// Test serializing different values
const testCases = [
  { name: 'empty array', value: [] },
  { name: 'single string', value: 'hello world' },
  { name: 'array with string', value: ['hello world'] },
  { name: 'array with two strings', value: ['hello', 'world'] },
  { name: 'nested array', value: [['hello', 'world']] },
];

for (const test of testCases) {
  try {
    const serialized = serialize(test.value);
    console.log(`✅ ${test.name}: ${serialized}`);

    // Try to deserialize it back
    const deserialized = deserialize(serialized);
    console.log(`   Deserialized: ${JSON.stringify(deserialized)}`);
  } catch (error) {
    console.log(`❌ ${test.name}: ${(error as Error).message}`);
  }
}

// Now test what the RPC system actually sees
console.log('\nNow testing RPC argument serialization:');

// The RPC system wraps arguments in RpcPayload.fromAppParams()
// which eventually calls Devaluator.devaluate()

import { Devaluator } from './capnweb-github/src/serialize.js';

const argumentsList = ['hello world'];
try {
  const devalued = Devaluator.devaluate(argumentsList);
  console.log('Arguments devalued:', JSON.stringify(devalued));
} catch (error) {
  console.log('Devaluation error:', (error as Error).message);
}