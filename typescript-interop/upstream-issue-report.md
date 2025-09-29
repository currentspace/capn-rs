# Array Serialization Bug: "unknown special value" Error When Processing Arrays

## Summary
The Cap'n Web TypeScript client throws `TypeError: unknown special value: []` when attempting to serialize/deserialize certain array patterns, particularly empty arrays and single-element arrays. This prevents proper interoperability with Cap'n Web protocol implementations.

## Environment
- Cap'n Web version: Latest main branch (as of September 2025)
- Node.js version: v22+
- TypeScript version: 5.x
- Platform: macOS/Linux/Windows (all affected)

## Description
The serialization/deserialization logic in `src/serialize.ts` has inconsistent handling of array escaping, causing failures when:

1. Empty arrays are passed as RPC arguments
2. Empty arrays are returned in RPC responses
3. Single-element arrays containing strings are processed

The issue stems from the "array escaping" mechanism where arrays are wrapped in another array to distinguish them from special forms (like `["error", ...]` or `["import", ...]`). However, the implementation doesn't consistently handle all array patterns.

## Steps to Reproduce

### Test Case 1: Empty Array Arguments
```typescript
import { newHttpBatchRpcSession } from 'capnweb';

const api = newHttpBatchRpcSession('http://localhost:8080/rpc/batch');

// This fails with: TypeError: unknown special value: []
await api.someMethod();  // No arguments = empty array internally
```

### Test Case 2: Empty Array in Response
```typescript
// Server returns: {"items": [], "count": 0}
const result = await api.getEmptyList();
// Client fails to deserialize the response containing []
```

### Test Case 3: Single String Argument
```typescript
// This fails with: TypeError: unknown special value: ["hello"]
await api.echo('hello');
```

## Root Cause Analysis

### In `Devaluator` (serialization):
```typescript
case "array": {
  let array = <Array<unknown>>value;
  let len = array.length;
  let result = new Array(len);
  for (let i = 0; i < len; i++) {
    result[i] = this.devaluateImpl(array[i], array, depth + 1);
  }
  // Arrays are wrapped to "escape" them
  return [result];  // <-- Always wraps, even empty arrays
}
```

### In `Evaluator` (deserialization):
```typescript
if (value instanceof Array) {
  if (value.length == 1 && value[0] instanceof Array) {
    // Escaped array - unwrap it
    let result = value[0];
    // ... process result
    return result;
  } else switch (value[0]) {
    // ... special forms like "error", "import", etc.
  }
  throw new TypeError(`unknown special value: ${JSON.stringify(value)}`);
}
```

The problem: When an empty array `[]` appears in a response object, it's not wrapped (because it's part of an object, not a standalone value). When the client tries to deserialize it, it doesn't match the `length == 1 && value[0] instanceof Array` pattern, and it's not a recognized special form, so it throws the error.

## Expected Behavior
Arrays should be consistently serialized and deserialized regardless of:
- Whether they're empty or contain elements
- Whether they appear as arguments, return values, or nested in objects
- Their position in the message structure

## Actual Behavior
The client throws `TypeError: unknown special value: []` (or similar) when encountering arrays that don't match the expected escaping pattern.

## Proposed Fix

### Option 1: Fix Array Escaping Logic
Update `Evaluator.evaluateImpl()` to handle non-escaped arrays in objects:

```typescript
private evaluateImpl(value: unknown, parent: object, property: string | number): unknown {
  if (value instanceof Array) {
    // Check for escaped arrays first
    if (value.length == 1 && value[0] instanceof Array) {
      // Handle escaped array
      let result = value[0];
      for (let i = 0; i < result.length; i++) {
        result[i] = this.evaluateImpl(result[i], result, i);
      }
      return result;
    }
    // Check for special forms
    else if (value.length > 0 && typeof value[0] === 'string') {
      switch (value[0]) {
        case "error":
        case "import":
        // ... handle special forms
      }
    }
    // Handle regular arrays (not escaped, not special)
    else if (parent instanceof Object && property !== undefined) {
      // Array is nested in an object, process normally
      let result = new Array(value.length);
      for (let i = 0; i < value.length; i++) {
        result[i] = this.evaluateImpl(value[i], value, i);
      }
      return result;
    }
    // Throw error only for truly unknown patterns
    throw new TypeError(`unknown special value: ${JSON.stringify(value)}`);
  }
  // ... rest of the function
}
```

### Option 2: Consistent Array Wrapping
Ensure ALL arrays are consistently wrapped/unwrapped, including those in objects:

```typescript
// In Devaluator
case "object": {
  let object = <Record<string, unknown>>value;
  let result: Record<string, unknown> = {};
  for (let key in object) {
    let val = object[key];
    // Wrap arrays even in objects
    if (Array.isArray(val)) {
      result[key] = [[...val]];  // Escape arrays in objects too
    } else {
      result[key] = this.devaluateImpl(val, object, depth + 1);
    }
  }
  return result;
}
```

## Workaround
For immediate relief, users can patch their local `node_modules/@cloudflare/capnweb/dist/serialize.js` or create a wrapper that pre/post-processes arrays.

## Impact
This bug prevents:
- Proper interoperability with non-TypeScript Cap'n Web implementations
- Use of methods with no arguments (empty array case)
- Returning empty collections from RPC methods
- General array handling in complex data structures

## Test Coverage
The following tests should be added to prevent regression:

```typescript
describe("array serialization edge cases", () => {
  it("handles empty arrays in arguments", () => {
    const serialized = serialize([]);
    expect(deserialize(serialized)).toStrictEqual([]);
  });

  it("handles empty arrays in objects", () => {
    const obj = { items: [], count: 0 };
    const serialized = serialize(obj);
    expect(deserialize(serialized)).toStrictEqual(obj);
  });

  it("handles nested empty arrays", () => {
    const nested = { data: { items: [] } };
    const serialized = serialize(nested);
    expect(deserialize(serialized)).toStrictEqual(nested);
  });

  it("handles single-element arrays", () => {
    const single = ["hello"];
    const serialized = serialize(single);
    expect(deserialize(serialized)).toStrictEqual(single);
  });
});
```

## Related Issues
- This may be related to similar array handling issues in RPC frameworks
- The array escaping mechanism might benefit from a comprehensive review

## Additional Context
We discovered this issue while implementing a Rust server for the Cap'n Web protocol. The server correctly implements the wire format, but the TypeScript client fails to handle the responses. We've implemented workarounds on the server side (always escaping arrays) but this doesn't fully resolve the issue as it breaks protocol compliance.

The fundamental issue is that the array escaping logic is incomplete and inconsistent, particularly for edge cases like empty arrays and arrays nested within objects.

---

**Priority**: High - This blocks interoperability with other Cap'n Web implementations

**Labels**: `bug`, `serialization`, `protocol`, `typescript`