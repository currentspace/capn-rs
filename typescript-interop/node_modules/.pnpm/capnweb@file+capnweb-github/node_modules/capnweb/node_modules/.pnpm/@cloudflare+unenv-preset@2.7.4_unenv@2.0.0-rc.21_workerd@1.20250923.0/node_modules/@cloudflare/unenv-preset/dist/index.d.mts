import * as unenv from 'unenv';
import { Preset } from 'unenv';

/**
 * Creates the Cloudflare preset for the given compatibility date and compatibility flags
 *
 * @param compatibilityDate workerd compatibility date
 * @param compatibilityFlags workerd compatibility flags
 * @returns The cloudflare preset
 */
declare function getCloudflarePreset({ compatibilityDate, compatibilityFlags, }: {
    compatibilityDate?: string;
    compatibilityFlags?: string[];
}): Preset;

/**
 * @deprecated Use getCloudflarePreset instead.
 */
declare const cloudflare: unenv.Preset;

export { cloudflare, getCloudflarePreset };
