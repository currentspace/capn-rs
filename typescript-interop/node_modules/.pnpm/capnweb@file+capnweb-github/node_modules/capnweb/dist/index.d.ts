import { IncomingMessage, ServerResponse, OutgoingHttpHeaders, OutgoingHttpHeader } from 'node:http';

// Copyright (c) 2025 Cloudflare, Inc.
// Licensed under the MIT license found in the LICENSE.txt file or at:
//     https://opensource.org/license/mit

// This file borrows heavily from `types/defines/rpc.d.ts` in workerd.

// Branded types for identifying `WorkerEntrypoint`/`DurableObject`/`Target`s.
// TypeScript uses *structural* typing meaning anything with the same shape as type `T` is a `T`.
// For the classes exported by `cloudflare:workers` we want *nominal* typing (i.e. we only want to
// accept `WorkerEntrypoint` from `cloudflare:workers`, not any other class with the same shape)
declare const __RPC_STUB_BRAND: '__RPC_STUB_BRAND';
declare const __RPC_TARGET_BRAND: '__RPC_TARGET_BRAND';
interface RpcTargetBranded {
  [__RPC_TARGET_BRAND]: never;
}

// Types that can be used through `Stub`s
type Stubable = RpcTargetBranded | ((...args: any[]) => any);

// Types that can be passed over RPC
// The reason for using a generic type here is to build a serializable subset of structured
//   cloneable composite types. This allows types defined with the "interface" keyword to pass the
//   serializable check as well. Otherwise, only types defined with the "type" keyword would pass.
type Serializable<T> =
  // Structured cloneables
  | BaseType
  // Structured cloneable composites
  | Map<
      T extends Map<infer U, unknown> ? Serializable<U> : never,
      T extends Map<unknown, infer U> ? Serializable<U> : never
    >
  | Set<T extends Set<infer U> ? Serializable<U> : never>
  | Array<T extends Array<infer U> ? Serializable<U> : never>
  | ReadonlyArray<T extends ReadonlyArray<infer U> ? Serializable<U> : never>
  | {
      [K in keyof T]: K extends number | string ? Serializable<T[K]> : never;
    }
  | Promise<T extends Promise<infer U> ? Serializable<U> : never>
  // Special types
  | Stub<Stubable>
  // Serialized as stubs, see `Stubify`
  | Stubable;

// Base type for all RPC stubs, including common memory management methods.
// `T` is used as a marker type for unwrapping `Stub`s later.
interface StubBase<T extends Serializable<T>> extends Disposable {
  [__RPC_STUB_BRAND]: T;
  dup(): this;
  onRpcBroken(callback: (error: any) => void): void;
}
type Stub<T extends Serializable<T>> =
    T extends object ? Provider<T> & StubBase<T> : StubBase<T>;

type TypedArray =
  | Uint8Array
  | Uint8ClampedArray
  | Uint16Array
  | Uint32Array
  | Int8Array
  | Int16Array
  | Int32Array
  | BigUint64Array
  | BigInt64Array
  | Float32Array
  | Float64Array;

// This represents all the types that can be sent as-is over an RPC boundary
type BaseType =
  | void
  | undefined
  | null
  | boolean
  | number
  | bigint
  | string
  | TypedArray
  | ArrayBuffer
  | DataView
  | Date
  | Error
  | RegExp
  | ReadableStream<Uint8Array>
  | WritableStream<Uint8Array>
  | Request
  | Response
  | Headers;
// Recursively rewrite all `Stubable` types with `Stub`s, and resolve promises.
// prettier-ignore
type Stubify<T> =
  T extends Stubable ? Stub<T>
  : T extends Promise<infer U> ? Stubify<U>
  : T extends StubBase<any> ? T
  : T extends Map<infer K, infer V> ? Map<Stubify<K>, Stubify<V>>
  : T extends Set<infer V> ? Set<Stubify<V>>
  : T extends Array<infer V> ? Array<Stubify<V>>
  : T extends ReadonlyArray<infer V> ? ReadonlyArray<Stubify<V>>
  : T extends BaseType ? T
  // When using "unknown" instead of "any", interfaces are not stubified.
  : T extends { [key: string | number]: any } ? { [K in keyof T]: Stubify<T[K]> }
  : T;

// Recursively rewrite all `Stub<T>`s with the corresponding `T`s.
// Note we use `StubBase` instead of `Stub` here to avoid circular dependencies:
// `Stub` depends on `Provider`, which depends on `Unstubify`, which would depend on `Stub`.
// prettier-ignore
type UnstubifyInner<T> =
  T extends StubBase<infer V> ? (T | V)  // can provide either stub or local RpcTarget
  : T extends Map<infer K, infer V> ? Map<Unstubify<K>, Unstubify<V>>
  : T extends Set<infer V> ? Set<Unstubify<V>>
  : T extends Array<infer V> ? Array<Unstubify<V>>
  : T extends ReadonlyArray<infer V> ? ReadonlyArray<Unstubify<V>>
  : T extends BaseType ? T
  : T extends { [key: string | number]: unknown } ? { [K in keyof T]: Unstubify<T[K]> }
  : T;

// You can put promises anywhere in the params and they'll be resolved before delivery.
// (This also covers RpcPromise, because it's defined as being a Promise.)
type Unstubify<T> = UnstubifyInner<T> | Promise<UnstubifyInner<T>>

type UnstubifyAll<A extends any[]> = { [I in keyof A]: Unstubify<A[I]> };

// Utility type for adding `Disposable`s to `object` types only.
// Note `unknown & T` is equivalent to `T`.
type MaybeDisposable<T> = T extends object ? Disposable : unknown;

// Type for method return or property on an RPC interface.
// - Stubable types are replaced by stubs.
// - Serializable types are passed by value, with stubable types replaced by stubs
//   and a top-level `Disposer`.
// Everything else can't be passed over RPC.
// Technically, we use custom thenables here, but they quack like `Promise`s.
// Intersecting with `(Maybe)Provider` allows pipelining.
// prettier-ignore
type Result<R> =
  R extends Stubable ? Promise<Stub<R>> & Provider<R> & StubBase<R>
  : R extends Serializable<R> ? Promise<Stubify<R> & MaybeDisposable<R>> & Provider<R> & StubBase<R>
  : never;

// Type for method or property on an RPC interface.
// For methods, unwrap `Stub`s in parameters, and rewrite returns to be `Result`s.
// Unwrapping `Stub`s allows calling with `Stubable` arguments.
// For properties, rewrite types to be `Result`s.
// In each case, unwrap `Promise`s.
type MethodOrProperty<V> = V extends (...args: infer P) => infer R
  ? (...args: UnstubifyAll<P>) => Result<Awaited<R>>
  : Result<Awaited<V>>;

// Type for the callable part of an `Provider` if `T` is callable.
// This is intersected with methods/properties.
type MaybeCallableProvider<T> = T extends (...args: any[]) => any
  ? MethodOrProperty<T>
  : unknown;

// Base type for all other types providing RPC-like interfaces.
// Rewrites all methods/properties to be `MethodOrProperty`s, while preserving callable types.
type Provider<T> = MaybeCallableProvider<T> &
  (T extends Array<infer U>
    ? {
        [key: number]: MethodOrProperty<U>;
      } & {
        map<V>(callback: (elem: U) => V): Result<Array<V>>;
      }
    : {
        [K in Exclude<
          keyof T,
          symbol | keyof StubBase<never>
        >]: MethodOrProperty<T[K]>;
      } & {
        map<V>(callback: (value: NonNullable<T>) => V): Result<Array<V>>;
      });

/**
 * Serialize a value, using Cap'n Web's underlying serialization. This won't be able to serialize
 * RPC stubs, but it will support basic data types.
 */
declare function serialize(value: unknown): string;
/**
 * Deserialize a value serialized using serialize().
 */
declare function deserialize(value: string): unknown;

/**
 * Interface for an RPC transport, which is a simple bidirectional message stream. Implement this
 * interface if the built-in transports (e.g. for HTTP batch and WebSocket) don't meet your needs.
 */
interface RpcTransport {
    /**
     * Sends a message to the other end.
     */
    send(message: string): Promise<void>;
    /**
     * Receives a message sent by the other end.
     *
     * If and when the transport becomes disconnected, this will reject. The thrown error will be
     * propagated to all outstanding calls and future calls on any stubs associated with the session.
     * If there are no outstanding calls (and none are made in the future), then the error does not
     * propagate anywhere -- this is considered a "clean" shutdown.
     */
    receive(): Promise<string>;
    /**
     * Indicates that the RPC system has suffered an error that prevents the session from continuing.
     * The transport should ideally try to send any queued messages if it can, and then close the
     * connection. (It's not strictly necessary to deliver queued messages, but the last message sent
     * before abort() is called is often an "abort" message, which communicates the error to the
     * peer, so if that is dropped, the peer may have less information about what happened.)
     */
    abort?(reason: any): void;
}
/**
 * Options to customize behavior of an RPC session. All functions which start a session should
 * optionally accept this.
 */
type RpcSessionOptions = {
    /**
     * If provided, this function will be called whenever an `Error` object is serialized (for any
     * reason, not just because it was thrown). This can be used to log errors, and also to redact
     * them.
     *
     * If `onSendError` returns an Error object, than object will be substituted in place of the
     * original. If it has a stack property, the stack will be sent to the client.
     *
     * If `onSendError` doesn't return anything (or is not provided at all), the default behavior is
     * to serialize the error with the stack omitted.
     */
    onSendError?: (error: Error) => Error | void;
};

/**
 * For use in Cloudflare Workers: Construct an HTTP response that starts a WebSocket RPC session
 * with the given `localMain`.
 */
declare function newWorkersWebSocketRpcResponse(request: Request, localMain?: any, options?: RpcSessionOptions): Response;

/**
 * Implements the server end of an HTTP batch session, using standard Fetch API types to represent
 * HTTP requests and responses.
 *
 * @param request The request received from the client initiating the session.
 * @param localMain The main stub or RpcTarget which the server wishes to expose to the client.
 * @param options Optional RPC session options.
 * @returns The HTTP response to return to the client. Note that the returned object has mutable
 *     headers, so you can modify them using e.g. `response.headers.set("Foo", "bar")`.
 */
declare function newHttpBatchRpcResponse(request: Request, localMain: any, options?: RpcSessionOptions): Promise<Response>;
/**
 * Implements the server end of an HTTP batch session using traditional Node.js HTTP APIs.
 *
 * @param request The request received from the client initiating the session.
 * @param response The response object, to which the response should be written.
 * @param localMain The main stub or RpcTarget which the server wishes to expose to the client.
 * @param options Optional RPC session options. You can also pass headers to set on the response.
 */
declare function nodeHttpBatchRpcResponse(request: IncomingMessage, response: ServerResponse, localMain: any, options?: RpcSessionOptions & {
    headers?: OutgoingHttpHeaders | OutgoingHttpHeader[];
}): Promise<void>;

/**
 * Represents a reference to a remote object, on which methods may be remotely invoked via RPC.
 *
 * `RpcStub` can represent any interface (when using TypeScript, you pass the specific interface
 * type as `T`, but this isn't known at runtime). The way this works is, `RpcStub` is actually a
 * `Proxy`. It makes itself appear as if every possible method / property name is defined. You can
 * invoke any method name, and the invocation will be sent to the server. If it turns out that no
 * such method exists on the remote object, an exception is thrown back. But the client does not
 * actually know, until that point, what methods exist.
 */
type RpcStub<T extends Serializable<T>> = Stub<T>;
declare const RpcStub: {
    new <T extends Serializable<T>>(value: T): RpcStub<T>;
};
/**
 * Represents the result of an RPC call.
 *
 * Also used to represent properties. That is, `stub.foo` evaluates to an `RpcPromise` for the
 * value of `foo`.
 *
 * This isn't actually a JavaScript `Promise`. It does, however, have `then()`, `catch()`, and
 * `finally()` methods, like `Promise` does, and because it has a `then()` method, JavaScript will
 * allow you to treat it like a promise, e.g. you can `await` it.
 *
 * An `RpcPromise` is also a proxy, just like `RpcStub`, where calling methods or awaiting
 * properties will make a pipelined network request.
 *
 * Note that and `RpcPromise` is "lazy": the actual final result is not requested from the server
 * until you actually `await` the promise (or call `then()`, etc. on it). This is an optimization:
 * if you only intend to use the promise for pipelining and you never await it, then there's no
 * need to transmit the resolution!
 */
type RpcPromise<T extends Serializable<T>> = Stub<T> & Promise<Stubify<T>>;
declare const RpcPromise: {};
/**
 * Use to construct an `RpcSession` on top of a custom `RpcTransport`.
 *
 * Most people won't use this. You only need it if you've implemented your own `RpcTransport`.
 */
interface RpcSession<T extends Serializable<T> = undefined> {
    getRemoteMain(): RpcStub<T>;
    getStats(): {
        imports: number;
        exports: number;
    };
    drain(): Promise<void>;
}
declare const RpcSession: {
    new <T extends Serializable<T> = undefined>(transport: RpcTransport, localMain?: any, options?: RpcSessionOptions): RpcSession<T>;
};
/**
 * Classes which are intended to be passed by reference and called over RPC must extend
 * `RpcTarget`. A class which does not extend `RpcTarget` (and which doesn't have built-in support
 * from the RPC system) cannot be passed in an RPC message at all; an exception will be thrown.
 *
 * Note that on Cloudflare Workers, this `RpcTarget` is an alias for the one exported from the
 * "cloudflare:workers" module, so they can be used interchangably.
 */
interface RpcTarget extends RpcTargetBranded {
}
declare const RpcTarget: {
    new (): RpcTarget;
};
/**
 * Empty interface used as default type parameter for sessions where the other side doesn't
 * necessarily export a main interface.
 */
interface Empty {
}
/**
 * Start a WebSocket session given either an already-open WebSocket or a URL.
 *
 * @param webSocket Either the `wss://` URL to connect to, or an already-open WebSocket object to
 * use.
 * @param localMain The main RPC interface to expose to the peer. Returns a stub for the main
 * interface exposed from the peer.
 */
declare let newWebSocketRpcSession: <T extends Serializable<T> = Empty>(webSocket: WebSocket | string, localMain?: any, options?: RpcSessionOptions) => RpcStub<T>;
/**
 * Initiate an HTTP batch session from the client side.
 *
 * The parameters to this method have exactly the same signature as `fetch()`, but the return
 * value is an RpcStub. You can customize anything about the request except for the method
 * (it will always be set to POST) and the body (which the RPC system will fill in).
 */
declare let newHttpBatchRpcSession: <T extends Serializable<T>>(urlOrRequest: string | Request, init?: RequestInit) => RpcStub<T>;
/**
 * Initiate an RPC session over a MessagePort, which is particularly useful for communicating
 * between an iframe and its parent frame in a browser context. Each side should call this function
 * on its own end of the MessageChannel.
 */
declare let newMessagePortRpcSession: <T extends Serializable<T> = Empty>(port: MessagePort, localMain?: any, options?: RpcSessionOptions) => RpcStub<T>;
/**
 * Implements unified handling of HTTP-batch and WebSocket responses for the Cloudflare Workers
 * Runtime.
 *
 * SECURITY WARNING: This function accepts cross-origin requests. If you do not want this, you
 * should validate the `Origin` header before calling this, or use `newHttpBatchRpcSession()` and
 * `newWebSocketRpcSession()` directly with appropriate security measures for each type of request.
 * But if your API uses in-band authorization (i.e. it has an RPC method that takes the user's
 * credentials as parameters and returns the authorized API), then cross-origin requests should
 * be safe.
 */
declare function newWorkersRpcResponse(request: Request, localMain: any): Promise<Response>;

export { RpcPromise, RpcSession, type RpcSessionOptions, RpcStub, RpcTarget, type RpcTransport, deserialize, newHttpBatchRpcResponse, newHttpBatchRpcSession, newMessagePortRpcSession, newWebSocketRpcSession, newWorkersRpcResponse, newWorkersWebSocketRpcResponse, nodeHttpBatchRpcResponse, serialize };
