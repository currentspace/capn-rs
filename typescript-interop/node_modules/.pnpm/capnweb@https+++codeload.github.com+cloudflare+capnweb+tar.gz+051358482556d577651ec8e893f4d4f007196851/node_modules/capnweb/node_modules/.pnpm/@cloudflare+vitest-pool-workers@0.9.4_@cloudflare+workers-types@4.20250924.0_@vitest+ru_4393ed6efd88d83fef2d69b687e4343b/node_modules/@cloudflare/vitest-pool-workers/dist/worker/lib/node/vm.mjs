// src/worker/lib/node/vm.ts
import assert from "node:assert";
var unsafeEval;
function _setUnsafeEval(newUnsafeEval) {
  unsafeEval = newUnsafeEval;
}
function runInThisContext(code, options) {
  assert(unsafeEval !== void 0);
  return unsafeEval.eval(code, options?.filename);
}
var vm_default = { runInThisContext };
export {
  _setUnsafeEval,
  vm_default as default,
  runInThisContext
};
//# sourceMappingURL=vm.mjs.map
