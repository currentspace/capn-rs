import { Transport, ClientConfig, Logger, CapId, Plan, Source, JsonValue, Message, CallMessage, ResultMessage } from './types.js';

/**
 * Cap'n Web TypeScript Client
 *
 * A complete TypeScript implementation of the Cap'n Web client
 * for interoperability testing with the Rust server.
 */

declare class CapnWebClient {
    private transport;
    private config;
    private logger;
    private nextCallId;
    private pendingCalls;
    private isListening;
    constructor(transport: Transport, config?: ClientConfig, logger?: Logger);
    connect(): Promise<void>;
    private startListening;
    private listenForResponses;
    private handleMessage;
    private handleResult;
    call(capId: CapId, member: string, args: unknown[]): Promise<unknown>;
    executePlan(plan: Plan, params?: Record<string, unknown>): Promise<unknown>;
    dispose(capIds: CapId[]): Promise<void>;
    close(): Promise<void>;
}
/**
 * Utility class for building Cap'n Web plans in TypeScript
 */
declare class PlanBuilder {
    private captures;
    private ops;
    private nextResult;
    capture(capId: CapId): CapRef;
    addOp(op: any): ResultRef;
    build(result: Source): Plan;
    object(fields: Record<string, Source>): ResultRef;
    array(items: Source[]): ResultRef;
}
declare class CapRef {
    private builder;
    private index;
    constructor(builder: PlanBuilder, index: number);
    call(member: string, args: Source[]): ResultRef;
    asSource(): Source;
}
declare class ResultRef {
    private builder;
    private index;
    constructor(builder: PlanBuilder, index: number);
    call(member: string, args: Source[]): ResultRef;
    asSource(): Source;
}
declare const Param: {
    path: (path: string[]) => Source;
    value: (value: JsonValue) => Source;
};
declare function isCallMessage(message: Message): message is CallMessage;
declare function isResultMessage(message: Message): message is ResultMessage;

export { CapRef, CapnWebClient, Param, PlanBuilder, ResultRef, isCallMessage, isResultMessage };
