/**
 * Standalone TypeScript Server for Rust Client Testing
 *
 * A standalone server that can be run to test Rust client interoperability.
 */

import { CapnWebServer, MockCalculator, MockUserManager } from './capnweb/server.js'

async function main(): Promise<void> {
  console.log('🔥 Starting TypeScript Cap\'n Web Server for Interop Testing')
  console.log('=' .repeat(60))

  const server = new CapnWebServer({
    port: 8081,
    host: 'localhost',
    path: '/ws'
  })

  // Register test capabilities that match the Rust server
  server.registerCapability(1, new MockCalculator())
  server.registerCapability(2, new MockCalculator()) // Scientific calculator
  server.registerCapability(100, new MockUserManager())

  console.log('📋 Registered capabilities:')
  console.log('   • Calculator (ID: 1) - Basic arithmetic operations')
  console.log('   • Scientific Calculator (ID: 2) - Advanced math functions')
  console.log('   • User Manager (ID: 100) - User management operations')
  console.log()

  try {
    await server.start()
    console.log('✅ TypeScript server started successfully!')
    console.log('🌐 Server Details:')
    console.log('   • Host: localhost')
    console.log('   • Port: 8081')
    console.log('   • WebSocket Path: /ws')
    console.log('   • Full URL: ws://localhost:8081/ws')
    console.log()

    console.log('🧪 Available Test Capabilities:')
    console.log()
    console.log('Calculator (ID: 1, 2):')
    console.log('   • add(a, b) → a + b')
    console.log('   • subtract(a, b) → a - b')
    console.log('   • multiply(a, b) → a * b')
    console.log('   • divide(a, b) → a / b (throws on division by zero)')
    console.log('   • power(base, exp) → base^exp')
    console.log('   • sqrt(n) → √n (throws on negative numbers)')
    console.log('   • factorial(n) → n! (throws on negative, max 20)')
    console.log()
    console.log('User Manager (ID: 100):')
    console.log('   • getUser(id) → User object')
    console.log('   • createUser(userData) → Created user object')
    console.log()

    console.log('🔌 Ready for Rust client connections!')
    console.log('💡 Test with Rust client:')
    console.log('   cd .. && cargo run --example calculator_client --features typescript-server')
    console.log()
    console.log('Press Ctrl+C to stop the server...')

    // Keep the server running
    await new Promise((resolve) => {
      process.on('SIGINT', () => {
        console.log('\n🛑 Received SIGINT, shutting down server...')
        resolve(undefined)
      })

      process.on('SIGTERM', () => {
        console.log('\n🛑 Received SIGTERM, shutting down server...')
        resolve(undefined)
      })
    })

  } catch (error) {
    console.error('💥 Failed to start server:', error)
    process.exit(1)
  } finally {
    console.log('🔄 Stopping TypeScript server...')
    await server.stop()
    console.log('✅ Server stopped successfully')
    process.exit(0)
  }
}

main().catch((error) => {
  console.error('💥 Unhandled error:', error)
  process.exit(1)
})