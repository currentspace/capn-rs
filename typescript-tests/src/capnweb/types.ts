/**
 * Cap'n Web Protocol Types
 *
 * TypeScript implementation of the Cap'n Web protocol for interoperability testing
 * with the Rust implementation.
 */

export type CapId = number
export type CallId = number
export type PromiseId = number

// Message Types
export interface CallMessage {
  call: {
    id: CallId
    target: Target
    member: string
    args: unknown[]
  }
}

export interface ResultMessage {
  result: {
    id: CallId
    success?: {
      value: unknown
    }
    error?: {
      error: RpcError
    }
  }
}

export interface CapRefMessage {
  capRef: {
    id: CapId
  }
}

export interface DisposeMessage {
  dispose: {
    caps: CapId[]
  }
}

export type Message = CallMessage | ResultMessage | CapRefMessage | DisposeMessage

// Target Types
export interface CapTarget {
  cap: CapId
}

export interface PromiseTarget {
  promise: PromiseId
}

export type Target = CapTarget | PromiseTarget

// Source Types
export interface CaptureSource {
  capture: {
    index: number
  }
}

export interface ResultSource {
  result: {
    index: number
  }
}

export interface ParamSource {
  param: {
    path: string[]
  }
}

export interface ByValueSource {
  byValue: {
    value: unknown
  }
}

export type Source = CaptureSource | ResultSource | ParamSource | ByValueSource

// Operation Types
export interface CallOp {
  call: {
    target: Source
    member: string
    args: Source[]
    result: number
  }
}

export interface ObjectOp {
  object: {
    fields: Record<string, Source>
    result: number
  }
}

export interface ArrayOp {
  array: {
    items: Source[]
    result: number
  }
}

export type Op = CallOp | ObjectOp | ArrayOp

// Plan Type
export interface Plan {
  captures: CapId[]
  ops: Op[]
  result: Source
}

// Error Types
export interface RpcError {
  code: string
  message: string
  data?: unknown
}

// Transport Types
export interface Transport {
  send(message: Message): Promise<void>
  receive(): Promise<Message | null>
  close(): Promise<void>
}

// Client Configuration
export interface ClientConfig {
  timeout?: number
  maxRetries?: number
  retryDelay?: number
}

// Server Configuration
export interface ServerConfig {
  port?: number
  host?: string
  path?: string
}

// Capability Interface
export interface Capability {
  call(member: string, args: unknown[]): Promise<unknown>
  dispose?(): Promise<void>
}

// Promise Types for Pipelining
export interface PendingPromise {
  id: PromiseId
  resolve: (value: unknown) => void
  reject: (error: Error) => void
  dependencies: Set<PromiseId>
}

// Test Types
export interface TestCase {
  name: string
  description: string
  plan?: Plan
  expectedResult?: unknown
  expectedError?: string
  capabilities: Record<CapId, MockCapability>
}

export interface MockCapability {
  name: string
  methods: Record<string, (args: unknown[]) => Promise<unknown>>
}

// Utility Types
export type JsonValue =
  | string
  | number
  | boolean
  | null
  | JsonValue[]
  | { [key: string]: JsonValue }

export interface Logger {
  debug(message: string, ...args: unknown[]): void
  info(message: string, ...args: unknown[]): void
  warn(message: string, ...args: unknown[]): void
  error(message: string, ...args: unknown[]): void
}