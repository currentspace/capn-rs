import { ServerConfig, Logger, CapId, Capability } from './types.js';

/**
 * Cap'n Web TypeScript Server
 *
 * A TypeScript server implementation for testing Rust client interoperability.
 */

declare class CapnWebServer {
    private wss;
    private capabilities;
    private config;
    private logger;
    constructor(config?: ServerConfig, logger?: Logger);
    registerCapability(capId: CapId, capability: Capability): void;
    start(): Promise<void>;
    private handleConnection;
    private handleMessage;
    private handleCall;
    private handleCapRef;
    private handleDispose;
    stop(): Promise<void>;
}
/**
 * Mock capabilities for testing
 */
declare class MockCalculator implements Capability {
    call(member: string, args: unknown[]): Promise<unknown>;
}
declare class MockUserManager implements Capability {
    private users;
    call(member: string, args: unknown[]): Promise<unknown>;
}

export { CapnWebServer, MockCalculator, MockUserManager };
