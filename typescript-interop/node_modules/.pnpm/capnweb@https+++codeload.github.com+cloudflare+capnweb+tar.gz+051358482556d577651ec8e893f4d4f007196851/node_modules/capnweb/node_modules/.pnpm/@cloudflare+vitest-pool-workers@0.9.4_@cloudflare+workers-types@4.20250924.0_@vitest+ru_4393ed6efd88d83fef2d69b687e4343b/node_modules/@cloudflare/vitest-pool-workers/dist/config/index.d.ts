import type { WorkersPoolOptions } from "../pool/config";
import type { Awaitable, inject } from "vitest";
import type { ConfigEnv, UserConfig, UserWorkspaceConfig } from "vitest/config";
type ConfigFn<T extends UserConfig> = (env: ConfigEnv) => T | Promise<T>;
export type AnyConfigExport<T extends UserConfig> = T | Promise<T> | ConfigFn<T>;
export interface WorkerPoolOptionsContext {
    inject: typeof inject;
}
export type WorkersUserConfig<T extends UserConfig> = T & {
    test?: {
        pool?: "@cloudflare/vitest-pool-workers";
        poolMatchGlobs?: never;
        poolOptions?: {
            workers?: WorkersPoolOptions | ((ctx: WorkerPoolOptionsContext) => Awaitable<WorkersPoolOptions>);
        };
    };
};
export type WorkersUserConfigExport = WorkersUserConfig<UserConfig>;
export type WorkersProjectConfigExport = WorkersUserConfig<UserWorkspaceConfig>;
export declare function defineWorkersConfig(config: WorkersUserConfigExport): WorkersUserConfigExport;
export declare function defineWorkersConfig(config: Promise<WorkersUserConfigExport>): Promise<WorkersUserConfigExport>;
export declare function defineWorkersConfig(config: ConfigFn<WorkersUserConfigExport>): ConfigFn<WorkersUserConfigExport>;
export declare function defineWorkersProject(config: WorkersProjectConfigExport): WorkersProjectConfigExport;
export declare function defineWorkersProject(config: Promise<WorkersProjectConfigExport>): Promise<WorkersProjectConfigExport>;
export declare function defineWorkersProject(config: ConfigFn<WorkersProjectConfigExport>): ConfigFn<WorkersProjectConfigExport>;
export * from "./d1";
export * from "./pages";
