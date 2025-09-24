(function () {
  'use strict';

  /* @__NO_SIDE_EFFECTS__ */
  function getBrowserState() {
  	// @ts-expect-error not typed global
  	return window.__vitest_browser_runner__;
  }

  const config = getBrowserState().config;
  const sessionId = getBrowserState().sessionId;
  const state = {
  	ctx: {
  		pool: "browser",
  		worker: "./browser.js",
  		workerId: 1,
  		config,
  		projectName: config.name || "",
  		files: [],
  		environment: {
  			name: "browser",
  			options: null
  		},
  		providedContext: {},
  		invalidates: []
  	},
  	onCancel: null,
  	config,
  	environment: {
  		name: "browser",
  		transformMode: "web",
  		setup() {
  			throw new Error("Not called in the browser");
  		}
  	},
  	onCleanup: (fn) => getBrowserState().cleanups.push(fn),
  	moduleCache: getBrowserState().moduleCache,
  	rpc: null,
  	durations: {
  		environment: 0,
  		prepare: performance.now()
  	},
  	providedContext: {}
  };
  // @ts-expect-error not typed global
  globalThis.__vitest_browser__ = true;
  // @ts-expect-error not typed global
  globalThis.__vitest_worker__ = state;
  getBrowserState().cdp = createCdp();
  function rpc() {
  	return state.rpc;
  }
  function createCdp() {
  	const listenersMap = new WeakMap();
  	function getId(listener) {
  		const id = listenersMap.get(listener) || crypto.randomUUID();
  		listenersMap.set(listener, id);
  		return id;
  	}
  	const listeners = {};
  	const cdp = {
  		send(method, params) {
  			return rpc().sendCdpEvent(sessionId, method, params);
  		},
  		on(event, listener) {
  			const listenerId = getId(listener);
  			listeners[event] = listeners[event] || [];
  			listeners[event].push(listener);
  			rpc().trackCdpEvent(sessionId, "on", event, listenerId).catch(error);
  			return cdp;
  		},
  		once(event, listener) {
  			const listenerId = getId(listener);
  			const handler = (data) => {
  				listener(data);
  				cdp.off(event, listener);
  			};
  			listeners[event] = listeners[event] || [];
  			listeners[event].push(handler);
  			rpc().trackCdpEvent(sessionId, "once", event, listenerId).catch(error);
  			return cdp;
  		},
  		off(event, listener) {
  			const listenerId = getId(listener);
  			if (listeners[event]) {
  				listeners[event] = listeners[event].filter((l) => l !== listener);
  			}
  			rpc().trackCdpEvent(sessionId, "off", event, listenerId).catch(error);
  			return cdp;
  		},
  		emit(event, payload) {
  			if (listeners[event]) {
  				listeners[event].forEach((l) => {
  					try {
  						l(payload);
  					} catch (err) {
  						error(err);
  					}
  				});
  			}
  		}
  	};
  	return cdp;
  }
  function error(err) {
  	window.dispatchEvent(new ErrorEvent("error", { error: err }));
  }

})();
