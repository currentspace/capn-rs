var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
import { b as resolve, e as executor, c as getWorkerState, a as getConfig, g as getBrowserState } from "./utils-Owv5OOOf.js";
import { onCancel, globalChannel, channel, client } from "@vitest/browser/client";
import { userEvent, page, server } from "@vitest/browser/context";
import { getSafeTimers, TraceMap, originalPositionFor, loadDiffConfig, loadSnapshotSerializers, takeCoverageInsideWorker, stringify, format, setupCommonEnv, startCoverageInsideWorker, stopCoverageInsideWorker, startTests, collectTests, SpyModule } from "vitest/internal/browser";
import { VitestTestRunner, NodeBenchmarkRunner } from "vitest/runners";
const scriptRel = "modulepreload";
const assetsURL = function(dep) {
  return "/" + dep;
};
const seen = {};
const __vitePreload = function preload(baseModule, deps, importerUrl) {
  let promise = Promise.resolve();
  if (deps && deps.length > 0) {
    let allSettled2 = function(promises2) {
      return Promise.all(
        promises2.map(
          (p2) => Promise.resolve(p2).then(
            (value) => ({ status: "fulfilled", value }),
            (reason) => ({ status: "rejected", reason })
          )
        )
      );
    };
    document.getElementsByTagName("link");
    const cspNonceMeta = document.querySelector(
      "meta[property=csp-nonce]"
    );
    const cspNonce = (cspNonceMeta == null ? void 0 : cspNonceMeta.nonce) || (cspNonceMeta == null ? void 0 : cspNonceMeta.getAttribute("nonce"));
    promise = allSettled2(
      deps.map((dep) => {
        dep = assetsURL(dep);
        if (dep in seen) return;
        seen[dep] = true;
        const isCss = dep.endsWith(".css");
        const cssSelector = isCss ? '[rel="stylesheet"]' : "";
        if (document.querySelector(`link[href="${dep}"]${cssSelector}`)) {
          return;
        }
        const link = document.createElement("link");
        link.rel = isCss ? "stylesheet" : scriptRel;
        if (!isCss) {
          link.as = "script";
        }
        link.crossOrigin = "";
        link.href = dep;
        if (cspNonce) {
          link.setAttribute("nonce", cspNonce);
        }
        document.head.appendChild(link);
        if (isCss) {
          return new Promise((res, rej) => {
            link.addEventListener("load", res);
            link.addEventListener(
              "error",
              () => rej(new Error(`Unable to preload CSS for ${dep}`))
            );
          });
        }
      })
    );
  }
  function handlePreloadError(err) {
    const e = new Event("vite:preloadError", {
      cancelable: true
    });
    e.payload = err;
    window.dispatchEvent(e);
    if (!e.defaultPrevented) {
      throw err;
    }
  }
  return promise.then((res) => {
    for (const item of res || []) {
      if (item.status !== "rejected") continue;
      handlePreloadError(item.reason);
    }
    return baseModule().catch(handlePreloadError);
  });
};
const { parse: $parse } = JSON;
const { keys } = Object;
const Primitive = String;
const primitive = "string";
const ignore = {};
const object = "object";
const noop = (_, value) => value;
const primitives = (value) => value instanceof Primitive ? Primitive(value) : value;
const Primitives = (_, value) => typeof value === primitive ? new Primitive(value) : value;
const revive = (input, parsed, output, $) => {
  const lazy = [];
  for (let ke = keys(output), { length } = ke, y = 0; y < length; y++) {
    const k = ke[y];
    const value = output[k];
    if (value instanceof Primitive) {
      const tmp = input[value];
      if (typeof tmp === object && !parsed.has(tmp)) {
        parsed.add(tmp);
        output[k] = ignore;
        lazy.push({ k, a: [input, parsed, tmp, $] });
      } else
        output[k] = $.call(output, k, tmp);
    } else if (output[k] !== ignore)
      output[k] = $.call(output, k, value);
  }
  for (let { length } = lazy, i = 0; i < length; i++) {
    const { k, a: a2 } = lazy[i];
    output[k] = $.call(output, k, revive.apply(null, a2));
  }
  return output;
};
const parse = (text, reviver) => {
  const input = $parse(text, Primitives).map(primitives);
  const value = input[0];
  const $ = noop;
  const tmp = typeof value === object && value ? revive(input, /* @__PURE__ */ new Set(), value, $) : value;
  return $.call({ "": tmp }, "", tmp);
};
function showPopupWarning(name, value, defaultValue) {
  return (...params) => {
    const formattedParams = params.map((p2) => JSON.stringify(p2)).join(", ");
    console.warn(`Vitest encountered a \`${name}(${formattedParams})\` call that it cannot handle by default, so it returned \`${value}\`. Read more in https://vitest.dev/guide/browser/#thread-blocking-dialogs.
If needed, mock the \`${name}\` call manually like:

\`\`\`
import { expect, vi } from "vitest"

vi.spyOn(window, "${name}")${defaultValue ? `.mockReturnValue(${JSON.stringify(defaultValue)})` : ""}
${name}(${formattedParams})
expect(${name}).toHaveBeenCalledWith(${formattedParams})
\`\`\``);
    return value;
  };
}
function setupDialogsSpy() {
  globalThis.alert = showPopupWarning("alert", void 0);
  globalThis.confirm = showPopupWarning("confirm", false, true);
  globalThis.prompt = showPopupWarning("prompt", null, "your value");
}
const { get } = Reflect;
function withSafeTimers(getTimers, fn) {
  const { setTimeout, clearTimeout, setImmediate, clearImmediate } = getTimers();
  const currentSetTimeout = globalThis.setTimeout;
  const currentClearTimeout = globalThis.clearTimeout;
  const currentSetImmediate = globalThis.setImmediate;
  const currentClearImmediate = globalThis.clearImmediate;
  try {
    globalThis.setTimeout = setTimeout;
    globalThis.clearTimeout = clearTimeout;
    globalThis.setImmediate = setImmediate;
    globalThis.clearImmediate = clearImmediate;
    const result = fn();
    return result;
  } finally {
    globalThis.setTimeout = currentSetTimeout;
    globalThis.clearTimeout = currentClearTimeout;
    globalThis.setImmediate = currentSetImmediate;
    globalThis.clearImmediate = currentClearImmediate;
  }
}
const promises = /* @__PURE__ */ new Set();
function createSafeRpc(client2) {
  return new Proxy(client2.rpc, {
    get(target, p2, handler) {
      if (p2 === "then") {
        return;
      }
      const sendCall = get(target, p2, handler);
      const safeSendCall = (...args) => withSafeTimers(getSafeTimers, async () => {
        const result = sendCall(...args);
        promises.add(result);
        try {
          return await result;
        } finally {
          promises.delete(result);
        }
      });
      safeSendCall.asEvent = sendCall.asEvent;
      return safeSendCall;
    }
  });
}
function rpc$2() {
  return globalThis.__vitest_worker__.rpc;
}
var traceMapping_umd$1 = { exports: {} };
var sourcemapCodec_umd$1 = { exports: {} };
var sourcemapCodec_umd = sourcemapCodec_umd$1.exports;
var hasRequiredSourcemapCodec_umd;
function requireSourcemapCodec_umd() {
  if (hasRequiredSourcemapCodec_umd) return sourcemapCodec_umd$1.exports;
  hasRequiredSourcemapCodec_umd = 1;
  (function(module, exports) {
    (function(global, factory) {
      factory(exports);
    })(sourcemapCodec_umd, function(exports2) {
      const comma = ",".charCodeAt(0);
      const semicolon = ";".charCodeAt(0);
      const chars2 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
      const intToChar2 = new Uint8Array(64);
      const charToInt2 = new Uint8Array(128);
      for (let i = 0; i < chars2.length; i++) {
        const c = chars2.charCodeAt(i);
        intToChar2[i] = c;
        charToInt2[c] = i;
      }
      function decodeInteger(reader, relative) {
        let value = 0;
        let shift = 0;
        let integer = 0;
        do {
          const c = reader.next();
          integer = charToInt2[c];
          value |= (integer & 31) << shift;
          shift += 5;
        } while (integer & 32);
        const shouldNegate = value & 1;
        value >>>= 1;
        if (shouldNegate) {
          value = -2147483648 | -value;
        }
        return relative + value;
      }
      function encodeInteger(builder, num, relative) {
        let delta = num - relative;
        delta = delta < 0 ? -delta << 1 | 1 : delta << 1;
        do {
          let clamped = delta & 31;
          delta >>>= 5;
          if (delta > 0)
            clamped |= 32;
          builder.write(intToChar2[clamped]);
        } while (delta > 0);
        return num;
      }
      function hasMoreVlq(reader, max) {
        if (reader.pos >= max)
          return false;
        return reader.peek() !== comma;
      }
      const bufLength = 1024 * 16;
      const td = typeof TextDecoder !== "undefined" ? /* @__PURE__ */ new TextDecoder() : typeof Buffer !== "undefined" ? {
        decode(buf) {
          const out = Buffer.from(buf.buffer, buf.byteOffset, buf.byteLength);
          return out.toString();
        }
      } : {
        decode(buf) {
          let out = "";
          for (let i = 0; i < buf.length; i++) {
            out += String.fromCharCode(buf[i]);
          }
          return out;
        }
      };
      class StringWriter {
        constructor() {
          this.pos = 0;
          this.out = "";
          this.buffer = new Uint8Array(bufLength);
        }
        write(v) {
          const { buffer } = this;
          buffer[this.pos++] = v;
          if (this.pos === bufLength) {
            this.out += td.decode(buffer);
            this.pos = 0;
          }
        }
        flush() {
          const { buffer, out, pos } = this;
          return pos > 0 ? out + td.decode(buffer.subarray(0, pos)) : out;
        }
      }
      class StringReader {
        constructor(buffer) {
          this.pos = 0;
          this.buffer = buffer;
        }
        next() {
          return this.buffer.charCodeAt(this.pos++);
        }
        peek() {
          return this.buffer.charCodeAt(this.pos);
        }
        indexOf(char) {
          const { buffer, pos } = this;
          const idx = buffer.indexOf(char, pos);
          return idx === -1 ? buffer.length : idx;
        }
      }
      const EMPTY = [];
      function decodeOriginalScopes(input) {
        const { length } = input;
        const reader = new StringReader(input);
        const scopes = [];
        const stack = [];
        let line = 0;
        for (; reader.pos < length; reader.pos++) {
          line = decodeInteger(reader, line);
          const column = decodeInteger(reader, 0);
          if (!hasMoreVlq(reader, length)) {
            const last = stack.pop();
            last[2] = line;
            last[3] = column;
            continue;
          }
          const kind = decodeInteger(reader, 0);
          const fields = decodeInteger(reader, 0);
          const hasName = fields & 1;
          const scope = hasName ? [line, column, 0, 0, kind, decodeInteger(reader, 0)] : [line, column, 0, 0, kind];
          let vars = EMPTY;
          if (hasMoreVlq(reader, length)) {
            vars = [];
            do {
              const varsIndex = decodeInteger(reader, 0);
              vars.push(varsIndex);
            } while (hasMoreVlq(reader, length));
          }
          scope.vars = vars;
          scopes.push(scope);
          stack.push(scope);
        }
        return scopes;
      }
      function encodeOriginalScopes(scopes) {
        const writer = new StringWriter();
        for (let i = 0; i < scopes.length; ) {
          i = _encodeOriginalScopes(scopes, i, writer, [0]);
        }
        return writer.flush();
      }
      function _encodeOriginalScopes(scopes, index2, writer, state) {
        const scope = scopes[index2];
        const { 0: startLine, 1: startColumn, 2: endLine, 3: endColumn, 4: kind, vars } = scope;
        if (index2 > 0)
          writer.write(comma);
        state[0] = encodeInteger(writer, startLine, state[0]);
        encodeInteger(writer, startColumn, 0);
        encodeInteger(writer, kind, 0);
        const fields = scope.length === 6 ? 1 : 0;
        encodeInteger(writer, fields, 0);
        if (scope.length === 6)
          encodeInteger(writer, scope[5], 0);
        for (const v of vars) {
          encodeInteger(writer, v, 0);
        }
        for (index2++; index2 < scopes.length; ) {
          const next = scopes[index2];
          const { 0: l, 1: c } = next;
          if (l > endLine || l === endLine && c >= endColumn) {
            break;
          }
          index2 = _encodeOriginalScopes(scopes, index2, writer, state);
        }
        writer.write(comma);
        state[0] = encodeInteger(writer, endLine, state[0]);
        encodeInteger(writer, endColumn, 0);
        return index2;
      }
      function decodeGeneratedRanges(input) {
        const { length } = input;
        const reader = new StringReader(input);
        const ranges = [];
        const stack = [];
        let genLine = 0;
        let definitionSourcesIndex = 0;
        let definitionScopeIndex = 0;
        let callsiteSourcesIndex = 0;
        let callsiteLine = 0;
        let callsiteColumn = 0;
        let bindingLine = 0;
        let bindingColumn = 0;
        do {
          const semi = reader.indexOf(";");
          let genColumn = 0;
          for (; reader.pos < semi; reader.pos++) {
            genColumn = decodeInteger(reader, genColumn);
            if (!hasMoreVlq(reader, semi)) {
              const last = stack.pop();
              last[2] = genLine;
              last[3] = genColumn;
              continue;
            }
            const fields = decodeInteger(reader, 0);
            const hasDefinition = fields & 1;
            const hasCallsite = fields & 2;
            const hasScope = fields & 4;
            let callsite = null;
            let bindings = EMPTY;
            let range;
            if (hasDefinition) {
              const defSourcesIndex = decodeInteger(reader, definitionSourcesIndex);
              definitionScopeIndex = decodeInteger(reader, definitionSourcesIndex === defSourcesIndex ? definitionScopeIndex : 0);
              definitionSourcesIndex = defSourcesIndex;
              range = [genLine, genColumn, 0, 0, defSourcesIndex, definitionScopeIndex];
            } else {
              range = [genLine, genColumn, 0, 0];
            }
            range.isScope = !!hasScope;
            if (hasCallsite) {
              const prevCsi = callsiteSourcesIndex;
              const prevLine = callsiteLine;
              callsiteSourcesIndex = decodeInteger(reader, callsiteSourcesIndex);
              const sameSource = prevCsi === callsiteSourcesIndex;
              callsiteLine = decodeInteger(reader, sameSource ? callsiteLine : 0);
              callsiteColumn = decodeInteger(reader, sameSource && prevLine === callsiteLine ? callsiteColumn : 0);
              callsite = [callsiteSourcesIndex, callsiteLine, callsiteColumn];
            }
            range.callsite = callsite;
            if (hasMoreVlq(reader, semi)) {
              bindings = [];
              do {
                bindingLine = genLine;
                bindingColumn = genColumn;
                const expressionsCount = decodeInteger(reader, 0);
                let expressionRanges;
                if (expressionsCount < -1) {
                  expressionRanges = [[decodeInteger(reader, 0)]];
                  for (let i = -1; i > expressionsCount; i--) {
                    const prevBl = bindingLine;
                    bindingLine = decodeInteger(reader, bindingLine);
                    bindingColumn = decodeInteger(reader, bindingLine === prevBl ? bindingColumn : 0);
                    const expression = decodeInteger(reader, 0);
                    expressionRanges.push([expression, bindingLine, bindingColumn]);
                  }
                } else {
                  expressionRanges = [[expressionsCount]];
                }
                bindings.push(expressionRanges);
              } while (hasMoreVlq(reader, semi));
            }
            range.bindings = bindings;
            ranges.push(range);
            stack.push(range);
          }
          genLine++;
          reader.pos = semi + 1;
        } while (reader.pos < length);
        return ranges;
      }
      function encodeGeneratedRanges(ranges) {
        if (ranges.length === 0)
          return "";
        const writer = new StringWriter();
        for (let i = 0; i < ranges.length; ) {
          i = _encodeGeneratedRanges(ranges, i, writer, [0, 0, 0, 0, 0, 0, 0]);
        }
        return writer.flush();
      }
      function _encodeGeneratedRanges(ranges, index2, writer, state) {
        const range = ranges[index2];
        const { 0: startLine, 1: startColumn, 2: endLine, 3: endColumn, isScope, callsite, bindings } = range;
        if (state[0] < startLine) {
          catchupLine(writer, state[0], startLine);
          state[0] = startLine;
          state[1] = 0;
        } else if (index2 > 0) {
          writer.write(comma);
        }
        state[1] = encodeInteger(writer, range[1], state[1]);
        const fields = (range.length === 6 ? 1 : 0) | (callsite ? 2 : 0) | (isScope ? 4 : 0);
        encodeInteger(writer, fields, 0);
        if (range.length === 6) {
          const { 4: sourcesIndex, 5: scopesIndex } = range;
          if (sourcesIndex !== state[2]) {
            state[3] = 0;
          }
          state[2] = encodeInteger(writer, sourcesIndex, state[2]);
          state[3] = encodeInteger(writer, scopesIndex, state[3]);
        }
        if (callsite) {
          const { 0: sourcesIndex, 1: callLine, 2: callColumn } = range.callsite;
          if (sourcesIndex !== state[4]) {
            state[5] = 0;
            state[6] = 0;
          } else if (callLine !== state[5]) {
            state[6] = 0;
          }
          state[4] = encodeInteger(writer, sourcesIndex, state[4]);
          state[5] = encodeInteger(writer, callLine, state[5]);
          state[6] = encodeInteger(writer, callColumn, state[6]);
        }
        if (bindings) {
          for (const binding of bindings) {
            if (binding.length > 1)
              encodeInteger(writer, -binding.length, 0);
            const expression = binding[0][0];
            encodeInteger(writer, expression, 0);
            let bindingStartLine = startLine;
            let bindingStartColumn = startColumn;
            for (let i = 1; i < binding.length; i++) {
              const expRange = binding[i];
              bindingStartLine = encodeInteger(writer, expRange[1], bindingStartLine);
              bindingStartColumn = encodeInteger(writer, expRange[2], bindingStartColumn);
              encodeInteger(writer, expRange[0], 0);
            }
          }
        }
        for (index2++; index2 < ranges.length; ) {
          const next = ranges[index2];
          const { 0: l, 1: c } = next;
          if (l > endLine || l === endLine && c >= endColumn) {
            break;
          }
          index2 = _encodeGeneratedRanges(ranges, index2, writer, state);
        }
        if (state[0] < endLine) {
          catchupLine(writer, state[0], endLine);
          state[0] = endLine;
          state[1] = 0;
        } else {
          writer.write(comma);
        }
        state[1] = encodeInteger(writer, endColumn, state[1]);
        return index2;
      }
      function catchupLine(writer, lastLine, line) {
        do {
          writer.write(semicolon);
        } while (++lastLine < line);
      }
      function decode(mappings) {
        const { length } = mappings;
        const reader = new StringReader(mappings);
        const decoded = [];
        let genColumn = 0;
        let sourcesIndex = 0;
        let sourceLine = 0;
        let sourceColumn = 0;
        let namesIndex = 0;
        do {
          const semi = reader.indexOf(";");
          const line = [];
          let sorted = true;
          let lastCol = 0;
          genColumn = 0;
          while (reader.pos < semi) {
            let seg;
            genColumn = decodeInteger(reader, genColumn);
            if (genColumn < lastCol)
              sorted = false;
            lastCol = genColumn;
            if (hasMoreVlq(reader, semi)) {
              sourcesIndex = decodeInteger(reader, sourcesIndex);
              sourceLine = decodeInteger(reader, sourceLine);
              sourceColumn = decodeInteger(reader, sourceColumn);
              if (hasMoreVlq(reader, semi)) {
                namesIndex = decodeInteger(reader, namesIndex);
                seg = [genColumn, sourcesIndex, sourceLine, sourceColumn, namesIndex];
              } else {
                seg = [genColumn, sourcesIndex, sourceLine, sourceColumn];
              }
            } else {
              seg = [genColumn];
            }
            line.push(seg);
            reader.pos++;
          }
          if (!sorted)
            sort(line);
          decoded.push(line);
          reader.pos = semi + 1;
        } while (reader.pos <= length);
        return decoded;
      }
      function sort(line) {
        line.sort(sortComparator);
      }
      function sortComparator(a2, b) {
        return a2[0] - b[0];
      }
      function encode(decoded) {
        const writer = new StringWriter();
        let sourcesIndex = 0;
        let sourceLine = 0;
        let sourceColumn = 0;
        let namesIndex = 0;
        for (let i = 0; i < decoded.length; i++) {
          const line = decoded[i];
          if (i > 0)
            writer.write(semicolon);
          if (line.length === 0)
            continue;
          let genColumn = 0;
          for (let j = 0; j < line.length; j++) {
            const segment = line[j];
            if (j > 0)
              writer.write(comma);
            genColumn = encodeInteger(writer, segment[0], genColumn);
            if (segment.length === 1)
              continue;
            sourcesIndex = encodeInteger(writer, segment[1], sourcesIndex);
            sourceLine = encodeInteger(writer, segment[2], sourceLine);
            sourceColumn = encodeInteger(writer, segment[3], sourceColumn);
            if (segment.length === 4)
              continue;
            namesIndex = encodeInteger(writer, segment[4], namesIndex);
          }
        }
        return writer.flush();
      }
      exports2.decode = decode;
      exports2.decodeGeneratedRanges = decodeGeneratedRanges;
      exports2.decodeOriginalScopes = decodeOriginalScopes;
      exports2.encode = encode;
      exports2.encodeGeneratedRanges = encodeGeneratedRanges;
      exports2.encodeOriginalScopes = encodeOriginalScopes;
      Object.defineProperty(exports2, "__esModule", { value: true });
    });
  })(sourcemapCodec_umd$1, sourcemapCodec_umd$1.exports);
  return sourcemapCodec_umd$1.exports;
}
var resolveUri_umd$1 = { exports: {} };
var resolveUri_umd = resolveUri_umd$1.exports;
var hasRequiredResolveUri_umd;
function requireResolveUri_umd() {
  if (hasRequiredResolveUri_umd) return resolveUri_umd$1.exports;
  hasRequiredResolveUri_umd = 1;
  (function(module, exports) {
    (function(global, factory) {
      module.exports = factory();
    })(resolveUri_umd, function() {
      const schemeRegex = /^[\w+.-]+:\/\//;
      const urlRegex = /^([\w+.-]+:)\/\/([^@/#?]*@)?([^:/#?]*)(:\d+)?(\/[^#?]*)?(\?[^#]*)?(#.*)?/;
      const fileRegex = /^file:(?:\/\/((?![a-z]:)[^/#?]*)?)?(\/?[^#?]*)(\?[^#]*)?(#.*)?/i;
      var UrlType2;
      (function(UrlType3) {
        UrlType3[UrlType3["Empty"] = 1] = "Empty";
        UrlType3[UrlType3["Hash"] = 2] = "Hash";
        UrlType3[UrlType3["Query"] = 3] = "Query";
        UrlType3[UrlType3["RelativePath"] = 4] = "RelativePath";
        UrlType3[UrlType3["AbsolutePath"] = 5] = "AbsolutePath";
        UrlType3[UrlType3["SchemeRelative"] = 6] = "SchemeRelative";
        UrlType3[UrlType3["Absolute"] = 7] = "Absolute";
      })(UrlType2 || (UrlType2 = {}));
      function isAbsoluteUrl(input) {
        return schemeRegex.test(input);
      }
      function isSchemeRelativeUrl(input) {
        return input.startsWith("//");
      }
      function isAbsolutePath(input) {
        return input.startsWith("/");
      }
      function isFileUrl(input) {
        return input.startsWith("file:");
      }
      function isRelative(input) {
        return /^[.?#]/.test(input);
      }
      function parseAbsoluteUrl(input) {
        const match = urlRegex.exec(input);
        return makeUrl(match[1], match[2] || "", match[3], match[4] || "", match[5] || "/", match[6] || "", match[7] || "");
      }
      function parseFileUrl(input) {
        const match = fileRegex.exec(input);
        const path = match[2];
        return makeUrl("file:", "", match[1] || "", "", isAbsolutePath(path) ? path : "/" + path, match[3] || "", match[4] || "");
      }
      function makeUrl(scheme, user, host, port, path, query, hash) {
        return {
          scheme,
          user,
          host,
          port,
          path,
          query,
          hash,
          type: UrlType2.Absolute
        };
      }
      function parseUrl(input) {
        if (isSchemeRelativeUrl(input)) {
          const url3 = parseAbsoluteUrl("http:" + input);
          url3.scheme = "";
          url3.type = UrlType2.SchemeRelative;
          return url3;
        }
        if (isAbsolutePath(input)) {
          const url3 = parseAbsoluteUrl("http://foo.com" + input);
          url3.scheme = "";
          url3.host = "";
          url3.type = UrlType2.AbsolutePath;
          return url3;
        }
        if (isFileUrl(input))
          return parseFileUrl(input);
        if (isAbsoluteUrl(input))
          return parseAbsoluteUrl(input);
        const url2 = parseAbsoluteUrl("http://foo.com/" + input);
        url2.scheme = "";
        url2.host = "";
        url2.type = input ? input.startsWith("?") ? UrlType2.Query : input.startsWith("#") ? UrlType2.Hash : UrlType2.RelativePath : UrlType2.Empty;
        return url2;
      }
      function stripPathFilename(path) {
        if (path.endsWith("/.."))
          return path;
        const index2 = path.lastIndexOf("/");
        return path.slice(0, index2 + 1);
      }
      function mergePaths(url2, base) {
        normalizePath(base, base.type);
        if (url2.path === "/") {
          url2.path = base.path;
        } else {
          url2.path = stripPathFilename(base.path) + url2.path;
        }
      }
      function normalizePath(url2, type) {
        const rel = type <= UrlType2.RelativePath;
        const pieces = url2.path.split("/");
        let pointer = 1;
        let positive = 0;
        let addTrailingSlash = false;
        for (let i = 1; i < pieces.length; i++) {
          const piece = pieces[i];
          if (!piece) {
            addTrailingSlash = true;
            continue;
          }
          addTrailingSlash = false;
          if (piece === ".")
            continue;
          if (piece === "..") {
            if (positive) {
              addTrailingSlash = true;
              positive--;
              pointer--;
            } else if (rel) {
              pieces[pointer++] = piece;
            }
            continue;
          }
          pieces[pointer++] = piece;
          positive++;
        }
        let path = "";
        for (let i = 1; i < pointer; i++) {
          path += "/" + pieces[i];
        }
        if (!path || addTrailingSlash && !path.endsWith("/..")) {
          path += "/";
        }
        url2.path = path;
      }
      function resolve2(input, base) {
        if (!input && !base)
          return "";
        const url2 = parseUrl(input);
        let inputType = url2.type;
        if (base && inputType !== UrlType2.Absolute) {
          const baseUrl = parseUrl(base);
          const baseType = baseUrl.type;
          switch (inputType) {
            case UrlType2.Empty:
              url2.hash = baseUrl.hash;
            // fall through
            case UrlType2.Hash:
              url2.query = baseUrl.query;
            // fall through
            case UrlType2.Query:
            case UrlType2.RelativePath:
              mergePaths(url2, baseUrl);
            // fall through
            case UrlType2.AbsolutePath:
              url2.user = baseUrl.user;
              url2.host = baseUrl.host;
              url2.port = baseUrl.port;
            // fall through
            case UrlType2.SchemeRelative:
              url2.scheme = baseUrl.scheme;
          }
          if (baseType > inputType)
            inputType = baseType;
        }
        normalizePath(url2, inputType);
        const queryHash = url2.query + url2.hash;
        switch (inputType) {
          // This is impossible, because of the empty checks at the start of the function.
          // case UrlType.Empty:
          case UrlType2.Hash:
          case UrlType2.Query:
            return queryHash;
          case UrlType2.RelativePath: {
            const path = url2.path.slice(1);
            if (!path)
              return queryHash || ".";
            if (isRelative(base || input) && !isRelative(path)) {
              return "./" + path + queryHash;
            }
            return path + queryHash;
          }
          case UrlType2.AbsolutePath:
            return url2.path + queryHash;
          default:
            return url2.scheme + "//" + url2.user + url2.host + url2.port + url2.path + queryHash;
        }
      }
      return resolve2;
    });
  })(resolveUri_umd$1);
  return resolveUri_umd$1.exports;
}
var traceMapping_umd = traceMapping_umd$1.exports;
var hasRequiredTraceMapping_umd;
function requireTraceMapping_umd() {
  if (hasRequiredTraceMapping_umd) return traceMapping_umd$1.exports;
  hasRequiredTraceMapping_umd = 1;
  (function(module, exports) {
    (function(global, factory) {
      factory(exports, requireSourcemapCodec_umd(), requireResolveUri_umd());
    })(traceMapping_umd, function(exports2, sourcemapCodec, resolveUri) {
      function resolve2(input, base) {
        if (base && !base.endsWith("/"))
          base += "/";
        return resolveUri(input, base);
      }
      function stripFilename(path) {
        if (!path)
          return "";
        const index2 = path.lastIndexOf("/");
        return path.slice(0, index2 + 1);
      }
      const COLUMN = 0;
      const SOURCES_INDEX = 1;
      const SOURCE_LINE = 2;
      const SOURCE_COLUMN = 3;
      const NAMES_INDEX = 4;
      const REV_GENERATED_LINE = 1;
      const REV_GENERATED_COLUMN = 2;
      function maybeSort(mappings, owned) {
        const unsortedIndex = nextUnsortedSegmentLine(mappings, 0);
        if (unsortedIndex === mappings.length)
          return mappings;
        if (!owned)
          mappings = mappings.slice();
        for (let i = unsortedIndex; i < mappings.length; i = nextUnsortedSegmentLine(mappings, i + 1)) {
          mappings[i] = sortSegments(mappings[i], owned);
        }
        return mappings;
      }
      function nextUnsortedSegmentLine(mappings, start) {
        for (let i = start; i < mappings.length; i++) {
          if (!isSorted(mappings[i]))
            return i;
        }
        return mappings.length;
      }
      function isSorted(line) {
        for (let j = 1; j < line.length; j++) {
          if (line[j][COLUMN] < line[j - 1][COLUMN]) {
            return false;
          }
        }
        return true;
      }
      function sortSegments(line, owned) {
        if (!owned)
          line = line.slice();
        return line.sort(sortComparator);
      }
      function sortComparator(a2, b) {
        return a2[COLUMN] - b[COLUMN];
      }
      let found = false;
      function binarySearch(haystack, needle, low, high) {
        while (low <= high) {
          const mid = low + (high - low >> 1);
          const cmp = haystack[mid][COLUMN] - needle;
          if (cmp === 0) {
            found = true;
            return mid;
          }
          if (cmp < 0) {
            low = mid + 1;
          } else {
            high = mid - 1;
          }
        }
        found = false;
        return low - 1;
      }
      function upperBound(haystack, needle, index2) {
        for (let i = index2 + 1; i < haystack.length; index2 = i++) {
          if (haystack[i][COLUMN] !== needle)
            break;
        }
        return index2;
      }
      function lowerBound(haystack, needle, index2) {
        for (let i = index2 - 1; i >= 0; index2 = i--) {
          if (haystack[i][COLUMN] !== needle)
            break;
        }
        return index2;
      }
      function memoizedState() {
        return {
          lastKey: -1,
          lastNeedle: -1,
          lastIndex: -1
        };
      }
      function memoizedBinarySearch(haystack, needle, state, key) {
        const { lastKey, lastNeedle, lastIndex } = state;
        let low = 0;
        let high = haystack.length - 1;
        if (key === lastKey) {
          if (needle === lastNeedle) {
            found = lastIndex !== -1 && haystack[lastIndex][COLUMN] === needle;
            return lastIndex;
          }
          if (needle >= lastNeedle) {
            low = lastIndex === -1 ? 0 : lastIndex;
          } else {
            high = lastIndex;
          }
        }
        state.lastKey = key;
        state.lastNeedle = needle;
        return state.lastIndex = binarySearch(haystack, needle, low, high);
      }
      function buildBySources(decoded, memos) {
        const sources = memos.map(buildNullArray);
        for (let i = 0; i < decoded.length; i++) {
          const line = decoded[i];
          for (let j = 0; j < line.length; j++) {
            const seg = line[j];
            if (seg.length === 1)
              continue;
            const sourceIndex2 = seg[SOURCES_INDEX];
            const sourceLine = seg[SOURCE_LINE];
            const sourceColumn = seg[SOURCE_COLUMN];
            const originalSource = sources[sourceIndex2];
            const originalLine = originalSource[sourceLine] || (originalSource[sourceLine] = []);
            const memo = memos[sourceIndex2];
            let index2 = upperBound(originalLine, sourceColumn, memoizedBinarySearch(originalLine, sourceColumn, memo, sourceLine));
            memo.lastIndex = ++index2;
            insert(originalLine, index2, [sourceColumn, i, seg[COLUMN]]);
          }
        }
        return sources;
      }
      function insert(array, index2, value) {
        for (let i = array.length; i > index2; i--) {
          array[i] = array[i - 1];
        }
        array[index2] = value;
      }
      function buildNullArray() {
        return { __proto__: null };
      }
      const AnyMap = function(map, mapUrl) {
        const parsed = parse2(map);
        if (!("sections" in parsed)) {
          return new TraceMap2(parsed, mapUrl);
        }
        const mappings = [];
        const sources = [];
        const sourcesContent = [];
        const names = [];
        const ignoreList = [];
        recurse(parsed, mapUrl, mappings, sources, sourcesContent, names, ignoreList, 0, 0, Infinity, Infinity);
        const joined = {
          version: 3,
          file: parsed.file,
          names,
          sources,
          sourcesContent,
          mappings,
          ignoreList
        };
        return presortedDecodedMap(joined);
      };
      function parse2(map) {
        return typeof map === "string" ? JSON.parse(map) : map;
      }
      function recurse(input, mapUrl, mappings, sources, sourcesContent, names, ignoreList, lineOffset, columnOffset, stopLine, stopColumn) {
        const { sections } = input;
        for (let i = 0; i < sections.length; i++) {
          const { map, offset } = sections[i];
          let sl = stopLine;
          let sc = stopColumn;
          if (i + 1 < sections.length) {
            const nextOffset = sections[i + 1].offset;
            sl = Math.min(stopLine, lineOffset + nextOffset.line);
            if (sl === stopLine) {
              sc = Math.min(stopColumn, columnOffset + nextOffset.column);
            } else if (sl < stopLine) {
              sc = columnOffset + nextOffset.column;
            }
          }
          addSection(map, mapUrl, mappings, sources, sourcesContent, names, ignoreList, lineOffset + offset.line, columnOffset + offset.column, sl, sc);
        }
      }
      function addSection(input, mapUrl, mappings, sources, sourcesContent, names, ignoreList, lineOffset, columnOffset, stopLine, stopColumn) {
        const parsed = parse2(input);
        if ("sections" in parsed)
          return recurse(...arguments);
        const map = new TraceMap2(parsed, mapUrl);
        const sourcesOffset = sources.length;
        const namesOffset = names.length;
        const decoded = decodedMappings(map);
        const { resolvedSources, sourcesContent: contents, ignoreList: ignores } = map;
        append(sources, resolvedSources);
        append(names, map.names);
        if (contents)
          append(sourcesContent, contents);
        else
          for (let i = 0; i < resolvedSources.length; i++)
            sourcesContent.push(null);
        if (ignores)
          for (let i = 0; i < ignores.length; i++)
            ignoreList.push(ignores[i] + sourcesOffset);
        for (let i = 0; i < decoded.length; i++) {
          const lineI = lineOffset + i;
          if (lineI > stopLine)
            return;
          const out = getLine(mappings, lineI);
          const cOffset = i === 0 ? columnOffset : 0;
          const line = decoded[i];
          for (let j = 0; j < line.length; j++) {
            const seg = line[j];
            const column = cOffset + seg[COLUMN];
            if (lineI === stopLine && column >= stopColumn)
              return;
            if (seg.length === 1) {
              out.push([column]);
              continue;
            }
            const sourcesIndex = sourcesOffset + seg[SOURCES_INDEX];
            const sourceLine = seg[SOURCE_LINE];
            const sourceColumn = seg[SOURCE_COLUMN];
            out.push(seg.length === 4 ? [column, sourcesIndex, sourceLine, sourceColumn] : [column, sourcesIndex, sourceLine, sourceColumn, namesOffset + seg[NAMES_INDEX]]);
          }
        }
      }
      function append(arr, other) {
        for (let i = 0; i < other.length; i++)
          arr.push(other[i]);
      }
      function getLine(arr, index2) {
        for (let i = arr.length; i <= index2; i++)
          arr[i] = [];
        return arr[index2];
      }
      const LINE_GTR_ZERO = "`line` must be greater than 0 (lines start at line 1)";
      const COL_GTR_EQ_ZERO = "`column` must be greater than or equal to 0 (columns start at column 0)";
      const LEAST_UPPER_BOUND = -1;
      const GREATEST_LOWER_BOUND = 1;
      class TraceMap2 {
        constructor(map, mapUrl) {
          const isString = typeof map === "string";
          if (!isString && map._decodedMemo)
            return map;
          const parsed = isString ? JSON.parse(map) : map;
          const { version, file, names, sourceRoot, sources, sourcesContent } = parsed;
          this.version = version;
          this.file = file;
          this.names = names || [];
          this.sourceRoot = sourceRoot;
          this.sources = sources;
          this.sourcesContent = sourcesContent;
          this.ignoreList = parsed.ignoreList || parsed.x_google_ignoreList || void 0;
          const from = resolve2(sourceRoot || "", stripFilename(mapUrl));
          this.resolvedSources = sources.map((s) => resolve2(s || "", from));
          const { mappings } = parsed;
          if (typeof mappings === "string") {
            this._encoded = mappings;
            this._decoded = void 0;
          } else {
            this._encoded = void 0;
            this._decoded = maybeSort(mappings, isString);
          }
          this._decodedMemo = memoizedState();
          this._bySources = void 0;
          this._bySourceMemos = void 0;
        }
      }
      function cast(map) {
        return map;
      }
      function encodedMappings(map) {
        var _a;
        var _b;
        return (_a = (_b = cast(map))._encoded) !== null && _a !== void 0 ? _a : _b._encoded = sourcemapCodec.encode(cast(map)._decoded);
      }
      function decodedMappings(map) {
        var _a;
        return (_a = cast(map))._decoded || (_a._decoded = sourcemapCodec.decode(cast(map)._encoded));
      }
      function traceSegment(map, line, column) {
        const decoded = decodedMappings(map);
        if (line >= decoded.length)
          return null;
        const segments = decoded[line];
        const index2 = traceSegmentInternal(segments, cast(map)._decodedMemo, line, column, GREATEST_LOWER_BOUND);
        return index2 === -1 ? null : segments[index2];
      }
      function originalPositionFor2(map, needle) {
        let { line, column, bias } = needle;
        line--;
        if (line < 0)
          throw new Error(LINE_GTR_ZERO);
        if (column < 0)
          throw new Error(COL_GTR_EQ_ZERO);
        const decoded = decodedMappings(map);
        if (line >= decoded.length)
          return OMapping(null, null, null, null);
        const segments = decoded[line];
        const index2 = traceSegmentInternal(segments, cast(map)._decodedMemo, line, column, bias || GREATEST_LOWER_BOUND);
        if (index2 === -1)
          return OMapping(null, null, null, null);
        const segment = segments[index2];
        if (segment.length === 1)
          return OMapping(null, null, null, null);
        const { names, resolvedSources } = map;
        return OMapping(resolvedSources[segment[SOURCES_INDEX]], segment[SOURCE_LINE] + 1, segment[SOURCE_COLUMN], segment.length === 5 ? names[segment[NAMES_INDEX]] : null);
      }
      function generatedPositionFor(map, needle) {
        const { source, line, column, bias } = needle;
        return generatedPosition(map, source, line, column, bias || GREATEST_LOWER_BOUND, false);
      }
      function allGeneratedPositionsFor(map, needle) {
        const { source, line, column, bias } = needle;
        return generatedPosition(map, source, line, column, bias || LEAST_UPPER_BOUND, true);
      }
      function eachMapping(map, cb) {
        const decoded = decodedMappings(map);
        const { names, resolvedSources } = map;
        for (let i = 0; i < decoded.length; i++) {
          const line = decoded[i];
          for (let j = 0; j < line.length; j++) {
            const seg = line[j];
            const generatedLine = i + 1;
            const generatedColumn = seg[0];
            let source = null;
            let originalLine = null;
            let originalColumn = null;
            let name = null;
            if (seg.length !== 1) {
              source = resolvedSources[seg[1]];
              originalLine = seg[2] + 1;
              originalColumn = seg[3];
            }
            if (seg.length === 5)
              name = names[seg[4]];
            cb({
              generatedLine,
              generatedColumn,
              source,
              originalLine,
              originalColumn,
              name
            });
          }
        }
      }
      function sourceIndex(map, source) {
        const { sources, resolvedSources } = map;
        let index2 = sources.indexOf(source);
        if (index2 === -1)
          index2 = resolvedSources.indexOf(source);
        return index2;
      }
      function sourceContentFor(map, source) {
        const { sourcesContent } = map;
        if (sourcesContent == null)
          return null;
        const index2 = sourceIndex(map, source);
        return index2 === -1 ? null : sourcesContent[index2];
      }
      function isIgnored(map, source) {
        const { ignoreList } = map;
        if (ignoreList == null)
          return false;
        const index2 = sourceIndex(map, source);
        return index2 === -1 ? false : ignoreList.includes(index2);
      }
      function presortedDecodedMap(map, mapUrl) {
        const tracer = new TraceMap2(clone(map, []), mapUrl);
        cast(tracer)._decoded = map.mappings;
        return tracer;
      }
      function decodedMap(map) {
        return clone(map, decodedMappings(map));
      }
      function encodedMap(map) {
        return clone(map, encodedMappings(map));
      }
      function clone(map, mappings) {
        return {
          version: map.version,
          file: map.file,
          names: map.names,
          sourceRoot: map.sourceRoot,
          sources: map.sources,
          sourcesContent: map.sourcesContent,
          mappings,
          ignoreList: map.ignoreList || map.x_google_ignoreList
        };
      }
      function OMapping(source, line, column, name) {
        return { source, line, column, name };
      }
      function GMapping(line, column) {
        return { line, column };
      }
      function traceSegmentInternal(segments, memo, line, column, bias) {
        let index2 = memoizedBinarySearch(segments, column, memo, line);
        if (found) {
          index2 = (bias === LEAST_UPPER_BOUND ? upperBound : lowerBound)(segments, column, index2);
        } else if (bias === LEAST_UPPER_BOUND)
          index2++;
        if (index2 === -1 || index2 === segments.length)
          return -1;
        return index2;
      }
      function sliceGeneratedPositions(segments, memo, line, column, bias) {
        let min = traceSegmentInternal(segments, memo, line, column, GREATEST_LOWER_BOUND);
        if (!found && bias === LEAST_UPPER_BOUND)
          min++;
        if (min === -1 || min === segments.length)
          return [];
        const matchedColumn = found ? column : segments[min][COLUMN];
        if (!found)
          min = lowerBound(segments, matchedColumn, min);
        const max = upperBound(segments, matchedColumn, min);
        const result = [];
        for (; min <= max; min++) {
          const segment = segments[min];
          result.push(GMapping(segment[REV_GENERATED_LINE] + 1, segment[REV_GENERATED_COLUMN]));
        }
        return result;
      }
      function generatedPosition(map, source, line, column, bias, all) {
        var _a;
        line--;
        if (line < 0)
          throw new Error(LINE_GTR_ZERO);
        if (column < 0)
          throw new Error(COL_GTR_EQ_ZERO);
        const { sources, resolvedSources } = map;
        let sourceIndex2 = sources.indexOf(source);
        if (sourceIndex2 === -1)
          sourceIndex2 = resolvedSources.indexOf(source);
        if (sourceIndex2 === -1)
          return all ? [] : GMapping(null, null);
        const generated = (_a = cast(map))._bySources || (_a._bySources = buildBySources(decodedMappings(map), cast(map)._bySourceMemos = sources.map(memoizedState)));
        const segments = generated[sourceIndex2][line];
        if (segments == null)
          return all ? [] : GMapping(null, null);
        const memo = cast(map)._bySourceMemos[sourceIndex2];
        if (all)
          return sliceGeneratedPositions(segments, memo, line, column, bias);
        const index2 = traceSegmentInternal(segments, memo, line, column, bias);
        if (index2 === -1)
          return GMapping(null, null);
        const segment = segments[index2];
        return GMapping(segment[REV_GENERATED_LINE] + 1, segment[REV_GENERATED_COLUMN]);
      }
      exports2.AnyMap = AnyMap;
      exports2.GREATEST_LOWER_BOUND = GREATEST_LOWER_BOUND;
      exports2.LEAST_UPPER_BOUND = LEAST_UPPER_BOUND;
      exports2.TraceMap = TraceMap2;
      exports2.allGeneratedPositionsFor = allGeneratedPositionsFor;
      exports2.decodedMap = decodedMap;
      exports2.decodedMappings = decodedMappings;
      exports2.eachMapping = eachMapping;
      exports2.encodedMap = encodedMap;
      exports2.encodedMappings = encodedMappings;
      exports2.generatedPositionFor = generatedPositionFor;
      exports2.isIgnored = isIgnored;
      exports2.originalPositionFor = originalPositionFor2;
      exports2.presortedDecodedMap = presortedDecodedMap;
      exports2.sourceContentFor = sourceContentFor;
      exports2.traceSegment = traceSegment;
    });
  })(traceMapping_umd$1, traceMapping_umd$1.exports);
  return traceMapping_umd$1.exports;
}
var traceMapping_umdExports = requireTraceMapping_umd();
function notNullish(v) {
  return v != null;
}
const CHROME_IE_STACK_REGEXP = /^\s*at .*(?:\S:\d+|\(native\))/m;
const SAFARI_NATIVE_CODE_REGEXP = /^(?:eval@)?(?:\[native code\])?$/;
const stackIgnorePatterns = [
  "node:internal",
  /\/packages\/\w+\/dist\//,
  /\/@vitest\/\w+\/dist\//,
  "/vitest/dist/",
  "/vitest/src/",
  "/vite-node/dist/",
  "/vite-node/src/",
  "/node_modules/chai/",
  "/node_modules/tinypool/",
  "/node_modules/tinyspy/",
  // browser related deps
  "/deps/chunk-",
  "/deps/@vitest",
  "/deps/loupe",
  "/deps/chai",
  /node:\w+/,
  /__vitest_test__/,
  /__vitest_browser__/,
  /\/deps\/vitest_/
];
function extractLocation(urlLike) {
  if (!urlLike.includes(":")) {
    return [urlLike];
  }
  const regExp = /(.+?)(?::(\d+))?(?::(\d+))?$/;
  const parts = regExp.exec(urlLike.replace(/^\(|\)$/g, ""));
  if (!parts) {
    return [urlLike];
  }
  let url2 = parts[1];
  if (url2.startsWith("async ")) {
    url2 = url2.slice(6);
  }
  if (url2.startsWith("http:") || url2.startsWith("https:")) {
    const urlObj = new URL(url2);
    urlObj.searchParams.delete("import");
    urlObj.searchParams.delete("browserv");
    url2 = urlObj.pathname + urlObj.hash + urlObj.search;
  }
  if (url2.startsWith("/@fs/")) {
    const isWindows = /^\/@fs\/[a-zA-Z]:\//.test(url2);
    url2 = url2.slice(isWindows ? 5 : 4);
  }
  return [url2, parts[2] || void 0, parts[3] || void 0];
}
function parseSingleFFOrSafariStack(raw) {
  let line = raw.trim();
  if (SAFARI_NATIVE_CODE_REGEXP.test(line)) {
    return null;
  }
  if (line.includes(" > eval")) {
    line = line.replace(
      / line (\d+)(?: > eval line \d+)* > eval:\d+:\d+/g,
      ":$1"
    );
  }
  if (!line.includes("@") && !line.includes(":")) {
    return null;
  }
  const functionNameRegex = /((.*".+"[^@]*)?[^@]*)(@)/;
  const matches = line.match(functionNameRegex);
  const functionName = matches && matches[1] ? matches[1] : void 0;
  const [url2, lineNumber, columnNumber] = extractLocation(
    line.replace(functionNameRegex, "")
  );
  if (!url2 || !lineNumber || !columnNumber) {
    return null;
  }
  return {
    file: url2,
    method: functionName || "",
    line: Number.parseInt(lineNumber),
    column: Number.parseInt(columnNumber)
  };
}
function parseSingleV8Stack(raw) {
  let line = raw.trim();
  if (!CHROME_IE_STACK_REGEXP.test(line)) {
    return null;
  }
  if (line.includes("(eval ")) {
    line = line.replace(/eval code/g, "eval").replace(/(\(eval at [^()]*)|(,.*$)/g, "");
  }
  let sanitizedLine = line.replace(/^\s+/, "").replace(/\(eval code/g, "(").replace(/^.*?\s+/, "");
  const location2 = sanitizedLine.match(/ (\(.+\)$)/);
  sanitizedLine = location2 ? sanitizedLine.replace(location2[0], "") : sanitizedLine;
  const [url2, lineNumber, columnNumber] = extractLocation(
    location2 ? location2[1] : sanitizedLine
  );
  let method = location2 && sanitizedLine || "";
  let file = url2 && ["eval", "<anonymous>"].includes(url2) ? void 0 : url2;
  if (!file || !lineNumber || !columnNumber) {
    return null;
  }
  if (method.startsWith("async ")) {
    method = method.slice(6);
  }
  if (file.startsWith("file://")) {
    file = file.slice(7);
  }
  file = file.startsWith("node:") || file.startsWith("internal:") ? file : resolve(file);
  if (method) {
    method = method.replace(/__vite_ssr_import_\d+__\./g, "");
  }
  return {
    method,
    file,
    line: Number.parseInt(lineNumber),
    column: Number.parseInt(columnNumber)
  };
}
function createStackString(stacks) {
  return stacks.map((stack) => {
    const line = `${stack.file}:${stack.line}:${stack.column}`;
    if (stack.method) {
      return `    at ${stack.method}(${line})`;
    }
    return `    at ${line}`;
  }).join("\n");
}
function parseStacktrace(stack, options = {}) {
  const { ignoreStackEntries = stackIgnorePatterns } = options;
  const stacks = !CHROME_IE_STACK_REGEXP.test(stack) ? parseFFOrSafariStackTrace(stack) : parseV8Stacktrace(stack);
  return stacks.map((stack2) => {
    var _a;
    if (options.getUrlId) {
      stack2.file = options.getUrlId(stack2.file);
    }
    const map = (_a = options.getSourceMap) == null ? void 0 : _a.call(options, stack2.file);
    if (!map || typeof map !== "object" || !map.version) {
      return shouldFilter(ignoreStackEntries, stack2.file) ? null : stack2;
    }
    const traceMap = new traceMapping_umdExports.TraceMap(map);
    const { line, column, source, name } = traceMapping_umdExports.originalPositionFor(traceMap, stack2);
    let file = stack2.file;
    if (source) {
      const fileUrl = stack2.file.startsWith("file://") ? stack2.file : `file://${stack2.file}`;
      const sourceRootUrl = map.sourceRoot ? new URL(map.sourceRoot, fileUrl) : fileUrl;
      file = new URL(source, sourceRootUrl).pathname;
      if (file.match(/\/\w:\//)) {
        file = file.slice(1);
      }
    }
    if (shouldFilter(ignoreStackEntries, file)) {
      return null;
    }
    if (line != null && column != null) {
      return {
        line,
        column,
        file,
        method: name || stack2.method
      };
    }
    return stack2;
  }).filter((s) => s != null);
}
function shouldFilter(ignoreStackEntries, file) {
  return ignoreStackEntries.some((p2) => file.match(p2));
}
function parseFFOrSafariStackTrace(stack) {
  return stack.split("\n").map((line) => parseSingleFFOrSafariStack(line)).filter(notNullish);
}
function parseV8Stacktrace(stack) {
  return stack.split("\n").map((line) => parseSingleV8Stack(line)).filter(notNullish);
}
class VitestBrowserSnapshotEnvironment {
  constructor() {
    __publicField(this, "sourceMaps", /* @__PURE__ */ new Map());
    __publicField(this, "traceMaps", /* @__PURE__ */ new Map());
  }
  addSourceMap(filepath, map) {
    this.sourceMaps.set(filepath, map);
  }
  getVersion() {
    return "1";
  }
  getHeader() {
    return `// Vitest Snapshot v${this.getVersion()}, https://vitest.dev/guide/snapshot.html`;
  }
  readSnapshotFile(filepath) {
    return rpc$1().readSnapshotFile(filepath);
  }
  saveSnapshotFile(filepath, snapshot) {
    return rpc$1().saveSnapshotFile(filepath, snapshot);
  }
  resolvePath(filepath) {
    return rpc$1().resolveSnapshotPath(filepath);
  }
  resolveRawPath(testPath, rawPath) {
    return rpc$1().resolveSnapshotRawPath(testPath, rawPath);
  }
  removeSnapshotFile(filepath) {
    return rpc$1().removeSnapshotFile(filepath);
  }
  processStackTrace(stack) {
    const map = this.sourceMaps.get(stack.file);
    if (!map) {
      return stack;
    }
    let traceMap = this.traceMaps.get(stack.file);
    if (!traceMap) {
      traceMap = new TraceMap(map);
      this.traceMaps.set(stack.file, traceMap);
    }
    const { line, column } = originalPositionFor(traceMap, stack);
    if (line != null && column != null) {
      return { ...stack, line, column };
    }
    return stack;
  }
}
function rpc$1() {
  return globalThis.__vitest_worker__.rpc;
}
const browserHashMap = /* @__PURE__ */ new Map();
function createBrowserRunner(runnerClass, mocker, state, coverageModule) {
  return class BrowserTestRunner extends runnerClass {
    constructor(options) {
      super(options.config);
      __publicField(this, "config");
      __publicField(this, "hashMap", browserHashMap);
      __publicField(this, "sourceMapCache", /* @__PURE__ */ new Map());
      __publicField(this, "method", "run");
      __publicField(this, "onBeforeTryTask", async (...args) => {
        var _a;
        await userEvent.cleanup();
        await ((_a = super.onBeforeTryTask) == null ? void 0 : _a.call(this, ...args));
      });
      __publicField(this, "onAfterRunTask", async (task) => {
        var _a, _b;
        await ((_a = super.onAfterRunTask) == null ? void 0 : _a.call(this, task));
        if (this.config.bail && ((_b = task.result) == null ? void 0 : _b.state) === "fail") {
          const previousFailures = await rpc$2().getCountOfFailedTests();
          const currentFailures = 1 + previousFailures;
          if (currentFailures >= this.config.bail) {
            rpc$2().cancelCurrentRun("test-failure");
            this.cancel("test-failure");
          }
        }
      });
      __publicField(this, "onTaskFinished", async (task) => {
        var _a, _b;
        if (this.config.browser.screenshotFailures && document.body.clientHeight > 0 && ((_a = task.result) == null ? void 0 : _a.state) === "fail") {
          const screenshot = await page.screenshot({
            timeout: ((_b = this.config.browser.providerOptions) == null ? void 0 : _b.actionTimeout) ?? 5e3
          }).catch((err) => {
            console.error("[vitest] Failed to take a screenshot", err);
          });
          if (screenshot) {
            task.meta.failScreenshotPath = screenshot;
          }
        }
      });
      __publicField(this, "cancel", (reason) => {
        var _a;
        (_a = super.cancel) == null ? void 0 : _a.call(this, reason);
        globalChannel.postMessage({ type: "cancel", reason });
      });
      __publicField(this, "onBeforeRunSuite", async (suite) => {
        var _a;
        await Promise.all([
          (_a = super.onBeforeRunSuite) == null ? void 0 : _a.call(this, suite),
          (async () => {
            if ("filepath" in suite) {
              const map = await rpc$2().getBrowserFileSourceMap(suite.filepath);
              this.sourceMapCache.set(suite.filepath, map);
              const snapshotEnvironment = this.config.snapshotOptions.snapshotEnvironment;
              if (snapshotEnvironment instanceof VitestBrowserSnapshotEnvironment) {
                snapshotEnvironment.addSourceMap(suite.filepath, map);
              }
            }
          })()
        ]);
      });
      __publicField(this, "onAfterRunFiles", async (files) => {
        var _a, _b;
        const [coverage] = await Promise.all([
          (_a = coverageModule == null ? void 0 : coverageModule.takeCoverage) == null ? void 0 : _a.call(coverageModule),
          mocker.invalidate(),
          (_b = super.onAfterRunFiles) == null ? void 0 : _b.call(this, files)
        ]);
        if (coverage) {
          await rpc$2().onAfterSuiteRun({
            coverage,
            testFiles: files.map((file) => file.name),
            transformMode: "browser",
            projectName: this.config.name
          });
        }
      });
      __publicField(this, "onCollectStart", (file) => {
        return rpc$2().onQueued(this.method, file);
      });
      __publicField(this, "onCollected", async (files) => {
        files.forEach((file) => {
          file.prepareDuration = state.durations.prepare;
          file.environmentLoad = state.durations.environment;
          state.durations.prepare = 0;
          state.durations.environment = 0;
        });
        if (this.config.includeTaskLocation) {
          try {
            await updateTestFilesLocations(files, this.sourceMapCache);
          } catch {
          }
        }
        return rpc$2().onCollected(this.method, files);
      });
      __publicField(this, "onTestAnnotate", (test, annotation) => {
        if (annotation.location) {
          const map = this.sourceMapCache.get(annotation.location.file);
          if (!map) {
            return rpc$2().onTaskAnnotate(test.id, annotation);
          }
          const traceMap = new TraceMap(map);
          const { line, column, source } = originalPositionFor(traceMap, annotation.location);
          if (line != null && column != null && source != null) {
            let file = annotation.location.file;
            if (source) {
              const fileUrl = annotation.location.file.startsWith("file://") ? annotation.location.file : `file://${annotation.location.file}`;
              const sourceRootUrl = map.sourceRoot ? new URL(map.sourceRoot, fileUrl) : fileUrl;
              file = new URL(source, sourceRootUrl).pathname;
            }
            annotation.location = {
              line,
              column: column + 1,
              // if the file path is on windows, we need to remove the starting slash
              file: file.match(/\/\w:\//) ? file.slice(1) : file
            };
          }
        }
        return rpc$2().onTaskAnnotate(test.id, annotation);
      });
      __publicField(this, "onTaskUpdate", (task, events) => {
        return rpc$2().onTaskUpdate(this.method, task, events);
      });
      __publicField(this, "importFile", async (filepath) => {
        let hash = this.hashMap.get(filepath);
        if (!hash) {
          hash = Date.now().toString();
          this.hashMap.set(filepath, hash);
        }
        const prefix = `/${/^\w:/.test(filepath) ? "@fs/" : ""}`;
        const query = `browserv=${hash}`;
        const importpath = `${prefix}${filepath}?${query}`.replace(/\/+/g, "/");
        try {
          await import(
            /* @vite-ignore */
            importpath
          );
        } catch (err) {
          throw new Error(`Failed to import test file ${filepath}`, { cause: err });
        }
      });
      this.config = options.config;
    }
    setMethod(method) {
      this.method = method;
    }
  };
}
let cachedRunner = null;
function getBrowserRunner() {
  return cachedRunner;
}
async function initiateRunner(state, mocker, config) {
  if (cachedRunner) {
    return cachedRunner;
  }
  const runnerClass = config.mode === "test" ? VitestTestRunner : NodeBenchmarkRunner;
  const BrowserRunner = createBrowserRunner(runnerClass, mocker, state, {
    takeCoverage: () => takeCoverageInsideWorker(config.coverage, executor)
  });
  if (!config.snapshotOptions.snapshotEnvironment) {
    config.snapshotOptions.snapshotEnvironment = new VitestBrowserSnapshotEnvironment();
  }
  const runner = new BrowserRunner({
    config
  });
  cachedRunner = runner;
  onCancel.then((reason) => {
    var _a;
    (_a = runner.cancel) == null ? void 0 : _a.call(runner, reason);
  });
  const [diffOptions] = await Promise.all([
    loadDiffConfig(config, executor),
    loadSnapshotSerializers(config, executor)
  ]);
  runner.config.diffOptions = diffOptions;
  getWorkerState().onFilterStackTrace = (stack) => {
    const stacks = parseStacktrace(stack, {
      getSourceMap(file) {
        return runner.sourceMapCache.get(file);
      }
    });
    return createStackString(stacks);
  };
  return runner;
}
async function getTraceMap(file, sourceMaps) {
  const result = sourceMaps.get(file) || await rpc$2().getBrowserFileSourceMap(file).then((map) => {
    sourceMaps.set(file, map);
    return map;
  });
  if (!result) {
    return null;
  }
  return new TraceMap(result);
}
async function updateTestFilesLocations(files, sourceMaps) {
  const promises2 = files.map(async (file) => {
    const traceMap = await getTraceMap(file.filepath, sourceMaps);
    if (!traceMap) {
      return null;
    }
    const updateLocation = (task) => {
      if (task.location) {
        const { line, column } = originalPositionFor(traceMap, task.location);
        if (line != null && column != null) {
          task.location = { line, column: task.each ? column : column + 1 };
        }
      }
      if ("tasks" in task) {
        task.tasks.forEach(updateLocation);
      }
    };
    file.tasks.forEach(updateLocation);
    return null;
  });
  await Promise.all(promises2);
}
const { Date: Date$1, console: console$1, performance: performance$1 } = globalThis;
function setupConsoleLogSpy() {
  const {
    log,
    info,
    error,
    dir,
    dirxml,
    trace,
    time,
    timeEnd,
    timeLog,
    warn,
    debug: debug2,
    count,
    countReset
  } = console$1;
  console$1.log = stdout(log);
  console$1.debug = stdout(debug2);
  console$1.info = stdout(info);
  console$1.error = stderr(error);
  console$1.warn = stderr(warn);
  console$1.dir = (item, options) => {
    dir(item, options);
    sendLog("stdout", formatInput(item));
  };
  console$1.dirxml = (...args) => {
    dirxml(...args);
    sendLog("stdout", processLog(args));
  };
  console$1.trace = (...args) => {
    var _a;
    trace(...args);
    const content = processLog(args);
    const error2 = new Error("$$Trace");
    const processor = ((_a = globalThis.__vitest_worker__) == null ? void 0 : _a.onFilterStackTrace) || ((s) => s || "");
    const stack = processor(error2.stack || "");
    sendLog("stderr", `${content}
${stack}`, true);
  };
  const timeLabels = {};
  console$1.time = (label = "default") => {
    time(label);
    const now2 = performance$1.now();
    timeLabels[label] = now2;
  };
  console$1.timeLog = (label = "default") => {
    timeLog(label);
    if (!(label in timeLabels)) {
      sendLog("stderr", `Timer "${label}" does not exist`);
    } else {
      sendLog("stdout", `${label}: ${timeLabels[label]} ms`);
    }
  };
  console$1.timeEnd = (label = "default") => {
    timeEnd(label);
    const end = performance$1.now();
    const start = timeLabels[label];
    if (!(label in timeLabels)) {
      sendLog("stderr", `Timer "${label}" does not exist`);
    } else if (typeof start !== "undefined") {
      const duration = end - start;
      sendLog("stdout", `${label}: ${duration} ms`);
    }
  };
  const countLabels = {};
  console$1.count = (label = "default") => {
    count(label);
    const counter = (countLabels[label] ?? 0) + 1;
    countLabels[label] = counter;
    sendLog("stdout", `${label}: ${counter}`);
  };
  console$1.countReset = (label = "default") => {
    countReset(label);
    countLabels[label] = 0;
  };
}
function stdout(base) {
  return (...args) => {
    base(...args);
    if (args[0] === "[WDIO]") {
      if (args[1] === "newShadowRoot" || args[1] === "removeShadowRoot") {
        return;
      }
    }
    sendLog("stdout", processLog(args));
  };
}
function stderr(base) {
  return (...args) => {
    base(...args);
    sendLog("stderr", processLog(args));
  };
}
function formatInput(input) {
  if (typeof input === "object") {
    return stringify(input, void 0, {
      printBasicPrototype: false,
      escapeString: false
    });
  }
  return format(input);
}
function processLog(args) {
  return args.map(formatInput).join(" ");
}
function sendLog(type, content, disableStack) {
  var _a, _b, _c;
  if (content.startsWith("[vite]")) {
    return;
  }
  const unknownTestId = "__vitest__unknown_test__";
  const taskId = ((_b = (_a = globalThis.__vitest_worker__) == null ? void 0 : _a.current) == null ? void 0 : _b.id) ?? unknownTestId;
  const origin = getConfig().printConsoleTrace && !disableStack ? (_c = new Error("STACK_TRACE").stack) == null ? void 0 : _c.split("\n").slice(1).join("\n") : void 0;
  const runner = getBrowserRunner();
  rpc$2().sendLog((runner == null ? void 0 : runner.method) || "run", {
    origin,
    content,
    browser: true,
    time: Date$1.now(),
    taskId,
    type,
    size: content.length
  });
}
class MockerRegistry {
  constructor() {
    __publicField(this, "registryByUrl", /* @__PURE__ */ new Map());
    __publicField(this, "registryById", /* @__PURE__ */ new Map());
  }
  clear() {
    this.registryByUrl.clear();
    this.registryById.clear();
  }
  keys() {
    return this.registryByUrl.keys();
  }
  add(mock) {
    this.registryByUrl.set(mock.url, mock);
    this.registryById.set(mock.id, mock);
  }
  register(typeOrEvent, raw, id, url2, factoryOrRedirect) {
    const type = typeof typeOrEvent === "object" ? typeOrEvent.type : typeOrEvent;
    if (typeof typeOrEvent === "object") {
      const event = typeOrEvent;
      if (event instanceof AutomockedModule || event instanceof AutospiedModule || event instanceof ManualMockedModule || event instanceof RedirectedModule) {
        throw new TypeError(`[vitest] Cannot register a mock that is already defined. Expected a JSON representation from \`MockedModule.toJSON\`, instead got "${event.type}". Use "registry.add()" to update a mock instead.`);
      }
      if (event.type === "automock") {
        const module = AutomockedModule.fromJSON(event);
        this.add(module);
        return module;
      } else if (event.type === "autospy") {
        const module = AutospiedModule.fromJSON(event);
        this.add(module);
        return module;
      } else if (event.type === "redirect") {
        const module = RedirectedModule.fromJSON(event);
        this.add(module);
        return module;
      } else if (event.type === "manual") {
        throw new Error(`Cannot set serialized manual mock. Define a factory function manually with \`ManualMockedModule.fromJSON()\`.`);
      } else {
        throw new Error(`Unknown mock type: ${event.type}`);
      }
    }
    if (typeof raw !== "string") {
      throw new TypeError("[vitest] Mocks require a raw string.");
    }
    if (typeof url2 !== "string") {
      throw new TypeError("[vitest] Mocks require a url string.");
    }
    if (typeof id !== "string") {
      throw new TypeError("[vitest] Mocks require an id string.");
    }
    if (type === "manual") {
      if (typeof factoryOrRedirect !== "function") {
        throw new TypeError("[vitest] Manual mocks require a factory function.");
      }
      const mock = new ManualMockedModule(raw, id, url2, factoryOrRedirect);
      this.add(mock);
      return mock;
    } else if (type === "automock" || type === "autospy") {
      const mock = type === "automock" ? new AutomockedModule(raw, id, url2) : new AutospiedModule(raw, id, url2);
      this.add(mock);
      return mock;
    } else if (type === "redirect") {
      if (typeof factoryOrRedirect !== "string") {
        throw new TypeError("[vitest] Redirect mocks require a redirect string.");
      }
      const mock = new RedirectedModule(raw, id, url2, factoryOrRedirect);
      this.add(mock);
      return mock;
    } else {
      throw new Error(`[vitest] Unknown mock type: ${type}`);
    }
  }
  delete(id) {
    this.registryByUrl.delete(id);
  }
  get(id) {
    return this.registryByUrl.get(id);
  }
  getById(id) {
    return this.registryById.get(id);
  }
  has(id) {
    return this.registryByUrl.has(id);
  }
}
class AutomockedModule {
  constructor(raw, id, url2) {
    __publicField(this, "type", "automock");
    this.raw = raw;
    this.id = id;
    this.url = url2;
  }
  static fromJSON(data) {
    return new AutospiedModule(data.raw, data.id, data.url);
  }
  toJSON() {
    return {
      type: this.type,
      url: this.url,
      raw: this.raw,
      id: this.id
    };
  }
}
class AutospiedModule {
  constructor(raw, id, url2) {
    __publicField(this, "type", "autospy");
    this.raw = raw;
    this.id = id;
    this.url = url2;
  }
  static fromJSON(data) {
    return new AutospiedModule(data.raw, data.id, data.url);
  }
  toJSON() {
    return {
      type: this.type,
      url: this.url,
      id: this.id,
      raw: this.raw
    };
  }
}
class RedirectedModule {
  constructor(raw, id, url2, redirect) {
    __publicField(this, "type", "redirect");
    this.raw = raw;
    this.id = id;
    this.url = url2;
    this.redirect = redirect;
  }
  static fromJSON(data) {
    return new RedirectedModule(data.raw, data.id, data.url, data.redirect);
  }
  toJSON() {
    return {
      type: this.type,
      url: this.url,
      raw: this.raw,
      id: this.id,
      redirect: this.redirect
    };
  }
}
class ManualMockedModule {
  constructor(raw, id, url2, factory) {
    __publicField(this, "cache");
    __publicField(this, "type", "manual");
    this.raw = raw;
    this.id = id;
    this.url = url2;
    this.factory = factory;
  }
  async resolve() {
    if (this.cache) {
      return this.cache;
    }
    let exports;
    try {
      exports = await this.factory();
    } catch (err) {
      const vitestError = new Error('[vitest] There was an error when mocking a module. If you are using "vi.mock" factory, make sure there are no top level variables inside, since this call is hoisted to top of the file. Read more: https://vitest.dev/api/vi.html#vi-mock');
      vitestError.cause = err;
      throw vitestError;
    }
    if (exports === null || typeof exports !== "object" || Array.isArray(exports)) {
      throw new TypeError(`[vitest] vi.mock("${this.raw}", factory?: () => unknown) is not returning an object. Did you mean to return an object with a "default" key?`);
    }
    return this.cache = exports;
  }
  static fromJSON(data, factory) {
    return new ManualMockedModule(data.raw, data.id, data.url, factory);
  }
  toJSON() {
    return {
      type: this.type,
      url: this.url,
      id: this.id,
      raw: this.raw
    };
  }
}
function mockObject(options, object2, mockExports = {}) {
  const finalizers = new Array();
  const refs = new RefTracker();
  const define = (container, key, value) => {
    try {
      container[key] = value;
      return true;
    } catch {
      return false;
    }
  };
  const mockPropertiesOf = (container, newContainer) => {
    const containerType = getType(container);
    const isModule = containerType === "Module" || !!container.__esModule;
    for (const { key: property, descriptor } of getAllMockableProperties(container, isModule, options.globalConstructors)) {
      if (!isModule && descriptor.get) {
        try {
          Object.defineProperty(newContainer, property, descriptor);
        } catch {
        }
        continue;
      }
      if (isSpecialProp(property, containerType)) {
        continue;
      }
      const value = container[property];
      const refId = refs.getId(value);
      if (refId !== void 0) {
        finalizers.push(() => define(newContainer, property, refs.getMockedValue(refId)));
        continue;
      }
      const type = getType(value);
      if (Array.isArray(value)) {
        define(newContainer, property, []);
        continue;
      }
      const isFunction = type.includes("Function") && typeof value === "function";
      if ((!isFunction || value._isMockFunction) && type !== "Object" && type !== "Module") {
        define(newContainer, property, value);
        continue;
      }
      if (!define(newContainer, property, isFunction ? value : {})) {
        continue;
      }
      if (isFunction) {
        let mockFunction = function() {
          if (this instanceof newContainer[property]) {
            for (const { key, descriptor: descriptor2 } of getAllMockableProperties(this, false, options.globalConstructors)) {
              if (descriptor2.get) {
                continue;
              }
              const value2 = this[key];
              const type2 = getType(value2);
              const isFunction2 = type2.includes("Function") && typeof value2 === "function";
              if (isFunction2) {
                const original = this[key];
                const mock2 = spyOn(this, key).mockImplementation(original);
                const origMockReset = mock2.mockReset;
                mock2.mockRestore = mock2.mockReset = () => {
                  origMockReset.call(mock2);
                  mock2.mockImplementation(original);
                  return mock2;
                };
              }
            }
          }
        };
        if (!options.spyOn) {
          throw new Error("[@vitest/mocker] `spyOn` is not defined. This is a Vitest error. Please open a new issue with reproduction.");
        }
        const spyOn = options.spyOn;
        const mock = spyOn(newContainer, property);
        if (options.type === "automock") {
          mock.mockImplementation(mockFunction);
          const origMockReset = mock.mockReset;
          mock.mockRestore = mock.mockReset = () => {
            origMockReset.call(mock);
            mock.mockImplementation(mockFunction);
            return mock;
          };
        }
        Object.defineProperty(newContainer[property], "length", { value: 0 });
      }
      refs.track(value, newContainer[property]);
      mockPropertiesOf(value, newContainer[property]);
    }
  };
  const mockedObject = mockExports;
  mockPropertiesOf(object2, mockedObject);
  for (const finalizer of finalizers) {
    finalizer();
  }
  return mockedObject;
}
class RefTracker {
  constructor() {
    __publicField(this, "idMap", /* @__PURE__ */ new Map());
    __publicField(this, "mockedValueMap", /* @__PURE__ */ new Map());
  }
  getId(value) {
    return this.idMap.get(value);
  }
  getMockedValue(id) {
    return this.mockedValueMap.get(id);
  }
  track(originalValue, mockedValue) {
    const newId = this.idMap.size;
    this.idMap.set(originalValue, newId);
    this.mockedValueMap.set(newId, mockedValue);
    return newId;
  }
}
function getType(value) {
  return Object.prototype.toString.apply(value).slice(8, -1);
}
function isSpecialProp(prop, parentType) {
  return parentType.includes("Function") && typeof prop === "string" && [
    "arguments",
    "callee",
    "caller",
    "length",
    "name"
  ].includes(prop);
}
function getAllMockableProperties(obj, isModule, constructors) {
  const { Map: Map2, Object: Object2, Function: Function2, RegExp: RegExp2, Array: Array2 } = constructors;
  const allProps = new Map2();
  let curr = obj;
  do {
    if (curr === Object2.prototype || curr === Function2.prototype || curr === RegExp2.prototype) {
      break;
    }
    collectOwnProperties(curr, (key) => {
      const descriptor = Object2.getOwnPropertyDescriptor(curr, key);
      if (descriptor) {
        allProps.set(key, {
          key,
          descriptor
        });
      }
    });
  } while (curr = Object2.getPrototypeOf(curr));
  if (isModule && !allProps.has("default") && "default" in obj) {
    const descriptor = Object2.getOwnPropertyDescriptor(obj, "default");
    if (descriptor) {
      allProps.set("default", {
        key: "default",
        descriptor
      });
    }
  }
  return Array2.from(allProps.values());
}
function collectOwnProperties(obj, collector) {
  const collect = typeof collector === "function" ? collector : (key) => collector.add(key);
  Object.getOwnPropertyNames(obj).forEach(collect);
  Object.getOwnPropertySymbols(obj).forEach(collect);
}
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
function normalizeString(path, allowAboveRoot) {
  let res = "";
  let lastSegmentLength = 0;
  let lastSlash = -1;
  let dots = 0;
  let char = null;
  for (let index2 = 0; index2 <= path.length; ++index2) {
    if (index2 < path.length) {
      char = path[index2];
    } else if (char === "/") {
      break;
    } else {
      char = "/";
    }
    if (char === "/") {
      if (lastSlash === index2 - 1 || dots === 1) ;
      else if (dots === 2) {
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
            lastSlash = index2;
            dots = 0;
            continue;
          } else if (res.length > 0) {
            res = "";
            lastSegmentLength = 0;
            lastSlash = index2;
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
          res += `/${path.slice(lastSlash + 1, index2)}`;
        } else {
          res = path.slice(lastSlash + 1, index2);
        }
        lastSegmentLength = index2 - lastSlash - 1;
      }
      lastSlash = index2;
      dots = 0;
    } else if (char === "." && dots !== -1) {
      ++dots;
    } else {
      dots = -1;
    }
  }
  return res;
}
const isAbsolute = function(p2) {
  return _IS_ABSOLUTE_RE.test(p2);
};
const extname = function(p2) {
  if (p2 === "..") return "";
  const match = _EXTNAME_RE.exec(normalizeWindowsPath(p2));
  return match && match[1] || "";
};
var f = {
  reset: [0, 0],
  bold: [1, 22, "\x1B[22m\x1B[1m"],
  dim: [2, 22, "\x1B[22m\x1B[2m"],
  italic: [3, 23],
  underline: [4, 24],
  inverse: [7, 27],
  hidden: [8, 28],
  strikethrough: [9, 29],
  black: [30, 39],
  red: [31, 39],
  green: [32, 39],
  yellow: [33, 39],
  blue: [34, 39],
  magenta: [35, 39],
  cyan: [36, 39],
  white: [37, 39],
  gray: [90, 39],
  bgBlack: [40, 49],
  bgRed: [41, 49],
  bgGreen: [42, 49],
  bgYellow: [43, 49],
  bgBlue: [44, 49],
  bgMagenta: [45, 49],
  bgCyan: [46, 49],
  bgWhite: [47, 49],
  blackBright: [90, 39],
  redBright: [91, 39],
  greenBright: [92, 39],
  yellowBright: [93, 39],
  blueBright: [94, 39],
  magentaBright: [95, 39],
  cyanBright: [96, 39],
  whiteBright: [97, 39],
  bgBlackBright: [100, 49],
  bgRedBright: [101, 49],
  bgGreenBright: [102, 49],
  bgYellowBright: [103, 49],
  bgBlueBright: [104, 49],
  bgMagentaBright: [105, 49],
  bgCyanBright: [106, 49],
  bgWhiteBright: [107, 49]
}, h = Object.entries(f);
function a(n) {
  return String(n);
}
a.open = "";
a.close = "";
function C(n = false) {
  let e = typeof process != "undefined" ? process : void 0, i = (e == null ? void 0 : e.env) || {}, g = (e == null ? void 0 : e.argv) || [];
  return !("NO_COLOR" in i || g.includes("--no-color")) && ("FORCE_COLOR" in i || g.includes("--color") || (e == null ? void 0 : e.platform) === "win32" || n && i.TERM !== "dumb" || "CI" in i) || typeof window != "undefined" && !!window.chrome;
}
function p(n = false) {
  let e = C(n), i = (r, t, c, o) => {
    let l = "", s = 0;
    do
      l += r.substring(s, o) + c, s = o + t.length, o = r.indexOf(t, s);
    while (~o);
    return l + r.substring(s);
  }, g = (r, t, c = r) => {
    let o = (l) => {
      let s = String(l), b = s.indexOf(t, r.length);
      return ~b ? r + i(s, t, c, b) + t : r + s + t;
    };
    return o.open = r, o.close = t, o;
  }, u = {
    isColorSupported: e
  }, d = (r) => `\x1B[${r}m`;
  for (let [r, t] of h)
    u[r] = e ? g(
      d(t[0]),
      d(t[1]),
      t[2]
    ) : a;
  return u;
}
p();
function _mergeNamespaces(n, m) {
  m.forEach(function(e) {
    e && typeof e !== "string" && !Array.isArray(e) && Object.keys(e).forEach(function(k) {
      if (k !== "default" && !(k in n)) {
        var d = Object.getOwnPropertyDescriptor(e, k);
        Object.defineProperty(n, k, d.get ? d : {
          enumerable: true,
          get: function() {
            return e[k];
          }
        });
      }
    });
  });
  return Object.freeze(n);
}
function getDefaultExportFromCjs(x) {
  return x && x.__esModule && Object.prototype.hasOwnProperty.call(x, "default") ? x["default"] : x;
}
var reactIs$1 = { exports: {} };
var reactIs_production = {};
/**
* @license React
* react-is.production.js
*
* Copyright (c) Meta Platforms, Inc. and affiliates.
*
* This source code is licensed under the MIT license found in the
* LICENSE file in the root directory of this source tree.
*/
var hasRequiredReactIs_production;
function requireReactIs_production() {
  if (hasRequiredReactIs_production) return reactIs_production;
  hasRequiredReactIs_production = 1;
  var REACT_ELEMENT_TYPE = Symbol.for("react.transitional.element"), REACT_PORTAL_TYPE = Symbol.for("react.portal"), REACT_FRAGMENT_TYPE = Symbol.for("react.fragment"), REACT_STRICT_MODE_TYPE = Symbol.for("react.strict_mode"), REACT_PROFILER_TYPE = Symbol.for("react.profiler");
  var REACT_CONSUMER_TYPE = Symbol.for("react.consumer"), REACT_CONTEXT_TYPE = Symbol.for("react.context"), REACT_FORWARD_REF_TYPE = Symbol.for("react.forward_ref"), REACT_SUSPENSE_TYPE = Symbol.for("react.suspense"), REACT_SUSPENSE_LIST_TYPE = Symbol.for("react.suspense_list"), REACT_MEMO_TYPE = Symbol.for("react.memo"), REACT_LAZY_TYPE = Symbol.for("react.lazy"), REACT_VIEW_TRANSITION_TYPE = Symbol.for("react.view_transition"), REACT_CLIENT_REFERENCE = Symbol.for("react.client.reference");
  function typeOf(object2) {
    if ("object" === typeof object2 && null !== object2) {
      var $$typeof = object2.$$typeof;
      switch ($$typeof) {
        case REACT_ELEMENT_TYPE:
          switch (object2 = object2.type, object2) {
            case REACT_FRAGMENT_TYPE:
            case REACT_PROFILER_TYPE:
            case REACT_STRICT_MODE_TYPE:
            case REACT_SUSPENSE_TYPE:
            case REACT_SUSPENSE_LIST_TYPE:
            case REACT_VIEW_TRANSITION_TYPE:
              return object2;
            default:
              switch (object2 = object2 && object2.$$typeof, object2) {
                case REACT_CONTEXT_TYPE:
                case REACT_FORWARD_REF_TYPE:
                case REACT_LAZY_TYPE:
                case REACT_MEMO_TYPE:
                  return object2;
                case REACT_CONSUMER_TYPE:
                  return object2;
                default:
                  return $$typeof;
              }
          }
        case REACT_PORTAL_TYPE:
          return $$typeof;
      }
    }
  }
  reactIs_production.ContextConsumer = REACT_CONSUMER_TYPE;
  reactIs_production.ContextProvider = REACT_CONTEXT_TYPE;
  reactIs_production.Element = REACT_ELEMENT_TYPE;
  reactIs_production.ForwardRef = REACT_FORWARD_REF_TYPE;
  reactIs_production.Fragment = REACT_FRAGMENT_TYPE;
  reactIs_production.Lazy = REACT_LAZY_TYPE;
  reactIs_production.Memo = REACT_MEMO_TYPE;
  reactIs_production.Portal = REACT_PORTAL_TYPE;
  reactIs_production.Profiler = REACT_PROFILER_TYPE;
  reactIs_production.StrictMode = REACT_STRICT_MODE_TYPE;
  reactIs_production.Suspense = REACT_SUSPENSE_TYPE;
  reactIs_production.SuspenseList = REACT_SUSPENSE_LIST_TYPE;
  reactIs_production.isContextConsumer = function(object2) {
    return typeOf(object2) === REACT_CONSUMER_TYPE;
  };
  reactIs_production.isContextProvider = function(object2) {
    return typeOf(object2) === REACT_CONTEXT_TYPE;
  };
  reactIs_production.isElement = function(object2) {
    return "object" === typeof object2 && null !== object2 && object2.$$typeof === REACT_ELEMENT_TYPE;
  };
  reactIs_production.isForwardRef = function(object2) {
    return typeOf(object2) === REACT_FORWARD_REF_TYPE;
  };
  reactIs_production.isFragment = function(object2) {
    return typeOf(object2) === REACT_FRAGMENT_TYPE;
  };
  reactIs_production.isLazy = function(object2) {
    return typeOf(object2) === REACT_LAZY_TYPE;
  };
  reactIs_production.isMemo = function(object2) {
    return typeOf(object2) === REACT_MEMO_TYPE;
  };
  reactIs_production.isPortal = function(object2) {
    return typeOf(object2) === REACT_PORTAL_TYPE;
  };
  reactIs_production.isProfiler = function(object2) {
    return typeOf(object2) === REACT_PROFILER_TYPE;
  };
  reactIs_production.isStrictMode = function(object2) {
    return typeOf(object2) === REACT_STRICT_MODE_TYPE;
  };
  reactIs_production.isSuspense = function(object2) {
    return typeOf(object2) === REACT_SUSPENSE_TYPE;
  };
  reactIs_production.isSuspenseList = function(object2) {
    return typeOf(object2) === REACT_SUSPENSE_LIST_TYPE;
  };
  reactIs_production.isValidElementType = function(type) {
    return "string" === typeof type || "function" === typeof type || type === REACT_FRAGMENT_TYPE || type === REACT_PROFILER_TYPE || type === REACT_STRICT_MODE_TYPE || type === REACT_SUSPENSE_TYPE || type === REACT_SUSPENSE_LIST_TYPE || "object" === typeof type && null !== type && (type.$$typeof === REACT_LAZY_TYPE || type.$$typeof === REACT_MEMO_TYPE || type.$$typeof === REACT_CONTEXT_TYPE || type.$$typeof === REACT_CONSUMER_TYPE || type.$$typeof === REACT_FORWARD_REF_TYPE || type.$$typeof === REACT_CLIENT_REFERENCE || void 0 !== type.getModuleId) ? true : false;
  };
  reactIs_production.typeOf = typeOf;
  return reactIs_production;
}
var hasRequiredReactIs$1;
function requireReactIs$1() {
  if (hasRequiredReactIs$1) return reactIs$1.exports;
  hasRequiredReactIs$1 = 1;
  {
    reactIs$1.exports = requireReactIs_production();
  }
  return reactIs$1.exports;
}
var reactIsExports$1 = requireReactIs$1();
var index$1 = /* @__PURE__ */ getDefaultExportFromCjs(reactIsExports$1);
var ReactIs19 = /* @__PURE__ */ _mergeNamespaces({
  __proto__: null,
  default: index$1
}, [reactIsExports$1]);
var reactIs = { exports: {} };
var reactIs_production_min = {};
/**
* @license React
* react-is.production.min.js
*
* Copyright (c) Facebook, Inc. and its affiliates.
*
* This source code is licensed under the MIT license found in the
* LICENSE file in the root directory of this source tree.
*/
var hasRequiredReactIs_production_min;
function requireReactIs_production_min() {
  if (hasRequiredReactIs_production_min) return reactIs_production_min;
  hasRequiredReactIs_production_min = 1;
  var b = Symbol.for("react.element"), c = Symbol.for("react.portal"), d = Symbol.for("react.fragment"), e = Symbol.for("react.strict_mode"), f2 = Symbol.for("react.profiler"), g = Symbol.for("react.provider"), h2 = Symbol.for("react.context"), k = Symbol.for("react.server_context"), l = Symbol.for("react.forward_ref"), m = Symbol.for("react.suspense"), n = Symbol.for("react.suspense_list"), p2 = Symbol.for("react.memo"), q = Symbol.for("react.lazy"), t = Symbol.for("react.offscreen"), u;
  u = Symbol.for("react.module.reference");
  function v(a2) {
    if ("object" === typeof a2 && null !== a2) {
      var r = a2.$$typeof;
      switch (r) {
        case b:
          switch (a2 = a2.type, a2) {
            case d:
            case f2:
            case e:
            case m:
            case n:
              return a2;
            default:
              switch (a2 = a2 && a2.$$typeof, a2) {
                case k:
                case h2:
                case l:
                case q:
                case p2:
                case g:
                  return a2;
                default:
                  return r;
              }
          }
        case c:
          return r;
      }
    }
  }
  reactIs_production_min.ContextConsumer = h2;
  reactIs_production_min.ContextProvider = g;
  reactIs_production_min.Element = b;
  reactIs_production_min.ForwardRef = l;
  reactIs_production_min.Fragment = d;
  reactIs_production_min.Lazy = q;
  reactIs_production_min.Memo = p2;
  reactIs_production_min.Portal = c;
  reactIs_production_min.Profiler = f2;
  reactIs_production_min.StrictMode = e;
  reactIs_production_min.Suspense = m;
  reactIs_production_min.SuspenseList = n;
  reactIs_production_min.isAsyncMode = function() {
    return false;
  };
  reactIs_production_min.isConcurrentMode = function() {
    return false;
  };
  reactIs_production_min.isContextConsumer = function(a2) {
    return v(a2) === h2;
  };
  reactIs_production_min.isContextProvider = function(a2) {
    return v(a2) === g;
  };
  reactIs_production_min.isElement = function(a2) {
    return "object" === typeof a2 && null !== a2 && a2.$$typeof === b;
  };
  reactIs_production_min.isForwardRef = function(a2) {
    return v(a2) === l;
  };
  reactIs_production_min.isFragment = function(a2) {
    return v(a2) === d;
  };
  reactIs_production_min.isLazy = function(a2) {
    return v(a2) === q;
  };
  reactIs_production_min.isMemo = function(a2) {
    return v(a2) === p2;
  };
  reactIs_production_min.isPortal = function(a2) {
    return v(a2) === c;
  };
  reactIs_production_min.isProfiler = function(a2) {
    return v(a2) === f2;
  };
  reactIs_production_min.isStrictMode = function(a2) {
    return v(a2) === e;
  };
  reactIs_production_min.isSuspense = function(a2) {
    return v(a2) === m;
  };
  reactIs_production_min.isSuspenseList = function(a2) {
    return v(a2) === n;
  };
  reactIs_production_min.isValidElementType = function(a2) {
    return "string" === typeof a2 || "function" === typeof a2 || a2 === d || a2 === f2 || a2 === e || a2 === m || a2 === n || a2 === t || "object" === typeof a2 && null !== a2 && (a2.$$typeof === q || a2.$$typeof === p2 || a2.$$typeof === g || a2.$$typeof === h2 || a2.$$typeof === l || a2.$$typeof === u || void 0 !== a2.getModuleId) ? true : false;
  };
  reactIs_production_min.typeOf = v;
  return reactIs_production_min;
}
var hasRequiredReactIs;
function requireReactIs() {
  if (hasRequiredReactIs) return reactIs.exports;
  hasRequiredReactIs = 1;
  {
    reactIs.exports = requireReactIs_production_min();
  }
  return reactIs.exports;
}
var reactIsExports = requireReactIs();
var index = /* @__PURE__ */ getDefaultExportFromCjs(reactIsExports);
var ReactIs18 = /* @__PURE__ */ _mergeNamespaces({
  __proto__: null,
  default: index
}, [reactIsExports]);
const reactIsMethods = [
  "isAsyncMode",
  "isConcurrentMode",
  "isContextConsumer",
  "isContextProvider",
  "isElement",
  "isForwardRef",
  "isFragment",
  "isLazy",
  "isMemo",
  "isPortal",
  "isProfiler",
  "isStrictMode",
  "isSuspense",
  "isSuspenseList",
  "isValidElementType"
];
Object.fromEntries(reactIsMethods.map((m) => [m, (v) => ReactIs18[m](v) || ReactIs19[m](v)]));
let getPromiseValue = () => "Promise{…}";
try {
  const { getPromiseDetails, kPending, kRejected } = process.binding("util");
  if (Array.isArray(getPromiseDetails(Promise.resolve()))) {
    getPromiseValue = (value, options) => {
      const [state, innerValue] = getPromiseDetails(value);
      if (state === kPending) {
        return "Promise{<pending>}";
      }
      return `Promise${state === kRejected ? "!" : ""}{${options.inspect(innerValue, options)}}`;
    };
  }
} catch (notNode) {
}
var jsTokens_1;
var hasRequiredJsTokens;
function requireJsTokens() {
  if (hasRequiredJsTokens) return jsTokens_1;
  hasRequiredJsTokens = 1;
  var Identifier, JSXIdentifier, JSXPunctuator, JSXString, JSXText, KeywordsWithExpressionAfter, KeywordsWithNoLineTerminatorAfter, LineTerminatorSequence, MultiLineComment, Newline, NumericLiteral, Punctuator, RegularExpressionLiteral, SingleLineComment, StringLiteral, Template, TokensNotPrecedingObjectLiteral, TokensPrecedingExpression, WhiteSpace;
  RegularExpressionLiteral = /\/(?![*\/])(?:\[(?:(?![\]\\]).|\\.)*\]|(?![\/\\]).|\\.)*(\/[$_\u200C\u200D\p{ID_Continue}]*|\\)?/uy;
  Punctuator = /--|\+\+|=>|\.{3}|\??\.(?!\d)|(?:&&|\|\||\?\?|[+\-%&|^]|\*{1,2}|<{1,2}|>{1,3}|!=?|={1,2}|\/(?![\/*]))=?|[?~,:;[\](){}]/y;
  Identifier = /(\x23?)(?=[$_\p{ID_Start}\\])(?:[$_\u200C\u200D\p{ID_Continue}]|\\u[\da-fA-F]{4}|\\u\{[\da-fA-F]+\})+/uy;
  StringLiteral = /(['"])(?:(?!\1)[^\\\n\r]|\\(?:\r\n|[^]))*(\1)?/y;
  NumericLiteral = /(?:0[xX][\da-fA-F](?:_?[\da-fA-F])*|0[oO][0-7](?:_?[0-7])*|0[bB][01](?:_?[01])*)n?|0n|[1-9](?:_?\d)*n|(?:(?:0(?!\d)|0\d*[89]\d*|[1-9](?:_?\d)*)(?:\.(?:\d(?:_?\d)*)?)?|\.\d(?:_?\d)*)(?:[eE][+-]?\d(?:_?\d)*)?|0[0-7]+/y;
  Template = /[`}](?:[^`\\$]|\\[^]|\$(?!\{))*(`|\$\{)?/y;
  WhiteSpace = /[\t\v\f\ufeff\p{Zs}]+/uy;
  LineTerminatorSequence = /\r?\n|[\r\u2028\u2029]/y;
  MultiLineComment = /\/\*(?:[^*]|\*(?!\/))*(\*\/)?/y;
  SingleLineComment = /\/\/.*/y;
  JSXPunctuator = /[<>.:={}]|\/(?![\/*])/y;
  JSXIdentifier = /[$_\p{ID_Start}][$_\u200C\u200D\p{ID_Continue}-]*/uy;
  JSXString = /(['"])(?:(?!\1)[^])*(\1)?/y;
  JSXText = /[^<>{}]+/y;
  TokensPrecedingExpression = /^(?:[\/+-]|\.{3}|\?(?:InterpolationIn(?:JSX|Template)|NoLineTerminatorHere|NonExpressionParenEnd|UnaryIncDec))?$|[{}([,;<>=*%&|^!~?:]$/;
  TokensNotPrecedingObjectLiteral = /^(?:=>|[;\]){}]|else|\?(?:NoLineTerminatorHere|NonExpressionParenEnd))?$/;
  KeywordsWithExpressionAfter = /^(?:await|case|default|delete|do|else|instanceof|new|return|throw|typeof|void|yield)$/;
  KeywordsWithNoLineTerminatorAfter = /^(?:return|throw|yield)$/;
  Newline = RegExp(LineTerminatorSequence.source);
  jsTokens_1 = function* (input, { jsx = false } = {}) {
    var braces, firstCodePoint, isExpression, lastIndex, lastSignificantToken, length, match, mode, nextLastIndex, nextLastSignificantToken, parenNesting, postfixIncDec, punctuator, stack;
    ({ length } = input);
    lastIndex = 0;
    lastSignificantToken = "";
    stack = [{ tag: "JS" }];
    braces = [];
    parenNesting = 0;
    postfixIncDec = false;
    while (lastIndex < length) {
      mode = stack[stack.length - 1];
      switch (mode.tag) {
        case "JS":
        case "JSNonExpressionParen":
        case "InterpolationInTemplate":
        case "InterpolationInJSX":
          if (input[lastIndex] === "/" && (TokensPrecedingExpression.test(lastSignificantToken) || KeywordsWithExpressionAfter.test(lastSignificantToken))) {
            RegularExpressionLiteral.lastIndex = lastIndex;
            if (match = RegularExpressionLiteral.exec(input)) {
              lastIndex = RegularExpressionLiteral.lastIndex;
              lastSignificantToken = match[0];
              postfixIncDec = true;
              yield {
                type: "RegularExpressionLiteral",
                value: match[0],
                closed: match[1] !== void 0 && match[1] !== "\\"
              };
              continue;
            }
          }
          Punctuator.lastIndex = lastIndex;
          if (match = Punctuator.exec(input)) {
            punctuator = match[0];
            nextLastIndex = Punctuator.lastIndex;
            nextLastSignificantToken = punctuator;
            switch (punctuator) {
              case "(":
                if (lastSignificantToken === "?NonExpressionParenKeyword") {
                  stack.push({
                    tag: "JSNonExpressionParen",
                    nesting: parenNesting
                  });
                }
                parenNesting++;
                postfixIncDec = false;
                break;
              case ")":
                parenNesting--;
                postfixIncDec = true;
                if (mode.tag === "JSNonExpressionParen" && parenNesting === mode.nesting) {
                  stack.pop();
                  nextLastSignificantToken = "?NonExpressionParenEnd";
                  postfixIncDec = false;
                }
                break;
              case "{":
                Punctuator.lastIndex = 0;
                isExpression = !TokensNotPrecedingObjectLiteral.test(lastSignificantToken) && (TokensPrecedingExpression.test(lastSignificantToken) || KeywordsWithExpressionAfter.test(lastSignificantToken));
                braces.push(isExpression);
                postfixIncDec = false;
                break;
              case "}":
                switch (mode.tag) {
                  case "InterpolationInTemplate":
                    if (braces.length === mode.nesting) {
                      Template.lastIndex = lastIndex;
                      match = Template.exec(input);
                      lastIndex = Template.lastIndex;
                      lastSignificantToken = match[0];
                      if (match[1] === "${") {
                        lastSignificantToken = "?InterpolationInTemplate";
                        postfixIncDec = false;
                        yield {
                          type: "TemplateMiddle",
                          value: match[0]
                        };
                      } else {
                        stack.pop();
                        postfixIncDec = true;
                        yield {
                          type: "TemplateTail",
                          value: match[0],
                          closed: match[1] === "`"
                        };
                      }
                      continue;
                    }
                    break;
                  case "InterpolationInJSX":
                    if (braces.length === mode.nesting) {
                      stack.pop();
                      lastIndex += 1;
                      lastSignificantToken = "}";
                      yield {
                        type: "JSXPunctuator",
                        value: "}"
                      };
                      continue;
                    }
                }
                postfixIncDec = braces.pop();
                nextLastSignificantToken = postfixIncDec ? "?ExpressionBraceEnd" : "}";
                break;
              case "]":
                postfixIncDec = true;
                break;
              case "++":
              case "--":
                nextLastSignificantToken = postfixIncDec ? "?PostfixIncDec" : "?UnaryIncDec";
                break;
              case "<":
                if (jsx && (TokensPrecedingExpression.test(lastSignificantToken) || KeywordsWithExpressionAfter.test(lastSignificantToken))) {
                  stack.push({ tag: "JSXTag" });
                  lastIndex += 1;
                  lastSignificantToken = "<";
                  yield {
                    type: "JSXPunctuator",
                    value: punctuator
                  };
                  continue;
                }
                postfixIncDec = false;
                break;
              default:
                postfixIncDec = false;
            }
            lastIndex = nextLastIndex;
            lastSignificantToken = nextLastSignificantToken;
            yield {
              type: "Punctuator",
              value: punctuator
            };
            continue;
          }
          Identifier.lastIndex = lastIndex;
          if (match = Identifier.exec(input)) {
            lastIndex = Identifier.lastIndex;
            nextLastSignificantToken = match[0];
            switch (match[0]) {
              case "for":
              case "if":
              case "while":
              case "with":
                if (lastSignificantToken !== "." && lastSignificantToken !== "?.") {
                  nextLastSignificantToken = "?NonExpressionParenKeyword";
                }
            }
            lastSignificantToken = nextLastSignificantToken;
            postfixIncDec = !KeywordsWithExpressionAfter.test(match[0]);
            yield {
              type: match[1] === "#" ? "PrivateIdentifier" : "IdentifierName",
              value: match[0]
            };
            continue;
          }
          StringLiteral.lastIndex = lastIndex;
          if (match = StringLiteral.exec(input)) {
            lastIndex = StringLiteral.lastIndex;
            lastSignificantToken = match[0];
            postfixIncDec = true;
            yield {
              type: "StringLiteral",
              value: match[0],
              closed: match[2] !== void 0
            };
            continue;
          }
          NumericLiteral.lastIndex = lastIndex;
          if (match = NumericLiteral.exec(input)) {
            lastIndex = NumericLiteral.lastIndex;
            lastSignificantToken = match[0];
            postfixIncDec = true;
            yield {
              type: "NumericLiteral",
              value: match[0]
            };
            continue;
          }
          Template.lastIndex = lastIndex;
          if (match = Template.exec(input)) {
            lastIndex = Template.lastIndex;
            lastSignificantToken = match[0];
            if (match[1] === "${") {
              lastSignificantToken = "?InterpolationInTemplate";
              stack.push({
                tag: "InterpolationInTemplate",
                nesting: braces.length
              });
              postfixIncDec = false;
              yield {
                type: "TemplateHead",
                value: match[0]
              };
            } else {
              postfixIncDec = true;
              yield {
                type: "NoSubstitutionTemplate",
                value: match[0],
                closed: match[1] === "`"
              };
            }
            continue;
          }
          break;
        case "JSXTag":
        case "JSXTagEnd":
          JSXPunctuator.lastIndex = lastIndex;
          if (match = JSXPunctuator.exec(input)) {
            lastIndex = JSXPunctuator.lastIndex;
            nextLastSignificantToken = match[0];
            switch (match[0]) {
              case "<":
                stack.push({ tag: "JSXTag" });
                break;
              case ">":
                stack.pop();
                if (lastSignificantToken === "/" || mode.tag === "JSXTagEnd") {
                  nextLastSignificantToken = "?JSX";
                  postfixIncDec = true;
                } else {
                  stack.push({ tag: "JSXChildren" });
                }
                break;
              case "{":
                stack.push({
                  tag: "InterpolationInJSX",
                  nesting: braces.length
                });
                nextLastSignificantToken = "?InterpolationInJSX";
                postfixIncDec = false;
                break;
              case "/":
                if (lastSignificantToken === "<") {
                  stack.pop();
                  if (stack[stack.length - 1].tag === "JSXChildren") {
                    stack.pop();
                  }
                  stack.push({ tag: "JSXTagEnd" });
                }
            }
            lastSignificantToken = nextLastSignificantToken;
            yield {
              type: "JSXPunctuator",
              value: match[0]
            };
            continue;
          }
          JSXIdentifier.lastIndex = lastIndex;
          if (match = JSXIdentifier.exec(input)) {
            lastIndex = JSXIdentifier.lastIndex;
            lastSignificantToken = match[0];
            yield {
              type: "JSXIdentifier",
              value: match[0]
            };
            continue;
          }
          JSXString.lastIndex = lastIndex;
          if (match = JSXString.exec(input)) {
            lastIndex = JSXString.lastIndex;
            lastSignificantToken = match[0];
            yield {
              type: "JSXString",
              value: match[0],
              closed: match[2] !== void 0
            };
            continue;
          }
          break;
        case "JSXChildren":
          JSXText.lastIndex = lastIndex;
          if (match = JSXText.exec(input)) {
            lastIndex = JSXText.lastIndex;
            lastSignificantToken = match[0];
            yield {
              type: "JSXText",
              value: match[0]
            };
            continue;
          }
          switch (input[lastIndex]) {
            case "<":
              stack.push({ tag: "JSXTag" });
              lastIndex++;
              lastSignificantToken = "<";
              yield {
                type: "JSXPunctuator",
                value: "<"
              };
              continue;
            case "{":
              stack.push({
                tag: "InterpolationInJSX",
                nesting: braces.length
              });
              lastIndex++;
              lastSignificantToken = "?InterpolationInJSX";
              postfixIncDec = false;
              yield {
                type: "JSXPunctuator",
                value: "{"
              };
              continue;
          }
      }
      WhiteSpace.lastIndex = lastIndex;
      if (match = WhiteSpace.exec(input)) {
        lastIndex = WhiteSpace.lastIndex;
        yield {
          type: "WhiteSpace",
          value: match[0]
        };
        continue;
      }
      LineTerminatorSequence.lastIndex = lastIndex;
      if (match = LineTerminatorSequence.exec(input)) {
        lastIndex = LineTerminatorSequence.lastIndex;
        postfixIncDec = false;
        if (KeywordsWithNoLineTerminatorAfter.test(lastSignificantToken)) {
          lastSignificantToken = "?NoLineTerminatorHere";
        }
        yield {
          type: "LineTerminatorSequence",
          value: match[0]
        };
        continue;
      }
      MultiLineComment.lastIndex = lastIndex;
      if (match = MultiLineComment.exec(input)) {
        lastIndex = MultiLineComment.lastIndex;
        if (Newline.test(match[0])) {
          postfixIncDec = false;
          if (KeywordsWithNoLineTerminatorAfter.test(lastSignificantToken)) {
            lastSignificantToken = "?NoLineTerminatorHere";
          }
        }
        yield {
          type: "MultiLineComment",
          value: match[0],
          closed: match[1] !== void 0
        };
        continue;
      }
      SingleLineComment.lastIndex = lastIndex;
      if (match = SingleLineComment.exec(input)) {
        lastIndex = SingleLineComment.lastIndex;
        postfixIncDec = false;
        yield {
          type: "SingleLineComment",
          value: match[0]
        };
        continue;
      }
      firstCodePoint = String.fromCodePoint(input.codePointAt(lastIndex));
      lastIndex += firstCodePoint.length;
      lastSignificantToken = firstCodePoint;
      postfixIncDec = false;
      yield {
        type: mode.tag.startsWith("JSX") ? "JSXInvalid" : "Invalid",
        value: firstCodePoint
      };
    }
    return void 0;
  };
  return jsTokens_1;
}
requireJsTokens();
var reservedWords = {
  keyword: [
    "break",
    "case",
    "catch",
    "continue",
    "debugger",
    "default",
    "do",
    "else",
    "finally",
    "for",
    "function",
    "if",
    "return",
    "switch",
    "throw",
    "try",
    "var",
    "const",
    "while",
    "with",
    "new",
    "this",
    "super",
    "class",
    "extends",
    "export",
    "import",
    "null",
    "true",
    "false",
    "in",
    "instanceof",
    "typeof",
    "void",
    "delete"
  ],
  strict: [
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield"
  ]
};
new Set(reservedWords.keyword);
new Set(reservedWords.strict);
const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const intToChar = new Uint8Array(64);
const charToInt = new Uint8Array(128);
for (let i = 0; i < chars.length; i++) {
  const c = chars.charCodeAt(i);
  intToChar[i] = c;
  charToInt[c] = i;
}
var UrlType;
(function(UrlType2) {
  UrlType2[UrlType2["Empty"] = 1] = "Empty";
  UrlType2[UrlType2["Hash"] = 2] = "Hash";
  UrlType2[UrlType2["Query"] = 3] = "Query";
  UrlType2[UrlType2["RelativePath"] = 4] = "RelativePath";
  UrlType2[UrlType2["AbsolutePath"] = 5] = "AbsolutePath";
  UrlType2[UrlType2["SchemeRelative"] = 6] = "SchemeRelative";
  UrlType2[UrlType2["Absolute"] = 7] = "Absolute";
})(UrlType || (UrlType = {}));
const { now } = Date;
class ModuleMocker {
  constructor(interceptor, rpc2, spyOn, config) {
    __publicField(this, "registry", new MockerRegistry());
    __publicField(this, "queue", /* @__PURE__ */ new Set());
    __publicField(this, "mockedIds", /* @__PURE__ */ new Set());
    this.interceptor = interceptor;
    this.rpc = rpc2;
    this.spyOn = spyOn;
    this.config = config;
  }
  async prepare() {
    if (!this.queue.size) {
      return;
    }
    await Promise.all([...this.queue.values()]);
  }
  async resolveFactoryModule(id) {
    const mock = this.registry.get(id);
    if (!mock || mock.type !== "manual") {
      throw new Error(`Mock ${id} wasn't registered. This is probably a Vitest error. Please, open a new issue with reproduction.`);
    }
    const result = await mock.resolve();
    return result;
  }
  getFactoryModule(id) {
    const mock = this.registry.get(id);
    if (!mock || mock.type !== "manual") {
      throw new Error(`Mock ${id} wasn't registered. This is probably a Vitest error. Please, open a new issue with reproduction.`);
    }
    if (!mock.cache) {
      throw new Error(`Mock ${id} wasn't resolved. This is probably a Vitest error. Please, open a new issue with reproduction.`);
    }
    return mock.cache;
  }
  async invalidate() {
    const ids = Array.from(this.mockedIds);
    if (!ids.length) {
      return;
    }
    await this.rpc.invalidate(ids);
    await this.interceptor.invalidate();
    this.registry.clear();
  }
  async importActual(id, importer) {
    const resolved = await this.rpc.resolveId(id, importer);
    if (resolved == null) {
      throw new Error(`[vitest] Cannot resolve "${id}" imported from "${importer}"`);
    }
    const ext = extname(resolved.id);
    const url2 = new URL(resolved.url, location.href);
    const query = `_vitest_original&ext${ext}`;
    const actualUrl = `${url2.pathname}${url2.search ? `${url2.search}&${query}` : `?${query}`}${url2.hash}`;
    return this.wrapDynamicImport(() => import(
      /* @vite-ignore */
      actualUrl
    )).then((mod) => {
      if (!resolved.optimized || typeof mod.default === "undefined") {
        return mod;
      }
      const m = mod.default;
      return (m === null || m === void 0 ? void 0 : m.__esModule) ? m : {
        ...typeof m === "object" && !Array.isArray(m) || typeof m === "function" ? m : {},
        default: m
      };
    });
  }
  async importMock(rawId, importer) {
    await this.prepare();
    const { resolvedId, resolvedUrl, redirectUrl } = await this.rpc.resolveMock(rawId, importer, { mock: "auto" });
    const mockUrl = this.resolveMockPath(cleanVersion(resolvedUrl));
    let mock = this.registry.get(mockUrl);
    if (!mock) {
      if (redirectUrl) {
        const resolvedRedirect = new URL(this.resolveMockPath(cleanVersion(redirectUrl)), location.href).toString();
        mock = new RedirectedModule(rawId, resolvedId, mockUrl, resolvedRedirect);
      } else {
        mock = new AutomockedModule(rawId, resolvedId, mockUrl);
      }
    }
    if (mock.type === "manual") {
      return await mock.resolve();
    }
    if (mock.type === "automock" || mock.type === "autospy") {
      const url2 = new URL(`/@id/${resolvedId}`, location.href);
      const query = url2.search ? `${url2.search}&t=${now()}` : `?t=${now()}`;
      const moduleObject = await __vitePreload(() => import(
        /* @vite-ignore */
        `${url2.pathname}${query}&mock=${mock.type}${url2.hash}`
      ), true ? [] : void 0);
      return this.mockObject(moduleObject, mock.type);
    }
    return import(
      /* @vite-ignore */
      mock.redirect
    );
  }
  mockObject(object2, moduleType = "automock") {
    return mockObject({
      globalConstructors: {
        Object,
        Function,
        Array,
        Map,
        RegExp
      },
      spyOn: this.spyOn,
      type: moduleType
    }, object2);
  }
  queueMock(rawId, importer, factoryOrOptions) {
    const promise = this.rpc.resolveMock(rawId, importer, { mock: typeof factoryOrOptions === "function" ? "factory" : (factoryOrOptions === null || factoryOrOptions === void 0 ? void 0 : factoryOrOptions.spy) ? "spy" : "auto" }).then(async ({ redirectUrl, resolvedId, resolvedUrl, needsInterop, mockType }) => {
      const mockUrl = this.resolveMockPath(cleanVersion(resolvedUrl));
      this.mockedIds.add(resolvedId);
      const factory = typeof factoryOrOptions === "function" ? async () => {
        const data = await factoryOrOptions();
        return needsInterop ? { default: data } : data;
      } : void 0;
      const mockRedirect = typeof redirectUrl === "string" ? new URL(this.resolveMockPath(cleanVersion(redirectUrl)), location.href).toString() : null;
      let module;
      if (mockType === "manual") {
        module = this.registry.register("manual", rawId, resolvedId, mockUrl, factory);
      } else if (mockType === "autospy") {
        module = this.registry.register("autospy", rawId, resolvedId, mockUrl);
      } else if (mockType === "redirect") {
        module = this.registry.register("redirect", rawId, resolvedId, mockUrl, mockRedirect);
      } else {
        module = this.registry.register("automock", rawId, resolvedId, mockUrl);
      }
      await this.interceptor.register(module);
    }).finally(() => {
      this.queue.delete(promise);
    });
    this.queue.add(promise);
  }
  queueUnmock(id, importer) {
    const promise = this.rpc.resolveId(id, importer).then(async (resolved) => {
      if (!resolved) {
        return;
      }
      const mockUrl = this.resolveMockPath(cleanVersion(resolved.url));
      this.mockedIds.add(resolved.id);
      this.registry.delete(mockUrl);
      await this.interceptor.delete(mockUrl);
    }).finally(() => {
      this.queue.delete(promise);
    });
    this.queue.add(promise);
  }
  // We need to await mock registration before importing the actual module
  // In case there is a mocked module in the import chain
  wrapDynamicImport(moduleFactory) {
    if (typeof moduleFactory === "function") {
      const promise = new Promise((resolve2, reject) => {
        this.prepare().finally(() => {
          moduleFactory().then(resolve2, reject);
        });
      });
      return promise;
    }
    return moduleFactory;
  }
  resolveMockPath(path) {
    const config = this.config;
    const fsRoot = join("/@fs/", config.root);
    if (path.startsWith(config.root)) {
      return path.slice(config.root.length);
    }
    if (path.startsWith(fsRoot)) {
      return path.slice(fsRoot.length);
    }
    return path;
  }
}
const versionRegexp = /(\?|&)v=\w{8}/;
function cleanVersion(url2) {
  return url2.replace(versionRegexp, "");
}
class VitestBrowserClientMocker extends ModuleMocker {
  // default "vi" utility tries to access mock context to avoid circular dependencies
  getMockContext() {
    return { callstack: null };
  }
  wrapDynamicImport(moduleFactory) {
    return getBrowserState().wrapModule(moduleFactory);
  }
}
function createModuleMockerInterceptor() {
  return {
    async register(module) {
      const state = getBrowserState();
      await rpc().registerMock(state.sessionId, module.toJSON());
    },
    async delete(id) {
      const state = getBrowserState();
      await rpc().unregisterMock(state.sessionId, id);
    },
    async invalidate() {
      const state = getBrowserState();
      await rpc().clearMocks(state.sessionId);
    }
  };
}
function rpc() {
  return getWorkerState().rpc;
}
getBrowserState().provider;
class CommandsManager {
  constructor() {
    __publicField(this, "_listeners", []);
  }
  onCommand(listener) {
    this._listeners.push(listener);
  }
  async triggerCommand(command, args, clientError = new Error("empty")) {
    var _a, _b;
    const state = getWorkerState();
    const rpc2 = state.rpc;
    const { sessionId } = getBrowserState();
    const filepath = state.filepath || ((_b = (_a = state.current) == null ? void 0 : _a.file) == null ? void 0 : _b.filepath);
    args = args.filter((arg) => arg !== void 0);
    if (this._listeners.length) {
      await Promise.all(this._listeners.map((listener) => listener(command, args)));
    }
    return rpc2.triggerCommand(sessionId, command, filepath, args).catch((err) => {
      var _a2;
      clientError.message = err.message;
      clientError.name = err.name;
      clientError.stack = (_a2 = clientError.stack) == null ? void 0 : _a2.replace(clientError.message, err.message);
      throw clientError;
    });
  }
}
const debugVar = getConfig().env.VITEST_BROWSER_DEBUG;
const debug = debugVar && debugVar !== "false" ? (...args) => {
  var _a, _b;
  return (_b = (_a = client.rpc).debug) == null ? void 0 : _b.call(_a, ...args.map(String));
} : void 0;
channel.addEventListener("message", async (e) => {
  await client.waitForConnection();
  const data = e.data;
  debug == null ? void 0 : debug("event from orchestrator", JSON.stringify(e.data));
  if (!isEvent(data)) {
    const error = new Error(`Unknown message: ${JSON.stringify(e.data)}`);
    unhandledError(error, "Uknown Iframe Message");
    return;
  }
  if (!("iframeId" in data) || data.iframeId !== getBrowserState().iframeId) {
    return;
  }
  switch (data.event) {
    case "execute": {
      const { method, files, context } = data;
      const state = getWorkerState();
      const parsedContext = parse(context);
      state.ctx.providedContext = parsedContext;
      state.providedContext = parsedContext;
      if (method === "collect") {
        await executeTests("collect", files).catch((err) => unhandledError(err, "Collect Error"));
      } else {
        await executeTests("run", files).catch((err) => unhandledError(err, "Run Error"));
      }
      break;
    }
    case "cleanup": {
      await cleanup().catch((err) => unhandledError(err, "Cleanup Error"));
      break;
    }
    case "prepare": {
      await prepare(data).catch((err) => unhandledError(err, "Prepare Error"));
      break;
    }
    case "viewport:done":
    case "viewport:fail":
    case "viewport": {
      break;
    }
    default: {
      const error = new Error(`Unknown event: ${data.event}`);
      unhandledError(error, "Uknown Event");
    }
  }
  channel.postMessage({
    event: `response:${data.event}`,
    iframeId: getBrowserState().iframeId
  });
});
const url = new URL(location.href);
const reloadStart = url.searchParams.get("__reloadStart");
const iframeId = url.searchParams.get("iframeId");
const commands = new CommandsManager();
getBrowserState().commands = commands;
getBrowserState().iframeId = iframeId;
let contextSwitched = false;
async function prepareTestEnvironment(options) {
  debug == null ? void 0 : debug("trying to resolve runner", `${reloadStart}`);
  const config = getConfig();
  const rpc2 = createSafeRpc(client);
  const state = getWorkerState();
  state.onCancel = onCancel;
  state.rpc = rpc2;
  const interceptor = createModuleMockerInterceptor();
  const mocker = new VitestBrowserClientMocker(
    interceptor,
    rpc2,
    SpyModule.spyOn,
    {
      root: getBrowserState().viteConfig.root
    }
  );
  globalThis.__vitest_mocker__ = mocker;
  setupConsoleLogSpy();
  setupDialogsSpy();
  const runner = await initiateRunner(state, mocker, config);
  getBrowserState().runner = runner;
  if (server.provider === "webdriverio") {
    let switchPromise = null;
    commands.onCommand(async () => {
      if (switchPromise) {
        await switchPromise;
      }
      if (!contextSwitched) {
        switchPromise = rpc2.wdioSwitchContext("iframe").finally(() => {
          switchPromise = null;
          contextSwitched = true;
        });
        await switchPromise;
      }
    });
  }
  state.durations.prepare = performance.now() - options.startTime;
  return {
    runner,
    config,
    state
  };
}
let preparedData;
async function executeTests(method, files) {
  if (!preparedData) {
    throw new Error(`Data was not properly initialized. This is a bug in Vitest. Please, open a new issue with reproduction.`);
  }
  debug == null ? void 0 : debug("runner resolved successfully");
  const { runner, state } = preparedData;
  state.ctx.files = files;
  runner.setMethod(method);
  const version = url.searchParams.get("browserv") || "";
  files.forEach((filename) => {
    const currentVersion = browserHashMap.get(filename);
    if (!currentVersion || currentVersion[1] !== version) {
      browserHashMap.set(filename, version);
    }
  });
  debug == null ? void 0 : debug("prepare time", state.durations.prepare, "ms");
  for (const file of files) {
    state.filepath = file;
    if (method === "run") {
      await startTests([file], runner);
    } else {
      await collectTests([file], runner);
    }
  }
}
async function prepare(options) {
  preparedData = await prepareTestEnvironment(options);
  debug == null ? void 0 : debug("runner resolved successfully");
  const { config, state } = preparedData;
  state.durations.prepare = performance.now() - state.durations.prepare;
  debug == null ? void 0 : debug("prepare time", state.durations.prepare, "ms");
  await Promise.all([
    setupCommonEnv(config),
    startCoverageInsideWorker(config.coverage, executor, { isolate: config.browser.isolate }),
    (async () => {
      const VitestIndex = await __vitePreload(() => import("vitest"), true ? [] : void 0);
      Object.defineProperty(window, "__vitest_index__", {
        value: VitestIndex,
        enumerable: false
      });
    })()
  ]);
}
async function cleanup() {
  const state = getWorkerState();
  const config = getConfig();
  const rpc2 = state.rpc;
  const cleanupSymbol = Symbol.for("vitest:component-cleanup");
  if (cleanupSymbol in page) {
    try {
      await page[cleanupSymbol]();
    } catch (error) {
      await unhandledError(error, "Cleanup Error");
    }
  }
  await userEvent.cleanup().catch((error) => unhandledError(error, "Cleanup Error"));
  await Promise.all(
    getBrowserState().cleanups.map((fn) => fn())
  ).catch((error) => unhandledError(error, "Cleanup Error"));
  if (contextSwitched) {
    await rpc2.wdioSwitchContext("parent").catch((error) => unhandledError(error, "Cleanup Error"));
  }
  state.environmentTeardownRun = true;
  await stopCoverageInsideWorker(config.coverage, executor, { isolate: config.browser.isolate }).catch((error) => {
    return unhandledError(error, "Coverage Error");
  });
}
function unhandledError(e, type) {
  return client.rpc.onUnhandledError({
    name: e.name,
    message: e.message,
    stack: e.stack
  }, type).catch(() => {
  });
}
function isEvent(data) {
  return typeof data === "object" && !!data && "event" in data;
}
