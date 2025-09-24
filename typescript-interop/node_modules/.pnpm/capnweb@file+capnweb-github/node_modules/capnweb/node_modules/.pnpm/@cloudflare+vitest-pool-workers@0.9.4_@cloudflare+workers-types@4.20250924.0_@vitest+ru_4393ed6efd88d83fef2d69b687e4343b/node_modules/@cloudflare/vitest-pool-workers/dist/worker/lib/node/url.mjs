// src/worker/lib/node/url.ts
import path from "node:path";
function fileURLToPath(url) {
  if (typeof url === "string") {
    url = new URL(url);
  } else if (!(url instanceof URL)) {
    throw new TypeError(`Expected path to be string | URL, got ${url}`);
  }
  if (url.protocol !== "file:") {
    throw new TypeError("Expected protocol to be file:");
  }
  return getPathFromURLPosix(url);
}
function getPathFromURLPosix(url) {
  if (url.hostname !== "") {
    throw new TypeError("Expected hostname to be empty");
  }
  const pathname = url.pathname;
  for (let n = 0; n < pathname.length; n++) {
    if (pathname[n] === "%") {
      const third = pathname.codePointAt(n + 2) | 32;
      if (pathname[n + 1] === "2" && third === 102) {
        throw new TypeError(
          "Expected pathname not to include encoded / characters"
        );
      }
    }
  }
  return decodeURIComponent(pathname);
}
var CHAR_FORWARD_SLASH = 47;
var percentRegEx = /%/g;
var backslashRegEx = /\\/g;
var newlineRegEx = /\n/g;
var carriageReturnRegEx = /\r/g;
var tabRegEx = /\t/g;
var questionRegex = /\?/g;
var hashRegex = /#/g;
function pathToFileURL(filepath) {
  let resolved = path.resolve(filepath);
  const filePathLast = filepath.charCodeAt(filepath.length - 1);
  if (filePathLast === CHAR_FORWARD_SLASH && resolved[resolved.length - 1] !== path.sep) {
    resolved += "/";
  }
  resolved = encodePathChars(resolved);
  if (resolved.indexOf("?") !== -1) {
    resolved = resolved.replace(questionRegex, "%3F");
  }
  if (resolved.indexOf("#") !== -1) {
    resolved = resolved.replace(hashRegex, "%23");
  }
  return new URL(`file://${resolved}`);
}
function encodePathChars(filepath) {
  if (filepath.indexOf("%") !== -1) {
    filepath = filepath.replace(percentRegEx, "%25");
  }
  if (filepath.indexOf("\\") !== -1) {
    filepath = filepath.replace(backslashRegEx, "%5C");
  }
  if (filepath.indexOf("\n") !== -1) {
    filepath = filepath.replace(newlineRegEx, "%0A");
  }
  if (filepath.indexOf("\r") !== -1) {
    filepath = filepath.replace(carriageReturnRegEx, "%0D");
  }
  if (filepath.indexOf("	") !== -1) {
    filepath = filepath.replace(tabRegEx, "%09");
  }
  return filepath;
}
var url_default = {
  fileURLToPath,
  pathToFileURL
};
export {
  CHAR_FORWARD_SLASH,
  url_default as default,
  fileURLToPath,
  pathToFileURL
};
//# sourceMappingURL=url.mjs.map
