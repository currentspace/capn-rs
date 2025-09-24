import { createManualModuleSource } from '@vitest/mocker/node';
import c from 'tinyrainbow';
import { createDebugger, isCSSRequest } from 'vitest/node';

const debug$1 = createDebugger("vitest:browser:playwright");
const playwrightBrowsers = [
	"firefox",
	"webkit",
	"chromium"
];
class PlaywrightBrowserProvider {
	name = "playwright";
	supportsParallelism = true;
	browser = null;
	browserName;
	project;
	options;
	contexts = new Map();
	pages = new Map();
	browserPromise = null;
	mocker;
	closing = false;
	getSupportedBrowsers() {
		return playwrightBrowsers;
	}
	initialize(project, { browser, options }) {
		this.closing = false;
		this.project = project;
		this.browserName = browser;
		this.options = options;
		this.mocker = this.createMocker();
	}
	async openBrowser() {
		await this._throwIfClosing();
		if (this.browserPromise) {
			debug$1?.("[%s] the browser is resolving, reusing the promise", this.browserName);
			return this.browserPromise;
		}
		if (this.browser) {
			debug$1?.("[%s] the browser is resolved, reusing it", this.browserName);
			return this.browser;
		}
		this.browserPromise = (async () => {
			const options = this.project.config.browser;
			const playwright = await import('playwright');
			if (this.options?.connect) {
				if (this.options.launch) {
					this.project.vitest.logger.warn(c.yellow(`Found both ${c.bold(c.italic(c.yellow("connect")))} and ${c.bold(c.italic(c.yellow("launch")))} options in browser instance configuration.
          Ignoring ${c.bold(c.italic(c.yellow("launch")))} options and using ${c.bold(c.italic(c.yellow("connect")))} mode.
          You probably want to remove one of the two options and keep only the one you want to use.`));
				}
				const browser = await playwright[this.browserName].connect(this.options.connect.wsEndpoint, this.options.connect.options);
				this.browser = browser;
				this.browserPromise = null;
				return this.browser;
			}
			const launchOptions = {
				...this.options?.launch,
				headless: options.headless
			};
			if (this.project.config.inspector.enabled) {
				// NodeJS equivalent defaults: https://nodejs.org/en/learn/getting-started/debugging#enable-inspector
				const port = this.project.config.inspector.port || 9229;
				const host = this.project.config.inspector.host || "127.0.0.1";
				launchOptions.args ||= [];
				launchOptions.args.push(`--remote-debugging-port=${port}`);
				launchOptions.args.push(`--remote-debugging-address=${host}`);
				this.project.vitest.logger.log(`Debugger listening on ws://${host}:${port}`);
			}
			// start Vitest UI maximized only on supported browsers
			if (this.project.config.browser.ui && this.browserName === "chromium") {
				if (!launchOptions.args) {
					launchOptions.args = [];
				}
				if (!launchOptions.args.includes("--start-maximized") && !launchOptions.args.includes("--start-fullscreen")) {
					launchOptions.args.push("--start-maximized");
				}
			}
			debug$1?.("[%s] initializing the browser with launch options: %O", this.browserName, launchOptions);
			this.browser = await playwright[this.browserName].launch(launchOptions);
			this.browserPromise = null;
			return this.browser;
		})();
		return this.browserPromise;
	}
	createMocker() {
		const idPreficates = new Map();
		const sessionIds = new Map();
		function createPredicate(sessionId, url) {
			const moduleUrl = new URL(url, "http://localhost");
			const predicate = (url) => {
				if (url.searchParams.has("_vitest_original")) {
					return false;
				}
				// different modules, ignore request
				if (url.pathname !== moduleUrl.pathname) {
					return false;
				}
				url.searchParams.delete("t");
				url.searchParams.delete("v");
				url.searchParams.delete("import");
				// different search params, ignore request
				if (url.searchParams.size !== moduleUrl.searchParams.size) {
					return false;
				}
				// check that all search params are the same
				for (const [param, value] of url.searchParams.entries()) {
					if (moduleUrl.searchParams.get(param) !== value) {
						return false;
					}
				}
				return true;
			};
			const ids = sessionIds.get(sessionId) || [];
			ids.push(moduleUrl.href);
			sessionIds.set(sessionId, ids);
			idPreficates.set(predicateKey(sessionId, moduleUrl.href), predicate);
			return predicate;
		}
		function predicateKey(sessionId, url) {
			return `${sessionId}:${url}`;
		}
		return {
			register: async (sessionId, module) => {
				const page = this.getPage(sessionId);
				await page.route(createPredicate(sessionId, module.url), async (route) => {
					if (module.type === "manual") {
						const exports = Object.keys(await module.resolve());
						const body = createManualModuleSource(module.url, exports);
						return route.fulfill({
							body,
							headers: getHeaders(this.project.browser.vite.config)
						});
					}
					// webkit doesn't support redirect responses
					// https://github.com/microsoft/playwright/issues/18318
					const isWebkit = this.browserName === "webkit";
					if (isWebkit) {
						let url;
						if (module.type === "redirect") {
							const redirect = new URL(module.redirect);
							url = redirect.href.slice(redirect.origin.length);
						} else {
							const request = new URL(route.request().url());
							request.searchParams.set("mock", module.type);
							url = request.href.slice(request.origin.length);
						}
						const result = await this.project.browser.vite.transformRequest(url).catch(() => null);
						if (!result) {
							return route.continue();
						}
						let content = result.code;
						if (result.map && "version" in result.map && result.map.mappings) {
							const type = isDirectCSSRequest(url) ? "css" : "js";
							content = getCodeWithSourcemap(type, content.toString(), result.map);
						}
						return route.fulfill({
							body: content,
							headers: getHeaders(this.project.browser.vite.config)
						});
					}
					if (module.type === "redirect") {
						return route.fulfill({
							status: 302,
							headers: { Location: module.redirect }
						});
					} else if (module.type === "automock" || module.type === "autospy") {
						const url = new URL(route.request().url());
						url.searchParams.set("mock", module.type);
						return route.fulfill({
							status: 302,
							headers: { Location: url.href }
						});
					} else ;
				});
			},
			delete: async (sessionId, id) => {
				const page = this.getPage(sessionId);
				const key = predicateKey(sessionId, id);
				const predicate = idPreficates.get(key);
				if (predicate) {
					await page.unroute(predicate).finally(() => idPreficates.delete(key));
				}
			},
			clear: async (sessionId) => {
				const page = this.getPage(sessionId);
				const ids = sessionIds.get(sessionId) || [];
				const promises = ids.map((id) => {
					const key = predicateKey(sessionId, id);
					const predicate = idPreficates.get(key);
					if (predicate) {
						return page.unroute(predicate).finally(() => idPreficates.delete(key));
					}
					return null;
				});
				await Promise.all(promises).finally(() => sessionIds.delete(sessionId));
			}
		};
	}
	async createContext(sessionId) {
		await this._throwIfClosing();
		if (this.contexts.has(sessionId)) {
			debug$1?.("[%s][%s] the context already exists, reusing it", sessionId, this.browserName);
			return this.contexts.get(sessionId);
		}
		const browser = await this.openBrowser();
		await this._throwIfClosing(browser);
		const { actionTimeout,...contextOptions } = this.options?.context ?? {};
		const options = {
			...contextOptions,
			ignoreHTTPSErrors: true
		};
		if (this.project.config.browser.ui) {
			options.viewport = null;
		}
		const context = await browser.newContext(options);
		await this._throwIfClosing(context);
		if (actionTimeout) {
			context.setDefaultTimeout(actionTimeout);
		}
		debug$1?.("[%s][%s] the context is ready", sessionId, this.browserName);
		this.contexts.set(sessionId, context);
		return context;
	}
	getPage(sessionId) {
		const page = this.pages.get(sessionId);
		if (!page) {
			throw new Error(`Page "${sessionId}" not found in ${this.browserName} browser.`);
		}
		return page;
	}
	getCommandsContext(sessionId) {
		const page = this.getPage(sessionId);
		return {
			page,
			context: this.contexts.get(sessionId),
			frame() {
				return new Promise((resolve, reject) => {
					const frame = page.frame("vitest-iframe");
					if (frame) {
						return resolve(frame);
					}
					const timeout = setTimeout(() => {
						const err = new Error(`Cannot find "vitest-iframe" on the page. This is a bug in Vitest, please report it.`);
						reject(err);
					}, 1e3).unref();
					page.on("frameattached", (frame) => {
						clearTimeout(timeout);
						resolve(frame);
					});
				});
			},
			get iframe() {
				return page.frameLocator("[data-vitest=\"true\"]");
			}
		};
	}
	async openBrowserPage(sessionId) {
		await this._throwIfClosing();
		if (this.pages.has(sessionId)) {
			debug$1?.("[%s][%s] the page already exists, closing the old one", sessionId, this.browserName);
			const page = this.pages.get(sessionId);
			await page.close();
			this.pages.delete(sessionId);
		}
		const context = await this.createContext(sessionId);
		const page = await context.newPage();
		debug$1?.("[%s][%s] the page is ready", sessionId, this.browserName);
		await this._throwIfClosing(page);
		this.pages.set(sessionId, page);
		if (process.env.VITEST_PW_DEBUG) {
			page.on("requestfailed", (request) => {
				console.error("[PW Error]", request.resourceType(), "request failed for", request.url(), "url:", request.failure()?.errorText);
			});
		}
		return page;
	}
	async openPage(sessionId, url, beforeNavigate) {
		debug$1?.("[%s][%s] creating the browser page for %s", sessionId, this.browserName, url);
		const browserPage = await this.openBrowserPage(sessionId);
		await beforeNavigate?.();
		debug$1?.("[%s][%s] browser page is created, opening %s", sessionId, this.browserName, url);
		await browserPage.goto(url, { timeout: 0 });
		await this._throwIfClosing(browserPage);
	}
	async _throwIfClosing(disposable) {
		if (this.closing) {
			debug$1?.("[%s] provider was closed, cannot perform the action on %s", this.browserName, String(disposable));
			await disposable?.close();
			this.pages.clear();
			this.contexts.clear();
			this.browser = null;
			this.browserPromise = null;
			throw new Error(`[vitest] The provider was closed.`);
		}
	}
	async getCDPSession(sessionid) {
		const page = this.getPage(sessionid);
		const cdp = await page.context().newCDPSession(page);
		return {
			async send(method, params) {
				const result = await cdp.send(method, params);
				return result;
			},
			on(event, listener) {
				cdp.on(event, listener);
			},
			off(event, listener) {
				cdp.off(event, listener);
			},
			once(event, listener) {
				cdp.once(event, listener);
			}
		};
	}
	async close() {
		debug$1?.("[%s] closing provider", this.browserName);
		this.closing = true;
		const browser = this.browser;
		this.browser = null;
		if (this.browserPromise) {
			await this.browserPromise;
			this.browserPromise = null;
		}
		await Promise.all([...this.pages.values()].map((p) => p.close()));
		this.pages.clear();
		await Promise.all([...this.contexts.values()].map((c) => c.close()));
		this.contexts.clear();
		await browser?.close();
		debug$1?.("[%s] provider is closed", this.browserName);
	}
}
function getHeaders(config) {
	const headers = { "Content-Type": "application/javascript" };
	for (const name in config.server.headers) {
		headers[name] = String(config.server.headers[name]);
	}
	return headers;
}
function getCodeWithSourcemap(type, code, map) {
	if (type === "js") {
		code += `\n//# sourceMappingURL=${genSourceMapUrl(map)}`;
	} else if (type === "css") {
		code += `\n/*# sourceMappingURL=${genSourceMapUrl(map)} */`;
	}
	return code;
}
function genSourceMapUrl(map) {
	if (typeof map !== "string") {
		map = JSON.stringify(map);
	}
	return `data:application/json;base64,${Buffer.from(map).toString("base64")}`;
}
const directRequestRE = /[?&]direct\b/;
function isDirectCSSRequest(request) {
	return isCSSRequest(request) && directRequestRE.test(request);
}

const debug = createDebugger("vitest:browser:wdio");
const webdriverBrowsers = [
	"firefox",
	"chrome",
	"edge",
	"safari"
];
class WebdriverBrowserProvider {
	name = "webdriverio";
	supportsParallelism = false;
	browser = null;
	browserName;
	project;
	options;
	closing = false;
	iframeSwitched = false;
	topLevelContext;
	getSupportedBrowsers() {
		return webdriverBrowsers;
	}
	async initialize(ctx, { browser, options }) {
		this.closing = false;
		this.project = ctx;
		this.browserName = browser;
		this.options = options;
	}
	isIframeSwitched() {
		return this.iframeSwitched;
	}
	async switchToTestFrame() {
		const page = this.browser;
		// support wdio@9
		if (page.switchFrame) {
			await page.switchFrame(page.$("iframe[data-vitest]"));
		} else {
			const iframe = await page.findElement("css selector", "iframe[data-vitest]");
			await page.switchToFrame(iframe);
		}
		this.iframeSwitched = true;
	}
	async switchToMainFrame() {
		const page = this.browser;
		if (page.switchFrame) {
			await page.switchFrame(null);
		} else {
			await page.switchToParentFrame();
		}
		this.iframeSwitched = false;
	}
	async setViewport(options) {
		if (this.topLevelContext == null || !this.browser) {
			throw new Error(`The browser has no open pages.`);
		}
		await this.browser.send({
			method: "browsingContext.setViewport",
			params: {
				context: this.topLevelContext,
				devicePixelRatio: 1,
				viewport: options
			}
		});
	}
	getCommandsContext() {
		return { browser: this.browser };
	}
	async openBrowser() {
		await this._throwIfClosing("opening the browser");
		if (this.browser) {
			debug?.("[%s] the browser is already opened, reusing it", this.browserName);
			return this.browser;
		}
		const options = this.project.config.browser;
		if (this.browserName === "safari") {
			if (options.headless) {
				throw new Error("You've enabled headless mode for Safari but it doesn't currently support it.");
			}
		}
		const { remote } = await import('webdriverio');
		const remoteOptions = {
			...this.options,
			logLevel: "error",
			capabilities: this.buildCapabilities()
		};
		debug?.("[%s] opening the browser with options: %O", this.browserName, remoteOptions);
		// TODO: close everything, if browser is closed from the outside
		this.browser = await remote(remoteOptions);
		await this._throwIfClosing();
		return this.browser;
	}
	buildCapabilities() {
		const capabilities = {
			...this.options?.capabilities,
			browserName: this.browserName
		};
		const headlessMap = {
			chrome: ["goog:chromeOptions", ["headless", "disable-gpu"]],
			firefox: ["moz:firefoxOptions", ["-headless"]],
			edge: ["ms:edgeOptions", ["--headless"]]
		};
		const options = this.project.config.browser;
		const browser = this.browserName;
		if (browser !== "safari" && options.headless) {
			const [key, args] = headlessMap[browser];
			const currentValues = (this.options?.capabilities)?.[key] || {};
			const newArgs = [...currentValues.args || [], ...args];
			capabilities[key] = {
				...currentValues,
				args: newArgs
			};
		}
		// start Vitest UI maximized only on supported browsers
		if (options.ui && (browser === "chrome" || browser === "edge")) {
			const key = browser === "chrome" ? "goog:chromeOptions" : "ms:edgeOptions";
			const args = capabilities[key]?.args || [];
			if (!args.includes("--start-maximized") && !args.includes("--start-fullscreen")) {
				args.push("--start-maximized");
			}
			capabilities[key] ??= {};
			capabilities[key].args = args;
		}
		return capabilities;
	}
	async openPage(sessionId, url) {
		await this._throwIfClosing("creating the browser");
		debug?.("[%s][%s] creating the browser page for %s", sessionId, this.browserName, url);
		const browserInstance = await this.openBrowser();
		debug?.("[%s][%s] browser page is created, opening %s", sessionId, this.browserName, url);
		await browserInstance.url(url);
		this.topLevelContext = await browserInstance.getWindowHandle();
		await this._throwIfClosing("opening the url");
	}
	async _throwIfClosing(action) {
		if (this.closing) {
			debug?.(`[%s] provider was closed, cannot perform the action${action ? ` ${action}` : ""}`, this.browserName);
			await (this.browser?.sessionId ? this.browser?.deleteSession?.() : null);
			throw new Error(`[vitest] The provider was closed.`);
		}
	}
	async close() {
		debug?.("[%s] closing provider", this.browserName);
		this.closing = true;
		await Promise.all([this.browser?.sessionId ? this.browser?.deleteSession?.() : null]);
		// TODO: right now process can only exit with timeout, if we use browser
		// needs investigating
		process.exit();
	}
}

export { PlaywrightBrowserProvider as P, WebdriverBrowserProvider as W };
