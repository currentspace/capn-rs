import { Transport, Logger, Message } from './types.js';

/**
 * WebSocket Transport Implementation
 *
 * Implements the Transport interface for WebSocket communication
 * with the Rust Cap'n Web server.
 */

declare class WebSocketTransport implements Transport {
    private url;
    private ws;
    private messageQueue;
    private responseQueue;
    private resolveQueue;
    private isConnected;
    private logger;
    constructor(url: string, logger?: Logger);
    connect(): Promise<void>;
    send(message: Message): Promise<void>;
    private sendMessage;
    receive(): Promise<Message | null>;
    close(): Promise<void>;
}
declare class HttpBatchTransport implements Transport {
    private url;
    private messageQueue;
    private logger;
    constructor(url: string, logger?: Logger);
    send(message: Message): Promise<void>;
    receive(): Promise<Message | null>;
    close(): Promise<void>;
}

export { HttpBatchTransport, WebSocketTransport };
