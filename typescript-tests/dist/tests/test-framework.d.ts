import { Logger } from '../capnweb/types.js';

/**
 * TypeScript Test Framework for Cap'n Web Interoperability
 *
 * A comprehensive test framework for verifying interoperability between
 * TypeScript and Rust Cap'n Web implementations.
 */

interface TestResult {
    name: string;
    passed: boolean;
    error?: Error;
    duration: number;
    details?: string;
}
interface TestSuite {
    name: string;
    results: TestResult[];
    totalPassed: number;
    totalFailed: number;
    totalDuration: number;
}
declare class InteropTestFramework {
    private logger;
    private suites;
    constructor(logger?: Logger);
    runTestSuite(name: string, testFunctions: Array<() => Promise<void>>): Promise<TestSuite>;
    private runSingleTest;
    private logSuiteResults;
    generateReport(): void;
    getAllResults(): TestSuite[];
    getOverallStats(): {
        totalTests: number;
        totalPassed: number;
        totalFailed: number;
        totalDuration: number;
        successRate: number;
    };
}
declare class InteropAssert {
    static equal(actual: unknown, expected: unknown, message?: string): void;
    static deepEqual(actual: unknown, expected: unknown, message?: string): void;
    static ok(value: unknown, message?: string): void;
    static throws(fn: () => void, message?: string): void;
    static rejects(promise: Promise<unknown>, message?: string): Promise<void>;
    static doesNotReject(promise: Promise<unknown>, message?: string): Promise<void>;
    static isNumber(value: unknown, message?: string): void;
    static isString(value: unknown, message?: string): void;
    static isObject(value: unknown, message?: string): void;
    static isArray(value: unknown, message?: string): void;
    static hasProperty(obj: object, property: string, message?: string): void;
    static approximatelyEqual(actual: number, expected: number, tolerance?: number, message?: string): void;
}
declare class Timer {
    private startTime;
    start(): void;
    elapsed(): number;
    static measure<T>(operation: () => Promise<T>): Promise<{
        result: T;
        duration: number;
    }>;
}
declare const wait: (ms: number) => Promise<void>;

export { InteropAssert, InteropTestFramework, type TestResult, type TestSuite, Timer, wait };
