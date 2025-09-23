import { WebSocketServer } from 'ws';

var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });
class CapnWebServer {
  static {
    __name(this, "CapnWebServer");
  }
  wss = null;
  capabilities = /* @__PURE__ */ new Map();
  config;
  logger;
  constructor(config = {}, logger) {
    this.config = {
      port: 8080,
      host: "localhost",
      path: "/ws",
      ...config
    };
    this.logger = logger || console;
  }
  registerCapability(capId, capability) {
    this.capabilities.set(capId, {
      capability,
      refCount: 0
    });
    this.logger.info(`Registered capability ${capId}`);
  }
  async start() {
    return new Promise((resolve, reject) => {
      try {
        this.wss = new WebSocketServer({
          port: this.config.port,
          host: this.config.host,
          path: this.config.path
        });
        this.wss.on("connection", (ws) => {
          this.logger.info("New WebSocket connection");
          this.handleConnection(ws);
        });
        this.wss.on("listening", () => {
          this.logger.info(`Server listening on ${this.config.host}:${this.config.port}${this.config.path}`);
          resolve();
        });
        this.wss.on("error", (error) => {
          this.logger.error("Server error:", error);
          reject(error);
        });
      } catch (error) {
        reject(error);
      }
    });
  }
  handleConnection(ws) {
    ws.on("message", async (data) => {
      try {
        const message = JSON.parse(data.toString());
        this.logger.debug("Received message:", message);
        const response = await this.handleMessage(message);
        if (response) {
          const responseJson = JSON.stringify(response);
          this.logger.debug("Sending response:", response);
          ws.send(responseJson);
        }
      } catch (error) {
        this.logger.error("Error handling message:", error);
        const errorResponse = {
          result: {
            id: 0,
            // We don't have the original call ID
            error: {
              error: {
                code: "INTERNAL_ERROR",
                message: error instanceof Error ? error.message : "Unknown error"
              }
            }
          }
        };
        ws.send(JSON.stringify(errorResponse));
      }
    });
    ws.on("close", () => {
      this.logger.info("WebSocket connection closed");
    });
    ws.on("error", (error) => {
      this.logger.error("WebSocket error:", error);
    });
  }
  async handleMessage(message) {
    if ("call" in message) {
      return this.handleCall(message);
    } else if ("capRef" in message) {
      return this.handleCapRef(message);
    } else if ("dispose" in message) {
      return this.handleDispose(message);
    } else {
      this.logger.warn("Unknown message type:", message);
      return null;
    }
  }
  async handleCall(message) {
    const { id, target, member, args } = message.call;
    try {
      let capId;
      if ("cap" in target) {
        capId = target.cap;
      } else {
        throw new Error("Promise targets not yet supported");
      }
      const registered = this.capabilities.get(capId);
      if (!registered) {
        throw new Error(`Capability ${capId} not found`);
      }
      const result = await registered.capability.call(member, args);
      return {
        result: {
          id,
          success: {
            value: result
          }
        }
      };
    } catch (error) {
      this.logger.error("Call failed:", error);
      return {
        result: {
          id,
          error: {
            error: {
              code: "CALL_FAILED",
              message: error instanceof Error ? error.message : "Unknown error",
              data: error instanceof Error ? { stack: error.stack } : void 0
            }
          }
        }
      };
    }
  }
  async handleCapRef(message) {
    const { id } = message.capRef;
    const registered = this.capabilities.get(id);
    if (registered) {
      registered.refCount++;
      this.logger.debug(`Incremented ref count for capability ${id} to ${registered.refCount}`);
    }
    return null;
  }
  async handleDispose(message) {
    const { caps } = message.dispose;
    for (const capId of caps) {
      const registered = this.capabilities.get(capId);
      if (registered) {
        registered.refCount = Math.max(0, registered.refCount - 1);
        this.logger.debug(`Decremented ref count for capability ${capId} to ${registered.refCount}`);
        if (registered.refCount === 0 && registered.capability.dispose) {
          try {
            await registered.capability.dispose();
            this.logger.info(`Disposed capability ${capId}`);
          } catch (error) {
            this.logger.error(`Error disposing capability ${capId}:`, error);
          }
        }
      }
    }
    return null;
  }
  async stop() {
    if (this.wss) {
      return new Promise((resolve) => {
        this.wss.close(() => {
          this.logger.info("Server stopped");
          resolve();
        });
      });
    }
  }
}
class MockCalculator {
  static {
    __name(this, "MockCalculator");
  }
  async call(member, args) {
    switch (member) {
      case "add":
        if (args.length !== 2) throw new Error("add requires 2 arguments");
        return args[0] + args[1];
      case "subtract":
        if (args.length !== 2) throw new Error("subtract requires 2 arguments");
        return args[0] - args[1];
      case "multiply":
        if (args.length !== 2) throw new Error("multiply requires 2 arguments");
        return args[0] * args[1];
      case "divide":
        if (args.length !== 2) throw new Error("divide requires 2 arguments");
        const divisor = args[1];
        if (divisor === 0) throw new Error("Division by zero");
        return args[0] / divisor;
      case "power":
        if (args.length !== 2) throw new Error("power requires 2 arguments");
        return Math.pow(args[0], args[1]);
      case "sqrt":
        if (args.length !== 1) throw new Error("sqrt requires 1 argument");
        const value = args[0];
        if (value < 0) throw new Error("Cannot take square root of negative number");
        return Math.sqrt(value);
      case "factorial":
        if (args.length !== 1) throw new Error("factorial requires 1 argument");
        const n = args[0];
        if (n < 0) throw new Error("Factorial not defined for negative numbers");
        if (n > 20) throw new Error("Factorial too large (max 20)");
        let result = 1;
        for (let i = 1; i <= n; i++) {
          result *= i;
        }
        return result;
      default:
        throw new Error(`Unknown method: ${member}`);
    }
  }
}
class MockUserManager {
  static {
    __name(this, "MockUserManager");
  }
  users = /* @__PURE__ */ new Map([
    [1, { id: 1, name: "Alice", email: "alice@example.com", role: "admin" }],
    [2, { id: 2, name: "Bob", email: "bob@example.com", role: "user" }],
    [3, { id: 3, name: "Charlie", email: "charlie@example.com", role: "user" }]
  ]);
  async call(member, args) {
    switch (member) {
      case "getUser":
        if (args.length !== 1) throw new Error("getUser requires 1 argument");
        const userId = args[0];
        const user = this.users.get(userId);
        if (!user) throw new Error("User not found");
        return user;
      case "createUser":
        if (args.length !== 1) throw new Error("createUser requires 1 argument");
        const userData = args[0];
        const newUser = {
          id: 999,
          name: userData.name || "Unknown",
          email: userData.email || "unknown@example.com",
          role: "user",
          created: true
        };
        return newUser;
      default:
        throw new Error(`Unknown method: ${member}`);
    }
  }
}

export { CapnWebServer, MockCalculator, MockUserManager };
//# sourceMappingURL=server.js.map
//# sourceMappingURL=server.js.map