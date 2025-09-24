// src/worker/lib/node/fs.ts
import assert from "node:assert";
function existsSync(_path) {
  return false;
}
function readdirSync(_path) {
  assert.fail("readdirSync() is not yet implemented in Workers");
}
function readFileSync(_path) {
  assert.fail("readFileSync() is not yet implemented in Workers");
}
function statSync(_path) {
  assert.fail("statSync() is not yet implemented in Workers");
}
var promises = {};
var fs_default = { existsSync };
export {
  fs_default as default,
  existsSync,
  promises,
  readFileSync,
  readdirSync,
  statSync
};
//# sourceMappingURL=fs.mjs.map
