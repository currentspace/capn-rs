import WebSocket from 'ws';

var __defProp = Object.defineProperty;
var __name = (target, value) => __defProp(target, "name", { value, configurable: true });
class WebSocketTransport {
  constructor(url, logger) {
    this.url = url;
    this.logger = logger || console;
  }
  static {
    __name(this, "WebSocketTransport");
  }
  ws = null;
  messageQueue = [];
  responseQueue = [];
  resolveQueue = [];
  isConnected = false;
  logger;
  async connect() {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(this.url);
      this.ws.on("open", () => {
        this.logger.info(`WebSocket connected to ${this.url}`);
        this.isConnected = true;
        resolve();
        while (this.messageQueue.length > 0) {
          const message = this.messageQueue.shift();
          this.sendMessage(message);
        }
      });
      this.ws.on("message", (data) => {
        try {
          const message = JSON.parse(data.toString());
          this.logger.debug("Received message:", message);
          if (this.resolveQueue.length > 0) {
            const resolve2 = this.resolveQueue.shift();
            resolve2(message);
          } else {
            this.responseQueue.push(message);
          }
        } catch (error) {
          this.logger.error("Failed to parse message:", error);
        }
      });
      this.ws.on("close", (code, reason) => {
        this.logger.info(`WebSocket closed: ${code} ${reason}`);
        this.isConnected = false;
        while (this.resolveQueue.length > 0) {
          const resolve2 = this.resolveQueue.shift();
          resolve2(null);
        }
      });
      this.ws.on("error", (error) => {
        this.logger.error("WebSocket error:", error);
        reject(error);
      });
      setTimeout(() => {
        if (!this.isConnected) {
          reject(new Error("WebSocket connection timeout"));
        }
      }, 1e4);
    });
  }
  async send(message) {
    if (!this.isConnected || !this.ws) {
      this.messageQueue.push(message);
      return;
    }
    this.sendMessage(message);
  }
  sendMessage(message) {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      this.messageQueue.push(message);
      return;
    }
    try {
      const json = JSON.stringify(message);
      this.logger.debug("Sending message:", message);
      this.ws.send(json);
    } catch (error) {
      this.logger.error("Failed to send message:", error);
      throw error;
    }
  }
  async receive() {
    if (this.responseQueue.length > 0) {
      return this.responseQueue.shift();
    }
    return new Promise((resolve) => {
      this.resolveQueue.push(resolve);
      setTimeout(() => {
        const index = this.resolveQueue.indexOf(resolve);
        if (index !== -1) {
          this.resolveQueue.splice(index, 1);
          resolve(null);
        }
      }, 3e4);
    });
  }
  async close() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.isConnected = false;
  }
}
class HttpBatchTransport {
  constructor(url, logger) {
    this.url = url;
    this.logger = logger || console;
  }
  static {
    __name(this, "HttpBatchTransport");
  }
  messageQueue = [];
  logger;
  async send(message) {
    this.messageQueue.push(message);
  }
  async receive() {
    if (this.messageQueue.length === 0) {
      return null;
    }
    try {
      const messages = [...this.messageQueue];
      this.messageQueue.length = 0;
      this.logger.debug("Sending batch request:", messages);
      const response = await fetch(this.url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(messages)
      });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      const results = await response.json();
      this.logger.debug("Received batch response:", results);
      return results[0] || null;
    } catch (error) {
      this.logger.error("HTTP batch request failed:", error);
      throw error;
    }
  }
  async close() {
  }
}

export { HttpBatchTransport, WebSocketTransport };
//# sourceMappingURL=websocket-transport.js.map
//# sourceMappingURL=websocket-transport.js.map