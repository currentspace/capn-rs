import type { WorkspaceProject } from "vitest/node";
export declare const WORKER_NAME_PREFIX = "vitest-pool-workers-";
export declare function isFileNotFoundError(e: unknown): boolean;
export declare function getProjectPath(project: WorkspaceProject): string | number;
export declare function getRelativeProjectPath(project: WorkspaceProject): string | number;
