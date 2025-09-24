// src/worker/lib/mlly.ts
function isObject(value) {
  return value !== null && typeof value === "object";
}
function resolvePathSync() {
  throw new Error("resolvePathSync() not yet implemented in worker");
}
function interopDefault(sourceModule) {
  if (!isObject(sourceModule) || !("default" in sourceModule) || !isObject(sourceModule.default)) {
    return sourceModule;
  }
  const newModule = sourceModule.default;
  for (const key in sourceModule) {
    if (key === "default") {
      try {
        if (!(key in newModule)) {
          Object.defineProperty(newModule, key, {
            enumerable: false,
            configurable: false,
            get() {
              return newModule;
            }
          });
        }
      } catch {
      }
    } else {
      try {
        if (!(key in newModule)) {
          Object.defineProperty(newModule, key, {
            enumerable: true,
            configurable: true,
            get() {
              return sourceModule[key];
            }
          });
        }
      } catch {
      }
    }
  }
  return newModule;
}
export {
  interopDefault,
  resolvePathSync
};
//# sourceMappingURL=mlly.mjs.map
