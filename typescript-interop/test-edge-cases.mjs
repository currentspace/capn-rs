import { deserialize } from './capnweb-github/dist/index.js';

console.log("Testing edge cases that might fail:\n");

// Test what happens if we try to deserialize various formats directly

const testCases = [
  { name: "Bare empty array", json: '[]', expectError: true },
  { name: "Escaped empty array", json: '[[]]', expectError: false },
  { name: "Bare array [1,2]", json: '[1,2]', expectError: true },
  { name: "Escaped array [[1,2]]", json: '[[1,2]]', expectError: false },
  { name: "Object with bare empty array", json: '{"items": []}', expectError: true },
  { name: "Object with escaped empty array", json: '{"items": [[]]}', expectError: false },
  { name: "Single string array (ambiguous)", json: '["hello"]', expectError: true },
  { name: "Escaped single string", json: '[["hello"]]', expectError: false },
];

for (const test of testCases) {
  try {
    const result = deserialize(test.json);
    if (test.expectError) {
      console.log(`✗ ${test.name}: Expected error but got: ${JSON.stringify(result)}`);
    } else {
      console.log(`✓ ${test.name}: ${JSON.stringify(result)}`);
    }
  } catch (e) {
    if (test.expectError) {
      console.log(`✓ ${test.name}: Correctly threw error: ${e.message}`);
    } else {
      console.log(`✗ ${test.name}: Unexpected error: ${e.message}`);
    }
  }
}
