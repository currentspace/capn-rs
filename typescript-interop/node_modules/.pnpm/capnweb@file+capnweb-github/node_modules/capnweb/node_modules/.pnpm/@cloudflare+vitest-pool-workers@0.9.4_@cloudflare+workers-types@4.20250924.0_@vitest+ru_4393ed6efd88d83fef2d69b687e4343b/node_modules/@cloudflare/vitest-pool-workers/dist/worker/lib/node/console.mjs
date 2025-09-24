// src/worker/lib/node/console.ts
import { Writable } from "node:stream";
import { formatWithOptions } from "node:util";
var originalConsole = console;
var Console = class {
  #stdout;
  #stderr;
  #inspectOptions;
  constructor(opts, stderr, ignoreErrors) {
    if (opts instanceof Writable) {
      opts = { stdout: opts, stderr, ignoreErrors };
    }
    this.#stdout = opts.stdout;
    this.#stderr = opts.stderr ?? this.#stdout;
    const colors = typeof opts.colorMode === "string" ? false : opts.colorMode ?? false;
    this.#inspectOptions = opts.inspectOptions ?? { colors };
    return new Proxy(this, {
      get(target, prop) {
        const value = target[prop];
        if (typeof value === "function") {
          return value.bind(target);
        }
        return value;
      }
    });
  }
  // Vitest expects this function to be called `value`:
  // https://github.com/vitest-dev/vitest/blob/v1.0.0-beta.5/packages/vitest/src/runtime/console.ts#L16
  value(stream, data) {
    stream.write(formatWithOptions(this.#inspectOptions, ...data) + "\n");
  }
  assert(condition, ...data) {
    originalConsole.assert(condition, ...data);
  }
  clear() {
    originalConsole.clear();
  }
  count(label) {
    originalConsole.count(label);
  }
  countReset(label) {
    originalConsole.countReset(label);
  }
  debug(...data) {
    if (this.#stdout === void 0) {
      originalConsole.debug(...data);
    } else {
      this.value(this.#stdout, data);
    }
  }
  dir(item, options) {
    originalConsole.dir(item, options);
  }
  dirxml(...data) {
    originalConsole.dirxml(...data);
  }
  error(...data) {
    if (this.#stderr === void 0) {
      originalConsole.error(...data);
    } else {
      this.value(this.#stderr, data);
    }
  }
  group(...data) {
    originalConsole.group(...data);
  }
  groupCollapsed(...data) {
    originalConsole.groupCollapsed(...data);
  }
  groupEnd() {
    originalConsole.groupEnd();
  }
  info(...data) {
    if (this.#stdout === void 0) {
      originalConsole.info(...data);
    } else {
      this.value(this.#stdout, data);
    }
  }
  log(...data) {
    if (this.#stdout === void 0) {
      originalConsole.log(...data);
    } else {
      this.value(this.#stdout, data);
    }
  }
  table(tabularData, properties) {
    originalConsole.table(tabularData, properties);
  }
  time(label) {
    originalConsole.time(label);
  }
  timeEnd(label) {
    originalConsole.timeEnd(label);
  }
  timeLog(label, ...data) {
    originalConsole.timeLog(label, ...data);
  }
  timeStamp(label) {
    originalConsole.timeStamp(label);
  }
  trace(...data) {
    if (this.#stdout === void 0) {
      originalConsole.trace(...data);
    } else {
      this.value(this.#stdout, data);
    }
  }
  warn(...data) {
    if (this.#stderr === void 0) {
      originalConsole.warn(...data);
    } else {
      this.value(this.#stderr, data);
    }
  }
};
export {
  Console
};
//# sourceMappingURL=console.mjs.map
