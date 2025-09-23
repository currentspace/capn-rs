/**
 * WebSocket Transport Implementation
 *
 * Implements the Transport interface for WebSocket communication
 * with the Rust Cap'n Web server.
 */

import WebSocket from 'ws'
import type { Transport, Message, Logger } from './types.js'

export class WebSocketTransport implements Transport {
  private ws: WebSocket | null = null
  private messageQueue: Message[] = []
  private responseQueue: Message[] = []
  private resolveQueue: Array<(message: Message | null) => void> = []
  private isConnected = false
  private logger: Logger

  constructor(
    private url: string,
    logger?: Logger
  ) {
    this.logger = logger || console
  }

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(this.url)

      this.ws.on('open', () => {
        this.logger.info(`WebSocket connected to ${this.url}`)
        this.isConnected = true
        resolve()

        // Send queued messages
        while (this.messageQueue.length > 0) {
          const message = this.messageQueue.shift()!
          this.sendMessage(message)
        }
      })

      this.ws.on('message', (data) => {
        try {
          const message = JSON.parse(data.toString()) as Message
          this.logger.debug('Received message:', message)

          // Handle queued receive requests
          if (this.resolveQueue.length > 0) {
            const resolve = this.resolveQueue.shift()!
            resolve(message)
          } else {
            this.responseQueue.push(message)
          }
        } catch (error) {
          this.logger.error('Failed to parse message:', error)
        }
      })

      this.ws.on('close', (code, reason) => {
        this.logger.info(`WebSocket closed: ${code} ${reason}`)
        this.isConnected = false

        // Resolve pending receives with null
        while (this.resolveQueue.length > 0) {
          const resolve = this.resolveQueue.shift()!
          resolve(null)
        }
      })

      this.ws.on('error', (error) => {
        this.logger.error('WebSocket error:', error)
        reject(error)
      })

      // Connection timeout
      setTimeout(() => {
        if (!this.isConnected) {
          reject(new Error('WebSocket connection timeout'))
        }
      }, 10000)
    })
  }

  async send(message: Message): Promise<void> {
    if (!this.isConnected || !this.ws) {
      this.messageQueue.push(message)
      return
    }

    this.sendMessage(message)
  }

  private sendMessage(message: Message): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      this.messageQueue.push(message)
      return
    }

    try {
      const json = JSON.stringify(message)
      this.logger.debug('Sending message:', message)
      this.ws.send(json)
    } catch (error) {
      this.logger.error('Failed to send message:', error)
      throw error
    }
  }

  async receive(): Promise<Message | null> {
    // Return queued response if available
    if (this.responseQueue.length > 0) {
      return this.responseQueue.shift()!
    }

    // Wait for next message
    return new Promise((resolve) => {
      this.resolveQueue.push(resolve)

      // Timeout after 30 seconds
      setTimeout(() => {
        const index = this.resolveQueue.indexOf(resolve)
        if (index !== -1) {
          this.resolveQueue.splice(index, 1)
          resolve(null)
        }
      }, 30000)
    })
  }

  async close(): Promise<void> {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
    this.isConnected = false
  }
}

export class HttpBatchTransport implements Transport {
  private messageQueue: Message[] = []
  private logger: Logger

  constructor(
    private url: string,
    logger?: Logger
  ) {
    this.logger = logger || console
  }

  async send(message: Message): Promise<void> {
    this.messageQueue.push(message)
  }

  async receive(): Promise<Message | null> {
    // HTTP batch transport sends all messages at once and receives responses
    if (this.messageQueue.length === 0) {
      return null
    }

    try {
      const messages = [...this.messageQueue]
      this.messageQueue.length = 0

      this.logger.debug('Sending batch request:', messages)

      const response = await fetch(this.url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(messages),
      })

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const results = await response.json() as Message[]
      this.logger.debug('Received batch response:', results)

      // For simplicity, return the first result
      // In a real implementation, you'd need to handle multiple responses
      return results[0] || null
    } catch (error) {
      this.logger.error('HTTP batch request failed:', error)
      throw error
    }
  }

  async close(): Promise<void> {
    // Nothing to close for HTTP transport
  }
}