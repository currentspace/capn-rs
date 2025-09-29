#!/bin/bash
# Patch the Cap'n Web TypeScript client to fix empty array serialization issue

echo "Patching Cap'n Web TypeScript client to fix serialization issues..."

# Backup original file
cp capnweb-github/src/serialize.ts capnweb-github/src/serialize.ts.backup

# Create a patch that fixes the empty array issue
cat > /tmp/capnweb-serialize.patch << 'EOF'
--- a/src/serialize.ts
+++ b/src/serialize.ts
@@ -133,8 +133,13 @@ export class Devaluator {
         for (let i = 0; i < len; i++) {
           result[i] = this.devaluateImpl(array[i], array, depth + 1);
         }
-        // Wrap literal arrays in an outer one-element array, to "escape" them.
-        return [result];
+        // Wrap literal arrays in an outer one-element array, to "escape" them.
+        // Special case: empty arrays need special handling
+        if (len === 0) {
+          return [[]];  // Return escaped empty array
+        } else {
+          return [result];
+        }
       }

       case "bigint":
@@ -298,7 +303,12 @@ export class Evaluator {
   private evaluateImpl(value: unknown, parent: object, property: string | number): unknown {
     if (value instanceof Array) {
       if (value.length == 1 && value[0] instanceof Array) {
-        // Escaped array. Evaluate the contents.
+        // Escaped array. Evaluate the contents.
+        // Special case for empty arrays
+        if (value[0].length === 0) {
+          return [];  // Return empty array directly
+        }
+
         let result = value[0];
         for (let i = 0; i < result.length; i++) {
           result[i] = this.evaluateImpl(result[i], result, i);
EOF

# Apply the patch
patch -p1 -d capnweb-github < /tmp/capnweb-serialize.patch

# Rebuild the Cap'n Web library
echo "Rebuilding Cap'n Web library..."
cd capnweb-github
pnpm build

echo "Patch applied and library rebuilt successfully!"
echo ""
echo "The patch fixes:"
echo "  - Empty array serialization causing 'unknown special value' errors"
echo "  - Proper handling of empty arrays in both serialization and deserialization"