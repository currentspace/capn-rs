// src/worker/lib/cloudflare/test-runner.ts
import assert from "node:assert";
import { NodeSnapshotEnvironment } from "@vitest/snapshot/environment";
import { resetMockAgent } from "cloudflare:mock-agent";
import {
  fetchMock,
  getSerializedOptions,
  internalEnv,
  registerHandlerAndGlobalWaitUntil,
  waitForGlobalWaitUntil
} from "cloudflare:test-internal";
import { vi } from "vitest";
import { VitestTestRunner } from "vitest/runners";
import workerdUnsafe from "workerd:unsafe";
var DEBUG = false;
var _ = (n) => " ".repeat(n);
var WorkersSnapshotEnvironment = class extends NodeSnapshotEnvironment {
  constructor(rpc) {
    super();
    this.rpc = rpc;
  }
  #fetch(method, path, body) {
    const encodedPath = encodeURIComponent(path);
    const url = `http://placeholder/snapshot?path=${encodedPath}`;
    return internalEnv.__VITEST_POOL_WORKERS_LOOPBACK_SERVICE.fetch(url, {
      method,
      body
    });
  }
  getHeader() {
    return `// Vitest Snapshot v${this.getVersion()}, https://vitest.dev/guide/snapshot.html`;
  }
  resolvePath(filePath) {
    return this.rpc.resolveSnapshotPath(filePath);
  }
  async prepareDirectory(dirPath) {
    const res = await this.#fetch("POST", dirPath);
    assert.strictEqual(res.status, 204);
  }
  async saveSnapshotFile(filePath, snapshot) {
    const res = await this.#fetch("PUT", filePath, snapshot);
    assert.strictEqual(res.status, 204);
  }
  async readSnapshotFile(filePath) {
    const res = await this.#fetch("GET", filePath);
    if (res.status === 404) {
      return null;
    }
    assert.strictEqual(res.status, 200);
    return await res.text();
  }
  async removeSnapshotFile(filePath) {
    const res = await this.#fetch("DELETE", filePath);
    assert.strictEqual(res.status, 204);
  }
};
var initialState;
var patchedPrepareStackTrace = false;
var getConsoleGetFileName = () => () => "node:internal/console/constructor";
function getTryKey({ repeats, retry }) {
  return `${repeats}:${retry}`;
}
var tryStates = /* @__PURE__ */ new WeakMap();
var waitUntilPatchedRpc = /* @__PURE__ */ new WeakSet();
function createWaitUntilRpc(rpc) {
  return new Proxy(rpc, {
    get(target, key, handler) {
      if (key === "then") {
        return;
      }
      const sendCall = Reflect.get(target, key, handler);
      const waitUntilSendCall = async (...args) => {
        const promise = sendCall(...args);
        registerHandlerAndGlobalWaitUntil(promise);
        return promise;
      };
      waitUntilSendCall.asEvent = sendCall.asEvent;
      return waitUntilSendCall;
    }
  });
}
var WorkersTestRunner = class extends VitestTestRunner {
  state;
  isolatedStorage;
  constructor(config) {
    super(config);
    const state = this.workerState;
    this.state = state;
    const { isolatedStorage } = getSerializedOptions();
    this.isolatedStorage = isolatedStorage ?? false;
    const opts = state.config.snapshotOptions;
    if (!(opts.snapshotEnvironment instanceof WorkersSnapshotEnvironment)) {
      opts.snapshotEnvironment = new WorkersSnapshotEnvironment(state.rpc);
    }
    if (!waitUntilPatchedRpc.has(state.rpc)) {
      waitUntilPatchedRpc.add(state.rpc);
      state.rpc = createWaitUntilRpc(state.rpc);
    }
    initialState ??= state;
    initialState.rpc = state.rpc;
    if (!patchedPrepareStackTrace) {
      patchedPrepareStackTrace = true;
      const originalPrepareStackTrace = Error.prepareStackTrace;
      assert(originalPrepareStackTrace !== void 0);
      Error.prepareStackTrace = (err, callSites) => {
        for (const callSite of callSites) {
          const fileName = callSite.getFileName();
          if (fileName?.endsWith("/dist/worker/lib/node/console.mjs")) {
            Object.defineProperty(callSite, "getFileName", {
              get: getConsoleGetFileName
            });
          }
        }
        return originalPrepareStackTrace(err, callSites);
      };
    }
  }
  async updateStackedStorage(action, source) {
    if (!this.isolatedStorage) {
      return;
    }
    await waitForGlobalWaitUntil();
    await workerdUnsafe.abortAllDurableObjects();
    const url = "http://placeholder/storage";
    const sourceString = `${source.file?.name ?? "an unknown file"}'s ${source.type} ${JSON.stringify(source.name)}`;
    const res = await internalEnv.__VITEST_POOL_WORKERS_LOOPBACK_SERVICE.fetch(
      url,
      {
        method: action === "pop" ? "DELETE" : "POST",
        headers: { "MF-Vitest-Source": sourceString }
      }
    );
    assert.strictEqual(res.status, 204, await res.text());
  }
  syncCurrentTaskWithInitialState() {
    assert(initialState !== void 0);
    initialState.current = this.state.current;
  }
  async onBeforeRunFiles() {
    if (DEBUG) {
      __console.log("onBeforeRunFiles");
      await scheduler.wait(100);
    }
    resetMockAgent(fetchMock);
    if (super.onBeforeRunFiles) {
      return super.onBeforeRunFiles();
    }
  }
  async onAfterRunFiles() {
    if (DEBUG) {
      __console.log("onAfterRunFiles");
      await scheduler.wait(100);
    }
    vi.resetModules();
    await waitForGlobalWaitUntil();
    return super.onAfterRunFiles?.();
  }
  async onBeforeRunSuite(suite) {
    if (DEBUG) {
      __console.log(`${_(2)}onBeforeRunSuite: ${suite.name}`);
      await scheduler.wait(100);
    }
    await this.updateStackedStorage("push", suite);
    return super.onBeforeRunSuite(suite);
  }
  async onAfterRunSuite(suite) {
    if (DEBUG) {
      __console.log(`${_(2)}onAfterRunSuite: ${suite.name}`);
      await scheduler.wait(100);
    }
    await this.updateStackedStorage("pop", suite);
    return super.onAfterRunSuite(suite);
  }
  async ensurePoppedActiveTryStorage(test, newActive) {
    const tries = tryStates.get(test);
    assert(tries !== void 0);
    const active = tries.active;
    if (newActive !== void 0) {
      tries.active = newActive;
    }
    if (active !== void 0 && !tries.popped.has(active)) {
      tries.popped.add(active);
      await this.updateStackedStorage("pop", test);
      return true;
    }
    return false;
  }
  async onBeforeRunTask(test) {
    if (DEBUG) {
      __console.log(`${_(4)}onBeforeRunTask: ${test.name}`);
      await scheduler.wait(100);
    }
    tryStates.set(test, { popped: /* @__PURE__ */ new Set() });
    if (this.isolatedStorage && test.concurrent) {
      const quotedName = JSON.stringify(test.name);
      const msg = [
        "Concurrent tests are unsupported with isolated storage. Please either:",
        `- Remove \`.concurrent\` from the ${quotedName} test`,
        `- Remove \`.concurrent\` from all \`describe()\` blocks containing the ${quotedName} test`,
        "- Remove `isolatedStorage: true` from your project's Vitest config"
      ];
      throw new Error(msg.join("\n"));
    }
    const result = await super.onBeforeRunTask(test);
    this.syncCurrentTaskWithInitialState();
    return result;
  }
  async onAfterRunTask(test) {
    if (DEBUG) {
      __console.log(`${_(4)}onAfterRunTask: ${test.name}`);
      await scheduler.wait(100);
    }
    await this.ensurePoppedActiveTryStorage(test);
    tryStates.delete(test);
    const result = await super.onAfterRunTask(test);
    this.syncCurrentTaskWithInitialState();
    return result;
  }
  // @ts-expect-error `VitestRunner` defines an additional `options` parameter
  //  that `VitestTestRunner` doesn't use
  async onBeforeTryTask(test, options) {
    if (DEBUG) {
      __console.log(`${_(6)}onBeforeTryTask: ${test.name}`, options);
      await scheduler.wait(100);
    }
    const newActive = getTryKey(options);
    await this.ensurePoppedActiveTryStorage(test, newActive);
    await this.updateStackedStorage("push", test);
    return super.onBeforeTryTask(test);
  }
  // @ts-expect-error `VitestRunner` defines an additional `options` parameter
  //  that `VitestTestRunner` doesn't use
  async onAfterTryTask(test, options) {
    if (DEBUG) {
      __console.log(`${_(6)}onAfterTryTask: ${test.name}`, options);
      await scheduler.wait(100);
    }
    assert(await this.ensurePoppedActiveTryStorage(test));
    return super.onAfterTryTask(test);
  }
};
export {
  createWaitUntilRpc,
  WorkersTestRunner as default
};
//# sourceMappingURL=test-runner.mjs.map
