// src/worker/lib/node/module.ts
var builtinModules = [];
function createRequire(filename) {
  return (specifier) => {
    const quotedSpecifier = JSON.stringify(specifier);
    const quotedFilename = JSON.stringify(filename);
    const message = `Attempted to \`require(${quotedSpecifier})\` from ${quotedFilename}`;
    const stack = (new Error(message).stack ?? "").replace("Error: ", "");
    throw new Error(
      `\`require()\` is not yet implemented in Workers.
${stack}`
    );
  };
}
var Module = class {
};
export {
  Module,
  builtinModules,
  createRequire
};
//# sourceMappingURL=module.mjs.map
