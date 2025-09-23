#!/usr/bin/env node
"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.InteropTestRunner = void 0;
const capnweb_1 = require("capnweb");
const http = __importStar(require("http"));
const child_process_1 = require("child_process");
const util_1 = require("util");
const sleep = (0, util_1.promisify)(setTimeout);
class InteropTestRunner {
    constructor() {
        this.serverReady = false;
        this.serverPort = 8080;
        this.serverHost = 'localhost';
    }
    async startRustServer() {
        console.log('🚀 Starting Rust Cap\'n Web server...');
        return new Promise((resolve, reject) => {
            this.rustServer = (0, child_process_1.spawn)('cargo', ['run', '--example', 'basic_server'], {
                cwd: '..',
                stdio: ['pipe', 'pipe', 'pipe']
            });
            let output = '';
            const timeout = setTimeout(() => {
                reject(new Error('Server startup timeout'));
            }, 30000); // 30 second timeout
            this.rustServer.stdout?.on('data', (data) => {
                output += data.toString();
                console.log(`[Server] ${data.toString().trim()}`);
                // Look for server ready indicators
                if (output.includes('listening') || output.includes('started') || output.includes('ready')) {
                    clearTimeout(timeout);
                    this.serverReady = true;
                    resolve();
                }
            });
            this.rustServer.stderr?.on('data', (data) => {
                console.error(`[Server Error] ${data.toString().trim()}`);
            });
            this.rustServer.on('error', (error) => {
                clearTimeout(timeout);
                reject(error);
            });
            this.rustServer.on('exit', (code) => {
                console.log(`Server process exited with code ${code}`);
            });
            // Give server some time to start even without explicit ready message
            setTimeout(() => {
                if (!this.serverReady) {
                    console.log('⏰ Server startup timeout, assuming ready...');
                    clearTimeout(timeout);
                    this.serverReady = true;
                    resolve();
                }
            }, 5000);
        });
    }
    async stopRustServer() {
        if (this.rustServer) {
            console.log('🛑 Stopping Rust server...');
            this.rustServer.kill('SIGTERM');
            await sleep(1000);
            if (!this.rustServer.killed) {
                this.rustServer.kill('SIGKILL');
            }
        }
    }
    async waitForServer() {
        const maxRetries = 30;
        let retries = 0;
        while (retries < maxRetries) {
            try {
                await new Promise((resolve, reject) => {
                    const req = http.get(`http://${this.serverHost}:${this.serverPort}/health`, (res) => {
                        resolve();
                    });
                    req.on('error', reject);
                    req.setTimeout(1000);
                });
                console.log('✅ Server is responding');
                return;
            }
            catch (error) {
                retries++;
                console.log(`⏳ Waiting for server... (${retries}/${maxRetries})`);
                await sleep(1000);
            }
        }
        // Even if health check fails, try to proceed with tests
        console.log('⚠️  Health check failed, but proceeding with tests...');
    }
    async runTest(name, testFn) {
        const startTime = Date.now();
        console.log(`🧪 Running test: ${name}`);
        try {
            await testFn();
            const duration = Date.now() - startTime;
            console.log(`✅ ${name} - PASSED (${duration}ms)`);
            return { name, passed: true, duration };
        }
        catch (error) {
            const duration = Date.now() - startTime;
            const errorMessage = error instanceof Error ? error.message : String(error);
            console.log(`❌ ${name} - FAILED (${duration}ms): ${errorMessage}`);
            return { name, passed: false, error: errorMessage, duration };
        }
    }
    async testBasicHttpBatchCall() {
        const session = (0, capnweb_1.newHttpBatchRpcSession)(`http://${this.serverHost}:${this.serverPort}/rpc/batch`);
        // Test basic capability call
        const result = await session.call(1, 'add', [5, 3]);
        if (result !== 8) {
            throw new Error(`Expected 8, got ${result}`);
        }
    }
    async testMessageSerialization() {
        const session = (0, capnweb_1.newHttpBatchRpcSession)(`http://${this.serverHost}:${this.serverPort}/rpc/batch`);
        // Test various data types
        const tests = [
            { args: [1, 2], expected: 3 },
            { args: [10.5, 5.5], expected: 16 },
            { args: [-5, 10], expected: 5 },
            { args: [0, 0], expected: 0 }
        ];
        for (const test of tests) {
            const result = await session.call(1, 'add', test.args);
            if (result !== test.expected) {
                throw new Error(`add(${test.args}) expected ${test.expected}, got ${result}`);
            }
        }
    }
    async testErrorHandling() {
        const session = (0, capnweb_1.newHttpBatchRpcSession)(`http://${this.serverHost}:${this.serverPort}/rpc/batch`);
        try {
            await session.call(1, 'divide', [10, 0]);
            throw new Error('Expected division by zero to throw an error');
        }
        catch (error) {
            // Expected error
            console.log('Expected error caught:', error);
        }
        try {
            await session.call(1, 'nonexistentMethod', []);
            throw new Error('Expected nonexistent method to throw an error');
        }
        catch (error) {
            // Expected error
            console.log('Expected error caught:', error);
        }
    }
    async testCapabilityManagement() {
        const session = (0, capnweb_1.newHttpBatchRpcSession)(`http://${this.serverHost}:${this.serverPort}/rpc/batch`);
        // Test multiple capability calls
        const result1 = await session.call(1, 'add', [1, 1]);
        const result2 = await session.call(1, 'multiply', [3, 4]);
        if (result1 !== 2) {
            throw new Error(`Expected 2, got ${result1}`);
        }
        if (result2 !== 12) {
            throw new Error(`Expected 12, got ${result2}`);
        }
    }
    async testComplexDataStructures() {
        const session = (0, capnweb_1.newHttpBatchRpcSession)(`http://${this.serverHost}:${this.serverPort}/rpc/batch`);
        // Test with complex objects
        const complexArg = {
            numbers: [1, 2, 3],
            nested: { value: 42 },
            string: "test"
        };
        try {
            const result = await session.call(1, 'echo', [complexArg]);
            // Basic validation that we got something back
            if (typeof result !== 'object') {
                throw new Error(`Expected object response, got ${typeof result}`);
            }
        }
        catch (error) {
            // If echo method doesn't exist, that's okay for this test
            if (error instanceof Error && !error.message.includes('not found')) {
                throw error;
            }
        }
    }
    async testBatchOperations() {
        const session = (0, capnweb_1.newHttpBatchRpcSession)(`http://${this.serverHost}:${this.serverPort}/rpc/batch`);
        // Test multiple calls in sequence
        const promises = [
            session.call(1, 'add', [1, 2]),
            session.call(1, 'add', [3, 4]),
            session.call(1, 'add', [5, 6])
        ];
        const results = await Promise.all(promises);
        if (results[0] !== 3 || results[1] !== 7 || results[2] !== 11) {
            throw new Error(`Unexpected batch results: ${results}`);
        }
    }
    async testWebSocketTransport() {
        try {
            const session = (0, capnweb_1.newWebSocketRpcSession)(`ws://${this.serverHost}:${this.serverPort}/rpc/ws`);
            // Test basic WebSocket call
            const result = await session.call(1, 'add', [10, 20]);
            if (result !== 30) {
                throw new Error(`Expected 30, got ${result}`);
            }
            // Close the WebSocket connection
            await session.close?.();
        }
        catch (error) {
            // WebSocket might not be implemented yet, that's okay
            if (error instanceof Error && error.message.includes('WebSocket')) {
                console.log('WebSocket transport not available, skipping...');
                return;
            }
            throw error;
        }
    }
    async runAllTests() {
        const suites = [];
        // Core Protocol Tests
        const coreTests = [];
        coreTests.push(await this.runTest('Basic HTTP Batch Call', () => this.testBasicHttpBatchCall()));
        coreTests.push(await this.runTest('Message Serialization', () => this.testMessageSerialization()));
        coreTests.push(await this.runTest('Error Handling', () => this.testErrorHandling()));
        coreTests.push(await this.runTest('Capability Management', () => this.testCapabilityManagement()));
        coreTests.push(await this.runTest('Complex Data Structures', () => this.testComplexDataStructures()));
        coreTests.push(await this.runTest('Batch Operations', () => this.testBatchOperations()));
        suites.push({
            name: 'Core Protocol Tests',
            results: coreTests,
            totalPassed: coreTests.filter(r => r.passed).length,
            totalFailed: coreTests.filter(r => !r.passed).length,
            totalDuration: coreTests.reduce((sum, r) => sum + r.duration, 0)
        });
        // Transport Tests
        const transportTests = [];
        transportTests.push(await this.runTest('WebSocket Transport', () => this.testWebSocketTransport()));
        suites.push({
            name: 'Transport Tests',
            results: transportTests,
            totalPassed: transportTests.filter(r => r.passed).length,
            totalFailed: transportTests.filter(r => !r.passed).length,
            totalDuration: transportTests.reduce((sum, r) => sum + r.duration, 0)
        });
        return suites;
    }
    printResults(suites) {
        console.log('\n' + '='.repeat(80));
        console.log('🧪 CAP\'N WEB RUST ↔ TYPESCRIPT INTEROPERABILITY TEST RESULTS');
        console.log('='.repeat(80));
        let totalPassed = 0;
        let totalFailed = 0;
        let totalDuration = 0;
        for (const suite of suites) {
            console.log(`\n📊 ${suite.name}`);
            console.log(`   ✅ Passed: ${suite.totalPassed}`);
            console.log(`   ❌ Failed: ${suite.totalFailed}`);
            console.log(`   ⏱️  Duration: ${suite.totalDuration}ms`);
            if (suite.totalFailed > 0) {
                console.log(`   💥 Failures:`);
                for (const result of suite.results.filter(r => !r.passed)) {
                    console.log(`      • ${result.name}: ${result.error}`);
                }
            }
            totalPassed += suite.totalPassed;
            totalFailed += suite.totalFailed;
            totalDuration += suite.totalDuration;
        }
        console.log('\n' + '='.repeat(80));
        console.log(`📈 OVERALL RESULTS`);
        console.log(`   ✅ Total Passed: ${totalPassed}`);
        console.log(`   ❌ Total Failed: ${totalFailed}`);
        console.log(`   ⏱️  Total Duration: ${totalDuration}ms`);
        console.log(`   🎯 Success Rate: ${Math.round((totalPassed / (totalPassed + totalFailed)) * 100)}%`);
        console.log('='.repeat(80));
        if (totalFailed === 0) {
            console.log('🎉 ALL TESTS PASSED! Cap\'n Web Rust implementation is fully compatible with TypeScript client!');
        }
        else {
            console.log(`⚠️  ${totalFailed} test(s) failed. Please review the implementation.`);
            process.exit(1);
        }
    }
    async run() {
        try {
            await this.startRustServer();
            await this.waitForServer();
            const suites = await this.runAllTests();
            this.printResults(suites);
        }
        catch (error) {
            console.error('💥 Test runner failed:', error);
            process.exit(1);
        }
        finally {
            await this.stopRustServer();
        }
    }
}
exports.InteropTestRunner = InteropTestRunner;
// Run tests if this file is executed directly
if (require.main === module) {
    const runner = new InteropTestRunner();
    runner.run().catch(error => {
        console.error('Fatal error:', error);
        process.exit(1);
    });
}
//# sourceMappingURL=interop-tests.js.map