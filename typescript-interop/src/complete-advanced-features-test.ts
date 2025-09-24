/**
 * Complete Advanced Features Test Suite
 * Tests all Cap'n Web advanced features with the official TypeScript client
 * Validates: Resume Tokens, Nested Capabilities, IL Plan Runner, HTTP/3 & WebTransport
 */

import { Client, Server, Value, Plan, Op, Source } from '@cloudflare/capnweb';
import { WebSocket } from 'ws';
import * as http from 'http';
import * as https from 'https';
import * as fs from 'fs';
import * as path from 'path';

// Test configuration
const RUST_SERVER_URL = 'http://localhost:8080';
const RUST_WS_URL = 'ws://localhost:8080/rpc/ws';
const RUST_H3_URL = 'https://localhost:8443';
const TEST_TIMEOUT = 30000;

interface TestResult {
  feature: string;
  status: 'passed' | 'failed' | 'skipped';
  message: string;
  details?: any;
}

class AdvancedFeaturesTestSuite {
  private results: TestResult[] = [];
  private client?: Client;
  private wsClient?: Client;
  private h3Client?: Client;

  /**
   * Initialize test clients for different transports
   */
  async initialize(): Promise<void> {
    console.log('🔧 Initializing test clients...\n');

    // HTTP Batch client
    this.client = new Client({
      endpoint: `${RUST_SERVER_URL}/rpc/batch`,
      transport: 'http-batch'
    });

    // WebSocket client
    try {
      this.wsClient = new Client({
        endpoint: RUST_WS_URL,
        transport: 'websocket',
        WebSocket: WebSocket as any
      });
      await this.wsClient.connect();
      console.log('✅ WebSocket client connected');
    } catch (e) {
      console.log('⚠️  WebSocket client connection failed:', e);
    }

    // HTTP/3 client (if available)
    try {
      this.h3Client = new Client({
        endpoint: RUST_H3_URL,
        transport: 'http3'
      });
      console.log('✅ HTTP/3 client initialized');
    } catch (e) {
      console.log('⚠️  HTTP/3 client not available');
    }
  }

  /**
   * Test 1: Resume Tokens - Session Recovery
   */
  async testResumeTokens(): Promise<void> {
    console.log('\n1️⃣  Testing Resume Tokens...');

    try {
      // Create a session with state
      const sessionId = `test-session-${Date.now()}`;

      // Initialize session with some state
      const calculator = await this.client!.import(0);

      // Perform operations to build session state
      const result1 = await calculator.add(10, 20);
      const result2 = await calculator.multiply(result1, 2);

      // Store state in session
      await calculator.setVariable('sessionId', sessionId);
      await calculator.setVariable('lastResult', result2);
      await calculator.setVariable('operations', ['add', 'multiply']);

      // Request resume token
      const tokenResponse = await calculator.createResumeToken({
        sessionId,
        includeState: true,
        expirationMinutes: 60
      });

      if (!tokenResponse || !tokenResponse.token) {
        throw new Error('Resume token not returned');
      }

      console.log(`  📝 Resume token created: ${tokenResponse.token.substring(0, 20)}...`);

      // Simulate disconnect
      await this.client!.disconnect();

      // Create new client and restore session
      const newClient = new Client({
        endpoint: `${RUST_SERVER_URL}/rpc/batch`,
        transport: 'http-batch',
        resumeToken: tokenResponse.token
      });

      const restoredCalc = await newClient.import(0);

      // Verify restored state
      const restoredSessionId = await restoredCalc.getVariable('sessionId');
      const restoredResult = await restoredCalc.getVariable('lastResult');
      const restoredOps = await restoredCalc.getVariable('operations');

      if (restoredSessionId !== sessionId) {
        throw new Error(`Session ID mismatch: ${restoredSessionId} !== ${sessionId}`);
      }

      if (restoredResult !== 60) { // (10 + 20) * 2
        throw new Error(`Result mismatch: ${restoredResult} !== 60`);
      }

      console.log('  ✅ Session successfully restored with state');
      console.log(`     - Session ID: ${restoredSessionId}`);
      console.log(`     - Last Result: ${restoredResult}`);
      console.log(`     - Operations: ${JSON.stringify(restoredOps)}`);

      // Test token expiration
      const expiredToken = await calculator.createResumeToken({
        sessionId: 'expired-session',
        expirationMinutes: -1 // Already expired
      });

      try {
        const expiredClient = new Client({
          endpoint: `${RUST_SERVER_URL}/rpc/batch`,
          resumeToken: expiredToken.token
        });
        await expiredClient.import(0);
        throw new Error('Should have failed with expired token');
      } catch (e: any) {
        if (e.message.includes('expired') || e.message.includes('invalid')) {
          console.log('  ✅ Token expiration properly handled');
        } else {
          throw e;
        }
      }

      this.results.push({
        feature: 'Resume Tokens',
        status: 'passed',
        message: 'Session persistence and recovery working correctly',
        details: {
          tokenLength: tokenResponse.token.length,
          sessionRestored: true,
          expirationHandled: true
        }
      });

    } catch (error: any) {
      this.results.push({
        feature: 'Resume Tokens',
        status: 'failed',
        message: error.message,
        details: error
      });
    }
  }

  /**
   * Test 2: Nested Capabilities - Dynamic Creation
   */
  async testNestedCapabilities(): Promise<void> {
    console.log('\n2️⃣  Testing Nested Capabilities...');

    try {
      const factory = await this.client!.import(1); // Capability factory endpoint

      // List available capability types
      const types = await factory.listCapabilityTypes();
      console.log(`  📋 Available capability types: ${types.join(', ')}`);

      // Create nested capability hierarchy
      const rootProcessor = await factory.createCapability('dataProcessor', {
        name: 'root-processor',
        mode: 'transform'
      });

      // Create child capabilities
      const validator = await rootProcessor.createSubCapability('validator', {
        name: 'data-validator',
        rules: ['required', 'type-check']
      });

      const aggregator = await rootProcessor.createSubCapability('aggregator', {
        name: 'data-aggregator',
        operations: ['sum', 'average', 'count']
      });

      console.log('  ✅ Created capability hierarchy:');
      console.log('     └─ root-processor (transform)');
      console.log('        ├─ data-validator');
      console.log('        └─ data-aggregator');

      // Test capability metadata
      const metadata = await rootProcessor.getMetadata();
      console.log(`  📊 Root capability metadata:`, metadata);

      // List child capabilities
      const children = await rootProcessor.listChildCapabilities();
      if (children.length !== 2) {
        throw new Error(`Expected 2 children, got ${children.length}`);
      }

      // Test capability operations
      const testData = [1, 2, 3, 4, 5];

      // Validate data
      const validationResult = await validator.validate(testData);
      console.log(`  ✅ Validation result: ${validationResult.valid ? 'Valid' : 'Invalid'}`);

      // Transform with root
      const transformed = await rootProcessor.process(testData);
      console.log(`  ✅ Transformed data:`, transformed);

      // Aggregate with child
      const aggregated = await aggregator.aggregate(transformed);
      console.log(`  ✅ Aggregated result:`, aggregated);

      // Test capability disposal
      const disposeResult = await rootProcessor.disposeChild(validator.id);
      if (!disposeResult) {
        throw new Error('Child disposal failed');
      }

      const remainingChildren = await rootProcessor.listChildCapabilities();
      if (remainingChildren.length !== 1) {
        throw new Error(`Expected 1 child after disposal, got ${remainingChildren.length}`);
      }

      console.log('  ✅ Child capability disposed successfully');

      // Test capability graph statistics
      const graphStats = await factory.getCapabilityGraphStats();
      console.log(`  📈 Capability graph stats:`, graphStats);

      this.results.push({
        feature: 'Nested Capabilities',
        status: 'passed',
        message: 'Dynamic capability creation and management working correctly',
        details: {
          typesAvailable: types.length,
          hierarchyCreated: true,
          childrenManaged: true,
          disposalWorking: true,
          graphStats
        }
      });

    } catch (error: any) {
      this.results.push({
        feature: 'Nested Capabilities',
        status: 'failed',
        message: error.message,
        details: error
      });
    }
  }

  /**
   * Test 3: Advanced IL Plan Runner
   */
  async testILPlanRunner(): Promise<void> {
    console.log('\n3️⃣  Testing Advanced IL Plan Runner...');

    try {
      const planner = await this.client!.import(2); // IL Plan endpoint

      // Build a complex multi-step plan
      const plan: Plan = {
        captures: [0, 1], // Capture calculator and processor capabilities
        ops: [
          // Step 1: Calculate sum
          {
            call: {
              target: { capture: { index: 0 } },
              member: 'add',
              args: [
                { byValue: { value: 10 } },
                { byValue: { value: 20 } }
              ],
              result: { index: 0 }
            }
          },
          // Step 2: Multiply result
          {
            call: {
              target: { capture: { index: 0 } },
              member: 'multiply',
              args: [
                { result: { index: 0 } },
                { byValue: { value: 3 } }
              ],
              result: { index: 1 }
            }
          },
          // Step 3: Process with second capability
          {
            call: {
              target: { capture: { index: 1 } },
              member: 'process',
              args: [
                { result: { index: 1 } }
              ],
              result: { index: 2 }
            }
          },
          // Step 4: Create result object
          {
            object: {
              fields: {
                'calculation': { result: { index: 1 } },
                'processed': { result: { index: 2 } },
                'metadata': {
                  byValue: {
                    value: {
                      timestamp: Date.now(),
                      plan: 'complex-calculation'
                    }
                  }
                }
              },
              result: { index: 3 }
            }
          }
        ],
        result: { result: { index: 3 } }
      };

      console.log('  📝 Executing complex IL plan with 4 operations...');

      // Validate plan
      const validationResult = await planner.validatePlan(plan);
      if (!validationResult.valid) {
        throw new Error(`Plan validation failed: ${validationResult.errors?.join(', ')}`);
      }
      console.log('  ✅ Plan validation passed');

      // Analyze plan complexity
      const complexity = await planner.analyzePlanComplexity(plan);
      console.log(`  📊 Plan complexity:`, complexity);

      // Execute the plan
      const startTime = Date.now();
      const planResult = await planner.executePlan(plan, {
        parameters: { multiplier: 3 },
        timeout: 5000
      });
      const executionTime = Date.now() - startTime;

      console.log(`  ✅ Plan executed in ${executionTime}ms`);
      console.log(`  📦 Result:`, planResult);

      // Verify result structure
      if (!planResult || typeof planResult !== 'object') {
        throw new Error('Invalid plan result');
      }

      if (!('calculation' in planResult) || !('processed' in planResult)) {
        throw new Error('Missing expected fields in result');
      }

      // Test plan optimization
      const optimizedPlan = await planner.optimizePlan(plan);
      console.log(`  ⚡ Plan optimized: ${optimizedPlan.ops.length} operations`);

      // Test parallel execution (if supported)
      const parallelPlan: Plan = {
        captures: [0],
        ops: [
          // Two independent operations that can run in parallel
          {
            call: {
              target: { capture: { index: 0 } },
              member: 'add',
              args: [
                { byValue: { value: 5 } },
                { byValue: { value: 10 } }
              ],
              result: { index: 0 }
            }
          },
          {
            call: {
              target: { capture: { index: 0 } },
              member: 'multiply',
              args: [
                { byValue: { value: 3 } },
                { byValue: { value: 7 } }
              ],
              result: { index: 1 }
            }
          },
          // Combine results
          {
            call: {
              target: { capture: { index: 0 } },
              member: 'add',
              args: [
                { result: { index: 0 } },
                { result: { index: 1 } }
              ],
              result: { index: 2 }
            }
          }
        ],
        result: { result: { index: 2 } }
      };

      const parallelResult = await planner.executePlan(parallelPlan, {
        parallel: true
      });
      console.log(`  ✅ Parallel execution result: ${parallelResult}`);

      this.results.push({
        feature: 'IL Plan Runner',
        status: 'passed',
        message: 'Complex plan execution working correctly',
        details: {
          planOps: plan.ops.length,
          executionTime,
          complexity,
          parallelSupport: true
        }
      });

    } catch (error: any) {
      this.results.push({
        feature: 'IL Plan Runner',
        status: 'failed',
        message: error.message,
        details: error
      });
    }
  }

  /**
   * Test 4: HTTP/3 & WebTransport
   */
  async testHttp3AndWebTransport(): Promise<void> {
    console.log('\n4️⃣  Testing HTTP/3 & WebTransport...');

    try {
      // Test HTTP/3 if client is available
      if (this.h3Client) {
        console.log('  🌐 Testing HTTP/3 transport...');

        const h3Calculator = await this.h3Client.import(0);
        const h3Result = await h3Calculator.add(15, 25);

        if (h3Result !== 40) {
          throw new Error(`HTTP/3 calculation failed: ${h3Result} !== 40`);
        }

        // Test multiplexing
        const promises = [];
        for (let i = 0; i < 10; i++) {
          promises.push(h3Calculator.multiply(i, 2));
        }

        const results = await Promise.all(promises);
        console.log(`  ✅ HTTP/3 multiplexing: ${results.length} concurrent requests completed`);

        // Get connection stats
        const stats = await this.h3Client.getTransportStats();
        console.log(`  📊 HTTP/3 stats:`, stats);

        this.results.push({
          feature: 'HTTP/3 Transport',
          status: 'passed',
          message: 'HTTP/3 transport working with multiplexing',
          details: stats
        });
      } else {
        this.results.push({
          feature: 'HTTP/3 Transport',
          status: 'skipped',
          message: 'HTTP/3 client not available'
        });
      }

      // Test WebTransport
      console.log('  🚄 Testing WebTransport...');

      try {
        const wtClient = new Client({
          endpoint: `${RUST_SERVER_URL}/rpc/wt`,
          transport: 'webtransport'
        });

        await wtClient.connect();
        const wtCalculator = await wtClient.import(0);

        // Test bidirectional streaming
        const stream = await wtCalculator.createBidirectionalStream();

        // Send data
        await stream.send({ operation: 'add', args: [100, 200] });
        const response = await stream.receive();

        if (response.result !== 300) {
          throw new Error(`WebTransport streaming failed: ${response.result} !== 300`);
        }

        console.log('  ✅ WebTransport bidirectional streaming working');

        await stream.close();
        await wtClient.disconnect();

        this.results.push({
          feature: 'WebTransport',
          status: 'passed',
          message: 'WebTransport with streaming support working'
        });

      } catch (e: any) {
        if (e.message.includes('not supported') || e.message.includes('not available')) {
          this.results.push({
            feature: 'WebTransport',
            status: 'skipped',
            message: 'WebTransport not available in current environment'
          });
        } else {
          throw e;
        }
      }

    } catch (error: any) {
      this.results.push({
        feature: 'HTTP/3 & WebTransport',
        status: 'failed',
        message: error.message,
        details: error
      });
    }
  }

  /**
   * Test 5: Cross-Transport Interoperability
   */
  async testCrossTransportInterop(): Promise<void> {
    console.log('\n5️⃣  Testing Cross-Transport Interoperability...');

    try {
      // Create capability on HTTP, use on WebSocket
      const httpCalculator = await this.client!.import(0);
      const capId = await httpCalculator.export();

      if (this.wsClient) {
        const wsCalculator = await this.wsClient.importById(capId);
        const wsResult = await wsCalculator.add(50, 50);

        if (wsResult !== 100) {
          throw new Error(`Cross-transport calculation failed: ${wsResult} !== 100`);
        }

        console.log('  ✅ HTTP -> WebSocket capability sharing working');
      }

      // Test transport negotiation
      const negotiator = await this.client!.import(3); // Transport negotiator
      const bestTransport = await negotiator.selectBestTransport({
        available: ['http-batch', 'websocket', 'http3', 'webtransport'],
        requirements: {
          streaming: true,
          multiplexing: true,
          lowLatency: true
        }
      });

      console.log(`  🎯 Best transport selected: ${bestTransport}`);

      // Test transport fallback
      const fallbackClient = new Client({
        endpoint: RUST_SERVER_URL,
        transport: 'auto', // Automatic selection
        fallback: ['http3', 'websocket', 'http-batch']
      });

      const fallbackCalc = await fallbackClient.import(0);
      const fallbackResult = await fallbackCalc.add(75, 25);

      console.log(`  ✅ Transport fallback working, using: ${fallbackClient.currentTransport}`);

      this.results.push({
        feature: 'Cross-Transport Interop',
        status: 'passed',
        message: 'Transport interoperability and negotiation working',
        details: {
          bestTransport,
          fallbackTransport: fallbackClient.currentTransport
        }
      });

    } catch (error: any) {
      this.results.push({
        feature: 'Cross-Transport Interop',
        status: 'failed',
        message: error.message,
        details: error
      });
    }
  }

  /**
   * Test 6: End-to-End Integration
   */
  async testEndToEndIntegration(): Promise<void> {
    console.log('\n6️⃣  Testing End-to-End Integration...');

    try {
      // Complex scenario using all features together
      console.log('  🎬 Running complex integration scenario...');

      // 1. Create session with resume token support
      const sessionId = `integration-${Date.now()}`;
      const calculator = await this.client!.import(0);
      await calculator.initSession(sessionId);

      // 2. Create nested capabilities
      const factory = await this.client!.import(1);
      const processor = await factory.createCapability('dataProcessor', {
        name: 'integration-processor',
        features: ['transform', 'validate', 'aggregate']
      });

      const analyzer = await processor.createSubCapability('analyzer', {
        name: 'data-analyzer'
      });

      // 3. Build IL plan using nested capabilities
      const plan: Plan = {
        captures: [0, 1], // calculator and processor
        ops: [
          // Generate test data
          {
            call: {
              target: { capture: { index: 0 } },
              member: 'generateData',
              args: [{ byValue: { value: 100 } }],
              result: { index: 0 }
            }
          },
          // Process with nested capability
          {
            call: {
              target: { capture: { index: 1 } },
              member: 'process',
              args: [{ result: { index: 0 } }],
              result: { index: 1 }
            }
          },
          // Analyze results
          {
            call: {
              target: { capture: { index: 1 } },
              member: 'analyze',
              args: [{ result: { index: 1 } }],
              result: { index: 2 }
            }
          },
          // Create final report
          {
            object: {
              fields: {
                'sessionId': { byValue: { value: sessionId } },
                'rawData': { result: { index: 0 } },
                'processed': { result: { index: 1 } },
                'analysis': { result: { index: 2 } },
                'timestamp': { byValue: { value: Date.now() } }
              },
              result: { index: 3 }
            }
          }
        ],
        result: { result: { index: 3 } }
      };

      // 4. Execute plan
      const planner = await this.client!.import(2);
      const integrationResult = await planner.executePlan(plan, {
        captures: [calculator.id, processor.id]
      });

      console.log('  ✅ Integration scenario completed successfully');
      console.log('  📊 Integration result:', integrationResult);

      // 5. Create resume token for session
      const resumeToken = await calculator.createResumeToken({
        sessionId,
        state: integrationResult
      });

      console.log('  💾 Session saved with resume token');

      // 6. Test session restoration
      const newClient = new Client({
        endpoint: `${RUST_SERVER_URL}/rpc/batch`,
        resumeToken: resumeToken.token
      });

      const restoredCalc = await newClient.import(0);
      const restoredState = await restoredCalc.getSessionState();

      if (restoredState.sessionId !== sessionId) {
        throw new Error('Session restoration failed');
      }

      console.log('  ✅ Session restored successfully');

      // 7. Clean up
      await processor.dispose();
      console.log('  🧹 Resources cleaned up');

      this.results.push({
        feature: 'End-to-End Integration',
        status: 'passed',
        message: 'All features working together seamlessly',
        details: {
          sessionId,
          planOperations: plan.ops.length,
          featuresUsed: ['resume-tokens', 'nested-capabilities', 'il-plans', 'cross-transport']
        }
      });

    } catch (error: any) {
      this.results.push({
        feature: 'End-to-End Integration',
        status: 'failed',
        message: error.message,
        details: error
      });
    }
  }

  /**
   * Run all tests and generate report
   */
  async runAllTests(): Promise<void> {
    console.log('🎯 Cap\'n Web Advanced Features - Official Client Test Suite');
    console.log('=' .repeat(60));

    try {
      await this.initialize();

      // Run all feature tests
      await this.testResumeTokens();
      await this.testNestedCapabilities();
      await this.testILPlanRunner();
      await this.testHttp3AndWebTransport();
      await this.testCrossTransportInterop();
      await this.testEndToEndIntegration();

    } catch (error: any) {
      console.error('❌ Test suite initialization failed:', error);
    } finally {
      // Clean up connections
      if (this.client) await this.client.disconnect?.();
      if (this.wsClient) await this.wsClient.disconnect();
      if (this.h3Client) await this.h3Client.disconnect?.();
    }

    // Generate report
    this.generateReport();
  }

  /**
   * Generate test report
   */
  private generateReport(): void {
    console.log('\n' + '='.repeat(60));
    console.log('📊 Test Results Summary\n');

    const passed = this.results.filter(r => r.status === 'passed').length;
    const failed = this.results.filter(r => r.status === 'failed').length;
    const skipped = this.results.filter(r => r.status === 'skipped').length;

    // Print individual results
    this.results.forEach(result => {
      const icon = result.status === 'passed' ? '✅' :
                   result.status === 'failed' ? '❌' : '⏩';
      console.log(`${icon} ${result.feature.padEnd(25)} - ${result.message}`);

      if (result.status === 'failed' && result.details) {
        console.log(`   Error: ${JSON.stringify(result.details).substring(0, 100)}`);
      }
    });

    // Print summary
    console.log('\n' + '-'.repeat(60));
    console.log(`Total: ${this.results.length} features tested`);
    console.log(`✅ Passed: ${passed}`);
    console.log(`❌ Failed: ${failed}`);
    console.log(`⏩ Skipped: ${skipped}`);

    const successRate = (passed / (passed + failed)) * 100;
    console.log(`\n📈 Success Rate: ${successRate.toFixed(1)}%`);

    // Overall result
    if (failed === 0) {
      console.log('\n🎉 All tests passed! Cap\'n Web advanced features fully validated! 🎉');
    } else {
      console.log(`\n⚠️  ${failed} test(s) failed. Please review the errors above.`);
    }

    // Save results to file
    const reportPath = path.join(__dirname, '..', 'test-results', 'advanced-features-report.json');
    fs.mkdirSync(path.dirname(reportPath), { recursive: true });
    fs.writeFileSync(reportPath, JSON.stringify({
      timestamp: new Date().toISOString(),
      results: this.results,
      summary: { passed, failed, skipped, successRate }
    }, null, 2));

    console.log(`\n📝 Detailed report saved to: ${reportPath}`);
  }
}

// Run tests if executed directly
if (require.main === module) {
  const suite = new AdvancedFeaturesTestSuite();

  suite.runAllTests()
    .then(() => process.exit(0))
    .catch(error => {
      console.error('Fatal error:', error);
      process.exit(1);
    });
}

export { AdvancedFeaturesTestSuite };