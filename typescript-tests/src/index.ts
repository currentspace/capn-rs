/**
 * Cap'n Web TypeScript Interoperability Test Suite
 *
 * Main entry point for running comprehensive interoperability tests
 * between TypeScript and Rust Cap'n Web implementations.
 */

import { runClientTests } from './tests/client-tests.js'
import { runServerTests } from './tests/server-tests.js'
import { wait } from './tests/test-framework.js'

interface TestConfiguration {
  runClientTests: boolean
  runServerTests: boolean
  waitForRustServer: number
  verbose: boolean
}

async function parseArgs(): Promise<TestConfiguration> {
  const args = process.argv.slice(2)

  const config: TestConfiguration = {
    runClientTests: true,
    runServerTests: true,
    waitForRustServer: 3000,
    verbose: false
  }

  for (let i = 0; i < args.length; i++) {
    const arg = args[i]

    switch (arg) {
      case '--client-only':
        config.runClientTests = true
        config.runServerTests = false
        break
      case '--server-only':
        config.runClientTests = false
        config.runServerTests = true
        break
      case '--wait':
        config.waitForRustServer = parseInt(args[++i] || '3000', 10)
        break
      case '--verbose':
        config.verbose = true
        break
      case '--help':
        console.log(`
Cap'n Web TypeScript Interoperability Test Suite

Usage: node dist/index.js [options]

Options:
  --client-only     Run only TypeScript client → Rust server tests
  --server-only     Run only TypeScript server ← Rust client tests
  --wait <ms>       Wait time for Rust server startup (default: 3000ms)
  --verbose         Enable verbose logging
  --help           Show this help message

Examples:
  node dist/index.js                    # Run all tests
  node dist/index.js --client-only      # Test TS client → Rust server
  node dist/index.js --server-only      # Test TS server ← Rust client
  node dist/index.js --wait 5000        # Wait 5 seconds for Rust server
`)
        process.exit(0)
        break
      default:
        console.warn(`Unknown argument: ${arg}`)
        break
    }
  }

  return config
}

async function checkRustServerAvailability(): Promise<boolean> {
  try {
    const response = await fetch('http://localhost:8080/health')
    return response.ok
  } catch {
    // Try WebSocket connection
    try {
      const ws = new (await import('ws')).default('ws://localhost:8080/ws')
      return new Promise((resolve) => {
        const timeout = setTimeout(() => {
          ws.close()
          resolve(false)
        }, 2000)

        ws.on('open', () => {
          clearTimeout(timeout)
          ws.close()
          resolve(true)
        })

        ws.on('error', () => {
          clearTimeout(timeout)
          resolve(false)
        })
      })
    } catch {
      return false
    }
  }
}

async function main(): Promise<void> {
  console.log('🌟 Cap\'n Web TypeScript ↔ Rust Interoperability Test Suite')
  console.log('=' .repeat(70))
  console.log()

  const config = await parseArgs()

  if (config.verbose) {
    console.log('📋 Test Configuration:')
    console.log(`   Client Tests: ${config.runClientTests ? '✅' : '❌'}`)
    console.log(`   Server Tests: ${config.runServerTests ? '✅' : '❌'}`)
    console.log(`   Rust Server Wait: ${config.waitForRustServer}ms`)
    console.log()
  }

  let totalTests = 0
  let totalPassed = 0
  let totalFailed = 0
  let totalDuration = 0

  const overallStart = Date.now()

  try {
    // Run TypeScript Client → Rust Server tests
    if (config.runClientTests) {
      console.log('🚀 PHASE 1: TypeScript Client → Rust Server Tests')
      console.log('-'.repeat(50))

      // Check if Rust server is running
      console.log('🔍 Checking for Rust server availability...')
      let serverAvailable = await checkRustServerAvailability()

      if (!serverAvailable) {
        console.log(`⏳ Rust server not ready, waiting ${config.waitForRustServer}ms...`)
        console.log('💡 Make sure to start the Rust server first:')
        console.log('   cd .. && cargo run --example calculator_server')
        console.log()

        await wait(config.waitForRustServer)
        serverAvailable = await checkRustServerAvailability()
      }

      if (serverAvailable) {
        console.log('✅ Rust server is available, proceeding with client tests...')
        const clientStart = Date.now()
        await runClientTests()
        const clientDuration = Date.now() - clientStart

        console.log(`⏱️  Client tests completed in ${clientDuration}ms`)
        totalDuration += clientDuration
      } else {
        console.error('❌ Rust server is not available. Skipping client tests.')
        console.error('   Start the Rust server with: cargo run --example calculator_server')
      }

      console.log()
    }

    // Run TypeScript Server ← Rust Client tests
    if (config.runServerTests) {
      console.log('🎯 PHASE 2: TypeScript Server ← Rust Client Tests')
      console.log('-'.repeat(50))

      const serverStart = Date.now()
      await runServerTests()
      const serverDuration = Date.now() - serverStart

      console.log(`⏱️  Server tests completed in ${serverDuration}ms`)
      totalDuration += serverDuration
      console.log()
    }

  } catch (error) {
    console.error('💥 Fatal error during test execution:')
    console.error(error)
    process.exit(1)
  }

  const overallDuration = Date.now() - overallStart

  // Final summary
  console.log('🏁 FINAL INTEROPERABILITY REPORT')
  console.log('='.repeat(70))
  console.log(`⏱️  Total Test Duration: ${overallDuration}ms`)

  if (totalTests > 0) {
    const successRate = (totalPassed / totalTests) * 100
    console.log(`📊 Overall Success Rate: ${successRate.toFixed(1)}%`)

    if (totalFailed === 0) {
      console.log()
      console.log('🎉 INTEROPERABILITY VERIFIED!')
      console.log('✅ TypeScript and Rust Cap\'n Web implementations are fully compatible!')
      console.log()
      console.log('🌟 Key achievements:')
      console.log('   • Protocol message format compatibility')
      console.log('   • Bidirectional communication verified')
      console.log('   • Error handling interoperability')
      console.log('   • Performance characteristics validated')
      console.log('   • Edge cases and robustness confirmed')
    } else {
      console.log()
      console.error('💥 INTEROPERABILITY ISSUES DETECTED!')
      console.error(`❌ ${totalFailed} tests failed`)
      console.error('🔧 Review the test results above for details on incompatibilities')
      process.exit(1)
    }
  } else {
    console.log('⚠️  No tests were executed')
    console.log('   Check server availability and configuration')
  }

  console.log()
  console.log('📚 For more information:')
  console.log('   • Cap\'n Web Specification: https://capnproto.org/capnweb')
  console.log('   • Rust Implementation: ../README.md')
  console.log('   • TypeScript Implementation: ./README.md')
  console.log()
}

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.log('\n🛑 Test suite interrupted by user')
  process.exit(130)
})

process.on('SIGTERM', () => {
  console.log('\n🛑 Test suite terminated')
  process.exit(143)
})

// Run the main function
main().catch((error) => {
  console.error('💥 Unhandled error in main:', error)
  process.exit(1)
})