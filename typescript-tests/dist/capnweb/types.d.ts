/**
 * Cap'n Web Protocol Types
 *
 * TypeScript implementation of the Cap'n Web protocol for interoperability testing
 * with the Rust implementation.
 */
type CapId = number;
type CallId = number;
type PromiseId = number;
interface CallMessage {
    call: {
        id: CallId;
        target: Target;
        member: string;
        args: unknown[];
    };
}
interface ResultMessage {
    result: {
        id: CallId;
        success?: {
            value: unknown;
        };
        error?: {
            error: RpcError;
        };
    };
}
interface CapRefMessage {
    capRef: {
        id: CapId;
    };
}
interface DisposeMessage {
    dispose: {
        caps: CapId[];
    };
}
type Message = CallMessage | ResultMessage | CapRefMessage | DisposeMessage;
interface CapTarget {
    cap: CapId;
}
interface PromiseTarget {
    promise: PromiseId;
}
type Target = CapTarget | PromiseTarget;
interface CaptureSource {
    capture: {
        index: number;
    };
}
interface ResultSource {
    result: {
        index: number;
    };
}
interface ParamSource {
    param: {
        path: string[];
    };
}
interface ByValueSource {
    byValue: {
        value: unknown;
    };
}
type Source = CaptureSource | ResultSource | ParamSource | ByValueSource;
interface CallOp {
    call: {
        target: Source;
        member: string;
        args: Source[];
        result: number;
    };
}
interface ObjectOp {
    object: {
        fields: Record<string, Source>;
        result: number;
    };
}
interface ArrayOp {
    array: {
        items: Source[];
        result: number;
    };
}
type Op = CallOp | ObjectOp | ArrayOp;
interface Plan {
    captures: CapId[];
    ops: Op[];
    result: Source;
}
interface RpcError {
    code: string;
    message: string;
    data?: unknown;
}
interface Transport {
    send(message: Message): Promise<void>;
    receive(): Promise<Message | null>;
    close(): Promise<void>;
}
interface ClientConfig {
    timeout?: number;
    maxRetries?: number;
    retryDelay?: number;
}
interface ServerConfig {
    port?: number;
    host?: string;
    path?: string;
}
interface Capability {
    call(member: string, args: unknown[]): Promise<unknown>;
    dispose?(): Promise<void>;
}
interface PendingPromise {
    id: PromiseId;
    resolve: (value: unknown) => void;
    reject: (error: Error) => void;
    dependencies: Set<PromiseId>;
}
interface TestCase {
    name: string;
    description: string;
    plan?: Plan;
    expectedResult?: unknown;
    expectedError?: string;
    capabilities: Record<CapId, MockCapability>;
}
interface MockCapability {
    name: string;
    methods: Record<string, (args: unknown[]) => Promise<unknown>>;
}
type JsonValue = string | number | boolean | null | JsonValue[] | {
    [key: string]: JsonValue;
};
interface Logger {
    debug(message: string, ...args: unknown[]): void;
    info(message: string, ...args: unknown[]): void;
    warn(message: string, ...args: unknown[]): void;
    error(message: string, ...args: unknown[]): void;
}

export type { ArrayOp, ByValueSource, CallId, CallMessage, CallOp, CapId, CapRefMessage, CapTarget, Capability, CaptureSource, ClientConfig, DisposeMessage, JsonValue, Logger, Message, MockCapability, ObjectOp, Op, ParamSource, PendingPromise, Plan, PromiseId, PromiseTarget, ResultMessage, ResultSource, RpcError, ServerConfig, Source, Target, TestCase, Transport };
