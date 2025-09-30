import { serialize, deserialize } from './capnweb-github/dist/index.js';

console.log("Testing array escaping behavior:\n");

// Test 1: Empty array
try {
  const empty = [];
  const serialized = serialize(empty);
  console.log("Empty array:", JSON.stringify(empty));
  console.log("Serialized:", serialized);
  console.log("Deserialized:", JSON.stringify(deserialize(serialized)));
  console.log("✓ Empty array works\n");
} catch (e) {
  console.log("✗ Empty array failed:", e.message, "\n");
}

// Test 2: Array in object
try {
  const objWithArray = { items: [], count: 0 };
  const serialized = serialize(objWithArray);
  console.log("Object with empty array:", JSON.stringify(objWithArray));
  console.log("Serialized:", serialized);
  console.log("Deserialized:", JSON.stringify(deserialize(serialized)));
  console.log("✓ Object with empty array works\n");
} catch (e) {
  console.log("✗ Object with empty array failed:", e.message, "\n");
}

// Test 3: Non-empty array
try {
  const arr = [1, 2, 3];
  const serialized = serialize(arr);
  console.log("Regular array:", JSON.stringify(arr));
  console.log("Serialized:", serialized);
  console.log("Deserialized:", JSON.stringify(deserialize(serialized)));
  console.log("✓ Regular array works\n");
} catch (e) {
  console.log("✗ Regular array failed:", e.message, "\n");
}

// Test 4: Nested array in object
try {
  const nested = { data: { items: ["a", "b"] } };
  const serialized = serialize(nested);
  console.log("Nested array in object:", JSON.stringify(nested));
  console.log("Serialized:", serialized);
  console.log("Deserialized:", JSON.stringify(deserialize(serialized)));
  console.log("✓ Nested array in object works\n");
} catch (e) {
  console.log("✗ Nested array in object failed:", e.message, "\n");
}

// Test 5: What SHOULD the wire format be for arrays in objects?
console.log("\n=== Wire Format Analysis ===");
console.log('Object {"items": [1,2]} serializes to:');
console.log(serialize({items: [1, 2]}));
