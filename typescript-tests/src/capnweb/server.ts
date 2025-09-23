/**
 * Cap'n Web TypeScript Server
 *
 * A TypeScript server implementation for testing Rust client interoperability.
 */

import WebSocket, { WebSocketServer } from 'ws'
import type {
  Message,
  CallMessage,
  ResultMessage,
  CapRefMessage,
  DisposeMessage,
  Capability,
  CapId,
  CallId,
  ServerConfig,
  Logger
} from './types.js'

interface RegisteredCapability {
  capability: Capability
  refCount: number
}

export class CapnWebServer {
  private wss: WebSocketServer | null = null
  private capabilities = new Map<CapId, RegisteredCapability>()
  private config: ServerConfig
  private logger: Logger

  constructor(config: ServerConfig = {}, logger?: Logger) {
    this.config = {
      port: 8080,
      host: 'localhost',
      path: '/ws',
      ...config
    }
    this.logger = logger || console
  }

  registerCapability(capId: CapId, capability: Capability): void {
    this.capabilities.set(capId, {
      capability,
      refCount: 0
    })
    this.logger.info(`Registered capability ${capId}`)
  }

  async start(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.wss = new WebSocketServer({
          port: this.config.port,
          host: this.config.host,
          path: this.config.path
        })

        this.wss.on('connection', (ws) => {
          this.logger.info('New WebSocket connection')
          this.handleConnection(ws)
        })

        this.wss.on('listening', () => {
          this.logger.info(`Server listening on ${this.config.host}:${this.config.port}${this.config.path}`)
          resolve()
        })

        this.wss.on('error', (error) => {
          this.logger.error('Server error:', error)
          reject(error)
        })
      } catch (error) {
        reject(error)
      }
    })
  }

  private handleConnection(ws: WebSocket): void {
    ws.on('message', async (data) => {
      try {
        const message = JSON.parse(data.toString()) as Message
        this.logger.debug('Received message:', message)

        const response = await this.handleMessage(message)
        if (response) {
          const responseJson = JSON.stringify(response)
          this.logger.debug('Sending response:', response)
          ws.send(responseJson)
        }
      } catch (error) {
        this.logger.error('Error handling message:', error)

        // Send error response if possible
        const errorResponse: ResultMessage = {
          result: {
            id: 0, // We don't have the original call ID
            error: {
              error: {
                code: 'INTERNAL_ERROR',
                message: error instanceof Error ? error.message : 'Unknown error'
              }
            }
          }
        }
        ws.send(JSON.stringify(errorResponse))
      }
    })

    ws.on('close', () => {
      this.logger.info('WebSocket connection closed')
    })

    ws.on('error', (error) => {
      this.logger.error('WebSocket error:', error)
    })
  }

  private async handleMessage(message: Message): Promise<Message | null> {
    if ('call' in message) {
      return this.handleCall(message as CallMessage)
    } else if ('capRef' in message) {
      return this.handleCapRef(message as CapRefMessage)
    } else if ('dispose' in message) {
      return this.handleDispose(message as DisposeMessage)
    } else {
      this.logger.warn('Unknown message type:', message)
      return null
    }
  }

  private async handleCall(message: CallMessage): Promise<ResultMessage> {
    const { id, target, member, args } = message.call

    try {
      // Extract capability ID from target
      let capId: CapId
      if ('cap' in target) {
        capId = target.cap
      } else {
        throw new Error('Promise targets not yet supported')
      }

      const registered = this.capabilities.get(capId)
      if (!registered) {
        throw new Error(`Capability ${capId} not found`)
      }

      const result = await registered.capability.call(member, args)

      return {
        result: {
          id,
          success: {
            value: result
          }
        }
      }
    } catch (error) {
      this.logger.error('Call failed:', error)

      return {
        result: {
          id,
          error: {
            error: {
              code: 'CALL_FAILED',
              message: error instanceof Error ? error.message : 'Unknown error',
              data: error instanceof Error ? { stack: error.stack } : undefined
            }
          }
        }
      }
    }
  }

  private async handleCapRef(message: CapRefMessage): Promise<Message | null> {
    const { id } = message.capRef

    const registered = this.capabilities.get(id)
    if (registered) {
      registered.refCount++
      this.logger.debug(`Incremented ref count for capability ${id} to ${registered.refCount}`)
    }

    // Cap refs don't require a response in the basic protocol
    return null
  }

  private async handleDispose(message: DisposeMessage): Promise<Message | null> {
    const { caps } = message.dispose

    for (const capId of caps) {
      const registered = this.capabilities.get(capId)
      if (registered) {
        registered.refCount = Math.max(0, registered.refCount - 1)
        this.logger.debug(`Decremented ref count for capability ${capId} to ${registered.refCount}`)

        if (registered.refCount === 0 && registered.capability.dispose) {
          try {
            await registered.capability.dispose()
            this.logger.info(`Disposed capability ${capId}`)
          } catch (error) {
            this.logger.error(`Error disposing capability ${capId}:`, error)
          }
        }
      }
    }

    // Dispose messages don't require a response
    return null
  }

  async stop(): Promise<void> {
    if (this.wss) {
      return new Promise((resolve) => {
        this.wss!.close(() => {
          this.logger.info('Server stopped')
          resolve()
        })
      })
    }
  }
}

/**
 * Mock capabilities for testing
 */
export class MockCalculator implements Capability {
  async call(member: string, args: unknown[]): Promise<unknown> {
    switch (member) {
      case 'add':
        if (args.length !== 2) throw new Error('add requires 2 arguments')
        return (args[0] as number) + (args[1] as number)

      case 'subtract':
        if (args.length !== 2) throw new Error('subtract requires 2 arguments')
        return (args[0] as number) - (args[1] as number)

      case 'multiply':
        if (args.length !== 2) throw new Error('multiply requires 2 arguments')
        return (args[0] as number) * (args[1] as number)

      case 'divide':
        if (args.length !== 2) throw new Error('divide requires 2 arguments')
        const divisor = args[1] as number
        if (divisor === 0) throw new Error('Division by zero')
        return (args[0] as number) / divisor

      case 'power':
        if (args.length !== 2) throw new Error('power requires 2 arguments')
        return Math.pow(args[0] as number, args[1] as number)

      case 'sqrt':
        if (args.length !== 1) throw new Error('sqrt requires 1 argument')
        const value = args[0] as number
        if (value < 0) throw new Error('Cannot take square root of negative number')
        return Math.sqrt(value)

      case 'factorial':
        if (args.length !== 1) throw new Error('factorial requires 1 argument')
        const n = args[0] as number
        if (n < 0) throw new Error('Factorial not defined for negative numbers')
        if (n > 20) throw new Error('Factorial too large (max 20)')
        let result = 1
        for (let i = 1; i <= n; i++) {
          result *= i
        }
        return result

      default:
        throw new Error(`Unknown method: ${member}`)
    }
  }
}

export class MockUserManager implements Capability {
  private users = new Map([
    [1, { id: 1, name: 'Alice', email: 'alice@example.com', role: 'admin' }],
    [2, { id: 2, name: 'Bob', email: 'bob@example.com', role: 'user' }],
    [3, { id: 3, name: 'Charlie', email: 'charlie@example.com', role: 'user' }]
  ])

  async call(member: string, args: unknown[]): Promise<unknown> {
    switch (member) {
      case 'getUser':
        if (args.length !== 1) throw new Error('getUser requires 1 argument')
        const userId = args[0] as number
        const user = this.users.get(userId)
        if (!user) throw new Error('User not found')
        return user

      case 'createUser':
        if (args.length !== 1) throw new Error('createUser requires 1 argument')
        const userData = args[0] as any
        const newUser = {
          id: 999,
          name: userData.name || 'Unknown',
          email: userData.email || 'unknown@example.com',
          role: 'user',
          created: true
        }
        return newUser

      default:
        throw new Error(`Unknown method: ${member}`)
    }
  }
}