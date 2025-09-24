import { Plugin } from 'vitest/config';
import { CDPSession, BrowserServerState as BrowserServerState$1, ProjectBrowser as ProjectBrowser$1, TestProject, BrowserProvider, Vitest, ResolvedConfig, Vite, BrowserCommand, BrowserScript, ProcessPool } from 'vitest/node';
import { StackTraceParserOptions } from '@vitest/utils/source-map';
import { ViteDevServer, HtmlTagDescriptor } from 'vite';
import { CancelReason, BrowserTesterOptions, TestExecutionMethod, RunnerTestFile, AfterSuiteRunMeta, UserConsoleLog, SnapshotResult, SerializedConfig, ErrorWithDiff, ParsedStack } from 'vitest';
import { MockedModuleSerialized } from '@vitest/mocker';
import { ServerIdResolution, ServerMockResolution } from '@vitest/mocker/node';
import { TestAnnotation, TaskResultPack, TaskEventPack } from '@vitest/runner';

type ArgumentsType<T> = T extends (...args: infer A) => any ? A : never;
type ReturnType<T> = T extends (...args: any) => infer R ? R : never;
type PromisifyFn<T> = ReturnType<T> extends Promise<any> ? T : (...args: ArgumentsType<T>) => Promise<Awaited<ReturnType<T>>>;
type BirpcFn<T> = PromisifyFn<T> & {
    /**
     * Send event without asking for response
     */
    asEvent: (...args: ArgumentsType<T>) => void;
};
type BirpcReturn<RemoteFunctions, LocalFunctions = Record<string, never>> = {
    [K in keyof RemoteFunctions]: BirpcFn<RemoteFunctions[K]>;
} & {
    $functions: LocalFunctions;
    $close: (error?: Error) => void;
    $closed: boolean;
};

interface WebSocketBrowserHandlers {
	resolveSnapshotPath: (testPath: string) => string;
	resolveSnapshotRawPath: (testPath: string, rawPath: string) => string;
	onUnhandledError: (error: unknown, type: string) => Promise<void>;
	onQueued: (method: TestExecutionMethod, file: RunnerTestFile) => void;
	onCollected: (method: TestExecutionMethod, files: RunnerTestFile[]) => Promise<void>;
	onTaskAnnotate: (testId: string, annotation: TestAnnotation) => Promise<TestAnnotation>;
	onTaskUpdate: (method: TestExecutionMethod, packs: TaskResultPack[], events: TaskEventPack[]) => void;
	onAfterSuiteRun: (meta: AfterSuiteRunMeta) => void;
	cancelCurrentRun: (reason: CancelReason) => void;
	getCountOfFailedTests: () => number;
	readSnapshotFile: (id: string) => Promise<string | null>;
	saveSnapshotFile: (id: string, content: string) => Promise<void>;
	removeSnapshotFile: (id: string) => Promise<void>;
	sendLog: (method: TestExecutionMethod, log: UserConsoleLog) => void;
	snapshotSaved: (snapshot: SnapshotResult) => void;
	debug: (...args: string[]) => void;
	resolveId: (id: string, importer?: string) => Promise<ServerIdResolution | null>;
	triggerCommand: <T>(sessionId: string, command: string, testPath: string | undefined, payload: unknown[]) => Promise<T>;
	resolveMock: (id: string, importer: string, options: {
		mock: "spy" | "factory" | "auto"
	}) => Promise<ServerMockResolution>;
	invalidate: (ids: string[]) => void;
	getBrowserFileSourceMap: (id: string) => SourceMap | null | {
		mappings: ""
	} | undefined;
	wdioSwitchContext: (direction: "iframe" | "parent") => void;
	registerMock: (sessionId: string, mock: MockedModuleSerialized) => void;
	unregisterMock: (sessionId: string, id: string) => void;
	clearMocks: (sessionId: string) => void;
	// cdp
	sendCdpEvent: (sessionId: string, event: string, payload?: Record<string, unknown>) => unknown;
	trackCdpEvent: (sessionId: string, type: "on" | "once" | "off", event: string, listenerId: string) => void;
}
interface WebSocketBrowserEvents {
	onCancel: (reason: CancelReason) => void;
	createTesters: (options: BrowserTesterOptions) => Promise<void>;
	cleanupTesters: () => Promise<void>;
	cdpEvent: (event: string, payload: unknown) => void;
	resolveManualMock: (url: string) => Promise<{
		url: string
		keys: string[]
		responseId: string
	}>;
}
type WebSocketBrowserRPC = BirpcReturn<WebSocketBrowserEvents, WebSocketBrowserHandlers>;
interface SourceMap {
	file: string;
	mappings: string;
	names: string[];
	sources: string[];
	sourcesContent?: string[];
	version: number;
	toString: () => string;
	toUrl: () => string;
}

declare class BrowserServerCDPHandler {
	private session;
	private tester;
	private listenerIds;
	private listeners;
	constructor(session: CDPSession, tester: WebSocketBrowserRPC);
	send(method: string, params?: Record<string, unknown>): Promise<unknown>;
	on(event: string, id: string, once?: boolean): void;
	off(event: string, id: string): void;
	once(event: string, listener: string): void;
}

declare class BrowserServerState implements BrowserServerState$1 {
	readonly orchestrators: Map<string, WebSocketBrowserRPC>;
	readonly testers: Map<string, WebSocketBrowserRPC>;
}

declare class ProjectBrowser implements ProjectBrowser$1 {
	project: TestProject;
	base: string;
	testerHtml: Promise<string> | string;
	testerFilepath: string;
	provider: BrowserProvider;
	vitest: Vitest;
	config: ResolvedConfig;
	children: Set<ProjectBrowser>;
	parent: ParentBrowserProject;
	state: BrowserServerState;
	constructor(project: TestProject, base: string);
	get vite(): ViteDevServer;
	wrapSerializedConfig(): SerializedConfig;
	initBrowserProvider(project: TestProject): Promise<void>;
	parseErrorStacktrace(e: ErrorWithDiff, options?: StackTraceParserOptions): ParsedStack[];
	parseStacktrace(trace: string, options?: StackTraceParserOptions): ParsedStack[];
	close(): Promise<void>;
}

declare class ParentBrowserProject {
	project: TestProject;
	base: string;
	orchestratorScripts: string | undefined;
	testerScripts: HtmlTagDescriptor[] | undefined;
	faviconUrl: string;
	prefixOrchestratorUrl: string;
	prefixTesterUrl: string;
	manifest: Promise<Vite.Manifest> | Vite.Manifest;
	vite: Vite.ViteDevServer;
	private stackTraceOptions;
	orchestratorHtml: Promise<string> | string;
	injectorJs: Promise<string> | string;
	errorCatcherUrl: string;
	locatorsUrl: string | undefined;
	matchersUrl: string;
	stateJs: Promise<string> | string;
	commands: Record<string, BrowserCommand<any>>;
	children: Set<ProjectBrowser>;
	vitest: Vitest;
	config: ResolvedConfig;
	// cache for non-vite source maps
	private sourceMapCache;
	constructor(project: TestProject, base: string);
	setServer(vite: Vite.ViteDevServer): void;
	spawn(project: TestProject): ProjectBrowser;
	parseErrorStacktrace(e: ErrorWithDiff, options?: StackTraceParserOptions): ParsedStack[];
	parseStacktrace(trace: string, options?: StackTraceParserOptions): ParsedStack[];
	readonly cdps: Map<string, BrowserServerCDPHandler>;
	private cdpSessionsPromises;
	ensureCDPHandler(sessionId: string, rpcId: string): Promise<BrowserServerCDPHandler>;
	removeCDPHandler(sessionId: string): void;
	formatScripts(scripts: BrowserScript[] | undefined): Promise<HtmlTagDescriptor[]>;
	resolveTesterUrl(pathname: string): {
		sessionId: string
		testFile: string
	};
	private retrieveSourceMapURL;
}

declare const distRoot: string;

declare function createBrowserPool(vitest: Vitest): ProcessPool;

declare function createBrowserServer(project: TestProject, configFile: string | undefined, prePlugins?: Plugin[], postPlugins?: Plugin[]): Promise<ParentBrowserProject>;

export { ProjectBrowser, createBrowserPool, createBrowserServer, distRoot };
