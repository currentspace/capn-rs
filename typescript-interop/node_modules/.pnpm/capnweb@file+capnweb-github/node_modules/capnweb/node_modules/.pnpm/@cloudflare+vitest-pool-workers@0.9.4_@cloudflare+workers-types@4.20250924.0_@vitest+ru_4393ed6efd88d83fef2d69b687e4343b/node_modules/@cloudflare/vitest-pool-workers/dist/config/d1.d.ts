import type { D1Migration } from "../shared/d1";
/**
 * Reads all migrations in `migrationsPath`, ordered by migration number.
 * Each migration will have its contents split into an array of SQL queries.
 */
export declare function readD1Migrations(migrationsPath: string): Promise<D1Migration[]>;
export type { D1Migration };
