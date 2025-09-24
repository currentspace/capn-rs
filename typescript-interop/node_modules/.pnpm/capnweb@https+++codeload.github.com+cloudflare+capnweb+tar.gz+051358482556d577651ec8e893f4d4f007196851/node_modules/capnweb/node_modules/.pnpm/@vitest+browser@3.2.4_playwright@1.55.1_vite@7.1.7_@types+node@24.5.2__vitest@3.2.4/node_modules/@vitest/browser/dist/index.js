import { ManualMockedModule, RedirectedModule, AutomockedModule, AutospiedModule, MockerRegistry } from '@vitest/mocker';
import { dynamicImportPlugin, ServerMockResolver, interceptorPlugin } from '@vitest/mocker/node';
import c from 'tinyrainbow';
import { getFilePoolName, distDir, resolveApiServerConfig, resolveFsAllow, isFileServingAllowed, createDebugger, isValidApiRequest, createViteLogger, createViteServer } from 'vitest/node';
import fs, { readFileSync, lstatSync, promises, existsSync } from 'node:fs';
import { createRequire } from 'node:module';
import { slash as slash$1, toArray, createDefer } from '@vitest/utils';
import MagicString from 'magic-string';
import sirv from 'sirv';
import * as vite from 'vite';
import { coverageConfigDefaults } from 'vitest/config';
import { fileURLToPath } from 'node:url';
import crypto from 'node:crypto';
import { mkdir, rm, readFile as readFile$1 } from 'node:fs/promises';
import { parseErrorStacktrace, parseStacktrace } from '@vitest/utils/source-map';
import { P as PlaywrightBrowserProvider, W as WebdriverBrowserProvider } from './webdriver-KA1WiV0q.js';
import { resolve as resolve$1, basename as basename$1, dirname as dirname$1, normalize as normalize$1 } from 'node:path';
import { WebSocketServer } from 'ws';
import * as nodeos from 'node:os';
import { performance } from 'node:perf_hooks';

var version = "3.2.4";

const _DRIVE_LETTER_START_RE = /^[A-Za-z]:\//;
function normalizeWindowsPath(input = "") {
  if (!input) {
    return input;
  }
  return input.replace(/\\/g, "/").replace(_DRIVE_LETTER_START_RE, (r) => r.toUpperCase());
}

const _UNC_REGEX = /^[/\\]{2}/;
const _IS_ABSOLUTE_RE = /^[/\\](?![/\\])|^[/\\]{2}(?!\.)|^[A-Za-z]:[/\\]/;
const _DRIVE_LETTER_RE = /^[A-Za-z]:$/;
const _ROOT_FOLDER_RE = /^\/([A-Za-z]:)?$/;
const _EXTNAME_RE = /.(\.[^./]+|\.)$/;
const normalize = function(path) {
  if (path.length === 0) {
    return ".";
  }
  path = normalizeWindowsPath(path);
  const isUNCPath = path.match(_UNC_REGEX);
  const isPathAbsolute = isAbsolute(path);
  const trailingSeparator = path[path.length - 1] === "/";
  path = normalizeString(path, !isPathAbsolute);
  if (path.length === 0) {
    if (isPathAbsolute) {
      return "/";
    }
    return trailingSeparator ? "./" : ".";
  }
  if (trailingSeparator) {
    path += "/";
  }
  if (_DRIVE_LETTER_RE.test(path)) {
    path += "/";
  }
  if (isUNCPath) {
    if (!isPathAbsolute) {
      return `//./${path}`;
    }
    return `//${path}`;
  }
  return isPathAbsolute && !isAbsolute(path) ? `/${path}` : path;
};
const join = function(...segments) {
  let path = "";
  for (const seg of segments) {
    if (!seg) {
      continue;
    }
    if (path.length > 0) {
      const pathTrailing = path[path.length - 1] === "/";
      const segLeading = seg[0] === "/";
      const both = pathTrailing && segLeading;
      if (both) {
        path += seg.slice(1);
      } else {
        path += pathTrailing || segLeading ? seg : `/${seg}`;
      }
    } else {
      path += seg;
    }
  }
  return normalize(path);
};
function cwd() {
  if (typeof process !== "undefined" && typeof process.cwd === "function") {
    return process.cwd().replace(/\\/g, "/");
  }
  return "/";
}
const resolve = function(...arguments_) {
  arguments_ = arguments_.map((argument) => normalizeWindowsPath(argument));
  let resolvedPath = "";
  let resolvedAbsolute = false;
  for (let index = arguments_.length - 1; index >= -1 && !resolvedAbsolute; index--) {
    const path = index >= 0 ? arguments_[index] : cwd();
    if (!path || path.length === 0) {
      continue;
    }
    resolvedPath = `${path}/${resolvedPath}`;
    resolvedAbsolute = isAbsolute(path);
  }
  resolvedPath = normalizeString(resolvedPath, !resolvedAbsolute);
  if (resolvedAbsolute && !isAbsolute(resolvedPath)) {
    return `/${resolvedPath}`;
  }
  return resolvedPath.length > 0 ? resolvedPath : ".";
};
function normalizeString(path, allowAboveRoot) {
  let res = "";
  let lastSegmentLength = 0;
  let lastSlash = -1;
  let dots = 0;
  let char = null;
  for (let index = 0; index <= path.length; ++index) {
    if (index < path.length) {
      char = path[index];
    } else if (char === "/") {
      break;
    } else {
      char = "/";
    }
    if (char === "/") {
      if (lastSlash === index - 1 || dots === 1) ; else if (dots === 2) {
        if (res.length < 2 || lastSegmentLength !== 2 || res[res.length - 1] !== "." || res[res.length - 2] !== ".") {
          if (res.length > 2) {
            const lastSlashIndex = res.lastIndexOf("/");
            if (lastSlashIndex === -1) {
              res = "";
              lastSegmentLength = 0;
            } else {
              res = res.slice(0, lastSlashIndex);
              lastSegmentLength = res.length - 1 - res.lastIndexOf("/");
            }
            lastSlash = index;
            dots = 0;
            continue;
          } else if (res.length > 0) {
            res = "";
            lastSegmentLength = 0;
            lastSlash = index;
            dots = 0;
            continue;
          }
        }
        if (allowAboveRoot) {
          res += res.length > 0 ? "/.." : "..";
          lastSegmentLength = 2;
        }
      } else {
        if (res.length > 0) {
          res += `/${path.slice(lastSlash + 1, index)}`;
        } else {
          res = path.slice(lastSlash + 1, index);
        }
        lastSegmentLength = index - lastSlash - 1;
      }
      lastSlash = index;
      dots = 0;
    } else if (char === "." && dots !== -1) {
      ++dots;
    } else {
      dots = -1;
    }
  }
  return res;
}
const isAbsolute = function(p) {
  return _IS_ABSOLUTE_RE.test(p);
};
const extname = function(p) {
  if (p === "..") return "";
  const match = _EXTNAME_RE.exec(normalizeWindowsPath(p));
  return match && match[1] || "";
};
const relative = function(from, to) {
  const _from = resolve(from).replace(_ROOT_FOLDER_RE, "$1").split("/");
  const _to = resolve(to).replace(_ROOT_FOLDER_RE, "$1").split("/");
  if (_to[0][1] === ":" && _from[0][1] === ":" && _from[0] !== _to[0]) {
    return _to.join("/");
  }
  const _fromCopy = [..._from];
  for (const segment of _fromCopy) {
    if (_to[0] !== segment) {
      break;
    }
    _from.shift();
    _to.shift();
  }
  return [..._from.map(() => ".."), ..._to].join("/");
};
const dirname = function(p) {
  const segments = normalizeWindowsPath(p).replace(/\/$/, "").split("/").slice(0, -1);
  if (segments.length === 1 && _DRIVE_LETTER_RE.test(segments[0])) {
    segments[0] += "/";
  }
  return segments.join("/") || (isAbsolute(p) ? "/" : ".");
};
const basename = function(p, extension) {
  const segments = normalizeWindowsPath(p).split("/");
  let lastSegment = "";
  for (let i = segments.length - 1; i >= 0; i--) {
    const val = segments[i];
    if (val) {
      lastSegment = val;
      break;
    }
  }
  return extension && lastSegment.endsWith(extension) ? lastSegment.slice(0, -extension.length) : lastSegment;
};

const pkgRoot = resolve(fileURLToPath(import.meta.url), "../..");
const distRoot = resolve(pkgRoot, "dist");

/// <reference types="../types/index.d.ts" />

// (c) 2020-present Andrea Giammarchi

const {parse: $parse, stringify: $stringify} = JSON;
const {keys} = Object;

const Primitive = String;   // it could be Number
const primitive = 'string'; // it could be 'number'

const ignore = {};
const object = 'object';

const noop = (_, value) => value;

const primitives = value => (
  value instanceof Primitive ? Primitive(value) : value
);

const Primitives = (_, value) => (
  typeof value === primitive ? new Primitive(value) : value
);

const revive = (input, parsed, output, $) => {
  const lazy = [];
  for (let ke = keys(output), {length} = ke, y = 0; y < length; y++) {
    const k = ke[y];
    const value = output[k];
    if (value instanceof Primitive) {
      const tmp = input[value];
      if (typeof tmp === object && !parsed.has(tmp)) {
        parsed.add(tmp);
        output[k] = ignore;
        lazy.push({k, a: [input, parsed, tmp, $]});
      }
      else
        output[k] = $.call(output, k, tmp);
    }
    else if (output[k] !== ignore)
      output[k] = $.call(output, k, value);
  }
  for (let {length} = lazy, i = 0; i < length; i++) {
    const {k, a} = lazy[i];
    output[k] = $.call(output, k, revive.apply(null, a));
  }
  return output;
};

const set = (known, input, value) => {
  const index = Primitive(input.push(value) - 1);
  known.set(value, index);
  return index;
};

/**
 * Converts a specialized flatted string into a JS value.
 * @param {string} text
 * @param {(this: any, key: string, value: any) => any} [reviver]
 * @returns {any}
 */
const parse = (text, reviver) => {
  const input = $parse(text, Primitives).map(primitives);
  const value = input[0];
  const $ = reviver || noop;
  const tmp = typeof value === object && value ?
              revive(input, new Set, value, $) :
              value;
  return $.call({'': tmp}, '', tmp);
};

/**
 * Converts a JS value into a specialized flatted string.
 * @param {any} value
 * @param {((this: any, key: string, value: any) => any) | (string | number)[] | null | undefined} [replacer]
 * @param {string | number | undefined} [space]
 * @returns {string}
 */
const stringify = (value, replacer, space) => {
  const $ = replacer && typeof replacer === object ?
            (k, v) => (k === '' || -1 < replacer.indexOf(k) ? v : void 0) :
            (replacer || noop);
  const known = new Map;
  const input = [];
  const output = [];
  let i = +set(known, input, $.call({'': value}, '', value));
  let firstRun = !i;
  while (i < input.length) {
    firstRun = true;
    output[i] = $stringify(input[i++], replace, space);
  }
  return '[' + output.join(',') + ']';
  function replace(key, value) {
    if (firstRun) {
      firstRun = !firstRun;
      return value;
    }
    const after = $.call(this, key, value);
    switch (typeof after) {
      case object:
        if (after === null) return after;
      case primitive:
        return known.get(after) || set(known, input, after);
    }
    return after;
  }
};

function replacer(code, values) {
	return code.replace(/\{\s*(\w+)\s*\}/g, (_, key) => values[key] ?? _);
}
const builtinProviders = [
	"webdriverio",
	"playwright",
	"preview"
];
async function getBrowserProvider(options, project) {
	if (options.provider == null || builtinProviders.includes(options.provider)) {
		const providers = await import('./providers.js');
		const provider = options.provider || "preview";
		return providers[provider];
	}
	let customProviderModule;
	try {
		customProviderModule = await project.import(options.provider);
	} catch (error) {
		throw new Error(`Failed to load custom BrowserProvider from ${options.provider}`, { cause: error });
	}
	if (customProviderModule.default == null) {
		throw new Error(`Custom BrowserProvider loaded from ${options.provider} was not the default export`);
	}
	return customProviderModule.default;
}
function slash(path) {
	return path.replace(/\\/g, "/").replace(/\/+/g, "/");
}

async function resolveOrchestrator(globalServer, url, res) {
	let sessionId = url.searchParams.get("sessionId");
	// it's possible to open the page without a context
	if (!sessionId) {
		const contexts = [...globalServer.children].flatMap((p) => [...p.state.orchestrators.keys()]);
		sessionId = contexts[contexts.length - 1] ?? "none";
	}
	// it's ok to not have a session here, especially in the preview provider
	// because the user could refresh the page which would remove the session id from the url
	const session = globalServer.vitest._browserSessions.getSession(sessionId);
	const browserProject = session?.project.browser || [...globalServer.children][0];
	if (!browserProject) {
		return;
	}
	// ignore uknown pages
	if (sessionId && sessionId !== "none" && !globalServer.vitest._browserSessions.sessionIds.has(sessionId)) {
		return;
	}
	const injectorJs = typeof globalServer.injectorJs === "string" ? globalServer.injectorJs : await globalServer.injectorJs;
	const injector = replacer(injectorJs, {
		__VITEST_PROVIDER__: JSON.stringify(browserProject.config.browser.provider || "preview"),
		__VITEST_CONFIG__: JSON.stringify(browserProject.wrapSerializedConfig()),
		__VITEST_VITE_CONFIG__: JSON.stringify({ root: browserProject.vite.config.root }),
		__VITEST_METHOD__: JSON.stringify("orchestrate"),
		__VITEST_TYPE__: "\"orchestrator\"",
		__VITEST_SESSION_ID__: JSON.stringify(sessionId),
		__VITEST_TESTER_ID__: "\"none\"",
		__VITEST_PROVIDED_CONTEXT__: JSON.stringify(stringify(browserProject.project.getProvidedContext())),
		__VITEST_API_TOKEN__: JSON.stringify(globalServer.vitest.config.api.token)
	});
	// disable CSP for the orchestrator as we are the ones controlling it
	res.removeHeader("Content-Security-Policy");
	if (!globalServer.orchestratorScripts) {
		globalServer.orchestratorScripts = (await globalServer.formatScripts(globalServer.config.browser.orchestratorScripts)).map((script) => {
			let html = "<script ";
			for (const attr in script.attrs || {}) {
				html += `${attr}="${script.attrs[attr]}" `;
			}
			html += `>${script.children}</script>`;
			return html;
		}).join("\n");
	}
	let baseHtml = typeof globalServer.orchestratorHtml === "string" ? globalServer.orchestratorHtml : await globalServer.orchestratorHtml;
	// if UI is enabled, use UI HTML and inject the orchestrator script
	if (globalServer.config.browser.ui) {
		const manifestContent = globalServer.manifest instanceof Promise ? await globalServer.manifest : globalServer.manifest;
		const jsEntry = manifestContent["orchestrator.html"].file;
		const base = browserProject.parent.vite.config.base || "/";
		baseHtml = baseHtml.replaceAll("./assets/", `${base}__vitest__/assets/`).replace("<!-- !LOAD_METADATA! -->", [
			"{__VITEST_INJECTOR__}",
			"{__VITEST_ERROR_CATCHER__}",
			"{__VITEST_SCRIPTS__}",
			`<script type="module" crossorigin src="${base}${jsEntry}"></script>`
		].join("\n"));
	}
	return replacer(baseHtml, {
		__VITEST_FAVICON__: globalServer.faviconUrl,
		__VITEST_TITLE__: "Vitest Browser Runner",
		__VITEST_SCRIPTS__: globalServer.orchestratorScripts,
		__VITEST_INJECTOR__: `<script type="module">${injector}</script>`,
		__VITEST_ERROR_CATCHER__: `<script type="module" src="${globalServer.errorCatcherUrl}"></script>`,
		__VITEST_SESSION_ID__: JSON.stringify(sessionId)
	});
}

function disableCache(res) {
	res.setHeader("Cache-Control", "no-cache, max-age=0, must-revalidate");
	res.setHeader("Content-Type", "text/html; charset=utf-8");
}
function allowIframes(res) {
	// remove custom iframe related headers to allow the iframe to load
	res.removeHeader("X-Frame-Options");
}

function createOrchestratorMiddleware(parentServer) {
	return async function vitestOrchestratorMiddleware(req, res, next) {
		if (!req.url) {
			return next();
		}
		const url = new URL(req.url, "http://localhost");
		if (url.pathname !== parentServer.prefixOrchestratorUrl) {
			return next();
		}
		const html = await resolveOrchestrator(parentServer, url, res);
		if (html) {
			disableCache(res);
			allowIframes(res);
			res.write(html, "utf-8");
			res.end();
		}
	};
}

async function resolveTester(globalServer, url, res, next) {
	const csp = res.getHeader("Content-Security-Policy");
	if (typeof csp === "string") {
		// add frame-ancestors to allow the iframe to be loaded by Vitest,
		// but keep the rest of the CSP
		res.setHeader("Content-Security-Policy", csp.replace(/frame-ancestors [^;]+/, "frame-ancestors *"));
	}
	const sessionId = url.searchParams.get("sessionId") || "none";
	const session = globalServer.vitest._browserSessions.getSession(sessionId);
	if (!session) {
		res.statusCode = 400;
		res.end("Invalid session ID");
		return;
	}
	const project = globalServer.vitest.getProjectByName(session.project.name || "");
	const browserProject = project.browser || [...globalServer.children][0];
	if (!browserProject) {
		res.statusCode = 400;
		res.end("Invalid session ID");
		return;
	}
	const injectorJs = typeof globalServer.injectorJs === "string" ? globalServer.injectorJs : await globalServer.injectorJs;
	const injector = replacer(injectorJs, {
		__VITEST_PROVIDER__: JSON.stringify(project.browser.provider.name),
		__VITEST_CONFIG__: JSON.stringify(browserProject.wrapSerializedConfig()),
		__VITEST_VITE_CONFIG__: JSON.stringify({ root: browserProject.vite.config.root }),
		__VITEST_TYPE__: "\"tester\"",
		__VITEST_METHOD__: JSON.stringify("none"),
		__VITEST_SESSION_ID__: JSON.stringify(sessionId),
		__VITEST_TESTER_ID__: JSON.stringify(crypto.randomUUID()),
		__VITEST_PROVIDED_CONTEXT__: "{}",
		__VITEST_API_TOKEN__: JSON.stringify(globalServer.vitest.config.api.token)
	});
	const testerHtml = typeof browserProject.testerHtml === "string" ? browserProject.testerHtml : await browserProject.testerHtml;
	try {
		const url = join("/@fs/", browserProject.testerFilepath);
		const indexhtml = await browserProject.vite.transformIndexHtml(url, testerHtml);
		const html = replacer(indexhtml, {
			__VITEST_FAVICON__: globalServer.faviconUrl,
			__VITEST_INJECTOR__: injector
		});
		return html;
	} catch (err) {
		session.fail(err);
		next(err);
	}
}

function createTesterMiddleware(browserServer) {
	return async function vitestTesterMiddleware(req, res, next) {
		if (!req.url) {
			return next();
		}
		const url = new URL(req.url, "http://localhost");
		if (url.pathname !== browserServer.prefixTesterUrl || !url.searchParams.has("sessionId")) {
			return next();
		}
		const html = await resolveTester(browserServer, url, res, next);
		if (html) {
			disableCache(res);
			allowIframes(res);
			res.write(html, "utf-8");
			res.end();
		}
	};
}

const VIRTUAL_ID_CONTEXT = "\0@vitest/browser/context";
const ID_CONTEXT = "@vitest/browser/context";
const __dirname = dirname(fileURLToPath(import.meta.url));
function BrowserContext(globalServer) {
	return {
		name: "vitest:browser:virtual-module:context",
		enforce: "pre",
		resolveId(id) {
			if (id === ID_CONTEXT) {
				return VIRTUAL_ID_CONTEXT;
			}
		},
		load(id) {
			if (id === VIRTUAL_ID_CONTEXT) {
				return generateContextFile.call(this, globalServer);
			}
		}
	};
}
async function generateContextFile(globalServer) {
	const commands = Object.keys(globalServer.commands);
	const provider = [...globalServer.children][0].provider || { name: "preview" };
	const providerName = provider.name;
	const commandsCode = commands.filter((command) => !command.startsWith("__vitest")).map((command) => {
		return `    ["${command}"]: (...args) => __vitest_browser_runner__.commands.triggerCommand("${command}", args),`;
	}).join("\n");
	const userEventNonProviderImport = await getUserEventImport(providerName, this.resolve.bind(this));
	const distContextPath = slash$1(`/@fs/${resolve(__dirname, "context.js")}`);
	return `
import { page, createUserEvent, cdp, locators } from '${distContextPath}'
${userEventNonProviderImport}

export const server = {
  platform: ${JSON.stringify(process.platform)},
  version: ${JSON.stringify(process.version)},
  provider: ${JSON.stringify(providerName)},
  browser: __vitest_browser_runner__.config.browser.name,
  commands: {
    ${commandsCode}
  },
  config: __vitest_browser_runner__.config,
}
export const commands = server.commands
export const userEvent = createUserEvent(_userEventSetup)
export { page, cdp, locators }
`;
}
async function getUserEventImport(provider, resolve) {
	if (provider !== "preview") {
		return "const _userEventSetup = undefined";
	}
	const resolved = await resolve("@testing-library/user-event", __dirname);
	if (!resolved) {
		throw new Error(`Failed to resolve user-event package from ${__dirname}`);
	}
	return `\
import { userEvent as __vitest_user_event__ } from '${slash$1(`/@fs/${resolved.id}`)}'
const _userEventSetup = __vitest_user_event__
`;
}

const versionRegexp = /(?:\?|&)v=\w{8}/;
var BrowserPlugin = (parentServer, base = "/") => {
	function isPackageExists(pkg, root) {
		return parentServer.vitest.packageInstaller.isPackageExists?.(pkg, { paths: [root] });
	}
	return [
		{
			enforce: "pre",
			name: "vitest:browser",
			async configureServer(server) {
				parentServer.setServer(server);
				// eslint-disable-next-line prefer-arrow-callback
				server.middlewares.use(function vitestHeaders(_req, res, next) {
					const headers = server.config.server.headers;
					if (headers) {
						for (const name in headers) {
							res.setHeader(name, headers[name]);
						}
					}
					next();
				});
				server.middlewares.use(createOrchestratorMiddleware(parentServer));
				server.middlewares.use(createTesterMiddleware(parentServer));
				server.middlewares.use(`${base}favicon.svg`, (_, res) => {
					const content = readFileSync(resolve(distRoot, "client/favicon.svg"));
					res.write(content, "utf-8");
					res.end();
				});
				const coverageFolder = resolveCoverageFolder(parentServer.vitest);
				const coveragePath = coverageFolder ? coverageFolder[1] : undefined;
				if (coveragePath && base === coveragePath) {
					throw new Error(`The ui base path and the coverage path cannot be the same: ${base}, change coverage.reportsDirectory`);
				}
				if (coverageFolder) {
					server.middlewares.use(coveragePath, sirv(coverageFolder[0], {
						single: true,
						dev: true,
						setHeaders: (res) => {
							res.setHeader("Cache-Control", "public,max-age=0,must-revalidate");
						}
					}));
				}
				const uiEnabled = parentServer.config.browser.ui;
				if (uiEnabled) {
					// eslint-disable-next-line prefer-arrow-callback
					server.middlewares.use(`${base}__screenshot-error`, function vitestBrowserScreenshotError(req, res) {
						if (!req.url) {
							res.statusCode = 404;
							res.end();
							return;
						}
						const url = new URL(req.url, "http://localhost");
						const id = url.searchParams.get("id");
						if (!id) {
							res.statusCode = 404;
							res.end();
							return;
						}
						const task = parentServer.vitest.state.idMap.get(id);
						const file = task?.meta.failScreenshotPath;
						if (!file) {
							res.statusCode = 404;
							res.end();
							return;
						}
						let stat;
						try {
							stat = lstatSync(file);
						} catch {}
						if (!stat?.isFile()) {
							res.statusCode = 404;
							res.end();
							return;
						}
						const ext = extname(file);
						const buffer = readFileSync(file);
						res.setHeader("Cache-Control", "public,max-age=0,must-revalidate");
						res.setHeader("Content-Length", buffer.length);
						res.setHeader("Content-Type", ext === "jpeg" || ext === "jpg" ? "image/jpeg" : ext === "webp" ? "image/webp" : "image/png");
						res.end(buffer);
					});
				}
				server.middlewares.use((req, res, next) => {
					// 9000 mega head move
					// Vite always caches optimized dependencies, but users might mock
					// them in _some_ tests, while keeping original modules in others
					// there is no way to configure that in Vite, so we patch it here
					// to always ignore the cache-control set by Vite in the next middleware
					if (req.url && versionRegexp.test(req.url) && !req.url.includes("chunk-")) {
						res.setHeader("Cache-Control", "no-cache");
						const setHeader = res.setHeader.bind(res);
						res.setHeader = function(name, value) {
							if (name === "Cache-Control") {
								return res;
							}
							return setHeader(name, value);
						};
					}
					next();
				});
			}
		},
		{
			name: "vitest:browser:tests",
			enforce: "pre",
			async config() {
				// this plugin can be used in different projects, but all of them
				// have the same `include` pattern, so it doesn't matter which project we use
				const project = parentServer.project;
				const { testFiles: allTestFiles } = await project.globTestFiles();
				const browserTestFiles = allTestFiles.filter((file) => getFilePoolName(project, file) === "browser");
				const setupFiles = toArray(project.config.setupFiles);
				// replace env values - cannot be reassign at runtime
				const define = {};
				for (const env in project.config.env || {}) {
					const stringValue = JSON.stringify(project.config.env[env]);
					define[`import.meta.env.${env}`] = stringValue;
				}
				const entries = [
					...browserTestFiles,
					...setupFiles,
					resolve(distDir, "index.js"),
					resolve(distDir, "browser.js"),
					resolve(distDir, "runners.js"),
					resolve(distDir, "utils.js"),
					...project.config.snapshotSerializers || []
				];
				const exclude = [
					"vitest",
					"vitest/internal/browser",
					"vitest/runners",
					"@vitest/browser",
					"@vitest/browser/client",
					"@vitest/utils",
					"@vitest/utils/source-map",
					"@vitest/runner",
					"@vitest/spy",
					"@vitest/utils/error",
					"@vitest/snapshot",
					"@vitest/expect",
					"std-env",
					"tinybench",
					"tinyspy",
					"tinyrainbow",
					"pathe",
					"msw",
					"msw/browser"
				];
				if (typeof project.config.diff === "string") {
					entries.push(project.config.diff);
				}
				if (parentServer.vitest.coverageProvider) {
					const coverage = parentServer.vitest.config.coverage;
					const provider = coverage.provider;
					if (provider === "v8") {
						const path = tryResolve("@vitest/coverage-v8", [parentServer.config.root]);
						if (path) {
							entries.push(path);
							exclude.push("@vitest/coverage-v8/browser");
						}
					} else if (provider === "istanbul") {
						const path = tryResolve("@vitest/coverage-istanbul", [parentServer.config.root]);
						if (path) {
							entries.push(path);
							exclude.push("@vitest/coverage-istanbul");
						}
					} else if (provider === "custom" && coverage.customProviderModule) {
						entries.push(coverage.customProviderModule);
					}
				}
				const include = [
					"vitest > expect-type",
					"vitest > @vitest/snapshot > magic-string",
					"vitest > chai",
					"vitest > chai > loupe",
					"vitest > @vitest/runner > strip-literal",
					"vitest > @vitest/utils > loupe",
					"@vitest/browser > @testing-library/user-event",
					"@vitest/browser > @testing-library/dom"
				];
				const fileRoot = browserTestFiles[0] ? dirname(browserTestFiles[0]) : project.config.root;
				const svelte = isPackageExists("vitest-browser-svelte", fileRoot);
				if (svelte) {
					exclude.push("vitest-browser-svelte");
				}
				// since we override the resolution in the esbuild plugin, Vite can no longer optimizer it
				const vue = isPackageExists("vitest-browser-vue", fileRoot);
				if (vue) {
					// we override them in the esbuild plugin so optimizer can no longer intercept it
					include.push("vitest-browser-vue", "vitest-browser-vue > @vue/test-utils", "vitest-browser-vue > @vue/test-utils > @vue/compiler-core");
				}
				const vueTestUtils = isPackageExists("@vue/test-utils", fileRoot);
				if (vueTestUtils) {
					include.push("@vue/test-utils");
				}
				return {
					define,
					resolve: { dedupe: ["vitest"] },
					optimizeDeps: {
						entries,
						exclude,
						include
					}
				};
			},
			async resolveId(id) {
				if (!/\?browserv=\w+$/.test(id)) {
					return;
				}
				let useId = id.slice(0, id.lastIndexOf("?"));
				if (useId.startsWith("/@fs/")) {
					useId = useId.slice(5);
				}
				if (/^\w:/.test(useId)) {
					useId = useId.replace(/\\/g, "/");
				}
				return useId;
			}
		},
		{
			name: "vitest:browser:resolve-virtual",
			async resolveId(rawId) {
				if (rawId === "/mockServiceWorker.js") {
					return this.resolve("msw/mockServiceWorker.js", distRoot, { skipSelf: true });
				}
			}
		},
		{
			name: "vitest:browser:assets",
			configureServer(server) {
				server.middlewares.use("/__vitest__", sirv(resolve(distRoot, "client/__vitest__")));
			},
			resolveId(id) {
				if (id.startsWith("/__vitest_browser__/")) {
					return resolve(distRoot, "client", id.slice(1));
				}
			},
			transform(code, id) {
				if (id.includes(parentServer.vite.config.cacheDir) && id.includes("loupe.js")) {
					// loupe bundle has a nastry require('util') call that leaves a warning in the console
					const utilRequire = "nodeUtil = require_util();";
					return code.replace(utilRequire, " ".repeat(utilRequire.length));
				}
			}
		},
		BrowserContext(parentServer),
		dynamicImportPlugin({
			globalThisAccessor: "\"__vitest_browser_runner__\"",
			filter(id) {
				if (id.includes(distRoot)) {
					return false;
				}
				return true;
			}
		}),
		{
			name: "vitest:browser:config",
			enforce: "post",
			async config(viteConfig) {
				// Enables using ignore hint for coverage providers with @preserve keyword
				if (viteConfig.esbuild !== false) {
					viteConfig.esbuild ||= {};
					viteConfig.esbuild.legalComments = "inline";
				}
				const defaultPort = parentServer.vitest._browserLastPort++;
				const api = resolveApiServerConfig(viteConfig.test?.browser || {}, defaultPort);
				viteConfig.server = {
					...viteConfig.server,
					port: defaultPort,
					...api,
					middlewareMode: false,
					open: false
				};
				viteConfig.server.fs ??= {};
				viteConfig.server.fs.allow = viteConfig.server.fs.allow || [];
				viteConfig.server.fs.allow.push(...resolveFsAllow(parentServer.vitest.config.root, parentServer.vitest.vite.config.configFile), distRoot);
				return { resolve: { alias: viteConfig.test?.alias } };
			}
		},
		{
			name: "vitest:browser:in-source-tests",
			transform(code, id) {
				const filename = cleanUrl(id);
				const project = parentServer.vitest.getProjectByName(parentServer.config.name);
				if (!project._isCachedTestFile(filename) || !code.includes("import.meta.vitest")) {
					return;
				}
				const s = new MagicString(code, { filename });
				s.prepend(`import.meta.vitest = __vitest_index__;\n`);
				return {
					code: s.toString(),
					map: s.generateMap({ hires: true })
				};
			}
		},
		{
			name: "vitest:browser:worker",
			transform(code, id, _options) {
				// https://github.com/vitejs/vite/blob/ba56cf43b5480f8519349f7d7fe60718e9af5f1a/packages/vite/src/node/plugins/worker.ts#L46
				if (/(?:\?|&)worker_file&type=\w+(?:&|$)/.test(id)) {
					const s = new MagicString(code);
					s.prepend("globalThis.__vitest_browser_runner__ = { wrapDynamicImport: f => f() };\n");
					return {
						code: s.toString(),
						map: s.generateMap({ hires: "boundary" })
					};
				}
			}
		},
		{
			name: "vitest:browser:transform-tester-html",
			enforce: "pre",
			async transformIndexHtml(html, ctx) {
				const projectBrowser = [...parentServer.children].find((server) => {
					return ctx.filename === server.testerFilepath;
				});
				if (!projectBrowser) {
					return;
				}
				if (!parentServer.testerScripts) {
					const testerScripts = await parentServer.formatScripts(parentServer.config.browser.testerScripts);
					parentServer.testerScripts = testerScripts;
				}
				const stateJs = typeof parentServer.stateJs === "string" ? parentServer.stateJs : await parentServer.stateJs;
				const testerTags = [];
				const isDefaultTemplate = resolve(distRoot, "client/tester/tester.html") === projectBrowser.testerFilepath;
				if (!isDefaultTemplate) {
					const manifestContent = parentServer.manifest instanceof Promise ? await parentServer.manifest : parentServer.manifest;
					const testerEntry = manifestContent["tester/tester.html"];
					testerTags.push({
						tag: "script",
						attrs: {
							type: "module",
							crossorigin: "",
							src: `${parentServer.base}${testerEntry.file}`
						},
						injectTo: "head"
					});
					for (const importName of testerEntry.imports || []) {
						const entryManifest = manifestContent[importName];
						if (entryManifest) {
							testerTags.push({
								tag: "link",
								attrs: {
									href: `${parentServer.base}${entryManifest.file}`,
									rel: "modulepreload",
									crossorigin: ""
								},
								injectTo: "head"
							});
						}
					}
				} else {
					// inject the reset style only in the default template,
					// allowing users to customize the style in their own template
					testerTags.push({
						tag: "style",
						children: `
html {
  padding: 0;
  margin: 0;
}
body {
  padding: 0;
  margin: 0;
  min-height: 100vh;
}`,
						injectTo: "head"
					});
				}
				return [
					{
						tag: "script",
						children: "{__VITEST_INJECTOR__}",
						injectTo: "head-prepend"
					},
					{
						tag: "script",
						children: stateJs,
						injectTo: "head-prepend"
					},
					{
						tag: "script",
						attrs: {
							type: "module",
							src: parentServer.errorCatcherUrl
						},
						injectTo: "head"
					},
					{
						tag: "script",
						attrs: {
							type: "module",
							src: parentServer.matchersUrl
						},
						injectTo: "head"
					},
					parentServer.locatorsUrl ? {
						tag: "script",
						attrs: {
							type: "module",
							src: parentServer.locatorsUrl
						},
						injectTo: "head"
					} : null,
					...parentServer.testerScripts,
					...testerTags
				].filter((s) => s != null);
			}
		},
		{
			name: "vitest:browser:support-testing-library",
			config() {
				const rolldownPlugin = {
					name: "vue-test-utils-rewrite",
					resolveId: {
						filter: { id: /^@vue\/(test-utils|compiler-core)$/ },
						handler(source, importer) {
							const resolved = getRequire().resolve(source, { paths: [importer] });
							return resolved;
						}
					}
				};
				const esbuildPlugin = {
					name: "test-utils-rewrite",
					setup(build) {
						// test-utils: resolve to CJS instead of the browser because the browser version expects a global Vue object
						// compiler-core: only CJS version allows slots as strings
						build.onResolve({ filter: /^@vue\/(test-utils|compiler-core)$/ }, (args) => {
							const resolved = getRequire().resolve(args.path, { paths: [args.importer] });
							return { path: resolved };
						});
					}
				};
				return { optimizeDeps: "rolldownVersion" in vite ? { rollupOptions: { plugins: [rolldownPlugin] } } : { esbuildOptions: { plugins: [esbuildPlugin] } } };
			}
		}
	];
};
function tryResolve(path, paths) {
	try {
		const _require = getRequire();
		return _require.resolve(path, { paths });
	} catch {
		return undefined;
	}
}
let _require;
function getRequire() {
	if (!_require) {
		_require = createRequire(import.meta.url);
	}
	return _require;
}
function resolveCoverageFolder(vitest) {
	const options = vitest.config;
	const htmlReporter = options.coverage?.enabled ? toArray(options.coverage.reporter).find((reporter) => {
		if (typeof reporter === "string") {
			return reporter === "html";
		}
		return reporter[0] === "html";
	}) : undefined;
	if (!htmlReporter) {
		return undefined;
	}
	// reportsDirectory not resolved yet
	const root = resolve(options.root || process.cwd(), options.coverage.reportsDirectory || coverageConfigDefaults.reportsDirectory);
	const subdir = Array.isArray(htmlReporter) && htmlReporter.length > 1 && "subdir" in htmlReporter[1] ? htmlReporter[1].subdir : undefined;
	if (!subdir || typeof subdir !== "string") {
		return [root, `/${basename(root)}/`];
	}
	return [resolve(root, subdir), `/${basename(root)}/${subdir}/`];
}
const postfixRE = /[?#].*$/;
function cleanUrl(url) {
	return url.replace(postfixRE, "");
}

class BrowserServerCDPHandler {
	listenerIds = {};
	listeners = {};
	constructor(session, tester) {
		this.session = session;
		this.tester = tester;
	}
	send(method, params) {
		return this.session.send(method, params);
	}
	on(event, id, once = false) {
		if (!this.listenerIds[event]) {
			this.listenerIds[event] = [];
		}
		this.listenerIds[event].push(id);
		if (!this.listeners[event]) {
			this.listeners[event] = (payload) => {
				this.tester.cdpEvent(event, payload);
				if (once) {
					this.off(event, id);
				}
			};
			this.session.on(event, this.listeners[event]);
		}
	}
	off(event, id) {
		if (!this.listenerIds[event]) {
			this.listenerIds[event] = [];
		}
		this.listenerIds[event] = this.listenerIds[event].filter((l) => l !== id);
		if (!this.listenerIds[event].length) {
			this.session.off(event, this.listeners[event]);
			delete this.listeners[event];
		}
	}
	once(event, listener) {
		this.on(event, listener, true);
	}
}

const clear = async (context, selector) => {
	if (context.provider instanceof PlaywrightBrowserProvider) {
		const { iframe } = context;
		const element = iframe.locator(selector);
		await element.clear();
	} else if (context.provider instanceof WebdriverBrowserProvider) {
		const browser = context.browser;
		const element = await browser.$(selector);
		await element.clearValue();
	} else {
		throw new TypeError(`Provider "${context.provider.name}" does not support clearing elements`);
	}
};

const click = async (context, selector, options = {}) => {
	const provider = context.provider;
	if (provider instanceof PlaywrightBrowserProvider) {
		const tester = context.iframe;
		await tester.locator(selector).click(options);
	} else if (provider instanceof WebdriverBrowserProvider) {
		const browser = context.browser;
		await browser.$(selector).click(options);
	} else {
		throw new TypeError(`Provider "${provider.name}" doesn't support click command`);
	}
};
const dblClick = async (context, selector, options = {}) => {
	const provider = context.provider;
	if (provider instanceof PlaywrightBrowserProvider) {
		const tester = context.iframe;
		await tester.locator(selector).dblclick(options);
	} else if (provider instanceof WebdriverBrowserProvider) {
		const browser = context.browser;
		await browser.$(selector).doubleClick();
	} else {
		throw new TypeError(`Provider "${provider.name}" doesn't support dblClick command`);
	}
};
const tripleClick = async (context, selector, options = {}) => {
	const provider = context.provider;
	if (provider instanceof PlaywrightBrowserProvider) {
		const tester = context.iframe;
		await tester.locator(selector).click({
			...options,
			clickCount: 3
		});
	} else if (provider instanceof WebdriverBrowserProvider) {
		const browser = context.browser;
		await browser.action("pointer", { parameters: { pointerType: "mouse" } }).move({ origin: browser.$(selector) }).down().up().pause(50).down().up().pause(50).down().up().pause(50).perform();
	} else {
		throw new TypeError(`Provider "${provider.name}" doesn't support tripleClick command`);
	}
};

const dragAndDrop = async (context, source, target, options_) => {
	if (context.provider instanceof PlaywrightBrowserProvider) {
		const frame = await context.frame();
		await frame.dragAndDrop(source, target, options_);
	} else if (context.provider instanceof WebdriverBrowserProvider) {
		const $source = context.browser.$(source);
		const $target = context.browser.$(target);
		const options = options_ || {};
		const duration = options.duration ?? 10;
		// https://github.com/webdriverio/webdriverio/issues/8022#issuecomment-1700919670
		await context.browser.action("pointer").move({
			duration: 0,
			origin: $source,
			x: options.sourceX ?? 0,
			y: options.sourceY ?? 0
		}).down({ button: 0 }).move({
			duration: 0,
			origin: "pointer",
			x: 0,
			y: 0
		}).pause(duration).move({
			duration: 0,
			origin: $target,
			x: options.targetX ?? 0,
			y: options.targetY ?? 0
		}).move({
			duration: 0,
			origin: "pointer",
			x: 1,
			y: 0
		}).move({
			duration: 0,
			origin: "pointer",
			x: -1,
			y: 0
		}).up({ button: 0 }).perform();
	} else {
		throw new TypeError(`Provider "${context.provider.name}" does not support dragging elements`);
	}
};

const fill = async (context, selector, text, options = {}) => {
	if (context.provider instanceof PlaywrightBrowserProvider) {
		const { iframe } = context;
		const element = iframe.locator(selector);
		await element.fill(text, options);
	} else if (context.provider instanceof WebdriverBrowserProvider) {
		const browser = context.browser;
		await browser.$(selector).setValue(text);
	} else {
		throw new TypeError(`Provider "${context.provider.name}" does not support filling inputs`);
	}
};

const types = {
    'application/andrew-inset': ['ez'],
    'application/appinstaller': ['appinstaller'],
    'application/applixware': ['aw'],
    'application/appx': ['appx'],
    'application/appxbundle': ['appxbundle'],
    'application/atom+xml': ['atom'],
    'application/atomcat+xml': ['atomcat'],
    'application/atomdeleted+xml': ['atomdeleted'],
    'application/atomsvc+xml': ['atomsvc'],
    'application/atsc-dwd+xml': ['dwd'],
    'application/atsc-held+xml': ['held'],
    'application/atsc-rsat+xml': ['rsat'],
    'application/automationml-aml+xml': ['aml'],
    'application/automationml-amlx+zip': ['amlx'],
    'application/bdoc': ['bdoc'],
    'application/calendar+xml': ['xcs'],
    'application/ccxml+xml': ['ccxml'],
    'application/cdfx+xml': ['cdfx'],
    'application/cdmi-capability': ['cdmia'],
    'application/cdmi-container': ['cdmic'],
    'application/cdmi-domain': ['cdmid'],
    'application/cdmi-object': ['cdmio'],
    'application/cdmi-queue': ['cdmiq'],
    'application/cpl+xml': ['cpl'],
    'application/cu-seeme': ['cu'],
    'application/cwl': ['cwl'],
    'application/dash+xml': ['mpd'],
    'application/dash-patch+xml': ['mpp'],
    'application/davmount+xml': ['davmount'],
    'application/dicom': ['dcm'],
    'application/docbook+xml': ['dbk'],
    'application/dssc+der': ['dssc'],
    'application/dssc+xml': ['xdssc'],
    'application/ecmascript': ['ecma'],
    'application/emma+xml': ['emma'],
    'application/emotionml+xml': ['emotionml'],
    'application/epub+zip': ['epub'],
    'application/exi': ['exi'],
    'application/express': ['exp'],
    'application/fdf': ['fdf'],
    'application/fdt+xml': ['fdt'],
    'application/font-tdpfr': ['pfr'],
    'application/geo+json': ['geojson'],
    'application/gml+xml': ['gml'],
    'application/gpx+xml': ['gpx'],
    'application/gxf': ['gxf'],
    'application/gzip': ['gz'],
    'application/hjson': ['hjson'],
    'application/hyperstudio': ['stk'],
    'application/inkml+xml': ['ink', 'inkml'],
    'application/ipfix': ['ipfix'],
    'application/its+xml': ['its'],
    'application/java-archive': ['jar', 'war', 'ear'],
    'application/java-serialized-object': ['ser'],
    'application/java-vm': ['class'],
    'application/javascript': ['*js'],
    'application/json': ['json', 'map'],
    'application/json5': ['json5'],
    'application/jsonml+json': ['jsonml'],
    'application/ld+json': ['jsonld'],
    'application/lgr+xml': ['lgr'],
    'application/lost+xml': ['lostxml'],
    'application/mac-binhex40': ['hqx'],
    'application/mac-compactpro': ['cpt'],
    'application/mads+xml': ['mads'],
    'application/manifest+json': ['webmanifest'],
    'application/marc': ['mrc'],
    'application/marcxml+xml': ['mrcx'],
    'application/mathematica': ['ma', 'nb', 'mb'],
    'application/mathml+xml': ['mathml'],
    'application/mbox': ['mbox'],
    'application/media-policy-dataset+xml': ['mpf'],
    'application/mediaservercontrol+xml': ['mscml'],
    'application/metalink+xml': ['metalink'],
    'application/metalink4+xml': ['meta4'],
    'application/mets+xml': ['mets'],
    'application/mmt-aei+xml': ['maei'],
    'application/mmt-usd+xml': ['musd'],
    'application/mods+xml': ['mods'],
    'application/mp21': ['m21', 'mp21'],
    'application/mp4': ['*mp4', '*mpg4', 'mp4s', 'm4p'],
    'application/msix': ['msix'],
    'application/msixbundle': ['msixbundle'],
    'application/msword': ['doc', 'dot'],
    'application/mxf': ['mxf'],
    'application/n-quads': ['nq'],
    'application/n-triples': ['nt'],
    'application/node': ['cjs'],
    'application/octet-stream': [
        'bin',
        'dms',
        'lrf',
        'mar',
        'so',
        'dist',
        'distz',
        'pkg',
        'bpk',
        'dump',
        'elc',
        'deploy',
        'exe',
        'dll',
        'deb',
        'dmg',
        'iso',
        'img',
        'msi',
        'msp',
        'msm',
        'buffer',
    ],
    'application/oda': ['oda'],
    'application/oebps-package+xml': ['opf'],
    'application/ogg': ['ogx'],
    'application/omdoc+xml': ['omdoc'],
    'application/onenote': [
        'onetoc',
        'onetoc2',
        'onetmp',
        'onepkg',
        'one',
        'onea',
    ],
    'application/oxps': ['oxps'],
    'application/p2p-overlay+xml': ['relo'],
    'application/patch-ops-error+xml': ['xer'],
    'application/pdf': ['pdf'],
    'application/pgp-encrypted': ['pgp'],
    'application/pgp-keys': ['asc'],
    'application/pgp-signature': ['sig', '*asc'],
    'application/pics-rules': ['prf'],
    'application/pkcs10': ['p10'],
    'application/pkcs7-mime': ['p7m', 'p7c'],
    'application/pkcs7-signature': ['p7s'],
    'application/pkcs8': ['p8'],
    'application/pkix-attr-cert': ['ac'],
    'application/pkix-cert': ['cer'],
    'application/pkix-crl': ['crl'],
    'application/pkix-pkipath': ['pkipath'],
    'application/pkixcmp': ['pki'],
    'application/pls+xml': ['pls'],
    'application/postscript': ['ai', 'eps', 'ps'],
    'application/provenance+xml': ['provx'],
    'application/pskc+xml': ['pskcxml'],
    'application/raml+yaml': ['raml'],
    'application/rdf+xml': ['rdf', 'owl'],
    'application/reginfo+xml': ['rif'],
    'application/relax-ng-compact-syntax': ['rnc'],
    'application/resource-lists+xml': ['rl'],
    'application/resource-lists-diff+xml': ['rld'],
    'application/rls-services+xml': ['rs'],
    'application/route-apd+xml': ['rapd'],
    'application/route-s-tsid+xml': ['sls'],
    'application/route-usd+xml': ['rusd'],
    'application/rpki-ghostbusters': ['gbr'],
    'application/rpki-manifest': ['mft'],
    'application/rpki-roa': ['roa'],
    'application/rsd+xml': ['rsd'],
    'application/rss+xml': ['rss'],
    'application/rtf': ['rtf'],
    'application/sbml+xml': ['sbml'],
    'application/scvp-cv-request': ['scq'],
    'application/scvp-cv-response': ['scs'],
    'application/scvp-vp-request': ['spq'],
    'application/scvp-vp-response': ['spp'],
    'application/sdp': ['sdp'],
    'application/senml+xml': ['senmlx'],
    'application/sensml+xml': ['sensmlx'],
    'application/set-payment-initiation': ['setpay'],
    'application/set-registration-initiation': ['setreg'],
    'application/shf+xml': ['shf'],
    'application/sieve': ['siv', 'sieve'],
    'application/smil+xml': ['smi', 'smil'],
    'application/sparql-query': ['rq'],
    'application/sparql-results+xml': ['srx'],
    'application/sql': ['sql'],
    'application/srgs': ['gram'],
    'application/srgs+xml': ['grxml'],
    'application/sru+xml': ['sru'],
    'application/ssdl+xml': ['ssdl'],
    'application/ssml+xml': ['ssml'],
    'application/swid+xml': ['swidtag'],
    'application/tei+xml': ['tei', 'teicorpus'],
    'application/thraud+xml': ['tfi'],
    'application/timestamped-data': ['tsd'],
    'application/toml': ['toml'],
    'application/trig': ['trig'],
    'application/ttml+xml': ['ttml'],
    'application/ubjson': ['ubj'],
    'application/urc-ressheet+xml': ['rsheet'],
    'application/urc-targetdesc+xml': ['td'],
    'application/voicexml+xml': ['vxml'],
    'application/wasm': ['wasm'],
    'application/watcherinfo+xml': ['wif'],
    'application/widget': ['wgt'],
    'application/winhlp': ['hlp'],
    'application/wsdl+xml': ['wsdl'],
    'application/wspolicy+xml': ['wspolicy'],
    'application/xaml+xml': ['xaml'],
    'application/xcap-att+xml': ['xav'],
    'application/xcap-caps+xml': ['xca'],
    'application/xcap-diff+xml': ['xdf'],
    'application/xcap-el+xml': ['xel'],
    'application/xcap-ns+xml': ['xns'],
    'application/xenc+xml': ['xenc'],
    'application/xfdf': ['xfdf'],
    'application/xhtml+xml': ['xhtml', 'xht'],
    'application/xliff+xml': ['xlf'],
    'application/xml': ['xml', 'xsl', 'xsd', 'rng'],
    'application/xml-dtd': ['dtd'],
    'application/xop+xml': ['xop'],
    'application/xproc+xml': ['xpl'],
    'application/xslt+xml': ['*xsl', 'xslt'],
    'application/xspf+xml': ['xspf'],
    'application/xv+xml': ['mxml', 'xhvml', 'xvml', 'xvm'],
    'application/yang': ['yang'],
    'application/yin+xml': ['yin'],
    'application/zip': ['zip'],
    'application/zip+dotlottie': ['lottie'],
    'audio/3gpp': ['*3gpp'],
    'audio/aac': ['adts', 'aac'],
    'audio/adpcm': ['adp'],
    'audio/amr': ['amr'],
    'audio/basic': ['au', 'snd'],
    'audio/midi': ['mid', 'midi', 'kar', 'rmi'],
    'audio/mobile-xmf': ['mxmf'],
    'audio/mp3': ['*mp3'],
    'audio/mp4': ['m4a', 'mp4a', 'm4b'],
    'audio/mpeg': ['mpga', 'mp2', 'mp2a', 'mp3', 'm2a', 'm3a'],
    'audio/ogg': ['oga', 'ogg', 'spx', 'opus'],
    'audio/s3m': ['s3m'],
    'audio/silk': ['sil'],
    'audio/wav': ['wav'],
    'audio/wave': ['*wav'],
    'audio/webm': ['weba'],
    'audio/xm': ['xm'],
    'font/collection': ['ttc'],
    'font/otf': ['otf'],
    'font/ttf': ['ttf'],
    'font/woff': ['woff'],
    'font/woff2': ['woff2'],
    'image/aces': ['exr'],
    'image/apng': ['apng'],
    'image/avci': ['avci'],
    'image/avcs': ['avcs'],
    'image/avif': ['avif'],
    'image/bmp': ['bmp', 'dib'],
    'image/cgm': ['cgm'],
    'image/dicom-rle': ['drle'],
    'image/dpx': ['dpx'],
    'image/emf': ['emf'],
    'image/fits': ['fits'],
    'image/g3fax': ['g3'],
    'image/gif': ['gif'],
    'image/heic': ['heic'],
    'image/heic-sequence': ['heics'],
    'image/heif': ['heif'],
    'image/heif-sequence': ['heifs'],
    'image/hej2k': ['hej2'],
    'image/ief': ['ief'],
    'image/jaii': ['jaii'],
    'image/jais': ['jais'],
    'image/jls': ['jls'],
    'image/jp2': ['jp2', 'jpg2'],
    'image/jpeg': ['jpg', 'jpeg', 'jpe'],
    'image/jph': ['jph'],
    'image/jphc': ['jhc'],
    'image/jpm': ['jpm', 'jpgm'],
    'image/jpx': ['jpx', 'jpf'],
    'image/jxl': ['jxl'],
    'image/jxr': ['jxr'],
    'image/jxra': ['jxra'],
    'image/jxrs': ['jxrs'],
    'image/jxs': ['jxs'],
    'image/jxsc': ['jxsc'],
    'image/jxsi': ['jxsi'],
    'image/jxss': ['jxss'],
    'image/ktx': ['ktx'],
    'image/ktx2': ['ktx2'],
    'image/pjpeg': ['jfif'],
    'image/png': ['png'],
    'image/sgi': ['sgi'],
    'image/svg+xml': ['svg', 'svgz'],
    'image/t38': ['t38'],
    'image/tiff': ['tif', 'tiff'],
    'image/tiff-fx': ['tfx'],
    'image/webp': ['webp'],
    'image/wmf': ['wmf'],
    'message/disposition-notification': ['disposition-notification'],
    'message/global': ['u8msg'],
    'message/global-delivery-status': ['u8dsn'],
    'message/global-disposition-notification': ['u8mdn'],
    'message/global-headers': ['u8hdr'],
    'message/rfc822': ['eml', 'mime', 'mht', 'mhtml'],
    'model/3mf': ['3mf'],
    'model/gltf+json': ['gltf'],
    'model/gltf-binary': ['glb'],
    'model/iges': ['igs', 'iges'],
    'model/jt': ['jt'],
    'model/mesh': ['msh', 'mesh', 'silo'],
    'model/mtl': ['mtl'],
    'model/obj': ['obj'],
    'model/prc': ['prc'],
    'model/step': ['step', 'stp', 'stpnc', 'p21', '210'],
    'model/step+xml': ['stpx'],
    'model/step+zip': ['stpz'],
    'model/step-xml+zip': ['stpxz'],
    'model/stl': ['stl'],
    'model/u3d': ['u3d'],
    'model/vrml': ['wrl', 'vrml'],
    'model/x3d+binary': ['*x3db', 'x3dbz'],
    'model/x3d+fastinfoset': ['x3db'],
    'model/x3d+vrml': ['*x3dv', 'x3dvz'],
    'model/x3d+xml': ['x3d', 'x3dz'],
    'model/x3d-vrml': ['x3dv'],
    'text/cache-manifest': ['appcache', 'manifest'],
    'text/calendar': ['ics', 'ifb'],
    'text/coffeescript': ['coffee', 'litcoffee'],
    'text/css': ['css'],
    'text/csv': ['csv'],
    'text/html': ['html', 'htm', 'shtml'],
    'text/jade': ['jade'],
    'text/javascript': ['js', 'mjs'],
    'text/jsx': ['jsx'],
    'text/less': ['less'],
    'text/markdown': ['md', 'markdown'],
    'text/mathml': ['mml'],
    'text/mdx': ['mdx'],
    'text/n3': ['n3'],
    'text/plain': ['txt', 'text', 'conf', 'def', 'list', 'log', 'in', 'ini'],
    'text/richtext': ['rtx'],
    'text/rtf': ['*rtf'],
    'text/sgml': ['sgml', 'sgm'],
    'text/shex': ['shex'],
    'text/slim': ['slim', 'slm'],
    'text/spdx': ['spdx'],
    'text/stylus': ['stylus', 'styl'],
    'text/tab-separated-values': ['tsv'],
    'text/troff': ['t', 'tr', 'roff', 'man', 'me', 'ms'],
    'text/turtle': ['ttl'],
    'text/uri-list': ['uri', 'uris', 'urls'],
    'text/vcard': ['vcard'],
    'text/vtt': ['vtt'],
    'text/wgsl': ['wgsl'],
    'text/xml': ['*xml'],
    'text/yaml': ['yaml', 'yml'],
    'video/3gpp': ['3gp', '3gpp'],
    'video/3gpp2': ['3g2'],
    'video/h261': ['h261'],
    'video/h263': ['h263'],
    'video/h264': ['h264'],
    'video/iso.segment': ['m4s'],
    'video/jpeg': ['jpgv'],
    'video/jpm': ['*jpm', '*jpgm'],
    'video/mj2': ['mj2', 'mjp2'],
    'video/mp2t': ['ts', 'm2t', 'm2ts', 'mts'],
    'video/mp4': ['mp4', 'mp4v', 'mpg4'],
    'video/mpeg': ['mpeg', 'mpg', 'mpe', 'm1v', 'm2v'],
    'video/ogg': ['ogv'],
    'video/quicktime': ['qt', 'mov'],
    'video/webm': ['webm'],
};
Object.freeze(types);

var __classPrivateFieldGet = (null && null.__classPrivateFieldGet) || function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _Mime_extensionToType, _Mime_typeToExtension, _Mime_typeToExtensions;
class Mime {
    constructor(...args) {
        _Mime_extensionToType.set(this, new Map());
        _Mime_typeToExtension.set(this, new Map());
        _Mime_typeToExtensions.set(this, new Map());
        for (const arg of args) {
            this.define(arg);
        }
    }
    define(typeMap, force = false) {
        for (let [type, extensions] of Object.entries(typeMap)) {
            type = type.toLowerCase();
            extensions = extensions.map((ext) => ext.toLowerCase());
            if (!__classPrivateFieldGet(this, _Mime_typeToExtensions, "f").has(type)) {
                __classPrivateFieldGet(this, _Mime_typeToExtensions, "f").set(type, new Set());
            }
            const allExtensions = __classPrivateFieldGet(this, _Mime_typeToExtensions, "f").get(type);
            let first = true;
            for (let extension of extensions) {
                const starred = extension.startsWith('*');
                extension = starred ? extension.slice(1) : extension;
                allExtensions?.add(extension);
                if (first) {
                    __classPrivateFieldGet(this, _Mime_typeToExtension, "f").set(type, extension);
                }
                first = false;
                if (starred)
                    continue;
                const currentType = __classPrivateFieldGet(this, _Mime_extensionToType, "f").get(extension);
                if (currentType && currentType != type && !force) {
                    throw new Error(`"${type} -> ${extension}" conflicts with "${currentType} -> ${extension}". Pass \`force=true\` to override this definition.`);
                }
                __classPrivateFieldGet(this, _Mime_extensionToType, "f").set(extension, type);
            }
        }
        return this;
    }
    getType(path) {
        if (typeof path !== 'string')
            return null;
        const last = path.replace(/^.*[/\\]/s, '').toLowerCase();
        const ext = last.replace(/^.*\./s, '').toLowerCase();
        const hasPath = last.length < path.length;
        const hasDot = ext.length < last.length - 1;
        if (!hasDot && hasPath)
            return null;
        return __classPrivateFieldGet(this, _Mime_extensionToType, "f").get(ext) ?? null;
    }
    getExtension(type) {
        if (typeof type !== 'string')
            return null;
        type = type?.split?.(';')[0];
        return ((type && __classPrivateFieldGet(this, _Mime_typeToExtension, "f").get(type.trim().toLowerCase())) ?? null);
    }
    getAllExtensions(type) {
        if (typeof type !== 'string')
            return null;
        return __classPrivateFieldGet(this, _Mime_typeToExtensions, "f").get(type.toLowerCase()) ?? null;
    }
    _freeze() {
        this.define = () => {
            throw new Error('define() not allowed for built-in Mime objects. See https://github.com/broofa/mime/blob/main/README.md#custom-mime-instances');
        };
        Object.freeze(this);
        for (const extensions of __classPrivateFieldGet(this, _Mime_typeToExtensions, "f").values()) {
            Object.freeze(extensions);
        }
        return this;
    }
    _getTestState() {
        return {
            types: __classPrivateFieldGet(this, _Mime_extensionToType, "f"),
            extensions: __classPrivateFieldGet(this, _Mime_typeToExtension, "f"),
        };
    }
}
_Mime_extensionToType = new WeakMap(), _Mime_typeToExtension = new WeakMap(), _Mime_typeToExtensions = new WeakMap();

var mime = new Mime(types)._freeze();

function assertFileAccess(path, project) {
	if (!isFileServingAllowed(path, project.vite) && !isFileServingAllowed(path, project.vitest.server)) {
		throw new Error(`Access denied to "${path}". See Vite config documentation for "server.fs": https://vitejs.dev/config/server-options.html#server-fs-strict.`);
	}
}
const readFile = async ({ project }, path, options = {}) => {
	const filepath = resolve$1(project.config.root, path);
	assertFileAccess(filepath, project);
	// never return a Buffer
	if (typeof options === "object" && !options.encoding) {
		options.encoding = "utf-8";
	}
	return promises.readFile(filepath, options);
};
const writeFile = async ({ project }, path, data, options) => {
	const filepath = resolve$1(project.config.root, path);
	assertFileAccess(filepath, project);
	const dir = dirname$1(filepath);
	if (!fs.existsSync(dir)) {
		await promises.mkdir(dir, { recursive: true });
	}
	await promises.writeFile(filepath, data, options);
};
const removeFile = async ({ project }, path) => {
	const filepath = resolve$1(project.config.root, path);
	assertFileAccess(filepath, project);
	await promises.rm(filepath);
};
const _fileInfo = async ({ project }, path, encoding) => {
	const filepath = resolve$1(project.config.root, path);
	assertFileAccess(filepath, project);
	const content = await promises.readFile(filepath, encoding || "base64");
	return {
		content,
		basename: basename$1(filepath),
		mime: mime.getType(filepath)
	};
};

const hover = async (context, selector, options = {}) => {
	if (context.provider instanceof PlaywrightBrowserProvider) {
		await context.iframe.locator(selector).hover(options);
	} else if (context.provider instanceof WebdriverBrowserProvider) {
		const browser = context.browser;
		await browser.$(selector).moveTo(options);
	} else {
		throw new TypeError(`Provider "${context.provider.name}" does not support hover`);
	}
};

var DOM_KEY_LOCATION = /*#__PURE__*/ function(DOM_KEY_LOCATION) {
    DOM_KEY_LOCATION[DOM_KEY_LOCATION["STANDARD"] = 0] = "STANDARD";
    DOM_KEY_LOCATION[DOM_KEY_LOCATION["LEFT"] = 1] = "LEFT";
    DOM_KEY_LOCATION[DOM_KEY_LOCATION["RIGHT"] = 2] = "RIGHT";
    DOM_KEY_LOCATION[DOM_KEY_LOCATION["NUMPAD"] = 3] = "NUMPAD";
    return DOM_KEY_LOCATION;
}({});

/**
 * Mapping for a default US-104-QWERTY keyboard
 */ const defaultKeyMap = [
    // alphanumeric block - writing system
    ...'0123456789'.split('').map((c)=>({
            code: `Digit${c}`,
            key: c
        })),
    ...')!@#$%^&*('.split('').map((c, i)=>({
            code: `Digit${i}`,
            key: c,
            shiftKey: true
        })),
    ...'abcdefghijklmnopqrstuvwxyz'.split('').map((c)=>({
            code: `Key${c.toUpperCase()}`,
            key: c
        })),
    ...'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').map((c)=>({
            code: `Key${c}`,
            key: c,
            shiftKey: true
        })),
    {
        code: 'BracketLeft',
        key: '['
    },
    {
        code: 'BracketLeft',
        key: '{',
        shiftKey: true
    },
    {
        code: 'BracketRight',
        key: ']'
    },
    {
        code: 'BracketRight',
        key: '}',
        shiftKey: true
    },
    // alphanumeric block - functional
    {
        code: 'Space',
        key: ' '
    },
    {
        code: 'AltLeft',
        key: 'Alt',
        location: DOM_KEY_LOCATION.LEFT
    },
    {
        code: 'AltRight',
        key: 'Alt',
        location: DOM_KEY_LOCATION.RIGHT
    },
    {
        code: 'ShiftLeft',
        key: 'Shift',
        location: DOM_KEY_LOCATION.LEFT
    },
    {
        code: 'ShiftRight',
        key: 'Shift',
        location: DOM_KEY_LOCATION.RIGHT
    },
    {
        code: 'ControlLeft',
        key: 'Control',
        location: DOM_KEY_LOCATION.LEFT
    },
    {
        code: 'ControlRight',
        key: 'Control',
        location: DOM_KEY_LOCATION.RIGHT
    },
    {
        code: 'MetaLeft',
        key: 'Meta',
        location: DOM_KEY_LOCATION.LEFT
    },
    {
        code: 'MetaRight',
        key: 'Meta',
        location: DOM_KEY_LOCATION.RIGHT
    },
    {
        code: 'OSLeft',
        key: 'OS',
        location: DOM_KEY_LOCATION.LEFT
    },
    {
        code: 'OSRight',
        key: 'OS',
        location: DOM_KEY_LOCATION.RIGHT
    },
    {
        code: 'ContextMenu',
        key: 'ContextMenu'
    },
    {
        code: 'Tab',
        key: 'Tab'
    },
    {
        code: 'CapsLock',
        key: 'CapsLock'
    },
    {
        code: 'Backspace',
        key: 'Backspace'
    },
    {
        code: 'Enter',
        key: 'Enter'
    },
    // function
    {
        code: 'Escape',
        key: 'Escape'
    },
    // arrows
    {
        code: 'ArrowUp',
        key: 'ArrowUp'
    },
    {
        code: 'ArrowDown',
        key: 'ArrowDown'
    },
    {
        code: 'ArrowLeft',
        key: 'ArrowLeft'
    },
    {
        code: 'ArrowRight',
        key: 'ArrowRight'
    },
    // control pad
    {
        code: 'Home',
        key: 'Home'
    },
    {
        code: 'End',
        key: 'End'
    },
    {
        code: 'Delete',
        key: 'Delete'
    },
    {
        code: 'PageUp',
        key: 'PageUp'
    },
    {
        code: 'PageDown',
        key: 'PageDown'
    },
    // Special keys that are not part of a default US-layout but included for specific behavior
    {
        code: 'Fn',
        key: 'Fn'
    },
    {
        code: 'Symbol',
        key: 'Symbol'
    },
    {
        code: 'AltRight',
        key: 'AltGraph'
    }
];

var bracketDict = /*#__PURE__*/ function(bracketDict) {
    bracketDict["{"] = "}";
    bracketDict["["] = "]";
    return bracketDict;
}(bracketDict || {});
/**
 * Read the next key definition from user input
 *
 * Describe key per `{descriptor}` or `[descriptor]`.
 * Everything else will be interpreted as a single character as descriptor - e.g. `a`.
 * Brackets `{` and `[` can be escaped by doubling - e.g. `foo[[bar` translates to `foo[bar`.
 * A previously pressed key can be released per `{/descriptor}`.
 * Keeping the key pressed can be written as `{descriptor>}`.
 * When keeping the key pressed you can choose how long the key is pressed `{descriptor>3}`.
 * You can then release the key per `{descriptor>3/}` or keep it pressed and continue with the next key.
 */ function readNextDescriptor(text, context) {
    let pos = 0;
    const startBracket = text[pos] in bracketDict ? text[pos] : '';
    pos += startBracket.length;
    const isEscapedChar = new RegExp(`^\\${startBracket}{2}`).test(text);
    const type = isEscapedChar ? '' : startBracket;
    return {
        type,
        ...type === '' ? readPrintableChar(text, pos) : readTag(text, pos, type)
    };
}
function readPrintableChar(text, pos, context) {
    const descriptor = text[pos];
    assertDescriptor(descriptor, text, pos);
    pos += descriptor.length;
    return {
        consumedLength: pos,
        descriptor,
        releasePrevious: false,
        releaseSelf: true,
        repeat: 1
    };
}
function readTag(text, pos, startBracket, context) {
    var _text_slice_match, _text_slice_match1;
    const releasePreviousModifier = text[pos] === '/' ? '/' : '';
    pos += releasePreviousModifier.length;
    const escapedDescriptor = startBracket === '{' && text[pos] === '\\';
    pos += Number(escapedDescriptor);
    const descriptor = escapedDescriptor ? text[pos] : (_text_slice_match = text.slice(pos).match(startBracket === '{' ? /^\w+|^[^}>/]/ : /^\w+/)) === null || _text_slice_match === undefined ? undefined : _text_slice_match[0];
    assertDescriptor(descriptor, text, pos);
    pos += descriptor.length;
    var _text_slice_match_;
    const repeatModifier = (_text_slice_match_ = (_text_slice_match1 = text.slice(pos).match(/^>\d+/)) === null || _text_slice_match1 === undefined ? undefined : _text_slice_match1[0]) !== null && _text_slice_match_ !== undefined ? _text_slice_match_ : '';
    pos += repeatModifier.length;
    const releaseSelfModifier = text[pos] === '/' || !repeatModifier && text[pos] === '>' ? text[pos] : '';
    pos += releaseSelfModifier.length;
    const expectedEndBracket = bracketDict[startBracket];
    const endBracket = text[pos] === expectedEndBracket ? expectedEndBracket : '';
    if (!endBracket) {
        throw new Error(getErrorMessage([
            !repeatModifier && 'repeat modifier',
            !releaseSelfModifier && 'release modifier',
            `"${expectedEndBracket}"`
        ].filter(Boolean).join(' or '), text[pos], text));
    }
    pos += endBracket.length;
    return {
        consumedLength: pos,
        descriptor,
        releasePrevious: !!releasePreviousModifier,
        repeat: repeatModifier ? Math.max(Number(repeatModifier.substr(1)), 1) : 1,
        releaseSelf: hasReleaseSelf(releaseSelfModifier, repeatModifier)
    };
}
function assertDescriptor(descriptor, text, pos, context) {
    if (!descriptor) {
        throw new Error(getErrorMessage('key descriptor', text[pos], text));
    }
}
function hasReleaseSelf(releaseSelfModifier, repeatModifier) {
    if (releaseSelfModifier) {
        return releaseSelfModifier === '/';
    }
    if (repeatModifier) {
        return false;
    }
}
function getErrorMessage(expected, found, text, context) {
    return `Expected ${expected} but found "${found !== null && found !== undefined ? found : ''}" in "${text}"
    See ${`https://testing-library.com/docs/user-event/keyboard`}
    for more information about how userEvent parses your input.`;
}

/**
 * Parse key definitions per `keyboardMap`
 *
 * Keys can be referenced by `{key}` or `{special}` as well as physical locations per `[code]`.
 * Everything else will be interpreted as a typed character - e.g. `a`.
 * Brackets `{` and `[` can be escaped by doubling - e.g. `foo[[bar` translates to `foo[bar`.
 * Keeping the key pressed can be written as `{key>}`.
 * When keeping the key pressed you can choose how long (how many keydown and keypress) the key is pressed `{key>3}`.
 * You can then release the key per `{key>3/}` or keep it pressed and continue with the next key.
 */ function parseKeyDef(keyboardMap, text) {
    const defs = [];
    do {
        const { type, descriptor, consumedLength, releasePrevious, releaseSelf = true, repeat } = readNextDescriptor(text);
        var _keyboardMap_find;
        const keyDef = (_keyboardMap_find = keyboardMap.find((def)=>{
            if (type === '[') {
                var _def_code;
                return ((_def_code = def.code) === null || _def_code === undefined ? undefined : _def_code.toLowerCase()) === descriptor.toLowerCase();
            } else if (type === '{') {
                var _def_key;
                return ((_def_key = def.key) === null || _def_key === undefined ? undefined : _def_key.toLowerCase()) === descriptor.toLowerCase();
            }
            return def.key === descriptor;
        })) !== null && _keyboardMap_find !== undefined ? _keyboardMap_find : {
            key: 'Unknown',
            code: 'Unknown',
            [type === '[' ? 'code' : 'key']: descriptor
        };
        defs.push({
            keyDef,
            releasePrevious,
            releaseSelf,
            repeat
        });
        text = text.slice(consumedLength);
    }while (text)
    return defs;
}

const keyboard = async (context, text, state) => {
	if (context.provider instanceof PlaywrightBrowserProvider) {
		const frame = await context.frame();
		await frame.evaluate(focusIframe);
	} else if (context.provider instanceof WebdriverBrowserProvider) {
		await context.browser.execute(focusIframe);
	}
	const pressed = new Set(state.unreleased);
	await keyboardImplementation(pressed, context.provider, context.sessionId, text, async () => {
		if (context.provider instanceof PlaywrightBrowserProvider) {
			const frame = await context.frame();
			await frame.evaluate(selectAll);
		} else if (context.provider instanceof WebdriverBrowserProvider) {
			await context.browser.execute(selectAll);
		} else {
			throw new TypeError(`Provider "${context.provider.name}" does not support selecting all text`);
		}
	}, true);
	return { unreleased: Array.from(pressed) };
};
const keyboardCleanup = async (context, state) => {
	const { provider, sessionId } = context;
	if (!state.unreleased) {
		return;
	}
	if (provider instanceof PlaywrightBrowserProvider) {
		const page = provider.getPage(sessionId);
		for (const key of state.unreleased) {
			await page.keyboard.up(key);
		}
	} else if (provider instanceof WebdriverBrowserProvider) {
		const keyboard = provider.browser.action("key");
		for (const key of state.unreleased) {
			keyboard.up(key);
		}
		await keyboard.perform();
	} else {
		throw new TypeError(`Provider "${context.provider.name}" does not support keyboard api`);
	}
};
// fallback to insertText for non US key
// https://github.com/microsoft/playwright/blob/50775698ae13642742f2a1e8983d1d686d7f192d/packages/playwright-core/src/server/input.ts#L95
const VALID_KEYS = new Set([
	"Escape",
	"F1",
	"F2",
	"F3",
	"F4",
	"F5",
	"F6",
	"F7",
	"F8",
	"F9",
	"F10",
	"F11",
	"F12",
	"Backquote",
	"`",
	"~",
	"Digit1",
	"1",
	"!",
	"Digit2",
	"2",
	"@",
	"Digit3",
	"3",
	"#",
	"Digit4",
	"4",
	"$",
	"Digit5",
	"5",
	"%",
	"Digit6",
	"6",
	"^",
	"Digit7",
	"7",
	"&",
	"Digit8",
	"8",
	"*",
	"Digit9",
	"9",
	"(",
	"Digit0",
	"0",
	")",
	"Minus",
	"-",
	"_",
	"Equal",
	"=",
	"+",
	"Backslash",
	"\\",
	"|",
	"Backspace",
	"Tab",
	"KeyQ",
	"q",
	"Q",
	"KeyW",
	"w",
	"W",
	"KeyE",
	"e",
	"E",
	"KeyR",
	"r",
	"R",
	"KeyT",
	"t",
	"T",
	"KeyY",
	"y",
	"Y",
	"KeyU",
	"u",
	"U",
	"KeyI",
	"i",
	"I",
	"KeyO",
	"o",
	"O",
	"KeyP",
	"p",
	"P",
	"BracketLeft",
	"[",
	"{",
	"BracketRight",
	"]",
	"}",
	"CapsLock",
	"KeyA",
	"a",
	"A",
	"KeyS",
	"s",
	"S",
	"KeyD",
	"d",
	"D",
	"KeyF",
	"f",
	"F",
	"KeyG",
	"g",
	"G",
	"KeyH",
	"h",
	"H",
	"KeyJ",
	"j",
	"J",
	"KeyK",
	"k",
	"K",
	"KeyL",
	"l",
	"L",
	"Semicolon",
	";",
	":",
	"Quote",
	"'",
	"\"",
	"Enter",
	"\n",
	"\r",
	"ShiftLeft",
	"Shift",
	"KeyZ",
	"z",
	"Z",
	"KeyX",
	"x",
	"X",
	"KeyC",
	"c",
	"C",
	"KeyV",
	"v",
	"V",
	"KeyB",
	"b",
	"B",
	"KeyN",
	"n",
	"N",
	"KeyM",
	"m",
	"M",
	"Comma",
	",",
	"<",
	"Period",
	".",
	">",
	"Slash",
	"/",
	"?",
	"ShiftRight",
	"ControlLeft",
	"Control",
	"MetaLeft",
	"Meta",
	"AltLeft",
	"Alt",
	"Space",
	" ",
	"AltRight",
	"AltGraph",
	"MetaRight",
	"ContextMenu",
	"ControlRight",
	"PrintScreen",
	"ScrollLock",
	"Pause",
	"PageUp",
	"PageDown",
	"Insert",
	"Delete",
	"Home",
	"End",
	"ArrowLeft",
	"ArrowUp",
	"ArrowRight",
	"ArrowDown",
	"NumLock",
	"NumpadDivide",
	"NumpadMultiply",
	"NumpadSubtract",
	"Numpad7",
	"Numpad8",
	"Numpad9",
	"Numpad4",
	"Numpad5",
	"Numpad6",
	"NumpadAdd",
	"Numpad1",
	"Numpad2",
	"Numpad3",
	"Numpad0",
	"NumpadDecimal",
	"NumpadEnter",
	"ControlOrMeta"
]);
async function keyboardImplementation(pressed, provider, sessionId, text, selectAll, skipRelease) {
	if (provider instanceof PlaywrightBrowserProvider) {
		const page = provider.getPage(sessionId);
		const actions = parseKeyDef(defaultKeyMap, text);
		for (const { releasePrevious, releaseSelf, repeat, keyDef } of actions) {
			const key = keyDef.key;
			// TODO: instead of calling down/up for each key, join non special
			// together, and call `type` once for all non special keys,
			// and then `press` for special keys
			if (pressed.has(key)) {
				if (VALID_KEYS.has(key)) {
					await page.keyboard.up(key);
				}
				pressed.delete(key);
			}
			if (!releasePrevious) {
				if (key === "selectall") {
					await selectAll();
					continue;
				}
				for (let i = 1; i <= repeat; i++) {
					if (VALID_KEYS.has(key)) {
						await page.keyboard.down(key);
					} else {
						await page.keyboard.insertText(key);
					}
				}
				if (releaseSelf) {
					if (VALID_KEYS.has(key)) {
						await page.keyboard.up(key);
					}
				} else {
					pressed.add(key);
				}
			}
		}
		if (!skipRelease && pressed.size) {
			for (const key of pressed) {
				if (VALID_KEYS.has(key)) {
					await page.keyboard.up(key);
				}
			}
		}
	} else if (provider instanceof WebdriverBrowserProvider) {
		const { Key } = await import('webdriverio');
		const browser = provider.browser;
		const actions = parseKeyDef(defaultKeyMap, text);
		let keyboard = browser.action("key");
		for (const { releasePrevious, releaseSelf, repeat, keyDef } of actions) {
			let key = keyDef.key;
			const special = Key[key];
			if (special) {
				key = special;
			}
			if (pressed.has(key)) {
				keyboard.up(key);
				pressed.delete(key);
			}
			if (!releasePrevious) {
				if (key === "selectall") {
					await keyboard.perform();
					keyboard = browser.action("key");
					await selectAll();
					continue;
				}
				for (let i = 1; i <= repeat; i++) {
					keyboard.down(key);
				}
				if (releaseSelf) {
					keyboard.up(key);
				} else {
					pressed.add(key);
				}
			}
		}
		// seems like webdriverio doesn't release keys automatically if skipRelease is true and all events are keyUp
		const allRelease = keyboard.toJSON().actions.every((action) => action.type === "keyUp");
		await keyboard.perform(allRelease ? false : skipRelease);
	}
	return { pressed };
}
function focusIframe() {
	if (!document.activeElement || document.activeElement.ownerDocument !== document || document.activeElement === document.body) {
		window.focus();
	}
}
function selectAll() {
	const element = document.activeElement;
	if (element && typeof element.select === "function") {
		element.select();
	}
}

const screenshot = async (context, name, options = {}) => {
	if (!context.testPath) {
		throw new Error(`Cannot take a screenshot without a test path`);
	}
	options.save ??= true;
	if (!options.save) {
		options.base64 = true;
	}
	const path = options.path ? resolve(dirname(context.testPath), options.path) : resolveScreenshotPath(context.testPath, name, context.project.config);
	const savePath = normalize$1(path);
	await mkdir(dirname(path), { recursive: true });
	if (context.provider instanceof PlaywrightBrowserProvider) {
		if (options.element) {
			const { element: selector,...config } = options;
			const element = context.iframe.locator(`${selector}`);
			const buffer = await element.screenshot({
				...config,
				path: options.save ? savePath : undefined
			});
			return returnResult(options, path, buffer);
		}
		const buffer = await context.iframe.locator("body").screenshot({
			...options,
			path: options.save ? savePath : undefined
		});
		return returnResult(options, path, buffer);
	}
	if (context.provider instanceof WebdriverBrowserProvider) {
		const page = context.provider.browser;
		const element = !options.element ? await page.$("body") : await page.$(`${options.element}`);
		const buffer = await element.saveScreenshot(savePath);
		if (!options.save) {
			await rm(savePath, { force: true });
		}
		return returnResult(options, path, buffer);
	}
	throw new Error(`Provider "${context.provider.name}" does not support screenshots`);
};
function resolveScreenshotPath(testPath, name, config) {
	const dir = dirname(testPath);
	const base = basename(testPath);
	if (config.browser.screenshotDirectory) {
		return resolve(config.browser.screenshotDirectory, relative(config.root, dir), base, name);
	}
	return resolve(dir, "__screenshots__", base, name);
}
function returnResult(options, path, buffer) {
	if (!options.save) {
		return buffer.toString("base64");
	}
	if (options.base64) {
		return {
			path,
			base64: buffer.toString("base64")
		};
	}
	return path;
}

const selectOptions = async (context, selector, userValues, options = {}) => {
	if (context.provider instanceof PlaywrightBrowserProvider) {
		const value = userValues;
		const { iframe } = context;
		const selectElement = iframe.locator(selector);
		const values = await Promise.all(value.map(async (v) => {
			if (typeof v === "string") {
				return v;
			}
			const elementHandler = await iframe.locator(v.element).elementHandle();
			if (!elementHandler) {
				throw new Error(`Element not found: ${v.element}`);
			}
			return elementHandler;
		}));
		await selectElement.selectOption(values, options);
	} else if (context.provider instanceof WebdriverBrowserProvider) {
		const values = userValues;
		if (!values.length) {
			return;
		}
		const browser = context.browser;
		if (values.length === 1 && "index" in values[0]) {
			const selectElement = browser.$(selector);
			await selectElement.selectByIndex(values[0].index);
		} else {
			throw new Error("Provider \"webdriverio\" doesn't support selecting multiple values at once");
		}
	} else {
		throw new TypeError(`Provider "${context.provider.name}" doesn't support selectOptions command`);
	}
};

const tab = async (context, options = {}) => {
	const provider = context.provider;
	if (provider instanceof PlaywrightBrowserProvider) {
		const page = context.page;
		await page.keyboard.press(options.shift === true ? "Shift+Tab" : "Tab");
		return;
	}
	if (provider instanceof WebdriverBrowserProvider) {
		const { Key } = await import('webdriverio');
		const browser = context.browser;
		await browser.keys(options.shift === true ? [Key.Shift, Key.Tab] : [Key.Tab]);
		return;
	}
	throw new Error(`Provider "${provider.name}" doesn't support tab command`);
};

const type = async (context, selector, text, options = {}) => {
	const { skipClick = false, skipAutoClose = false } = options;
	const unreleased = new Set(Reflect.get(options, "unreleased") ?? []);
	if (context.provider instanceof PlaywrightBrowserProvider) {
		const { iframe } = context;
		const element = iframe.locator(selector);
		if (!skipClick) {
			await element.focus();
		}
		await keyboardImplementation(unreleased, context.provider, context.sessionId, text, () => element.selectText(), skipAutoClose);
	} else if (context.provider instanceof WebdriverBrowserProvider) {
		const browser = context.browser;
		const element = browser.$(selector);
		if (!skipClick && !await element.isFocused()) {
			await element.click();
		}
		await keyboardImplementation(unreleased, context.provider, context.sessionId, text, () => browser.execute(() => {
			const element = document.activeElement;
			if (element && typeof element.select === "function") {
				element.select();
			}
		}), skipAutoClose);
	} else {
		throw new TypeError(`Provider "${context.provider.name}" does not support typing`);
	}
	return { unreleased: Array.from(unreleased) };
};

const upload = async (context, selector, files, options) => {
	const testPath = context.testPath;
	if (!testPath) {
		throw new Error(`Cannot upload files outside of a test`);
	}
	const root = context.project.config.root;
	if (context.provider instanceof PlaywrightBrowserProvider) {
		const { iframe } = context;
		const playwrightFiles = files.map((file) => {
			if (typeof file === "string") {
				return resolve(root, file);
			}
			return {
				name: file.name,
				mimeType: file.mimeType,
				buffer: Buffer.from(file.base64, "base64")
			};
		});
		await iframe.locator(selector).setInputFiles(playwrightFiles, options);
	} else if (context.provider instanceof WebdriverBrowserProvider) {
		for (const file of files) {
			if (typeof file !== "string") {
				throw new TypeError(`The "${context.provider.name}" provider doesn't support uploading files objects. Provide a file path instead.`);
			}
		}
		const element = context.browser.$(selector);
		for (const file of files) {
			const filepath = resolve(root, file);
			const remoteFilePath = await context.browser.uploadFile(filepath);
			await element.addValue(remoteFilePath);
		}
	} else {
		throw new TypeError(`Provider "${context.provider.name}" does not support uploading files via userEvent.upload`);
	}
};

const viewport = async (context, options) => {
	if (context.provider instanceof WebdriverBrowserProvider) {
		await context.provider.setViewport(options);
	} else {
		throw new TypeError(`Provider ${context.provider.name} doesn't support "viewport" command`);
	}
};

var builtinCommands = {
	readFile,
	removeFile,
	writeFile,
	__vitest_fileInfo: _fileInfo,
	__vitest_upload: upload,
	__vitest_click: click,
	__vitest_dblClick: dblClick,
	__vitest_tripleClick: tripleClick,
	__vitest_screenshot: screenshot,
	__vitest_type: type,
	__vitest_clear: clear,
	__vitest_fill: fill,
	__vitest_tab: tab,
	__vitest_keyboard: keyboard,
	__vitest_selectOptions: selectOptions,
	__vitest_dragAndDrop: dragAndDrop,
	__vitest_hover: hover,
	__vitest_cleanup: keyboardCleanup,
	__vitest_viewport: viewport
};

class BrowserServerState {
	orchestrators = new Map();
	testers = new Map();
}

class ProjectBrowser {
	testerHtml;
	testerFilepath;
	provider;
	vitest;
	config;
	children = new Set();
	parent;
	state = new BrowserServerState();
	constructor(project, base) {
		this.project = project;
		this.base = base;
		this.vitest = project.vitest;
		this.config = project.config;
		const pkgRoot = resolve(fileURLToPath(import.meta.url), "../..");
		const distRoot = resolve(pkgRoot, "dist");
		const testerHtmlPath = project.config.browser.testerHtmlPath ? resolve(project.config.root, project.config.browser.testerHtmlPath) : resolve(distRoot, "client/tester/tester.html");
		if (!existsSync(testerHtmlPath)) {
			throw new Error(`Tester HTML file "${testerHtmlPath}" doesn't exist.`);
		}
		this.testerFilepath = testerHtmlPath;
		this.testerHtml = readFile$1(testerHtmlPath, "utf8").then((html) => this.testerHtml = html);
	}
	get vite() {
		return this.parent.vite;
	}
	wrapSerializedConfig() {
		const config = wrapConfig(this.project.serializedConfig);
		config.env ??= {};
		config.env.VITEST_BROWSER_DEBUG = process.env.VITEST_BROWSER_DEBUG || "";
		return config;
	}
	async initBrowserProvider(project) {
		if (this.provider) {
			return;
		}
		const Provider = await getBrowserProvider(project.config.browser, project);
		this.provider = new Provider();
		const browser = project.config.browser.name;
		const name = project.name ? `[${project.name}] ` : "";
		if (!browser) {
			throw new Error(`${name}Browser name is required. Please, set \`test.browser.instances[].browser\` option manually.`);
		}
		const supportedBrowsers = this.provider.getSupportedBrowsers();
		if (supportedBrowsers.length && !supportedBrowsers.includes(browser)) {
			throw new Error(`${name}Browser "${browser}" is not supported by the browser provider "${this.provider.name}". Supported browsers: ${supportedBrowsers.join(", ")}.`);
		}
		const providerOptions = project.config.browser.providerOptions;
		await this.provider.initialize(project, {
			browser,
			options: providerOptions
		});
	}
	parseErrorStacktrace(e, options = {}) {
		return this.parent.parseErrorStacktrace(e, options);
	}
	parseStacktrace(trace, options = {}) {
		return this.parent.parseStacktrace(trace, options);
	}
	async close() {
		await this.parent.vite.close();
	}
}
function wrapConfig(config) {
	return {
		...config,
		testNamePattern: config.testNamePattern ? config.testNamePattern.toString() : undefined
	};
}

class ParentBrowserProject {
	orchestratorScripts;
	testerScripts;
	faviconUrl;
	prefixOrchestratorUrl;
	prefixTesterUrl;
	manifest;
	vite;
	stackTraceOptions;
	orchestratorHtml;
	injectorJs;
	errorCatcherUrl;
	locatorsUrl;
	matchersUrl;
	stateJs;
	commands = {};
	children = new Set();
	vitest;
	config;
	// cache for non-vite source maps
	sourceMapCache = new Map();
	constructor(project, base) {
		this.project = project;
		this.base = base;
		this.vitest = project.vitest;
		this.config = project.config;
		this.stackTraceOptions = {
			frameFilter: project.config.onStackTrace,
			getSourceMap: (id) => {
				if (this.sourceMapCache.has(id)) {
					return this.sourceMapCache.get(id);
				}
				const result = this.vite.moduleGraph.getModuleById(id)?.transformResult;
				// this can happen for bundled dependencies in node_modules/.vite
				if (result && !result.map) {
					const sourceMapUrl = this.retrieveSourceMapURL(result.code);
					if (!sourceMapUrl) {
						return null;
					}
					const filepathDir = dirname(id);
					const sourceMapPath = resolve(filepathDir, sourceMapUrl);
					const map = JSON.parse(readFileSync(sourceMapPath, "utf-8"));
					this.sourceMapCache.set(id, map);
					return map;
				}
				return result?.map;
			},
			getUrlId: (id) => {
				const mod = this.vite.moduleGraph.getModuleById(id);
				if (mod) {
					return id;
				}
				const resolvedPath = resolve(this.vite.config.root, id.slice(1));
				const modUrl = this.vite.moduleGraph.getModuleById(resolvedPath);
				if (modUrl) {
					return resolvedPath;
				}
				// some browsers (looking at you, safari) don't report queries in stack traces
				// the next best thing is to try the first id that this file resolves to
				const files = this.vite.moduleGraph.getModulesByFile(resolvedPath);
				if (files && files.size) {
					return files.values().next().value.id;
				}
				return id;
			}
		};
		for (const [name, command] of Object.entries(builtinCommands)) {
			this.commands[name] ??= command;
		}
		// validate names because they can't be used as identifiers
		for (const command in project.config.browser.commands) {
			if (!/^[a-z_$][\w$]*$/i.test(command)) {
				throw new Error(`Invalid command name "${command}". Only alphanumeric characters, $ and _ are allowed.`);
			}
			this.commands[command] = project.config.browser.commands[command];
		}
		this.prefixTesterUrl = `${base || "/"}`;
		this.prefixOrchestratorUrl = `${base}__vitest_test__/`;
		this.faviconUrl = `${base}__vitest__/favicon.svg`;
		this.manifest = (async () => {
			return JSON.parse(await readFile$1(`${distRoot}/client/.vite/manifest.json`, "utf8"));
		})().then((manifest) => this.manifest = manifest);
		this.orchestratorHtml = (project.config.browser.ui ? readFile$1(resolve(distRoot, "client/__vitest__/index.html"), "utf8") : readFile$1(resolve(distRoot, "client/orchestrator.html"), "utf8")).then((html) => this.orchestratorHtml = html);
		this.injectorJs = readFile$1(resolve(distRoot, "client/esm-client-injector.js"), "utf8").then((js) => this.injectorJs = js);
		this.errorCatcherUrl = join("/@fs/", resolve(distRoot, "client/error-catcher.js"));
		const builtinProviders = [
			"playwright",
			"webdriverio",
			"preview"
		];
		const providerName = project.config.browser.provider || "preview";
		if (builtinProviders.includes(providerName)) {
			this.locatorsUrl = join("/@fs/", distRoot, "locators", `${providerName}.js`);
		}
		this.matchersUrl = join("/@fs/", distRoot, "expect-element.js");
		this.stateJs = readFile$1(resolve(distRoot, "state.js"), "utf-8").then((js) => this.stateJs = js);
	}
	setServer(vite) {
		this.vite = vite;
	}
	spawn(project) {
		if (!this.vite) {
			throw new Error(`Cannot spawn child server without a parent dev server.`);
		}
		const clone = new ProjectBrowser(project, "/");
		clone.parent = this;
		this.children.add(clone);
		return clone;
	}
	parseErrorStacktrace(e, options = {}) {
		return parseErrorStacktrace(e, {
			...this.stackTraceOptions,
			...options
		});
	}
	parseStacktrace(trace, options = {}) {
		return parseStacktrace(trace, {
			...this.stackTraceOptions,
			...options
		});
	}
	cdps = new Map();
	cdpSessionsPromises = new Map();
	async ensureCDPHandler(sessionId, rpcId) {
		const cachedHandler = this.cdps.get(rpcId);
		if (cachedHandler) {
			return cachedHandler;
		}
		const browserSession = this.vitest._browserSessions.getSession(sessionId);
		if (!browserSession) {
			throw new Error(`Session "${sessionId}" not found.`);
		}
		const browser = browserSession.project.browser;
		const provider = browser.provider;
		if (!provider) {
			throw new Error(`Browser provider is not defined for the project "${browserSession.project.name}".`);
		}
		if (!provider.getCDPSession) {
			throw new Error(`CDP is not supported by the provider "${provider.name}".`);
		}
		const session = await this.cdpSessionsPromises.get(rpcId) ?? await (async () => {
			const promise = provider.getCDPSession(sessionId).finally(() => {
				this.cdpSessionsPromises.delete(rpcId);
			});
			this.cdpSessionsPromises.set(rpcId, promise);
			return promise;
		})();
		const rpc = browser.state.testers.get(rpcId);
		if (!rpc) {
			throw new Error(`Tester RPC "${rpcId}" was not established.`);
		}
		const handler = new BrowserServerCDPHandler(session, rpc);
		this.cdps.set(rpcId, handler);
		return handler;
	}
	removeCDPHandler(sessionId) {
		this.cdps.delete(sessionId);
	}
	async formatScripts(scripts) {
		if (!scripts?.length) {
			return [];
		}
		const server = this.vite;
		const promises = scripts.map(async ({ content, src, async, id, type = "module" }, index) => {
			const srcLink = (src ? (await server.pluginContainer.resolveId(src))?.id : undefined) || src;
			const transformId = srcLink || join(server.config.root, `virtual__${id || `injected-${index}.js`}`);
			await server.moduleGraph.ensureEntryFromUrl(transformId);
			const contentProcessed = content && type === "module" ? (await server.pluginContainer.transform(content, transformId)).code : content;
			return {
				tag: "script",
				attrs: {
					type,
					...async ? { async: "" } : {},
					...srcLink ? { src: srcLink.startsWith("http") ? srcLink : slash(`/@fs/${srcLink}`) } : {}
				},
				injectTo: "head",
				children: contentProcessed || ""
			};
		});
		return await Promise.all(promises);
	}
	resolveTesterUrl(pathname) {
		const [sessionId, testFile] = pathname.slice(this.prefixTesterUrl.length).split("/");
		const decodedTestFile = decodeURIComponent(testFile);
		return {
			sessionId,
			testFile: decodedTestFile
		};
	}
	retrieveSourceMapURL(source) {
		const re = /\/\/[@#]\s*sourceMappingURL=([^\s'"]+)\s*$|\/\*[@#]\s*sourceMappingURL=[^\s*'"]+\s*\*\/\s*$/gm;
		// Keep executing the search to find the *last* sourceMappingURL to avoid
		// picking up sourceMappingURLs from comments, strings, etc.
		let lastMatch, match;
		// eslint-disable-next-line no-cond-assign
		while (match = re.exec(source)) {
			lastMatch = match;
		}
		if (!lastMatch) {
			return null;
		}
		return lastMatch[1];
	}
}

const TYPE_REQUEST = "q";
const TYPE_RESPONSE = "s";
const DEFAULT_TIMEOUT = 6e4;
function defaultSerialize(i) {
  return i;
}
const defaultDeserialize = defaultSerialize;
const { clearTimeout, setTimeout } = globalThis;
const random = Math.random.bind(Math);
function createBirpc(functions, options) {
  const {
    post,
    on,
    off = () => {
    },
    eventNames = [],
    serialize = defaultSerialize,
    deserialize = defaultDeserialize,
    resolver,
    bind = "rpc",
    timeout = DEFAULT_TIMEOUT
  } = options;
  const rpcPromiseMap = /* @__PURE__ */ new Map();
  let _promise;
  let closed = false;
  const rpc = new Proxy({}, {
    get(_, method) {
      if (method === "$functions")
        return functions;
      if (method === "$close")
        return close;
      if (method === "$closed")
        return closed;
      if (method === "then" && !eventNames.includes("then") && !("then" in functions))
        return undefined;
      const sendEvent = (...args) => {
        post(serialize({ m: method, a: args, t: TYPE_REQUEST }));
      };
      if (eventNames.includes(method)) {
        sendEvent.asEvent = sendEvent;
        return sendEvent;
      }
      const sendCall = async (...args) => {
        if (closed)
          throw new Error(`[birpc] rpc is closed, cannot call "${method}"`);
        if (_promise) {
          try {
            await _promise;
          } finally {
            _promise = undefined;
          }
        }
        return new Promise((resolve, reject) => {
          const id = nanoid();
          let timeoutId;
          if (timeout >= 0) {
            timeoutId = setTimeout(() => {
              try {
                const handleResult = options.onTimeoutError?.(method, args);
                if (handleResult !== true)
                  throw new Error(`[birpc] timeout on calling "${method}"`);
              } catch (e) {
                reject(e);
              }
              rpcPromiseMap.delete(id);
            }, timeout);
            if (typeof timeoutId === "object")
              timeoutId = timeoutId.unref?.();
          }
          rpcPromiseMap.set(id, { resolve, reject, timeoutId, method });
          post(serialize({ m: method, a: args, i: id, t: "q" }));
        });
      };
      sendCall.asEvent = sendEvent;
      return sendCall;
    }
  });
  function close(error) {
    closed = true;
    rpcPromiseMap.forEach(({ reject, method }) => {
      reject(error || new Error(`[birpc] rpc is closed, cannot call "${method}"`));
    });
    rpcPromiseMap.clear();
    off(onMessage);
  }
  async function onMessage(data, ...extra) {
    let msg;
    try {
      msg = deserialize(data);
    } catch (e) {
      if (options.onGeneralError?.(e) !== true)
        throw e;
      return;
    }
    if (msg.t === TYPE_REQUEST) {
      const { m: method, a: args } = msg;
      let result, error;
      const fn = resolver ? resolver(method, functions[method]) : functions[method];
      if (!fn) {
        error = new Error(`[birpc] function "${method}" not found`);
      } else {
        try {
          result = await fn.apply(bind === "rpc" ? rpc : functions, args);
        } catch (e) {
          error = e;
        }
      }
      if (msg.i) {
        if (error && options.onError)
          options.onError(error, method, args);
        if (error && options.onFunctionError) {
          if (options.onFunctionError(error, method, args) === true)
            return;
        }
        if (!error) {
          try {
            post(serialize({ t: TYPE_RESPONSE, i: msg.i, r: result }), ...extra);
            return;
          } catch (e) {
            error = e;
            if (options.onGeneralError?.(e, method, args) !== true)
              throw e;
          }
        }
        try {
          post(serialize({ t: TYPE_RESPONSE, i: msg.i, e: error }), ...extra);
        } catch (e) {
          if (options.onGeneralError?.(e, method, args) !== true)
            throw e;
        }
      }
    } else {
      const { i: ack, r: result, e: error } = msg;
      const promise = rpcPromiseMap.get(ack);
      if (promise) {
        clearTimeout(promise.timeoutId);
        if (error)
          promise.reject(error);
        else
          promise.resolve(result);
      }
      rpcPromiseMap.delete(ack);
    }
  }
  _promise = on(onMessage);
  return rpc;
}
const urlAlphabet = "useandom-26T198340PX75pxJACKVERYMINDBUSHWOLF_GQZbfghjklqvwyzrict";
function nanoid(size = 21) {
  let id = "";
  let i = size;
  while (i--)
    id += urlAlphabet[random() * 64 | 0];
  return id;
}

const debug$1 = createDebugger("vitest:browser:api");
const BROWSER_API_PATH = "/__vitest_browser_api__";
function setupBrowserRpc(globalServer, defaultMockerRegistry) {
	const vite = globalServer.vite;
	const vitest = globalServer.vitest;
	const wss = new WebSocketServer({ noServer: true });
	vite.httpServer?.on("upgrade", (request, socket, head) => {
		if (!request.url) {
			return;
		}
		const { pathname, searchParams } = new URL(request.url, "http://localhost");
		if (pathname !== BROWSER_API_PATH) {
			return;
		}
		if (!isValidApiRequest(vitest.config, request)) {
			socket.destroy();
			return;
		}
		const type = searchParams.get("type");
		const rpcId = searchParams.get("rpcId");
		const sessionId = searchParams.get("sessionId");
		const projectName = searchParams.get("projectName");
		if (type !== "tester" && type !== "orchestrator") {
			return error(new Error(`[vitest] Type query in ${request.url} is invalid. Type should be either "tester" or "orchestrator".`));
		}
		if (!sessionId || !rpcId || projectName == null) {
			return error(new Error(`[vitest] Invalid URL ${request.url}. "projectName", "sessionId" and "rpcId" queries are required.`));
		}
		if (!vitest._browserSessions.sessionIds.has(sessionId)) {
			const ids = [...vitest._browserSessions.sessionIds].join(", ");
			return error(new Error(`[vitest] Unknown session id "${sessionId}". Expected one of ${ids}.`));
		}
		if (type === "orchestrator") {
			const session = vitest._browserSessions.getSession(sessionId);
			// it's possible the session was already resolved by the preview provider
			session?.connected();
		}
		const project = vitest.getProjectByName(projectName);
		if (!project) {
			return error(new Error(`[vitest] Project "${projectName}" not found.`));
		}
		wss.handleUpgrade(request, socket, head, (ws) => {
			wss.emit("connection", ws, request);
			const rpc = setupClient(project, rpcId, ws);
			const state = project.browser.state;
			const clients = type === "tester" ? state.testers : state.orchestrators;
			clients.set(rpcId, rpc);
			debug$1?.("[%s] Browser API connected to %s", rpcId, type);
			ws.on("close", () => {
				debug$1?.("[%s] Browser API disconnected from %s", rpcId, type);
				clients.delete(rpcId);
				globalServer.removeCDPHandler(rpcId);
				if (type === "orchestrator") {
					vitest._browserSessions.destroySession(sessionId);
				}
				// this will reject any hanging methods if there are any
				rpc.$close(new Error(`[vitest] Browser connection was closed while running tests. Was the page closed unexpectedly?`));
			});
		});
	});
	// we don't throw an error inside a stream because this can segfault the process
	function error(err) {
		console.error(err);
		vitest.state.catchError(err, "RPC Error");
	}
	function checkFileAccess(path) {
		if (!isFileServingAllowed(path, vite)) {
			throw new Error(`Access denied to "${path}". See Vite config documentation for "server.fs": https://vitejs.dev/config/server-options.html#server-fs-strict.`);
		}
	}
	function setupClient(project, rpcId, ws) {
		const mockResolver = new ServerMockResolver(globalServer.vite, { moduleDirectories: project.config.server?.deps?.moduleDirectories });
		const mocker = project.browser?.provider.mocker;
		const rpc = createBirpc({
			async onUnhandledError(error, type) {
				if (error && typeof error === "object") {
					const _error = error;
					_error.stacks = globalServer.parseErrorStacktrace(_error);
				}
				vitest.state.catchError(error, type);
			},
			async onQueued(method, file) {
				if (method === "collect") {
					vitest.state.collectFiles(project, [file]);
				} else {
					await vitest._testRun.enqueued(project, file);
				}
			},
			async onCollected(method, files) {
				if (method === "collect") {
					vitest.state.collectFiles(project, files);
				} else {
					await vitest._testRun.collected(project, files);
				}
			},
			async onTaskAnnotate(id, annotation) {
				return vitest._testRun.annotate(id, annotation);
			},
			async onTaskUpdate(method, packs, events) {
				if (method === "collect") {
					vitest.state.updateTasks(packs);
				} else {
					await vitest._testRun.updated(packs, events);
				}
			},
			onAfterSuiteRun(meta) {
				vitest.coverageProvider?.onAfterSuiteRun(meta);
			},
			async sendLog(method, log) {
				if (method === "collect") {
					vitest.state.updateUserLog(log);
				} else {
					await vitest._testRun.log(log);
				}
			},
			resolveSnapshotPath(testPath) {
				return vitest.snapshot.resolvePath(testPath, { config: project.serializedConfig });
			},
			resolveSnapshotRawPath(testPath, rawPath) {
				return vitest.snapshot.resolveRawPath(testPath, rawPath);
			},
			snapshotSaved(snapshot) {
				vitest.snapshot.add(snapshot);
			},
			async readSnapshotFile(snapshotPath) {
				checkFileAccess(snapshotPath);
				if (!existsSync(snapshotPath)) {
					return null;
				}
				return promises.readFile(snapshotPath, "utf-8");
			},
			async saveSnapshotFile(id, content) {
				checkFileAccess(id);
				await promises.mkdir(dirname(id), { recursive: true });
				return promises.writeFile(id, content, "utf-8");
			},
			async removeSnapshotFile(id) {
				checkFileAccess(id);
				if (!existsSync(id)) {
					throw new Error(`Snapshot file "${id}" does not exist.`);
				}
				return promises.unlink(id);
			},
			getBrowserFileSourceMap(id) {
				const mod = globalServer.vite.moduleGraph.getModuleById(id);
				return mod?.transformResult?.map;
			},
			cancelCurrentRun(reason) {
				vitest.cancelCurrentRun(reason);
			},
			async resolveId(id, importer) {
				return mockResolver.resolveId(id, importer);
			},
			debug(...args) {
				vitest.logger.console.debug(...args);
			},
			getCountOfFailedTests() {
				return vitest.state.getCountOfFailedTests();
			},
			async wdioSwitchContext(direction) {
				const provider = project.browser.provider;
				if (!provider) {
					throw new Error("Commands are only available for browser tests.");
				}
				if (provider.name !== "webdriverio") {
					throw new Error("Switch context is only available for WebDriverIO provider.");
				}
				if (direction === "iframe") {
					await provider.switchToTestFrame();
				} else {
					await provider.switchToMainFrame();
				}
			},
			async triggerCommand(sessionId, command, testPath, payload) {
				debug$1?.("[%s] Triggering command \"%s\"", sessionId, command);
				const provider = project.browser.provider;
				if (!provider) {
					throw new Error("Commands are only available for browser tests.");
				}
				const commands = globalServer.commands;
				if (!commands || !commands[command]) {
					throw new Error(`Unknown command "${command}".`);
				}
				const context = Object.assign({
					testPath,
					project,
					provider,
					contextId: sessionId,
					sessionId
				}, provider.getCommandsContext(sessionId));
				return await commands[command](context, ...payload);
			},
			resolveMock(rawId, importer, options) {
				return mockResolver.resolveMock(rawId, importer, options);
			},
			invalidate(ids) {
				return mockResolver.invalidate(ids);
			},
			async registerMock(sessionId, module) {
				if (!mocker) {
					// make sure modules are not processed yet in case they were imported before
					// and were not mocked
					mockResolver.invalidate([module.id]);
					if (module.type === "manual") {
						const mock = ManualMockedModule.fromJSON(module, async () => {
							try {
								const { keys } = await rpc.resolveManualMock(module.url);
								return Object.fromEntries(keys.map((key) => [key, null]));
							} catch (err) {
								vitest.state.catchError(err, "Manual Mock Resolver Error");
								return {};
							}
						});
						defaultMockerRegistry.add(mock);
					} else {
						if (module.type === "redirect") {
							const redirectUrl = new URL(module.redirect);
							module.redirect = join(vite.config.root, redirectUrl.pathname);
						}
						defaultMockerRegistry.register(module);
					}
					return;
				}
				if (module.type === "manual") {
					const manualModule = ManualMockedModule.fromJSON(module, async () => {
						const { keys } = await rpc.resolveManualMock(module.url);
						return Object.fromEntries(keys.map((key) => [key, null]));
					});
					await mocker.register(sessionId, manualModule);
				} else if (module.type === "redirect") {
					await mocker.register(sessionId, RedirectedModule.fromJSON(module));
				} else if (module.type === "automock") {
					await mocker.register(sessionId, AutomockedModule.fromJSON(module));
				} else if (module.type === "autospy") {
					await mocker.register(sessionId, AutospiedModule.fromJSON(module));
				}
			},
			clearMocks(sessionId) {
				if (!mocker) {
					return defaultMockerRegistry.clear();
				}
				return mocker.clear(sessionId);
			},
			unregisterMock(sessionId, id) {
				if (!mocker) {
					return defaultMockerRegistry.delete(id);
				}
				return mocker.delete(sessionId, id);
			},
			async sendCdpEvent(sessionId, event, payload) {
				const cdp = await globalServer.ensureCDPHandler(sessionId, rpcId);
				return cdp.send(event, payload);
			},
			async trackCdpEvent(sessionId, type, event, listenerId) {
				const cdp = await globalServer.ensureCDPHandler(sessionId, rpcId);
				cdp[type](event, listenerId);
			}
		}, {
			post: (msg) => ws.send(msg),
			on: (fn) => ws.on("message", fn),
			eventNames: ["onCancel", "cdpEvent"],
			serialize: (data) => stringify(data, stringifyReplace),
			timeout: -1,
			deserialize: parse,
			onTimeoutError(functionName) {
				throw new Error(`[vitest-api]: Timeout calling "${functionName}"`);
			}
		});
		vitest.onCancel((reason) => rpc.onCancel(reason));
		return rpc;
	}
}
// Serialization support utils.
function cloneByOwnProperties(value) {
	// Clones the value's properties into a new Object. The simpler approach of
	// Object.assign() won't work in the case that properties are not enumerable.
	return Object.getOwnPropertyNames(value).reduce((clone, prop) => ({
		...clone,
		[prop]: value[prop]
	}), {});
}
/**
* Replacer function for serialization methods such as JS.stringify() or
* flatted.stringify().
*/
function stringifyReplace(key, value) {
	if (value instanceof Error) {
		const cloned = cloneByOwnProperties(value);
		return {
			name: value.name,
			message: value.message,
			stack: value.stack,
			...cloned
		};
	} else {
		return value;
	}
}

const debug = createDebugger("vitest:browser:pool");
function createBrowserPool(vitest) {
	const providers = new Set();
	const numCpus = typeof nodeos.availableParallelism === "function" ? nodeos.availableParallelism() : nodeos.cpus().length;
	const threadsCount = vitest.config.watch ? Math.max(Math.floor(numCpus / 2), 1) : Math.max(numCpus - 1, 1);
	const projectPools = new WeakMap();
	const ensurePool = (project) => {
		if (projectPools.has(project)) {
			return projectPools.get(project);
		}
		debug?.("creating pool for project %s", project.name);
		const resolvedUrls = project.browser.vite.resolvedUrls;
		const origin = resolvedUrls?.local[0] ?? resolvedUrls?.network[0];
		if (!origin) {
			throw new Error(`Can't find browser origin URL for project "${project.name}"`);
		}
		const pool = new BrowserPool(project, {
			maxWorkers: getThreadsCount(project),
			origin
		});
		projectPools.set(project, pool);
		vitest.onCancel(() => {
			pool.cancel();
		});
		return pool;
	};
	const runWorkspaceTests = async (method, specs) => {
		const groupedFiles = new Map();
		for (const { project, moduleId } of specs) {
			const files = groupedFiles.get(project) || [];
			files.push(moduleId);
			groupedFiles.set(project, files);
		}
		let isCancelled = false;
		vitest.onCancel(() => {
			isCancelled = true;
		});
		const initialisedPools = await Promise.all([...groupedFiles.entries()].map(async ([project, files]) => {
			await project._initBrowserProvider();
			if (!project.browser) {
				throw new TypeError(`The browser server was not initialized${project.name ? ` for the "${project.name}" project` : ""}. This is a bug in Vitest. Please, open a new issue with reproduction.`);
			}
			if (isCancelled) {
				return;
			}
			debug?.("provider is ready for %s project", project.name);
			const pool = ensurePool(project);
			vitest.state.clearFiles(project, files);
			providers.add(project.browser.provider);
			return {
				pool,
				provider: project.browser.provider,
				runTests: () => pool.runTests(method, files)
			};
		}));
		if (isCancelled) {
			return;
		}
		const parallelPools = [];
		const nonParallelPools = [];
		for (const result of initialisedPools) {
			if (!result) {
				return;
			}
			if (result.provider.mocker && result.provider.supportsParallelism) {
				parallelPools.push(result.runTests);
			} else {
				nonParallelPools.push(result.runTests);
			}
		}
		await Promise.all(parallelPools.map((runTests) => runTests()));
		for (const runTests of nonParallelPools) {
			if (isCancelled) {
				return;
			}
			await runTests();
		}
	};
	function getThreadsCount(project) {
		const config = project.config.browser;
		if (!config.headless || !config.fileParallelism || !project.browser.provider.supportsParallelism) {
			return 1;
		}
		if (project.config.maxWorkers) {
			return project.config.maxWorkers;
		}
		return threadsCount;
	}
	return {
		name: "browser",
		async close() {
			await Promise.all([...providers].map((provider) => provider.close()));
			vitest._browserSessions.sessionIds.clear();
			providers.clear();
			vitest.projects.forEach((project) => {
				project.browser?.state.orchestrators.forEach((orchestrator) => {
					orchestrator.$close();
				});
			});
			debug?.("browser pool closed all providers");
		},
		runTests: (files) => runWorkspaceTests("run", files),
		collectTests: (files) => runWorkspaceTests("collect", files)
	};
}
function escapePathToRegexp(path) {
	return path.replace(/[/\\.?*()^${}|[\]+]/g, "\\$&");
}
class BrowserPool {
	_queue = [];
	_promise;
	_providedContext;
	readySessions = new Set();
	constructor(project, options) {
		this.project = project;
		this.options = options;
	}
	cancel() {
		this._queue = [];
	}
	reject(error) {
		this._promise?.reject(error);
		this._promise = undefined;
		this.cancel();
	}
	get orchestrators() {
		return this.project.browser.state.orchestrators;
	}
	async runTests(method, files) {
		this._promise ??= createDefer();
		if (!files.length) {
			debug?.("no tests found, finishing test run immediately");
			this._promise.resolve();
			return this._promise;
		}
		this._providedContext = stringify(this.project.getProvidedContext());
		this._queue.push(...files);
		this.readySessions.forEach((sessionId) => {
			if (this._queue.length) {
				this.readySessions.delete(sessionId);
				this.runNextTest(method, sessionId);
			}
		});
		if (this.orchestrators.size >= this.options.maxWorkers) {
			debug?.("all orchestrators are ready, not creating more");
			return this._promise;
		}
		// open the minimum amount of tabs
		// if there is only 1 file running, we don't need 8 tabs running
		const workerCount = Math.min(this.options.maxWorkers - this.orchestrators.size, files.length);
		const promises = [];
		for (let i = 0; i < workerCount; i++) {
			const sessionId = crypto.randomUUID();
			this.project.vitest._browserSessions.sessionIds.add(sessionId);
			const project = this.project.name;
			debug?.("[%s] creating session for %s", sessionId, project);
			const page = this.openPage(sessionId).then(() => {
				// start running tests on the page when it's ready
				this.runNextTest(method, sessionId);
			});
			promises.push(page);
		}
		await Promise.all(promises);
		debug?.("all sessions are created");
		return this._promise;
	}
	async openPage(sessionId) {
		const sessionPromise = this.project.vitest._browserSessions.createSession(sessionId, this.project, this);
		const browser = this.project.browser;
		const url = new URL("/__vitest_test__/", this.options.origin);
		url.searchParams.set("sessionId", sessionId);
		const pagePromise = browser.provider.openPage(sessionId, url.toString());
		await Promise.all([sessionPromise, pagePromise]);
	}
	getOrchestrator(sessionId) {
		const orchestrator = this.orchestrators.get(sessionId);
		if (!orchestrator) {
			throw new Error(`Orchestrator not found for session ${sessionId}. This is a bug in Vitest. Please, open a new issue with reproduction.`);
		}
		return orchestrator;
	}
	finishSession(sessionId) {
		this.readySessions.add(sessionId);
		// the last worker finished running tests
		if (this.readySessions.size === this.orchestrators.size) {
			this._promise?.resolve();
			this._promise = undefined;
			debug?.("[%s] all tests finished running", sessionId);
		} else {
			debug?.(`did not finish sessions for ${sessionId}: |ready - %s| |overall - %s|`, [...this.readySessions].join(", "), [...this.orchestrators.keys()].join(", "));
		}
	}
	runNextTest(method, sessionId) {
		const file = this._queue.shift();
		if (!file) {
			debug?.("[%s] no more tests to run", sessionId);
			const isolate = this.project.config.browser.isolate;
			// we don't need to cleanup testers if isolation is enabled,
			// because cleanup is done at the end of every test
			if (isolate) {
				this.finishSession(sessionId);
				return;
			}
			// we need to cleanup testers first because there is only
			// one iframe and it does the cleanup only after everything is completed
			const orchestrator = this.getOrchestrator(sessionId);
			orchestrator.cleanupTesters().catch((error) => this.reject(error)).finally(() => this.finishSession(sessionId));
			return;
		}
		if (!this._promise) {
			throw new Error(`Unexpected empty queue`);
		}
		const startTime = performance.now();
		const orchestrator = this.getOrchestrator(sessionId);
		debug?.("[%s] run test %s", sessionId, file);
		this.setBreakpoint(sessionId, file).then(() => {
			// this starts running tests inside the orchestrator
			orchestrator.createTesters({
				method,
				files: [file],
				providedContext: this._providedContext || "[{}]",
				startTime
			}).then(() => {
				debug?.("[%s] test %s finished running", sessionId, file);
				this.runNextTest(method, sessionId);
			}).catch((error) => {
				// if user cancells the test run manually, ignore the error and exit gracefully
				if (this.project.vitest.isCancelling && error instanceof Error && error.message.startsWith("Browser connection was closed while running tests")) {
					this.cancel();
					this._promise?.resolve();
					this._promise = undefined;
					debug?.("[%s] browser connection was closed", sessionId);
					return;
				}
				debug?.("[%s] error during %s test run: %s", sessionId, file, error);
				this.reject(error);
			});
		}).catch((err) => this.reject(err));
	}
	async setBreakpoint(sessionId, file) {
		if (!this.project.config.inspector.waitForDebugger) {
			return;
		}
		const provider = this.project.browser.provider;
		if (!provider.getCDPSession) {
			throw new Error("Unable to set breakpoint, CDP not supported");
		}
		debug?.("[%s] set breakpoint for %s", sessionId, file);
		const session = await provider.getCDPSession(sessionId);
		await session.send("Debugger.enable", {});
		await session.send("Debugger.setBreakpointByUrl", {
			lineNumber: 0,
			urlRegex: escapePathToRegexp(file)
		});
	}
}

async function createBrowserServer(project, configFile, prePlugins = [], postPlugins = []) {
	if (project.vitest.version !== version) {
		project.vitest.logger.warn(c.yellow(`Loaded ${c.inverse(c.yellow(` vitest@${project.vitest.version} `))} and ${c.inverse(c.yellow(` @vitest/browser@${version} `))}.` + "\nRunning mixed versions is not supported and may lead into bugs" + "\nUpdate your dependencies and make sure the versions match."));
	}
	const server = new ParentBrowserProject(project, "/");
	const configPath = typeof configFile === "string" ? configFile : false;
	const logLevel = process.env.VITEST_BROWSER_DEBUG ?? "info";
	const logger = createViteLogger(project.vitest.logger, logLevel, { allowClearScreen: false });
	const mockerRegistry = new MockerRegistry();
	const vite = await createViteServer({
		...project.options,
		base: "/",
		root: project.config.root,
		logLevel,
		customLogger: {
			...logger,
			info(msg, options) {
				logger.info(msg, options);
				if (msg.includes("optimized dependencies changed. reloading")) {
					logger.warn([c.yellow(`\n${c.bold("[vitest]")} Vite unexpectedly reloaded a test. This may cause tests to fail, lead to flaky behaviour or duplicated test runs.\n`), c.yellow(`For a stable experience, please add mentioned dependencies to your config\'s ${c.bold("`optimizeDeps.include`")} field manually.\n\n`)].join(""));
				}
			}
		},
		mode: project.config.mode,
		configFile: configPath,
		configLoader: project.vite.config.inlineConfig.configLoader,
		server: {
			hmr: false,
			watch: null
		},
		cacheDir: project.vite.config.cacheDir,
		plugins: [
			...prePlugins,
			...project.options?.plugins || [],
			BrowserPlugin(server),
			interceptorPlugin({ registry: mockerRegistry }),
			...postPlugins
		]
	});
	await vite.listen();
	setupBrowserRpc(server, mockerRegistry);
	return server;
}

export { createBrowserPool, createBrowserServer, distRoot };
