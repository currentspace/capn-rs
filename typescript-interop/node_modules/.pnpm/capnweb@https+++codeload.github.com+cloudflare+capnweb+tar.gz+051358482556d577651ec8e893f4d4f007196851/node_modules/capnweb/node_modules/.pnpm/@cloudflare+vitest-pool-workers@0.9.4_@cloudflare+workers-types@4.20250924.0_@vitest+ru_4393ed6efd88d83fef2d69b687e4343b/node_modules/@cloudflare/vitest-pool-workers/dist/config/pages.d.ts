import type { Request, Response } from "miniflare";
export declare function buildPagesASSETSBinding(assetsPath: string): Promise<(request: Request) => Promise<Response>>;
