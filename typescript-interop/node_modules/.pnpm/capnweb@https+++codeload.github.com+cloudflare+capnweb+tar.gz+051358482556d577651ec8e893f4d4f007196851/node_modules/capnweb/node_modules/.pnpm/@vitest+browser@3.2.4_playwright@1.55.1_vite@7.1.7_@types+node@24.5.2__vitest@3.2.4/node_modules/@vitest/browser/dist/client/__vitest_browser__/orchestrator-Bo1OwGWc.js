var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
import { g as getBrowserState, a as getConfig, r as relative } from "./utils-Owv5OOOf.js";
import { channel, globalChannel, client } from "@vitest/browser/client";
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
const ansiColors = {
  bold: ["1", "22"],
  dim: ["2", "22"],
  italic: ["3", "23"],
  underline: ["4", "24"],
  // 5 & 6 are blinking
  inverse: ["7", "27"],
  hidden: ["8", "28"],
  strike: ["9", "29"],
  // 10-20 are fonts
  // 21-29 are resets for 1-9
  black: ["30", "39"],
  red: ["31", "39"],
  green: ["32", "39"],
  yellow: ["33", "39"],
  blue: ["34", "39"],
  magenta: ["35", "39"],
  cyan: ["36", "39"],
  white: ["37", "39"],
  brightblack: ["30;1", "39"],
  brightred: ["31;1", "39"],
  brightgreen: ["32;1", "39"],
  brightyellow: ["33;1", "39"],
  brightblue: ["34;1", "39"],
  brightmagenta: ["35;1", "39"],
  brightcyan: ["36;1", "39"],
  brightwhite: ["37;1", "39"],
  grey: ["90", "39"]
};
const styles = {
  special: "cyan",
  number: "yellow",
  bigint: "yellow",
  boolean: "yellow",
  undefined: "grey",
  null: "bold",
  string: "green",
  symbol: "green",
  date: "magenta",
  regexp: "red"
};
const truncator = "…";
function colorise(value, styleType) {
  const color = ansiColors[styles[styleType]] || ansiColors[styleType] || "";
  if (!color) {
    return String(value);
  }
  return `\x1B[${color[0]}m${String(value)}\x1B[${color[1]}m`;
}
function normaliseOptions({
  showHidden = false,
  depth = 2,
  colors = false,
  customInspect = true,
  showProxy = false,
  maxArrayLength = Infinity,
  breakLength = Infinity,
  seen = [],
  // eslint-disable-next-line no-shadow
  truncate: truncate2 = Infinity,
  stylize = String
} = {}, inspect2) {
  const options = {
    showHidden: Boolean(showHidden),
    depth: Number(depth),
    colors: Boolean(colors),
    customInspect: Boolean(customInspect),
    showProxy: Boolean(showProxy),
    maxArrayLength: Number(maxArrayLength),
    breakLength: Number(breakLength),
    truncate: Number(truncate2),
    seen,
    inspect: inspect2,
    stylize
  };
  if (options.colors) {
    options.stylize = colorise;
  }
  return options;
}
function isHighSurrogate(char) {
  return char >= "\uD800" && char <= "\uDBFF";
}
function truncate(string, length, tail = truncator) {
  string = String(string);
  const tailLength = tail.length;
  const stringLength = string.length;
  if (tailLength > length && stringLength > tailLength) {
    return tail;
  }
  if (stringLength > length && stringLength > tailLength) {
    let end = length - tailLength;
    if (end > 0 && isHighSurrogate(string[end - 1])) {
      end = end - 1;
    }
    return `${string.slice(0, end)}${tail}`;
  }
  return string;
}
function inspectList(list, options, inspectItem, separator = ", ") {
  inspectItem = inspectItem || options.inspect;
  const size = list.length;
  if (size === 0)
    return "";
  const originalLength = options.truncate;
  let output = "";
  let peek = "";
  let truncated = "";
  for (let i = 0; i < size; i += 1) {
    const last = i + 1 === list.length;
    const secondToLast = i + 2 === list.length;
    truncated = `${truncator}(${list.length - i})`;
    const value = list[i];
    options.truncate = originalLength - output.length - (last ? 0 : separator.length);
    const string = peek || inspectItem(value, options) + (last ? "" : separator);
    const nextLength = output.length + string.length;
    const truncatedLength = nextLength + truncated.length;
    if (last && nextLength > originalLength && output.length + truncated.length <= originalLength) {
      break;
    }
    if (!last && !secondToLast && truncatedLength > originalLength) {
      break;
    }
    peek = last ? "" : inspectItem(list[i + 1], options) + (secondToLast ? "" : separator);
    if (!last && secondToLast && truncatedLength > originalLength && nextLength + peek.length > originalLength) {
      break;
    }
    output += string;
    if (!last && !secondToLast && nextLength + peek.length >= originalLength) {
      truncated = `${truncator}(${list.length - i - 1})`;
      break;
    }
    truncated = "";
  }
  return `${output}${truncated}`;
}
function quoteComplexKey(key) {
  if (key.match(/^[a-zA-Z_][a-zA-Z_0-9]*$/)) {
    return key;
  }
  return JSON.stringify(key).replace(/'/g, "\\'").replace(/\\"/g, '"').replace(/(^"|"$)/g, "'");
}
function inspectProperty([key, value], options) {
  options.truncate -= 2;
  if (typeof key === "string") {
    key = quoteComplexKey(key);
  } else if (typeof key !== "number") {
    key = `[${options.inspect(key, options)}]`;
  }
  options.truncate -= key.length;
  value = options.inspect(value, options);
  return `${key}: ${value}`;
}
function inspectArray(array, options) {
  const nonIndexProperties = Object.keys(array).slice(array.length);
  if (!array.length && !nonIndexProperties.length)
    return "[]";
  options.truncate -= 4;
  const listContents = inspectList(array, options);
  options.truncate -= listContents.length;
  let propertyContents = "";
  if (nonIndexProperties.length) {
    propertyContents = inspectList(nonIndexProperties.map((key) => [key, array[key]]), options, inspectProperty);
  }
  return `[ ${listContents}${propertyContents ? `, ${propertyContents}` : ""} ]`;
}
const getArrayName = (array) => {
  if (typeof Buffer === "function" && array instanceof Buffer) {
    return "Buffer";
  }
  if (array[Symbol.toStringTag]) {
    return array[Symbol.toStringTag];
  }
  return array.constructor.name;
};
function inspectTypedArray(array, options) {
  const name = getArrayName(array);
  options.truncate -= name.length + 4;
  const nonIndexProperties = Object.keys(array).slice(array.length);
  if (!array.length && !nonIndexProperties.length)
    return `${name}[]`;
  let output = "";
  for (let i = 0; i < array.length; i++) {
    const string = `${options.stylize(truncate(array[i], options.truncate), "number")}${i === array.length - 1 ? "" : ", "}`;
    options.truncate -= string.length;
    if (array[i] !== array.length && options.truncate <= 3) {
      output += `${truncator}(${array.length - array[i] + 1})`;
      break;
    }
    output += string;
  }
  let propertyContents = "";
  if (nonIndexProperties.length) {
    propertyContents = inspectList(nonIndexProperties.map((key) => [key, array[key]]), options, inspectProperty);
  }
  return `${name}[ ${output}${propertyContents ? `, ${propertyContents}` : ""} ]`;
}
function inspectDate(dateObject, options) {
  const stringRepresentation = dateObject.toJSON();
  if (stringRepresentation === null) {
    return "Invalid Date";
  }
  const split = stringRepresentation.split("T");
  const date = split[0];
  return options.stylize(`${date}T${truncate(split[1], options.truncate - date.length - 1)}`, "date");
}
function inspectFunction(func, options) {
  const functionType = func[Symbol.toStringTag] || "Function";
  const name = func.name;
  if (!name) {
    return options.stylize(`[${functionType}]`, "special");
  }
  return options.stylize(`[${functionType} ${truncate(name, options.truncate - 11)}]`, "special");
}
function inspectMapEntry([key, value], options) {
  options.truncate -= 4;
  key = options.inspect(key, options);
  options.truncate -= key.length;
  value = options.inspect(value, options);
  return `${key} => ${value}`;
}
function mapToEntries(map) {
  const entries = [];
  map.forEach((value, key) => {
    entries.push([key, value]);
  });
  return entries;
}
function inspectMap(map, options) {
  if (map.size === 0)
    return "Map{}";
  options.truncate -= 7;
  return `Map{ ${inspectList(mapToEntries(map), options, inspectMapEntry)} }`;
}
const isNaN = Number.isNaN || ((i) => i !== i);
function inspectNumber(number, options) {
  if (isNaN(number)) {
    return options.stylize("NaN", "number");
  }
  if (number === Infinity) {
    return options.stylize("Infinity", "number");
  }
  if (number === -Infinity) {
    return options.stylize("-Infinity", "number");
  }
  if (number === 0) {
    return options.stylize(1 / number === Infinity ? "+0" : "-0", "number");
  }
  return options.stylize(truncate(String(number), options.truncate), "number");
}
function inspectBigInt(number, options) {
  let nums = truncate(number.toString(), options.truncate - 1);
  if (nums !== truncator)
    nums += "n";
  return options.stylize(nums, "bigint");
}
function inspectRegExp(value, options) {
  const flags = value.toString().split("/")[2];
  const sourceLength = options.truncate - (2 + flags.length);
  const source = value.source;
  return options.stylize(`/${truncate(source, sourceLength)}/${flags}`, "regexp");
}
function arrayFromSet(set) {
  const values = [];
  set.forEach((value) => {
    values.push(value);
  });
  return values;
}
function inspectSet(set, options) {
  if (set.size === 0)
    return "Set{}";
  options.truncate -= 7;
  return `Set{ ${inspectList(arrayFromSet(set), options)} }`;
}
const stringEscapeChars = new RegExp("['\\u0000-\\u001f\\u007f-\\u009f\\u00ad\\u0600-\\u0604\\u070f\\u17b4\\u17b5\\u200c-\\u200f\\u2028-\\u202f\\u2060-\\u206f\\ufeff\\ufff0-\\uffff]", "g");
const escapeCharacters = {
  "\b": "\\b",
  "	": "\\t",
  "\n": "\\n",
  "\f": "\\f",
  "\r": "\\r",
  "'": "\\'",
  "\\": "\\\\"
};
const hex = 16;
function escape(char) {
  return escapeCharacters[char] || `\\u${`0000${char.charCodeAt(0).toString(hex)}`.slice(-4)}`;
}
function inspectString(string, options) {
  if (stringEscapeChars.test(string)) {
    string = string.replace(stringEscapeChars, escape);
  }
  return options.stylize(`'${truncate(string, options.truncate - 2)}'`, "string");
}
function inspectSymbol(value) {
  if ("description" in Symbol.prototype) {
    return value.description ? `Symbol(${value.description})` : "Symbol()";
  }
  return value.toString();
}
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
function inspectObject$1(object, options) {
  const properties = Object.getOwnPropertyNames(object);
  const symbols = Object.getOwnPropertySymbols ? Object.getOwnPropertySymbols(object) : [];
  if (properties.length === 0 && symbols.length === 0) {
    return "{}";
  }
  options.truncate -= 4;
  options.seen = options.seen || [];
  if (options.seen.includes(object)) {
    return "[Circular]";
  }
  options.seen.push(object);
  const propertyContents = inspectList(properties.map((key) => [key, object[key]]), options, inspectProperty);
  const symbolContents = inspectList(symbols.map((key) => [key, object[key]]), options, inspectProperty);
  options.seen.pop();
  let sep = "";
  if (propertyContents && symbolContents) {
    sep = ", ";
  }
  return `{ ${propertyContents}${sep}${symbolContents} }`;
}
const toStringTag = typeof Symbol !== "undefined" && Symbol.toStringTag ? Symbol.toStringTag : false;
function inspectClass(value, options) {
  let name = "";
  if (toStringTag && toStringTag in value) {
    name = value[toStringTag];
  }
  name = name || value.constructor.name;
  if (!name || name === "_class") {
    name = "<Anonymous Class>";
  }
  options.truncate -= name.length;
  return `${name}${inspectObject$1(value, options)}`;
}
function inspectArguments(args, options) {
  if (args.length === 0)
    return "Arguments[]";
  options.truncate -= 13;
  return `Arguments[ ${inspectList(args, options)} ]`;
}
const errorKeys = [
  "stack",
  "line",
  "column",
  "name",
  "message",
  "fileName",
  "lineNumber",
  "columnNumber",
  "number",
  "description",
  "cause"
];
function inspectObject(error, options) {
  const properties = Object.getOwnPropertyNames(error).filter((key) => errorKeys.indexOf(key) === -1);
  const name = error.name;
  options.truncate -= name.length;
  let message = "";
  if (typeof error.message === "string") {
    message = truncate(error.message, options.truncate);
  } else {
    properties.unshift("message");
  }
  message = message ? `: ${message}` : "";
  options.truncate -= message.length + 5;
  options.seen = options.seen || [];
  if (options.seen.includes(error)) {
    return "[Circular]";
  }
  options.seen.push(error);
  const propertyContents = inspectList(properties.map((key) => [key, error[key]]), options, inspectProperty);
  return `${name}${message}${propertyContents ? ` { ${propertyContents} }` : ""}`;
}
function inspectAttribute([key, value], options) {
  options.truncate -= 3;
  if (!value) {
    return `${options.stylize(String(key), "yellow")}`;
  }
  return `${options.stylize(String(key), "yellow")}=${options.stylize(`"${value}"`, "string")}`;
}
function inspectNodeCollection(collection, options) {
  return inspectList(collection, options, inspectNode, "\n");
}
function inspectNode(node, options) {
  switch (node.nodeType) {
    case 1:
      return inspectHTML(node, options);
    case 3:
      return options.inspect(node.data, options);
    default:
      return options.inspect(node, options);
  }
}
function inspectHTML(element, options) {
  const properties = element.getAttributeNames();
  const name = element.tagName.toLowerCase();
  const head = options.stylize(`<${name}`, "special");
  const headClose = options.stylize(`>`, "special");
  const tail = options.stylize(`</${name}>`, "special");
  options.truncate -= name.length * 2 + 5;
  let propertyContents = "";
  if (properties.length > 0) {
    propertyContents += " ";
    propertyContents += inspectList(properties.map((key) => [key, element.getAttribute(key)]), options, inspectAttribute, " ");
  }
  options.truncate -= propertyContents.length;
  const truncate2 = options.truncate;
  let children = inspectNodeCollection(element.children, options);
  if (children && children.length > truncate2) {
    children = `${truncator}(${element.children.length})`;
  }
  return `${head}${propertyContents}${headClose}${children}${tail}`;
}
const symbolsSupported = typeof Symbol === "function" && typeof Symbol.for === "function";
const chaiInspect = symbolsSupported ? Symbol.for("chai/inspect") : "@@chai/inspect";
const nodeInspect = Symbol.for("nodejs.util.inspect.custom");
const constructorMap = /* @__PURE__ */ new WeakMap();
const stringTagMap = {};
const baseTypesMap = {
  undefined: (value, options) => options.stylize("undefined", "undefined"),
  null: (value, options) => options.stylize("null", "null"),
  boolean: (value, options) => options.stylize(String(value), "boolean"),
  Boolean: (value, options) => options.stylize(String(value), "boolean"),
  number: inspectNumber,
  Number: inspectNumber,
  bigint: inspectBigInt,
  BigInt: inspectBigInt,
  string: inspectString,
  String: inspectString,
  function: inspectFunction,
  Function: inspectFunction,
  symbol: inspectSymbol,
  // A Symbol polyfill will return `Symbol` not `symbol` from typedetect
  Symbol: inspectSymbol,
  Array: inspectArray,
  Date: inspectDate,
  Map: inspectMap,
  Set: inspectSet,
  RegExp: inspectRegExp,
  Promise: getPromiseValue,
  // WeakSet, WeakMap are totally opaque to us
  WeakSet: (value, options) => options.stylize("WeakSet{…}", "special"),
  WeakMap: (value, options) => options.stylize("WeakMap{…}", "special"),
  Arguments: inspectArguments,
  Int8Array: inspectTypedArray,
  Uint8Array: inspectTypedArray,
  Uint8ClampedArray: inspectTypedArray,
  Int16Array: inspectTypedArray,
  Uint16Array: inspectTypedArray,
  Int32Array: inspectTypedArray,
  Uint32Array: inspectTypedArray,
  Float32Array: inspectTypedArray,
  Float64Array: inspectTypedArray,
  Generator: () => "",
  DataView: () => "",
  ArrayBuffer: () => "",
  Error: inspectObject,
  HTMLCollection: inspectNodeCollection,
  NodeList: inspectNodeCollection
};
const inspectCustom = (value, options, type) => {
  if (chaiInspect in value && typeof value[chaiInspect] === "function") {
    return value[chaiInspect](options);
  }
  if (nodeInspect in value && typeof value[nodeInspect] === "function") {
    return value[nodeInspect](options.depth, options);
  }
  if ("inspect" in value && typeof value.inspect === "function") {
    return value.inspect(options.depth, options);
  }
  if ("constructor" in value && constructorMap.has(value.constructor)) {
    return constructorMap.get(value.constructor)(value, options);
  }
  if (stringTagMap[type]) {
    return stringTagMap[type](value, options);
  }
  return "";
};
const toString = Object.prototype.toString;
function inspect$1(value, opts = {}) {
  const options = normaliseOptions(opts, inspect$1);
  const { customInspect } = options;
  let type = value === null ? "null" : typeof value;
  if (type === "object") {
    type = toString.call(value).slice(8, -1);
  }
  if (type in baseTypesMap) {
    return baseTypesMap[type](value, options);
  }
  if (customInspect && value) {
    const output = inspectCustom(value, options, type);
    if (output) {
      if (typeof output === "string")
        return output;
      return inspect$1(output, options);
    }
  }
  const proto = value ? Object.getPrototypeOf(value) : false;
  if (proto === Object.prototype || proto === null) {
    return inspectObject$1(value, options);
  }
  if (value && typeof HTMLElement === "function" && value instanceof HTMLElement) {
    return inspectHTML(value, options);
  }
  if ("constructor" in value) {
    if (value.constructor !== Object) {
      return inspectClass(value, options);
    }
    return inspectObject$1(value, options);
  }
  if (value === Object(value)) {
    return inspectObject$1(value, options);
  }
  return options.stylize(String(value), type);
}
const formatRegExp = /%[sdjifoOc%]/g;
function format(...args) {
  if (typeof args[0] !== "string") {
    const objects = [];
    for (let i2 = 0; i2 < args.length; i2++) {
      objects.push(inspect(args[i2], {
        depth: 0,
        colors: false
      }));
    }
    return objects.join(" ");
  }
  const len = args.length;
  let i = 1;
  const template = args[0];
  let str = String(template).replace(formatRegExp, (x) => {
    if (x === "%%") {
      return "%";
    }
    if (i >= len) {
      return x;
    }
    switch (x) {
      case "%s": {
        const value = args[i++];
        if (typeof value === "bigint") {
          return `${value.toString()}n`;
        }
        if (typeof value === "number" && value === 0 && 1 / value < 0) {
          return "-0";
        }
        if (typeof value === "object" && value !== null) {
          if (typeof value.toString === "function" && value.toString !== Object.prototype.toString) {
            return value.toString();
          }
          return inspect(value, {
            depth: 0,
            colors: false
          });
        }
        return String(value);
      }
      case "%d": {
        const value = args[i++];
        if (typeof value === "bigint") {
          return `${value.toString()}n`;
        }
        return Number(value).toString();
      }
      case "%i": {
        const value = args[i++];
        if (typeof value === "bigint") {
          return `${value.toString()}n`;
        }
        return Number.parseInt(String(value)).toString();
      }
      case "%f":
        return Number.parseFloat(String(args[i++])).toString();
      case "%o":
        return inspect(args[i++], {
          showHidden: true,
          showProxy: true
        });
      case "%O":
        return inspect(args[i++]);
      case "%c": {
        i++;
        return "";
      }
      case "%j":
        try {
          return JSON.stringify(args[i++]);
        } catch (err) {
          const m = err.message;
          if (m.includes("circular structure") || m.includes("cyclic structures") || m.includes("cyclic object")) {
            return "[Circular]";
          }
          throw err;
        }
      default:
        return x;
    }
  });
  for (let x = args[i]; i < len; x = args[++i]) {
    if (x === null || typeof x !== "object") {
      str += ` ${x}`;
    } else {
      str += ` ${inspect(x)}`;
    }
  }
  return str;
}
function inspect(obj, options = {}) {
  if (options.truncate === 0) {
    options.truncate = Number.POSITIVE_INFINITY;
  }
  return inspect$1(obj, options);
}
function objDisplay(obj, options = {}) {
  if (typeof options.truncate === "undefined") {
    options.truncate = 40;
  }
  const str = inspect(obj, options);
  const type = Object.prototype.toString.call(obj);
  if (options.truncate && str.length >= options.truncate) {
    if (type === "[object Function]") {
      const fn = obj;
      return !fn.name ? "[Function]" : `[Function: ${fn.name}]`;
    } else if (type === "[object Array]") {
      return `[ Array(${obj.length}) ]`;
    } else if (type === "[object Object]") {
      const keys = Object.keys(obj);
      const kstr = keys.length > 2 ? `${keys.splice(0, 2).join(", ")}, ...` : keys.join(", ");
      return `{ Object (${kstr}) }`;
    } else {
      return str;
    }
  }
  return str;
}
function toArray(array) {
  if (array === null || array === void 0) {
    array = [];
  }
  if (Array.isArray(array)) {
    return array;
  }
  return [array];
}
function isObject(item) {
  return item != null && typeof item === "object" && !Array.isArray(item);
}
function objectAttr(source, path, defaultValue = void 0) {
  const paths = path.replace(/\[(\d+)\]/g, ".$1").split(".");
  let result = source;
  for (const p2 of paths) {
    result = new Object(result)[p2];
    if (result === void 0) {
      return defaultValue;
    }
  }
  return result;
}
function createDefer() {
  let resolve2 = null;
  let reject = null;
  const p2 = new Promise((_resolve, _reject) => {
    resolve2 = _resolve;
    reject = _reject;
  });
  p2.resolve = resolve2;
  p2.reject = reject;
  return p2;
}
function isNegativeNaN(val) {
  if (!Number.isNaN(val)) {
    return false;
  }
  const f64 = new Float64Array(1);
  f64[0] = val;
  const u32 = new Uint32Array(f64.buffer);
  const isNegative = u32[1] >>> 31 === 1;
  return isNegative;
}
var jsTokens_1$1;
var hasRequiredJsTokens$1;
function requireJsTokens$1() {
  if (hasRequiredJsTokens$1) return jsTokens_1$1;
  hasRequiredJsTokens$1 = 1;
  var Identifier, JSXIdentifier, JSXPunctuator, JSXString, JSXText, KeywordsWithExpressionAfter, KeywordsWithNoLineTerminatorAfter, LineTerminatorSequence, MultiLineComment, Newline, NumericLiteral, Punctuator, RegularExpressionLiteral, SingleLineComment, StringLiteral, Template, TokensNotPrecedingObjectLiteral, TokensPrecedingExpression, WhiteSpace;
  RegularExpressionLiteral = /\/(?![*\/])(?:\[(?:(?![\]\\]).|\\.)*\]|(?![\/\\]).|\\.)*(\/[$_\u200C\u200D\p{ID_Continue}]*|\\)?/yu;
  Punctuator = /--|\+\+|=>|\.{3}|\??\.(?!\d)|(?:&&|\|\||\?\?|[+\-%&|^]|\*{1,2}|<{1,2}|>{1,3}|!=?|={1,2}|\/(?![\/*]))=?|[?~,:;[\](){}]/y;
  Identifier = /(\x23?)(?=[$_\p{ID_Start}\\])(?:[$_\u200C\u200D\p{ID_Continue}]|\\u[\da-fA-F]{4}|\\u\{[\da-fA-F]+\})+/yu;
  StringLiteral = /(['"])(?:(?!\1)[^\\\n\r]|\\(?:\r\n|[^]))*(\1)?/y;
  NumericLiteral = /(?:0[xX][\da-fA-F](?:_?[\da-fA-F])*|0[oO][0-7](?:_?[0-7])*|0[bB][01](?:_?[01])*)n?|0n|[1-9](?:_?\d)*n|(?:(?:0(?!\d)|0\d*[89]\d*|[1-9](?:_?\d)*)(?:\.(?:\d(?:_?\d)*)?)?|\.\d(?:_?\d)*)(?:[eE][+-]?\d(?:_?\d)*)?|0[0-7]+/y;
  Template = /[`}](?:[^`\\$]|\\[^]|\$(?!\{))*(`|\$\{)?/y;
  WhiteSpace = /[\t\v\f\ufeff\p{Zs}]+/yu;
  LineTerminatorSequence = /\r?\n|[\r\u2028\u2029]/y;
  MultiLineComment = /\/\*(?:[^*]|\*(?!\/))*(\*\/)?/y;
  SingleLineComment = /\/\/.*/y;
  JSXPunctuator = /[<>.:={}]|\/(?![\/*])/y;
  JSXIdentifier = /[$_\p{ID_Start}][$_\u200C\u200D\p{ID_Continue}-]*/yu;
  JSXString = /(['"])(?:(?!\1)[^])*(\1)?/y;
  JSXText = /[^<>{}]+/y;
  TokensPrecedingExpression = /^(?:[\/+-]|\.{3}|\?(?:InterpolationIn(?:JSX|Template)|NoLineTerminatorHere|NonExpressionParenEnd|UnaryIncDec))?$|[{}([,;<>=*%&|^!~?:]$/;
  TokensNotPrecedingObjectLiteral = /^(?:=>|[;\]){}]|else|\?(?:NoLineTerminatorHere|NonExpressionParenEnd))?$/;
  KeywordsWithExpressionAfter = /^(?:await|case|default|delete|do|else|instanceof|new|return|throw|typeof|void|yield)$/;
  KeywordsWithNoLineTerminatorAfter = /^(?:return|throw|yield)$/;
  Newline = RegExp(LineTerminatorSequence.source);
  jsTokens_1$1 = function* (input, { jsx = false } = {}) {
    var braces, firstCodePoint, isExpression, lastIndex, lastSignificantToken, length, match, mode, nextLastIndex, nextLastSignificantToken, parenNesting, postfixIncDec, punctuator, stack;
    ({ length } = input);
    lastIndex = 0;
    lastSignificantToken = "";
    stack = [
      { tag: "JS" }
    ];
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
  return jsTokens_1$1;
}
requireJsTokens$1();
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
const SAFE_TIMERS_SYMBOL = Symbol("vitest:SAFE_TIMERS");
function getSafeTimers() {
  const { setTimeout: safeSetTimeout, setInterval: safeSetInterval, clearInterval: safeClearInterval, clearTimeout: safeClearTimeout, setImmediate: safeSetImmediate, clearImmediate: safeClearImmediate, queueMicrotask: safeQueueMicrotask } = globalThis[SAFE_TIMERS_SYMBOL] || globalThis;
  const { nextTick: safeNextTick } = globalThis[SAFE_TIMERS_SYMBOL] || globalThis.process || { nextTick: (cb) => cb() };
  return {
    nextTick: safeNextTick,
    setTimeout: safeSetTimeout,
    setInterval: safeSetInterval,
    clearInterval: safeClearInterval,
    clearTimeout: safeClearTimeout,
    setImmediate: safeSetImmediate,
    clearImmediate: safeClearImmediate,
    queueMicrotask: safeQueueMicrotask
  };
}
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
const _DRIVE_LETTER_START_RE = /^[A-Za-z]:\//;
function normalizeWindowsPath(input = "") {
  if (!input) {
    return input;
  }
  return input.replace(/\\/g, "/").replace(_DRIVE_LETTER_START_RE, (r) => r.toUpperCase());
}
const _IS_ABSOLUTE_RE = /^[/\\](?![/\\])|^[/\\]{2}(?!\.)|^[A-Za-z]:[/\\]/;
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
      if (lastSlash === index - 1 || dots === 1) ;
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
const isAbsolute = function(p2) {
  return _IS_ABSOLUTE_RE.test(p2);
};
const CHROME_IE_STACK_REGEXP = /^\s*at .*(?:\S:\d+|\(native\))/m;
const SAFARI_NATIVE_CODE_REGEXP = /^(?:eval@)?(?:\[native code\])?$/;
function extractLocation(urlLike) {
  if (!urlLike.includes(":")) {
    return [urlLike];
  }
  const regExp = /(.+?)(?::(\d+))?(?::(\d+))?$/;
  const parts = regExp.exec(urlLike.replace(/^\(|\)$/g, ""));
  if (!parts) {
    return [urlLike];
  }
  let url = parts[1];
  if (url.startsWith("async ")) {
    url = url.slice(6);
  }
  if (url.startsWith("http:") || url.startsWith("https:")) {
    const urlObj = new URL(url);
    urlObj.searchParams.delete("import");
    urlObj.searchParams.delete("browserv");
    url = urlObj.pathname + urlObj.hash + urlObj.search;
  }
  if (url.startsWith("/@fs/")) {
    const isWindows = /^\/@fs\/[a-zA-Z]:\//.test(url);
    url = url.slice(isWindows ? 5 : 4);
  }
  return [
    url,
    parts[2] || void 0,
    parts[3] || void 0
  ];
}
function parseSingleFFOrSafariStack(raw) {
  let line = raw.trim();
  if (SAFARI_NATIVE_CODE_REGEXP.test(line)) {
    return null;
  }
  if (line.includes(" > eval")) {
    line = line.replace(/ line (\d+)(?: > eval line \d+)* > eval:\d+:\d+/g, ":$1");
  }
  if (!line.includes("@") && !line.includes(":")) {
    return null;
  }
  const functionNameRegex = /((.*".+"[^@]*)?[^@]*)(@)/;
  const matches = line.match(functionNameRegex);
  const functionName = matches && matches[1] ? matches[1] : void 0;
  const [url, lineNumber, columnNumber] = extractLocation(line.replace(functionNameRegex, ""));
  if (!url || !lineNumber || !columnNumber) {
    return null;
  }
  return {
    file: url,
    method: functionName || "",
    line: Number.parseInt(lineNumber),
    column: Number.parseInt(columnNumber)
  };
}
function parseSingleStack(raw) {
  const line = raw.trim();
  if (!CHROME_IE_STACK_REGEXP.test(line)) {
    return parseSingleFFOrSafariStack(line);
  }
  return parseSingleV8Stack(line);
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
  const location = sanitizedLine.match(/ (\(.+\)$)/);
  sanitizedLine = location ? sanitizedLine.replace(location[0], "") : sanitizedLine;
  const [url, lineNumber, columnNumber] = extractLocation(location ? location[1] : sanitizedLine);
  let method = location && sanitizedLine || "";
  let file = url && ["eval", "<anonymous>"].includes(url) ? void 0 : url;
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
function getDefaultExportFromCjs(x) {
  return x && x.__esModule && Object.prototype.hasOwnProperty.call(x, "default") ? x["default"] : x;
}
var jsTokens_1;
var hasRequiredJsTokens;
function requireJsTokens() {
  if (hasRequiredJsTokens) return jsTokens_1;
  hasRequiredJsTokens = 1;
  var HashbangComment, Identifier, JSXIdentifier, JSXPunctuator, JSXString, JSXText, KeywordsWithExpressionAfter, KeywordsWithNoLineTerminatorAfter, LineTerminatorSequence, MultiLineComment, Newline, NumericLiteral, Punctuator, RegularExpressionLiteral, SingleLineComment, StringLiteral, Template, TokensNotPrecedingObjectLiteral, TokensPrecedingExpression, WhiteSpace;
  RegularExpressionLiteral = /\/(?![*\/])(?:\[(?:[^\]\\\n\r\u2028\u2029]+|\\.)*\]?|[^\/[\\\n\r\u2028\u2029]+|\\.)*(\/[$_\u200C\u200D\p{ID_Continue}]*|\\)?/yu;
  Punctuator = /--|\+\+|=>|\.{3}|\??\.(?!\d)|(?:&&|\|\||\?\?|[+\-%&|^]|\*{1,2}|<{1,2}|>{1,3}|!=?|={1,2}|\/(?![\/*]))=?|[?~,:;[\](){}]/y;
  Identifier = /(\x23?)(?=[$_\p{ID_Start}\\])(?:[$_\u200C\u200D\p{ID_Continue}]+|\\u[\da-fA-F]{4}|\\u\{[\da-fA-F]+\})+/yu;
  StringLiteral = /(['"])(?:[^'"\\\n\r]+|(?!\1)['"]|\\(?:\r\n|[^]))*(\1)?/y;
  NumericLiteral = /(?:0[xX][\da-fA-F](?:_?[\da-fA-F])*|0[oO][0-7](?:_?[0-7])*|0[bB][01](?:_?[01])*)n?|0n|[1-9](?:_?\d)*n|(?:(?:0(?!\d)|0\d*[89]\d*|[1-9](?:_?\d)*)(?:\.(?:\d(?:_?\d)*)?)?|\.\d(?:_?\d)*)(?:[eE][+-]?\d(?:_?\d)*)?|0[0-7]+/y;
  Template = /[`}](?:[^`\\$]+|\\[^]|\$(?!\{))*(`|\$\{)?/y;
  WhiteSpace = /[\t\v\f\ufeff\p{Zs}]+/yu;
  LineTerminatorSequence = /\r?\n|[\r\u2028\u2029]/y;
  MultiLineComment = /\/\*(?:[^*]+|\*(?!\/))*(\*\/)?/y;
  SingleLineComment = /\/\/.*/y;
  HashbangComment = /^#!.*/;
  JSXPunctuator = /[<>.:={}]|\/(?![\/*])/y;
  JSXIdentifier = /[$_\p{ID_Start}][$_\u200C\u200D\p{ID_Continue}-]*/yu;
  JSXString = /(['"])(?:[^'"]+|(?!\1)['"])*(\1)?/y;
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
    stack = [
      { tag: "JS" }
    ];
    braces = [];
    parenNesting = 0;
    postfixIncDec = false;
    if (match = HashbangComment.exec(input)) {
      yield {
        type: "HashbangComment",
        value: match[0]
      };
      lastIndex = match[0].length;
    }
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
var jsTokensExports = requireJsTokens();
const jsTokens = /* @__PURE__ */ getDefaultExportFromCjs(jsTokensExports);
function stripLiteralJsTokens(code, options) {
  const FILL = " ";
  const FILL_COMMENT = " ";
  let result = "";
  const tokens = [];
  for (const token of jsTokens(code, { jsx: false })) {
    tokens.push(token);
    if (token.type === "SingleLineComment") {
      result += FILL_COMMENT.repeat(token.value.length);
      continue;
    }
    if (token.type === "MultiLineComment") {
      result += token.value.replace(/[^\n]/g, FILL_COMMENT);
      continue;
    }
    if (token.type === "StringLiteral") {
      if (!token.closed) {
        result += token.value;
        continue;
      }
      const body = token.value.slice(1, -1);
      {
        result += token.value[0] + FILL.repeat(body.length) + token.value[token.value.length - 1];
        continue;
      }
    }
    if (token.type === "NoSubstitutionTemplate") {
      const body = token.value.slice(1, -1);
      {
        result += `\`${body.replace(/[^\n]/g, FILL)}\``;
        continue;
      }
    }
    if (token.type === "RegularExpressionLiteral") {
      const body = token.value;
      {
        result += body.replace(/\/(.*)\/(\w?)$/g, (_, $1, $2) => `/${FILL.repeat($1.length)}/${$2}`);
        continue;
      }
    }
    if (token.type === "TemplateHead") {
      const body = token.value.slice(1, -2);
      {
        result += `\`${body.replace(/[^\n]/g, FILL)}\${`;
        continue;
      }
    }
    if (token.type === "TemplateTail") {
      const body = token.value.slice(0, -2);
      {
        result += `}${body.replace(/[^\n]/g, FILL)}\``;
        continue;
      }
    }
    if (token.type === "TemplateMiddle") {
      const body = token.value.slice(1, -2);
      {
        result += `}${body.replace(/[^\n]/g, FILL)}\${`;
        continue;
      }
    }
    result += token.value;
  }
  return {
    result,
    tokens
  };
}
function stripLiteral(code, options) {
  return stripLiteralDetailed(code).result;
}
function stripLiteralDetailed(code, options) {
  return stripLiteralJsTokens(code);
}
class PendingError extends Error {
  constructor(message, task, note) {
    super(message);
    __publicField(this, "code", "VITEST_PENDING");
    __publicField(this, "taskId");
    this.message = message;
    this.note = note;
    this.taskId = task.id;
  }
}
const fnMap = /* @__PURE__ */ new WeakMap();
const testFixtureMap = /* @__PURE__ */ new WeakMap();
const hooksMap = /* @__PURE__ */ new WeakMap();
function setFn(key, fn) {
  fnMap.set(key, fn);
}
function setTestFixture(key, fixture) {
  testFixtureMap.set(key, fixture);
}
function getTestFixture(key) {
  return testFixtureMap.get(key);
}
function setHooks(key, hooks) {
  hooksMap.set(key, hooks);
}
function getHooks(key) {
  return hooksMap.get(key);
}
function mergeScopedFixtures(testFixtures, scopedFixtures) {
  const scopedFixturesMap = scopedFixtures.reduce((map, fixture) => {
    map[fixture.prop] = fixture;
    return map;
  }, {});
  const newFixtures = {};
  testFixtures.forEach((fixture) => {
    const useFixture = scopedFixturesMap[fixture.prop] || { ...fixture };
    newFixtures[useFixture.prop] = useFixture;
  });
  for (const fixtureKep in newFixtures) {
    var _fixture$deps;
    const fixture = newFixtures[fixtureKep];
    fixture.deps = (_fixture$deps = fixture.deps) === null || _fixture$deps === void 0 ? void 0 : _fixture$deps.map((dep) => newFixtures[dep.prop]);
  }
  return Object.values(newFixtures);
}
function mergeContextFixtures(fixtures, context, runner2) {
  const fixtureOptionKeys = [
    "auto",
    "injected",
    "scope"
  ];
  const fixtureArray = Object.entries(fixtures).map(([prop, value]) => {
    const fixtureItem = { value };
    if (Array.isArray(value) && value.length >= 2 && isObject(value[1]) && Object.keys(value[1]).some((key) => fixtureOptionKeys.includes(key))) {
      var _runner$injectValue;
      Object.assign(fixtureItem, value[1]);
      const userValue = value[0];
      fixtureItem.value = fixtureItem.injected ? ((_runner$injectValue = runner2.injectValue) === null || _runner$injectValue === void 0 ? void 0 : _runner$injectValue.call(runner2, prop)) ?? userValue : userValue;
    }
    fixtureItem.scope = fixtureItem.scope || "test";
    if (fixtureItem.scope === "worker" && !runner2.getWorkerContext) {
      fixtureItem.scope = "file";
    }
    fixtureItem.prop = prop;
    fixtureItem.isFn = typeof fixtureItem.value === "function";
    return fixtureItem;
  });
  if (Array.isArray(context.fixtures)) {
    context.fixtures = context.fixtures.concat(fixtureArray);
  } else {
    context.fixtures = fixtureArray;
  }
  fixtureArray.forEach((fixture) => {
    if (fixture.isFn) {
      const usedProps = getUsedProps(fixture.value);
      if (usedProps.length) {
        fixture.deps = context.fixtures.filter(({ prop }) => prop !== fixture.prop && usedProps.includes(prop));
      }
      if (fixture.scope !== "test") {
        var _fixture$deps2;
        (_fixture$deps2 = fixture.deps) === null || _fixture$deps2 === void 0 ? void 0 : _fixture$deps2.forEach((dep) => {
          if (!dep.isFn) {
            return;
          }
          if (fixture.scope === "worker" && dep.scope === "worker") {
            return;
          }
          if (fixture.scope === "file" && dep.scope !== "test") {
            return;
          }
          throw new SyntaxError(`cannot use the ${dep.scope} fixture "${dep.prop}" inside the ${fixture.scope} fixture "${fixture.prop}"`);
        });
      }
    }
  });
  return context;
}
const fixtureValueMaps = /* @__PURE__ */ new Map();
const cleanupFnArrayMap = /* @__PURE__ */ new Map();
function withFixtures(runner2, fn, testContext) {
  return (hookContext) => {
    const context = hookContext || testContext;
    if (!context) {
      return fn({});
    }
    const fixtures = getTestFixture(context);
    if (!(fixtures === null || fixtures === void 0 ? void 0 : fixtures.length)) {
      return fn(context);
    }
    const usedProps = getUsedProps(fn);
    const hasAutoFixture = fixtures.some(({ auto }) => auto);
    if (!usedProps.length && !hasAutoFixture) {
      return fn(context);
    }
    if (!fixtureValueMaps.get(context)) {
      fixtureValueMaps.set(context, /* @__PURE__ */ new Map());
    }
    const fixtureValueMap = fixtureValueMaps.get(context);
    if (!cleanupFnArrayMap.has(context)) {
      cleanupFnArrayMap.set(context, []);
    }
    const cleanupFnArray = cleanupFnArrayMap.get(context);
    const usedFixtures = fixtures.filter(({ prop, auto }) => auto || usedProps.includes(prop));
    const pendingFixtures = resolveDeps(usedFixtures);
    if (!pendingFixtures.length) {
      return fn(context);
    }
    async function resolveFixtures() {
      for (const fixture of pendingFixtures) {
        if (fixtureValueMap.has(fixture)) {
          continue;
        }
        const resolvedValue = await resolveFixtureValue(runner2, fixture, context, cleanupFnArray);
        context[fixture.prop] = resolvedValue;
        fixtureValueMap.set(fixture, resolvedValue);
        if (fixture.scope === "test") {
          cleanupFnArray.unshift(() => {
            fixtureValueMap.delete(fixture);
          });
        }
      }
    }
    return resolveFixtures().then(() => fn(context));
  };
}
const globalFixturePromise = /* @__PURE__ */ new WeakMap();
function resolveFixtureValue(runner2, fixture, context, cleanupFnArray) {
  var _runner$getWorkerCont;
  const fileContext = getFileContext(context.task.file);
  const workerContext = (_runner$getWorkerCont = runner2.getWorkerContext) === null || _runner$getWorkerCont === void 0 ? void 0 : _runner$getWorkerCont.call(runner2);
  if (!fixture.isFn) {
    var _fixture$prop;
    fileContext[_fixture$prop = fixture.prop] ?? (fileContext[_fixture$prop] = fixture.value);
    if (workerContext) {
      var _fixture$prop2;
      workerContext[_fixture$prop2 = fixture.prop] ?? (workerContext[_fixture$prop2] = fixture.value);
    }
    return fixture.value;
  }
  if (fixture.scope === "test") {
    return resolveFixtureFunction(fixture.value, context, cleanupFnArray);
  }
  if (globalFixturePromise.has(fixture)) {
    return globalFixturePromise.get(fixture);
  }
  let fixtureContext;
  if (fixture.scope === "worker") {
    if (!workerContext) {
      throw new TypeError("[@vitest/runner] The worker context is not available in the current test runner. Please, provide the `getWorkerContext` method when initiating the runner.");
    }
    fixtureContext = workerContext;
  } else {
    fixtureContext = fileContext;
  }
  if (fixture.prop in fixtureContext) {
    return fixtureContext[fixture.prop];
  }
  if (!cleanupFnArrayMap.has(fixtureContext)) {
    cleanupFnArrayMap.set(fixtureContext, []);
  }
  const cleanupFnFileArray = cleanupFnArrayMap.get(fixtureContext);
  const promise = resolveFixtureFunction(fixture.value, fixtureContext, cleanupFnFileArray).then((value) => {
    fixtureContext[fixture.prop] = value;
    globalFixturePromise.delete(fixture);
    return value;
  });
  globalFixturePromise.set(fixture, promise);
  return promise;
}
async function resolveFixtureFunction(fixtureFn, context, cleanupFnArray) {
  const useFnArgPromise = createDefer();
  let isUseFnArgResolved = false;
  const fixtureReturn = fixtureFn(context, async (useFnArg) => {
    isUseFnArgResolved = true;
    useFnArgPromise.resolve(useFnArg);
    const useReturnPromise = createDefer();
    cleanupFnArray.push(async () => {
      useReturnPromise.resolve();
      await fixtureReturn;
    });
    await useReturnPromise;
  }).catch((e) => {
    if (!isUseFnArgResolved) {
      useFnArgPromise.reject(e);
      return;
    }
    throw e;
  });
  return useFnArgPromise;
}
function resolveDeps(fixtures, depSet = /* @__PURE__ */ new Set(), pendingFixtures = []) {
  fixtures.forEach((fixture) => {
    if (pendingFixtures.includes(fixture)) {
      return;
    }
    if (!fixture.isFn || !fixture.deps) {
      pendingFixtures.push(fixture);
      return;
    }
    if (depSet.has(fixture)) {
      throw new Error(`Circular fixture dependency detected: ${fixture.prop} <- ${[...depSet].reverse().map((d) => d.prop).join(" <- ")}`);
    }
    depSet.add(fixture);
    resolveDeps(fixture.deps, depSet, pendingFixtures);
    pendingFixtures.push(fixture);
    depSet.clear();
  });
  return pendingFixtures;
}
function getUsedProps(fn) {
  let fnString = stripLiteral(fn.toString());
  if (/__async\((?:this|null), (?:null|arguments|\[[_0-9, ]*\]), function\*/.test(fnString)) {
    fnString = fnString.split(/__async\((?:this|null),/)[1];
  }
  const match = fnString.match(/[^(]*\(([^)]*)/);
  if (!match) {
    return [];
  }
  const args = splitByComma(match[1]);
  if (!args.length) {
    return [];
  }
  let first = args[0];
  if ("__VITEST_FIXTURE_INDEX__" in fn) {
    first = args[fn.__VITEST_FIXTURE_INDEX__];
    if (!first) {
      return [];
    }
  }
  if (!(first.startsWith("{") && first.endsWith("}"))) {
    throw new Error(`The first argument inside a fixture must use object destructuring pattern, e.g. ({ test } => {}). Instead, received "${first}".`);
  }
  const _first = first.slice(1, -1).replace(/\s/g, "");
  const props = splitByComma(_first).map((prop) => {
    return prop.replace(/:.*|=.*/g, "");
  });
  const last = props.at(-1);
  if (last && last.startsWith("...")) {
    throw new Error(`Rest parameters are not supported in fixtures, received "${last}".`);
  }
  return props;
}
function splitByComma(s) {
  const result = [];
  const stack = [];
  let start = 0;
  for (let i = 0; i < s.length; i++) {
    if (s[i] === "{" || s[i] === "[") {
      stack.push(s[i] === "{" ? "}" : "]");
    } else if (s[i] === stack[stack.length - 1]) {
      stack.pop();
    } else if (!stack.length && s[i] === ",") {
      const token = s.substring(start, i).trim();
      if (token) {
        result.push(token);
      }
      start = i + 1;
    }
  }
  const lastToken = s.substring(start).trim();
  if (lastToken) {
    result.push(lastToken);
  }
  return result;
}
function createChainable(keys, fn) {
  function create(context) {
    const chain2 = function(...args) {
      return fn.apply(context, args);
    };
    Object.assign(chain2, fn);
    chain2.withContext = () => chain2.bind(context);
    chain2.setContext = (key, value) => {
      context[key] = value;
    };
    chain2.mergeContext = (ctx) => {
      Object.assign(context, ctx);
    };
    for (const key of keys) {
      Object.defineProperty(chain2, key, { get() {
        return create({
          ...context,
          [key]: true
        });
      } });
    }
    return chain2;
  }
  const chain = create({});
  chain.fn = fn;
  return chain;
}
const suite = createSuite();
createTest(function(name, optionsOrFn, optionsOrTest) {
  getCurrentSuite().test.fn.call(this, formatName(name), optionsOrFn, optionsOrTest);
});
let runner;
let defaultSuite;
let currentTestFilepath;
function assert(condition, message) {
  if (!condition) {
    throw new Error(`Vitest failed to find ${message}. This is a bug in Vitest. Please, open an issue with reproduction.`);
  }
}
function getTestFilepath() {
  return currentTestFilepath;
}
function getRunner() {
  assert(runner, "the runner");
  return runner;
}
function getCurrentSuite() {
  const currentSuite = collectorContext.currentSuite || defaultSuite;
  assert(currentSuite, "the current suite");
  return currentSuite;
}
function createSuiteHooks() {
  return {
    beforeAll: [],
    afterAll: [],
    beforeEach: [],
    afterEach: []
  };
}
function parseArguments(optionsOrFn, optionsOrTest) {
  let options = {};
  let fn = () => {
  };
  if (typeof optionsOrTest === "object") {
    if (typeof optionsOrFn === "object") {
      throw new TypeError("Cannot use two objects as arguments. Please provide options and a function callback in that order.");
    }
    console.warn("Using an object as a third argument is deprecated. Vitest 4 will throw an error if the third argument is not a timeout number. Please use the second argument for options. See more at https://vitest.dev/guide/migration");
    options = optionsOrTest;
  } else if (typeof optionsOrTest === "number") {
    options = { timeout: optionsOrTest };
  } else if (typeof optionsOrFn === "object") {
    options = optionsOrFn;
  }
  if (typeof optionsOrFn === "function") {
    if (typeof optionsOrTest === "function") {
      throw new TypeError("Cannot use two functions as arguments. Please use the second argument for options.");
    }
    fn = optionsOrFn;
  } else if (typeof optionsOrTest === "function") {
    fn = optionsOrTest;
  }
  return {
    options,
    handler: fn
  };
}
function createSuiteCollector(name, factory = () => {
}, mode, each, suiteOptions, parentCollectorFixtures) {
  const tasks = [];
  let suite2;
  initSuite();
  const task = function(name2 = "", options = {}) {
    var _collectorContext$cur;
    const timeout = (options === null || options === void 0 ? void 0 : options.timeout) ?? runner.config.testTimeout;
    const task2 = {
      id: "",
      name: name2,
      suite: (_collectorContext$cur = collectorContext.currentSuite) === null || _collectorContext$cur === void 0 ? void 0 : _collectorContext$cur.suite,
      each: options.each,
      fails: options.fails,
      context: void 0,
      type: "test",
      file: void 0,
      timeout,
      retry: options.retry ?? runner.config.retry,
      repeats: options.repeats,
      mode: options.only ? "only" : options.skip ? "skip" : options.todo ? "todo" : "run",
      meta: options.meta ?? /* @__PURE__ */ Object.create(null),
      annotations: []
    };
    const handler = options.handler;
    if (options.concurrent || !options.sequential && runner.config.sequence.concurrent) {
      task2.concurrent = true;
    }
    task2.shuffle = suiteOptions === null || suiteOptions === void 0 ? void 0 : suiteOptions.shuffle;
    const context = createTestContext(task2, runner);
    Object.defineProperty(task2, "context", {
      value: context,
      enumerable: false
    });
    setTestFixture(context, options.fixtures);
    const limit = Error.stackTraceLimit;
    Error.stackTraceLimit = 15;
    const stackTraceError = new Error("STACK_TRACE_ERROR");
    Error.stackTraceLimit = limit;
    if (handler) {
      setFn(task2, withTimeout(withAwaitAsyncAssertions(withFixtures(runner, handler, context), task2), timeout, false, stackTraceError, (_, error) => abortIfTimeout([context], error)));
    }
    if (runner.config.includeTaskLocation) {
      const error = stackTraceError.stack;
      const stack = findTestFileStackTrace(error);
      if (stack) {
        task2.location = stack;
      }
    }
    tasks.push(task2);
    return task2;
  };
  const test = createTest(function(name2, optionsOrFn, optionsOrTest) {
    let { options, handler } = parseArguments(optionsOrFn, optionsOrTest);
    if (typeof suiteOptions === "object") {
      options = Object.assign({}, suiteOptions, options);
    }
    options.concurrent = this.concurrent || !this.sequential && (options === null || options === void 0 ? void 0 : options.concurrent);
    options.sequential = this.sequential || !this.concurrent && (options === null || options === void 0 ? void 0 : options.sequential);
    const test2 = task(formatName(name2), {
      ...this,
      ...options,
      handler
    });
    test2.type = "test";
  });
  let collectorFixtures = parentCollectorFixtures;
  const collector = {
    type: "collector",
    name,
    mode,
    suite: suite2,
    options: suiteOptions,
    test,
    tasks,
    collect,
    task,
    clear,
    on: addHook,
    fixtures() {
      return collectorFixtures;
    },
    scoped(fixtures) {
      const parsed = mergeContextFixtures(fixtures, { fixtures: collectorFixtures }, runner);
      if (parsed.fixtures) {
        collectorFixtures = parsed.fixtures;
      }
    }
  };
  function addHook(name2, ...fn) {
    getHooks(suite2)[name2].push(...fn);
  }
  function initSuite(includeLocation) {
    var _collectorContext$cur2;
    if (typeof suiteOptions === "number") {
      suiteOptions = { timeout: suiteOptions };
    }
    suite2 = {
      id: "",
      type: "suite",
      name,
      suite: (_collectorContext$cur2 = collectorContext.currentSuite) === null || _collectorContext$cur2 === void 0 ? void 0 : _collectorContext$cur2.suite,
      mode,
      each,
      file: void 0,
      shuffle: suiteOptions === null || suiteOptions === void 0 ? void 0 : suiteOptions.shuffle,
      tasks: [],
      meta: /* @__PURE__ */ Object.create(null),
      concurrent: suiteOptions === null || suiteOptions === void 0 ? void 0 : suiteOptions.concurrent
    };
    setHooks(suite2, createSuiteHooks());
  }
  function clear() {
    tasks.length = 0;
    initSuite();
  }
  async function collect(file) {
    if (!file) {
      throw new TypeError("File is required to collect tasks.");
    }
    if (factory) {
      await runWithSuite(collector, () => factory(test));
    }
    const allChildren = [];
    for (const i of tasks) {
      allChildren.push(i.type === "collector" ? await i.collect(file) : i);
    }
    suite2.file = file;
    suite2.tasks = allChildren;
    allChildren.forEach((task2) => {
      task2.file = file;
    });
    return suite2;
  }
  collectTask(collector);
  return collector;
}
function withAwaitAsyncAssertions(fn, task) {
  return async (...args) => {
    const fnResult = await fn(...args);
    if (task.promises) {
      const result = await Promise.allSettled(task.promises);
      const errors = result.map((r) => r.status === "rejected" ? r.reason : void 0).filter(Boolean);
      if (errors.length) {
        throw errors;
      }
    }
    return fnResult;
  };
}
function createSuite() {
  function suiteFn(name, factoryOrOptions, optionsOrFactory) {
    var _currentSuite$options;
    const mode = this.only ? "only" : this.skip ? "skip" : this.todo ? "todo" : "run";
    const currentSuite = collectorContext.currentSuite || defaultSuite;
    let { options, handler: factory } = parseArguments(factoryOrOptions, optionsOrFactory);
    const isConcurrentSpecified = options.concurrent || this.concurrent || options.sequential === false;
    const isSequentialSpecified = options.sequential || this.sequential || options.concurrent === false;
    options = {
      ...currentSuite === null || currentSuite === void 0 ? void 0 : currentSuite.options,
      ...options,
      shuffle: this.shuffle ?? options.shuffle ?? (currentSuite === null || currentSuite === void 0 || (_currentSuite$options = currentSuite.options) === null || _currentSuite$options === void 0 ? void 0 : _currentSuite$options.shuffle) ?? void 0
    };
    const isConcurrent = isConcurrentSpecified || options.concurrent && !isSequentialSpecified;
    const isSequential = isSequentialSpecified || options.sequential && !isConcurrentSpecified;
    options.concurrent = isConcurrent && !isSequential;
    options.sequential = isSequential && !isConcurrent;
    return createSuiteCollector(formatName(name), factory, mode, this.each, options, currentSuite === null || currentSuite === void 0 ? void 0 : currentSuite.fixtures());
  }
  suiteFn.each = function(cases, ...args) {
    const suite2 = this.withContext();
    this.setContext("each", true);
    if (Array.isArray(cases) && args.length) {
      cases = formatTemplateString(cases, args);
    }
    return (name, optionsOrFn, fnOrOptions) => {
      const _name = formatName(name);
      const arrayOnlyCases = cases.every(Array.isArray);
      const { options, handler } = parseArguments(optionsOrFn, fnOrOptions);
      const fnFirst = typeof optionsOrFn === "function" && typeof fnOrOptions === "object";
      cases.forEach((i, idx) => {
        const items = Array.isArray(i) ? i : [i];
        if (fnFirst) {
          if (arrayOnlyCases) {
            suite2(formatTitle(_name, items, idx), () => handler(...items), options);
          } else {
            suite2(formatTitle(_name, items, idx), () => handler(i), options);
          }
        } else {
          if (arrayOnlyCases) {
            suite2(formatTitle(_name, items, idx), options, () => handler(...items));
          } else {
            suite2(formatTitle(_name, items, idx), options, () => handler(i));
          }
        }
      });
      this.setContext("each", void 0);
    };
  };
  suiteFn.for = function(cases, ...args) {
    if (Array.isArray(cases) && args.length) {
      cases = formatTemplateString(cases, args);
    }
    return (name, optionsOrFn, fnOrOptions) => {
      const name_ = formatName(name);
      const { options, handler } = parseArguments(optionsOrFn, fnOrOptions);
      cases.forEach((item, idx) => {
        suite(formatTitle(name_, toArray(item), idx), options, () => handler(item));
      });
    };
  };
  suiteFn.skipIf = (condition) => condition ? suite.skip : suite;
  suiteFn.runIf = (condition) => condition ? suite : suite.skip;
  return createChainable([
    "concurrent",
    "sequential",
    "shuffle",
    "skip",
    "only",
    "todo"
  ], suiteFn);
}
function createTaskCollector(fn, context) {
  const taskFn = fn;
  taskFn.each = function(cases, ...args) {
    const test = this.withContext();
    this.setContext("each", true);
    if (Array.isArray(cases) && args.length) {
      cases = formatTemplateString(cases, args);
    }
    return (name, optionsOrFn, fnOrOptions) => {
      const _name = formatName(name);
      const arrayOnlyCases = cases.every(Array.isArray);
      const { options, handler } = parseArguments(optionsOrFn, fnOrOptions);
      const fnFirst = typeof optionsOrFn === "function" && typeof fnOrOptions === "object";
      cases.forEach((i, idx) => {
        const items = Array.isArray(i) ? i : [i];
        if (fnFirst) {
          if (arrayOnlyCases) {
            test(formatTitle(_name, items, idx), () => handler(...items), options);
          } else {
            test(formatTitle(_name, items, idx), () => handler(i), options);
          }
        } else {
          if (arrayOnlyCases) {
            test(formatTitle(_name, items, idx), options, () => handler(...items));
          } else {
            test(formatTitle(_name, items, idx), options, () => handler(i));
          }
        }
      });
      this.setContext("each", void 0);
    };
  };
  taskFn.for = function(cases, ...args) {
    const test = this.withContext();
    if (Array.isArray(cases) && args.length) {
      cases = formatTemplateString(cases, args);
    }
    return (name, optionsOrFn, fnOrOptions) => {
      const _name = formatName(name);
      const { options, handler } = parseArguments(optionsOrFn, fnOrOptions);
      cases.forEach((item, idx) => {
        const handlerWrapper = (ctx) => handler(item, ctx);
        handlerWrapper.__VITEST_FIXTURE_INDEX__ = 1;
        handlerWrapper.toString = () => handler.toString();
        test(formatTitle(_name, toArray(item), idx), options, handlerWrapper);
      });
    };
  };
  taskFn.skipIf = function(condition) {
    return condition ? this.skip : this;
  };
  taskFn.runIf = function(condition) {
    return condition ? this : this.skip;
  };
  taskFn.scoped = function(fixtures) {
    const collector = getCurrentSuite();
    collector.scoped(fixtures);
  };
  taskFn.extend = function(fixtures) {
    const _context = mergeContextFixtures(fixtures, context || {}, runner);
    const originalWrapper = fn;
    return createTest(function(name, optionsOrFn, optionsOrTest) {
      const collector = getCurrentSuite();
      const scopedFixtures = collector.fixtures();
      const context2 = { ...this };
      if (scopedFixtures) {
        context2.fixtures = mergeScopedFixtures(context2.fixtures || [], scopedFixtures);
      }
      const { handler, options } = parseArguments(optionsOrFn, optionsOrTest);
      const timeout = options.timeout ?? void 0;
      originalWrapper.call(context2, formatName(name), handler, timeout);
    }, _context);
  };
  const _test = createChainable([
    "concurrent",
    "sequential",
    "skip",
    "only",
    "todo",
    "fails"
  ], taskFn);
  if (context) {
    _test.mergeContext(context);
  }
  return _test;
}
function createTest(fn, context) {
  return createTaskCollector(fn, context);
}
function formatName(name) {
  return typeof name === "string" ? name : typeof name === "function" ? name.name || "<anonymous>" : String(name);
}
function formatTitle(template, items, idx) {
  if (template.includes("%#") || template.includes("%$")) {
    template = template.replace(/%%/g, "__vitest_escaped_%__").replace(/%#/g, `${idx}`).replace(/%\$/g, `${idx + 1}`).replace(/__vitest_escaped_%__/g, "%%");
  }
  const count = template.split("%").length - 1;
  if (template.includes("%f")) {
    const placeholders = template.match(/%f/g) || [];
    placeholders.forEach((_, i) => {
      if (isNegativeNaN(items[i]) || Object.is(items[i], -0)) {
        let occurrence = 0;
        template = template.replace(/%f/g, (match) => {
          occurrence++;
          return occurrence === i + 1 ? "-%f" : match;
        });
      }
    });
  }
  let formatted = format(template, ...items.slice(0, count));
  const isObjectItem = isObject(items[0]);
  formatted = formatted.replace(/\$([$\w.]+)/g, (_, key) => {
    const isArrayKey = /^\d+$/.test(key);
    if (!isObjectItem && !isArrayKey) {
      return `$${key}`;
    }
    const arrayElement = isArrayKey ? objectAttr(items, key) : void 0;
    const value = isObjectItem ? objectAttr(items[0], key, arrayElement) : arrayElement;
    return objDisplay(value, { truncate: void 0 });
  });
  return formatted;
}
function formatTemplateString(cases, args) {
  const header = cases.join("").trim().replace(/ /g, "").split("\n").map((i) => i.split("|"))[0];
  const res = [];
  for (let i = 0; i < Math.floor(args.length / header.length); i++) {
    const oneCase = {};
    for (let j = 0; j < header.length; j++) {
      oneCase[header[j]] = args[i * header.length + j];
    }
    res.push(oneCase);
  }
  return res;
}
function findTestFileStackTrace(error) {
  const testFilePath = getTestFilepath();
  const lines = error.split("\n").slice(1);
  for (const line of lines) {
    const stack = parseSingleStack(line);
    if (stack && stack.file === testFilePath) {
      return {
        line: stack.line,
        column: stack.column
      };
    }
  }
}
function generateHash(str) {
  let hash = 0;
  if (str.length === 0) {
    return `${hash}`;
  }
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash;
  }
  return `${hash}`;
}
globalThis.performance ? globalThis.performance.now.bind(globalThis.performance) : Date.now;
globalThis.performance ? globalThis.performance.now.bind(globalThis.performance) : Date.now;
getSafeTimers();
const packs = /* @__PURE__ */ new Map();
const eventsPacks = [];
const pendingTasksUpdates = [];
function sendTasksUpdate(runner2) {
  if (packs.size) {
    var _runner$onTaskUpdate;
    const taskPacks = Array.from(packs).map(([id, task]) => {
      return [
        id,
        task[0],
        task[1]
      ];
    });
    const p2 = (_runner$onTaskUpdate = runner2.onTaskUpdate) === null || _runner$onTaskUpdate === void 0 ? void 0 : _runner$onTaskUpdate.call(runner2, taskPacks, eventsPacks);
    if (p2) {
      pendingTasksUpdates.push(p2);
      p2.then(() => pendingTasksUpdates.splice(pendingTasksUpdates.indexOf(p2), 1), () => {
      });
    }
    eventsPacks.length = 0;
    packs.clear();
  }
}
async function finishSendTasksUpdate(runner2) {
  sendTasksUpdate(runner2);
  await Promise.all(pendingTasksUpdates);
}
const now = Date.now;
const collectorContext = {
  currentSuite: null
};
function collectTask(task) {
  var _collectorContext$cur;
  (_collectorContext$cur = collectorContext.currentSuite) === null || _collectorContext$cur === void 0 ? void 0 : _collectorContext$cur.tasks.push(task);
}
async function runWithSuite(suite2, fn) {
  const prev = collectorContext.currentSuite;
  collectorContext.currentSuite = suite2;
  await fn();
  collectorContext.currentSuite = prev;
}
function withTimeout(fn, timeout, isHook = false, stackTraceError, onTimeout) {
  if (timeout <= 0 || timeout === Number.POSITIVE_INFINITY) {
    return fn;
  }
  const { setTimeout, clearTimeout } = getSafeTimers();
  return function runWithTimeout(...args) {
    const startTime = now();
    const runner2 = getRunner();
    runner2._currentTaskStartTime = startTime;
    runner2._currentTaskTimeout = timeout;
    return new Promise((resolve_, reject_) => {
      var _timer$unref;
      const timer = setTimeout(() => {
        clearTimeout(timer);
        rejectTimeoutError();
      }, timeout);
      (_timer$unref = timer.unref) === null || _timer$unref === void 0 ? void 0 : _timer$unref.call(timer);
      function rejectTimeoutError() {
        const error = makeTimeoutError(isHook, timeout, stackTraceError);
        onTimeout === null || onTimeout === void 0 ? void 0 : onTimeout(args, error);
        reject_(error);
      }
      function resolve2(result) {
        runner2._currentTaskStartTime = void 0;
        runner2._currentTaskTimeout = void 0;
        clearTimeout(timer);
        if (now() - startTime >= timeout) {
          rejectTimeoutError();
          return;
        }
        resolve_(result);
      }
      function reject(error) {
        runner2._currentTaskStartTime = void 0;
        runner2._currentTaskTimeout = void 0;
        clearTimeout(timer);
        reject_(error);
      }
      try {
        const result = fn(...args);
        if (typeof result === "object" && result != null && typeof result.then === "function") {
          result.then(resolve2, reject);
        } else {
          resolve2(result);
        }
      } catch (error) {
        reject(error);
      }
    });
  };
}
const abortControllers = /* @__PURE__ */ new WeakMap();
function abortIfTimeout([context], error) {
  if (context) {
    abortContextSignal(context, error);
  }
}
function abortContextSignal(context, error) {
  const abortController = abortControllers.get(context);
  abortController === null || abortController === void 0 ? void 0 : abortController.abort(error);
}
function createTestContext(test, runner2) {
  var _runner$extendTaskCon;
  const context = function() {
    throw new Error("done() callback is deprecated, use promise instead");
  };
  let abortController = abortControllers.get(context);
  if (!abortController) {
    abortController = new AbortController();
    abortControllers.set(context, abortController);
  }
  context.signal = abortController.signal;
  context.task = test;
  context.skip = (condition, note) => {
    if (condition === false) {
      return void 0;
    }
    test.result ?? (test.result = { state: "skip" });
    test.result.pending = true;
    throw new PendingError("test is skipped; abort execution", test, typeof condition === "string" ? condition : note);
  };
  async function annotate(message, location, type, attachment) {
    const annotation = {
      message,
      type: type || "notice"
    };
    if (attachment) {
      if (!attachment.body && !attachment.path) {
        throw new TypeError(`Test attachment requires body or path to be set. Both are missing.`);
      }
      if (attachment.body && attachment.path) {
        throw new TypeError(`Test attachment requires only one of "body" or "path" to be set. Both are specified.`);
      }
      annotation.attachment = attachment;
      if (attachment.body instanceof Uint8Array) {
        attachment.body = encodeUint8Array(attachment.body);
      }
    }
    if (location) {
      annotation.location = location;
    }
    if (!runner2.onTestAnnotate) {
      throw new Error(`Test runner doesn't support test annotations.`);
    }
    await finishSendTasksUpdate(runner2);
    const resolvedAnnotation = await runner2.onTestAnnotate(test, annotation);
    test.annotations.push(resolvedAnnotation);
    return resolvedAnnotation;
  }
  context.annotate = (message, type, attachment) => {
    if (test.result && test.result.state !== "run") {
      throw new Error(`Cannot annotate tests outside of the test run. The test "${test.name}" finished running with the "${test.result.state}" state already.`);
    }
    let location;
    const stack = new Error("STACK_TRACE").stack;
    const index = stack.includes("STACK_TRACE") ? 2 : 1;
    const stackLine = stack.split("\n")[index];
    const parsed = parseSingleStack(stackLine);
    if (parsed) {
      location = {
        file: parsed.file,
        line: parsed.line,
        column: parsed.column
      };
    }
    if (typeof type === "object") {
      return recordAsyncAnnotation(test, annotate(message, location, void 0, type));
    } else {
      return recordAsyncAnnotation(test, annotate(message, location, type, attachment));
    }
  };
  context.onTestFailed = (handler, timeout) => {
    test.onFailed || (test.onFailed = []);
    test.onFailed.push(withTimeout(handler, timeout ?? runner2.config.hookTimeout, true, new Error("STACK_TRACE_ERROR"), (_, error) => abortController.abort(error)));
  };
  context.onTestFinished = (handler, timeout) => {
    test.onFinished || (test.onFinished = []);
    test.onFinished.push(withTimeout(handler, timeout ?? runner2.config.hookTimeout, true, new Error("STACK_TRACE_ERROR"), (_, error) => abortController.abort(error)));
  };
  return ((_runner$extendTaskCon = runner2.extendTaskContext) === null || _runner$extendTaskCon === void 0 ? void 0 : _runner$extendTaskCon.call(runner2, context)) || context;
}
function makeTimeoutError(isHook, timeout, stackTraceError) {
  const message = `${isHook ? "Hook" : "Test"} timed out in ${timeout}ms.
If this is a long-running ${isHook ? "hook" : "test"}, pass a timeout value as the last argument or configure it globally with "${isHook ? "hookTimeout" : "testTimeout"}".`;
  const error = new Error(message);
  if (stackTraceError === null || stackTraceError === void 0 ? void 0 : stackTraceError.stack) {
    error.stack = stackTraceError.stack.replace(error.message, stackTraceError.message);
  }
  return error;
}
const fileContexts = /* @__PURE__ */ new WeakMap();
function getFileContext(file) {
  const context = fileContexts.get(file);
  if (!context) {
    throw new Error(`Cannot find file context for ${file.name}`);
  }
  return context;
}
const table = [];
for (let i = 65; i < 91; i++) {
  table.push(String.fromCharCode(i));
}
for (let i = 97; i < 123; i++) {
  table.push(String.fromCharCode(i));
}
for (let i = 0; i < 10; i++) {
  table.push(i.toString(10));
}
function encodeUint8Array(bytes) {
  let base64 = "";
  const len = bytes.byteLength;
  for (let i = 0; i < len; i += 3) {
    if (len === i + 1) {
      const a2 = (bytes[i] & 252) >> 2;
      const b = (bytes[i] & 3) << 4;
      base64 += table[a2];
      base64 += table[b];
      base64 += "==";
    } else if (len === i + 2) {
      const a2 = (bytes[i] & 252) >> 2;
      const b = (bytes[i] & 3) << 4 | (bytes[i + 1] & 240) >> 4;
      const c = (bytes[i + 1] & 15) << 2;
      base64 += table[a2];
      base64 += table[b];
      base64 += table[c];
      base64 += "=";
    } else {
      const a2 = (bytes[i] & 252) >> 2;
      const b = (bytes[i] & 3) << 4 | (bytes[i + 1] & 240) >> 4;
      const c = (bytes[i + 1] & 15) << 2 | (bytes[i + 2] & 192) >> 6;
      const d = bytes[i + 2] & 63;
      base64 += table[a2];
      base64 += table[b];
      base64 += table[c];
      base64 += table[d];
    }
  }
  return base64;
}
function recordAsyncAnnotation(test, promise) {
  promise = promise.finally(() => {
    if (!test.promises) {
      return;
    }
    const index = test.promises.indexOf(promise);
    if (index !== -1) {
      test.promises.splice(index, 1);
    }
  });
  if (!test.promises) {
    test.promises = [];
  }
  test.promises.push(promise);
  return promise;
}
function getUiAPI() {
  return window.__vitest_ui_api__;
}
const ID_ALL = "__vitest_all__";
class IframeOrchestrator {
  constructor() {
    __publicField(this, "cancelled", false);
    __publicField(this, "recreateNonIsolatedIframe", false);
    __publicField(this, "iframes", /* @__PURE__ */ new Map());
    debug("init orchestrator", getBrowserState().sessionId);
    channel.addEventListener(
      "message",
      (e) => this.onIframeEvent(e)
    );
    globalChannel.addEventListener(
      "message",
      (e) => this.onGlobalChannelEvent(e)
    );
  }
  async createTesters(options) {
    this.cancelled = false;
    const config = getConfig();
    debug("create testers", options.files.join(", "));
    const container = await getContainer(config);
    if (config.browser.ui) {
      container.className = "absolute origin-top mt-[8px]";
      container.parentElement.setAttribute("data-ready", "true");
      if (container.textContent) {
        container.textContent = "";
      }
    }
    if (config.browser.isolate === false) {
      await this.runNonIsolatedTests(container, options);
      return;
    }
    this.iframes.forEach((iframe) => iframe.remove());
    this.iframes.clear();
    for (let i = 0; i < options.files.length; i++) {
      if (this.cancelled) {
        return;
      }
      const file = options.files[i];
      debug("create iframe", file);
      await this.runIsolatedTestInIframe(
        container,
        file,
        options
      );
    }
  }
  async cleanupTesters() {
    const config = getConfig();
    if (config.browser.isolate) {
      const files = Array.from(this.iframes.keys());
      const ui = getUiAPI();
      if (ui && files[0]) {
        const id = generateFileId(files[0]);
        ui.setCurrentFileId(id);
      }
      return;
    }
    const iframe = this.iframes.get(ID_ALL);
    if (!iframe) {
      return;
    }
    await sendEventToIframe({
      event: "cleanup",
      iframeId: ID_ALL
    });
    this.recreateNonIsolatedIframe = true;
  }
  async runNonIsolatedTests(container, options) {
    if (this.recreateNonIsolatedIframe) {
      this.recreateNonIsolatedIframe = false;
      this.iframes.get(ID_ALL).remove();
      this.iframes.delete(ID_ALL);
      debug("recreate non-isolated iframe");
    }
    if (!this.iframes.has(ID_ALL)) {
      debug("preparing non-isolated iframe");
      await this.prepareIframe(container, ID_ALL, options.startTime);
    }
    const config = getConfig();
    const { width, height } = config.browser.viewport;
    const iframe = this.iframes.get(ID_ALL);
    await setIframeViewport(iframe, width, height);
    debug("run non-isolated tests", options.files.join(", "));
    await sendEventToIframe({
      event: "execute",
      iframeId: ID_ALL,
      files: options.files,
      method: options.method,
      context: options.providedContext
    });
  }
  async runIsolatedTestInIframe(container, file, options) {
    const config = getConfig();
    const { width, height } = config.browser.viewport;
    if (this.iframes.has(file)) {
      this.iframes.get(file).remove();
      this.iframes.delete(file);
    }
    const iframe = await this.prepareIframe(container, file, options.startTime);
    await setIframeViewport(iframe, width, height);
    await sendEventToIframe({
      event: "execute",
      files: [file],
      method: options.method,
      iframeId: file,
      context: options.providedContext
    });
    await sendEventToIframe({
      event: "cleanup",
      iframeId: file
    });
  }
  async prepareIframe(container, iframeId, startTime) {
    const iframe = this.createTestIframe(iframeId);
    container.appendChild(iframe);
    await new Promise((resolve2, reject) => {
      iframe.onload = () => {
        this.iframes.set(iframeId, iframe);
        sendEventToIframe({
          event: "prepare",
          iframeId,
          startTime
        }).then(resolve2, reject);
      };
      iframe.onerror = (e) => {
        if (typeof e === "string") {
          reject(new Error(e));
        } else if (e instanceof ErrorEvent) {
          reject(e.error);
        } else {
          reject(new Error(`Cannot load the iframe ${iframeId}.`));
        }
      };
    });
    return iframe;
  }
  createTestIframe(iframeId) {
    const iframe = document.createElement("iframe");
    const src = `/?sessionId=${getBrowserState().sessionId}&iframeId=${iframeId}`;
    iframe.setAttribute("loading", "eager");
    iframe.setAttribute("src", src);
    iframe.setAttribute("data-vitest", "true");
    iframe.style.border = "none";
    iframe.style.width = "100%";
    iframe.style.height = "100%";
    iframe.setAttribute("allowfullscreen", "true");
    iframe.setAttribute("allow", "clipboard-write;");
    iframe.setAttribute("name", "vitest-iframe");
    return iframe;
  }
  async onGlobalChannelEvent(e) {
    debug("global channel event", JSON.stringify(e.data));
    switch (e.data.type) {
      case "cancel": {
        this.cancelled = true;
        break;
      }
    }
  }
  async onIframeEvent(e) {
    debug("iframe event", JSON.stringify(e.data));
    switch (e.data.event) {
      case "viewport": {
        const { width, height, iframeId: id } = e.data;
        const iframe = this.iframes.get(id);
        if (!iframe) {
          const error = `Cannot find iframe with id ${id}`;
          channel.postMessage({
            event: "viewport:fail",
            iframeId: id,
            error
          });
          await client.rpc.onUnhandledError(
            {
              name: "Teardown Error",
              message: error
            },
            "Teardown Error"
          );
          break;
        }
        await setIframeViewport(iframe, width, height);
        channel.postMessage({ event: "viewport:done", iframeId: id });
        break;
      }
      default: {
        if (typeof e.data.event === "string" && e.data.event.startsWith("response:")) {
          break;
        }
        await client.rpc.onUnhandledError(
          {
            name: "Unexpected Event",
            message: `Unexpected event: ${e.data.event}`
          },
          "Unexpected Event"
        );
      }
    }
  }
}
getBrowserState().orchestrator = new IframeOrchestrator();
async function getContainer(config) {
  if (config.browser.ui) {
    const element = document.querySelector("#tester-ui");
    if (!element) {
      return new Promise((resolve2) => {
        queueMicrotask(() => {
          resolve2(getContainer(config));
        });
      });
    }
    return element;
  }
  return document.querySelector("#vitest-tester");
}
async function sendEventToIframe(event) {
  channel.postMessage(event);
  return new Promise((resolve2) => {
    channel.addEventListener(
      "message",
      function handler(e) {
        if (e.data.iframeId === event.iframeId && e.data.event === `response:${event.event}`) {
          resolve2();
          channel.removeEventListener("message", handler);
        }
      }
    );
  });
}
function generateFileId(file) {
  const config = getConfig();
  const project = config.name || "";
  const path = relative(config.root, file);
  return generateHash(`${path}${project}`);
}
async function setIframeViewport(iframe, width, height) {
  var _a, _b;
  const ui = getUiAPI();
  if (ui) {
    await ui.setIframeViewport(width, height);
  } else if (getBrowserState().provider === "webdriverio") {
    (_a = iframe.parentElement) == null ? void 0 : _a.setAttribute("data-scale", "1");
    await client.rpc.triggerCommand(
      getBrowserState().sessionId,
      "__vitest_viewport",
      void 0,
      [{ width, height }]
    );
  } else {
    const scale = Math.min(
      1,
      iframe.parentElement.parentElement.clientWidth / width,
      iframe.parentElement.parentElement.clientHeight / height
    );
    iframe.parentElement.style.cssText = `
      width: ${width}px;
      height: ${height}px;
      transform: scale(${scale});
      transform-origin: left top;
    `;
    (_b = iframe.parentElement) == null ? void 0 : _b.setAttribute("data-scale", String(scale));
    await new Promise((r) => requestAnimationFrame(r));
  }
}
function debug(...args) {
  const debug2 = getConfig().env.VITEST_BROWSER_DEBUG;
  if (debug2 && debug2 !== "false") {
    client.rpc.debug(...args.map(String));
  }
}
