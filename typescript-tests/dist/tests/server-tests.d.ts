/**
 * TypeScript Server Tests Against Rust Client
 *
 * Tests the TypeScript Cap'n Web server against the Rust client implementation
 * to verify bidirectional interoperability.
 */
declare class TypeScriptServerTests {
    private framework;
    private server;
    private rustClientProcess;
    constructor();
    runAllTests(): Promise<void>;
    private setupServer;
    private cleanup;
    private testServerStartup;
    private testCapabilityRegistration;
    private testServerCalculatorOperations;
    private testServerErrorHandling;
    private testServerUserManagement;
    private testRustClientConnection;
    private testRustClientCalculatorCalls;
    private testRustClientErrorScenarios;
    private testComplexDataStructures;
    private testConcurrentRustClients;
    private testLongRunningOperations;
    private runRustClientTest;
}
declare function runServerTests(): Promise<void>;

export { TypeScriptServerTests, runServerTests };
