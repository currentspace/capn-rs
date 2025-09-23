/**
 * TypeScript Client Tests Against Rust Server
 *
 * Tests the TypeScript Cap'n Web client against the Rust server implementation
 * to verify complete interoperability.
 */
declare class TypeScriptClientTests {
    private framework;
    private client;
    private transport;
    constructor();
    runAllTests(): Promise<void>;
    private setupClient;
    private cleanup;
    private testConnection;
    private testConnectionTimeout;
    private testBasicCalculatorOperations;
    private testCalculatorErrorHandling;
    private testMultipleSequentialCalls;
    private testConcurrentCalls;
    private testAdvancedMathOperations;
    private testFactorialOperations;
    private testErrorConditions;
    private testUserManagement;
    private testUserNotFound;
    private testCreateUser;
    private testPerformance;
    private testHighVolumeOperations;
    private testLargeNumbers;
    private testInvalidArguments;
    private testInvalidMethods;
}
declare function runClientTests(): Promise<void>;

export { TypeScriptClientTests, runClientTests };
