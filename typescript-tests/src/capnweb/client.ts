/**
 * Cap'n Web TypeScript Client
 *
 * A complete TypeScript implementation of the Cap'n Web client
 * for interoperability testing with the Rust server.
 */

import type {
  Transport,
  Message,
  CallMessage,
  ResultMessage,
  Plan,
  Source,
  CapId,
  CallId,
  ClientConfig,
  Logger,
  JsonValue
} from './types.js'

interface PendingCall {
  resolve: (value: unknown) => void
  reject: (error: Error) => void
  timeout?: NodeJS.Timeout
}

export class CapnWebClient {
  private transport: Transport
  private config: ClientConfig
  private logger: Logger
  private nextCallId = 1
  private pendingCalls = new Map<CallId, PendingCall>()
  private isListening = false

  constructor(transport: Transport, config: ClientConfig = {}, logger?: Logger) {
    this.transport = transport
    this.config = {
      timeout: 30000,
      maxRetries: 3,
      retryDelay: 1000,
      ...config
    }
    this.logger = logger || console
  }

  async connect(): Promise<void> {
    // Start listening for responses if not already listening
    if (!this.isListening) {
      this.startListening()
    }
  }

  private startListening(): void {
    this.isListening = true
    this.listenForResponses()
  }

  private async listenForResponses(): Promise<void> {
    try {
      while (this.isListening) {
        const message = await this.transport.receive()
        if (!message) {
          await new Promise(resolve => setTimeout(resolve, 100))
          continue
        }

        await this.handleMessage(message)
      }
    } catch (error) {
      this.logger.error('Error listening for responses:', error)
      // Retry listening after a delay
      if (this.isListening) {
        setTimeout(() => this.listenForResponses(), 1000)
      }
    }
  }

  private async handleMessage(message: Message): Promise<void> {
    if ('result' in message) {
      await this.handleResult(message as ResultMessage)
    } else {
      this.logger.warn('Received unexpected message type:', message)
    }
  }

  private async handleResult(message: ResultMessage): Promise<void> {
    const { id } = message.result
    const pendingCall = this.pendingCalls.get(id)

    if (!pendingCall) {
      this.logger.warn(`Received result for unknown call ID: ${id}`)
      return
    }

    this.pendingCalls.delete(id)

    if (pendingCall.timeout) {
      clearTimeout(pendingCall.timeout)
    }

    if (message.result.success) {
      pendingCall.resolve(message.result.success.value)
    } else if (message.result.error) {
      const error = new Error(message.result.error.error.message)
      ;(error as any).code = message.result.error.error.code
      ;(error as any).data = message.result.error.error.data
      pendingCall.reject(error)
    } else {
      pendingCall.reject(new Error('Invalid result message format'))
    }
  }

  async call(capId: CapId, member: string, args: unknown[]): Promise<unknown> {
    const callId = this.nextCallId++

    const message: CallMessage = {
      call: {
        id: callId,
        target: { cap: capId },
        member,
        args
      }
    }

    return new Promise((resolve, reject) => {
      // Set up timeout
      const timeout = setTimeout(() => {
        this.pendingCalls.delete(callId)
        reject(new Error(`Call timeout after ${this.config.timeout}ms`))
      }, this.config.timeout)

      this.pendingCalls.set(callId, { resolve, reject, timeout })

      // Send the message
      this.transport.send(message).catch(error => {
        this.pendingCalls.delete(callId)
        clearTimeout(timeout)
        reject(error)
      })
    })
  }

  async executePlan(plan: Plan, params?: Record<string, unknown>): Promise<unknown> {
    // For now, we'll execute plans by converting them to individual calls
    // A full implementation would send the plan to a plan execution endpoint
    this.logger.debug('Executing plan:', plan)

    // This is a simplified implementation
    // In practice, you'd send the plan to the server for execution
    throw new Error('Plan execution not yet implemented in TypeScript client')
  }

  async dispose(capIds: CapId[]): Promise<void> {
    const message = {
      dispose: {
        caps: capIds
      }
    }

    await this.transport.send(message)
    this.logger.debug(`Disposed capabilities: ${capIds.join(', ')}`)
  }

  async close(): Promise<void> {
    this.isListening = false

    // Reject all pending calls
    for (const [callId, pendingCall] of this.pendingCalls) {
      if (pendingCall.timeout) {
        clearTimeout(pendingCall.timeout)
      }
      pendingCall.reject(new Error('Client closed'))
    }
    this.pendingCalls.clear()

    await this.transport.close()
  }
}

/**
 * Utility class for building Cap'n Web plans in TypeScript
 */
export class PlanBuilder {
  private captures: CapId[] = []
  private ops: any[] = []
  private nextResult = 0

  capture(capId: CapId): CapRef {
    const index = this.captures.length
    this.captures.push(capId)
    return new CapRef(this, index)
  }

  addOp(op: any): ResultRef {
    const resultIndex = this.nextResult++
    this.ops.push({ ...op, result: resultIndex })
    return new ResultRef(this, resultIndex)
  }

  build(result: Source): Plan {
    return {
      captures: this.captures,
      ops: this.ops,
      result
    }
  }

  object(fields: Record<string, Source>): ResultRef {
    return this.addOp({
      object: {
        fields
      }
    })
  }

  array(items: Source[]): ResultRef {
    return this.addOp({
      array: {
        items
      }
    })
  }
}

export class CapRef {
  constructor(
    private builder: PlanBuilder,
    private index: number
  ) {}

  call(member: string, args: Source[]): ResultRef {
    return this.builder.addOp({
      call: {
        target: { capture: { index: this.index } },
        member,
        args
      }
    })
  }

  asSource(): Source {
    return { capture: { index: this.index } }
  }
}

export class ResultRef {
  constructor(
    private builder: PlanBuilder,
    private index: number
  ) {}

  call(member: string, args: Source[]): ResultRef {
    return this.builder.addOp({
      call: {
        target: { result: { index: this.index } },
        member,
        args
      }
    })
  }

  asSource(): Source {
    return { result: { index: this.index } }
  }
}

// Utility functions for creating sources
export const Param = {
  path: (path: string[]): Source => ({
    param: { path }
  }),

  value: (value: JsonValue): Source => ({
    byValue: { value }
  })
}

// Type guards
export function isCallMessage(message: Message): message is CallMessage {
  return 'call' in message
}

export function isResultMessage(message: Message): message is ResultMessage {
  return 'result' in message
}