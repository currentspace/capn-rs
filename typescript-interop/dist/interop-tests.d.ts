#!/usr/bin/env node
interface TestResult {
    name: string;
    passed: boolean;
    error?: string;
    duration: number;
}
interface TestSuite {
    name: string;
    results: TestResult[];
    totalPassed: number;
    totalFailed: number;
    totalDuration: number;
}
declare class InteropTestRunner {
    private rustServer?;
    private serverReady;
    private readonly serverPort;
    private readonly serverHost;
    startRustServer(): Promise<void>;
    stopRustServer(): Promise<void>;
    waitForServer(): Promise<void>;
    runTest(name: string, testFn: () => Promise<void>): Promise<TestResult>;
    testBasicHttpBatchCall(): Promise<void>;
    testMessageSerialization(): Promise<void>;
    testErrorHandling(): Promise<void>;
    testCapabilityManagement(): Promise<void>;
    testComplexDataStructures(): Promise<void>;
    testBatchOperations(): Promise<void>;
    testWebSocketTransport(): Promise<void>;
    runAllTests(): Promise<TestSuite[]>;
    printResults(suites: TestSuite[]): void;
    run(): Promise<void>;
}
export { InteropTestRunner, TestResult, TestSuite };
//# sourceMappingURL=interop-tests.d.ts.map