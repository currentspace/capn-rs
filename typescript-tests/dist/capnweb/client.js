var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });
class CapnWebClient {
  static {
    __name(this, "CapnWebClient");
  }
  transport;
  config;
  logger;
  nextCallId = 1;
  pendingCalls = /* @__PURE__ */ new Map();
  isListening = false;
  constructor(transport, config = {}, logger) {
    this.transport = transport;
    this.config = {
      timeout: 3e4,
      maxRetries: 3,
      retryDelay: 1e3,
      ...config
    };
    this.logger = logger || console;
  }
  async connect() {
    if (!this.isListening) {
      this.startListening();
    }
  }
  startListening() {
    this.isListening = true;
    this.listenForResponses();
  }
  async listenForResponses() {
    try {
      while (this.isListening) {
        const message = await this.transport.receive();
        if (!message) {
          await new Promise((resolve) => setTimeout(resolve, 100));
          continue;
        }
        await this.handleMessage(message);
      }
    } catch (error) {
      this.logger.error("Error listening for responses:", error);
      if (this.isListening) {
        setTimeout(() => this.listenForResponses(), 1e3);
      }
    }
  }
  async handleMessage(message) {
    if ("result" in message) {
      await this.handleResult(message);
    } else {
      this.logger.warn("Received unexpected message type:", message);
    }
  }
  async handleResult(message) {
    const { id } = message.result;
    const pendingCall = this.pendingCalls.get(id);
    if (!pendingCall) {
      this.logger.warn(`Received result for unknown call ID: ${id}`);
      return;
    }
    this.pendingCalls.delete(id);
    if (pendingCall.timeout) {
      clearTimeout(pendingCall.timeout);
    }
    if (message.result.success) {
      pendingCall.resolve(message.result.success.value);
    } else if (message.result.error) {
      const error = new Error(message.result.error.error.message);
      error.code = message.result.error.error.code;
      error.data = message.result.error.error.data;
      pendingCall.reject(error);
    } else {
      pendingCall.reject(new Error("Invalid result message format"));
    }
  }
  async call(capId, member, args) {
    const callId = this.nextCallId++;
    const message = {
      call: {
        id: callId,
        target: { cap: capId },
        member,
        args
      }
    };
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pendingCalls.delete(callId);
        reject(new Error(`Call timeout after ${this.config.timeout}ms`));
      }, this.config.timeout);
      this.pendingCalls.set(callId, { resolve, reject, timeout });
      this.transport.send(message).catch((error) => {
        this.pendingCalls.delete(callId);
        clearTimeout(timeout);
        reject(error);
      });
    });
  }
  async executePlan(plan, params) {
    this.logger.debug("Executing plan:", plan);
    throw new Error("Plan execution not yet implemented in TypeScript client");
  }
  async dispose(capIds) {
    const message = {
      dispose: {
        caps: capIds
      }
    };
    await this.transport.send(message);
    this.logger.debug(`Disposed capabilities: ${capIds.join(", ")}`);
  }
  async close() {
    this.isListening = false;
    for (const [callId, pendingCall] of this.pendingCalls) {
      if (pendingCall.timeout) {
        clearTimeout(pendingCall.timeout);
      }
      pendingCall.reject(new Error("Client closed"));
    }
    this.pendingCalls.clear();
    await this.transport.close();
  }
}
class PlanBuilder {
  static {
    __name(this, "PlanBuilder");
  }
  captures = [];
  ops = [];
  nextResult = 0;
  capture(capId) {
    const index = this.captures.length;
    this.captures.push(capId);
    return new CapRef(this, index);
  }
  addOp(op) {
    const resultIndex = this.nextResult++;
    this.ops.push({ ...op, result: resultIndex });
    return new ResultRef(this, resultIndex);
  }
  build(result) {
    return {
      captures: this.captures,
      ops: this.ops,
      result
    };
  }
  object(fields) {
    return this.addOp({
      object: {
        fields
      }
    });
  }
  array(items) {
    return this.addOp({
      array: {
        items
      }
    });
  }
}
class CapRef {
  constructor(builder, index) {
    this.builder = builder;
    this.index = index;
  }
  static {
    __name(this, "CapRef");
  }
  call(member, args) {
    return this.builder.addOp({
      call: {
        target: { capture: { index: this.index } },
        member,
        args
      }
    });
  }
  asSource() {
    return { capture: { index: this.index } };
  }
}
class ResultRef {
  constructor(builder, index) {
    this.builder = builder;
    this.index = index;
  }
  static {
    __name(this, "ResultRef");
  }
  call(member, args) {
    return this.builder.addOp({
      call: {
        target: { result: { index: this.index } },
        member,
        args
      }
    });
  }
  asSource() {
    return { result: { index: this.index } };
  }
}
const Param = {
  path: /* @__PURE__ */ __name((path) => ({
    param: { path }
  }), "path"),
  value: /* @__PURE__ */ __name((value) => ({
    byValue: { value }
  }), "value")
};
function isCallMessage(message) {
  return "call" in message;
}
__name(isCallMessage, "isCallMessage");
function isResultMessage(message) {
  return "result" in message;
}
__name(isResultMessage, "isResultMessage");

export { CapRef, CapnWebClient, Param, PlanBuilder, ResultRef, isCallMessage, isResultMessage };
//# sourceMappingURL=client.js.map
//# sourceMappingURL=client.js.map