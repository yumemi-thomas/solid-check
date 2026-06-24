"use strict";

const fs = require("fs");
const path = require("path");

const native = requireNative();

function transform(code, options) {
  if (typeof code !== "string") {
    throw new TypeError("jsx-dom-expressions-compiler transform() expects source code as a string");
  }

  const nativeOptions = validateOptions(code, options);
  if (nativeOptions?.skip) {
    return {
      code,
      map: null
    };
  }
  const result = native.transform(code, nativeOptions);
  return {
    code: result.code,
    map: result.map ?? null
  };
}

function transformAsync(code, options) {
  return Promise.resolve().then(() => transform(code, options));
}

const nativeOptionKeys = new Set([
  "filename",
  "moduleName",
  "generate",
  "hydratable",
  "dev",
  "sourceMap",
  "contextToCustomElements",
  "delegateEvents",
  "delegatedEvents",
  "omitQuotes",
  "omitAttributeSpacing",
  "inlineStyles",
  "effectWrapper",
  "wrapConditionals",
  "memoWrapper",
  "staticMarker",
  "omitNestedClosingTags",
  "omitLastClosingTag",
  "builtIns",
  "renderers"
]);

const compatibleBabelDefaults = new Map([]);

function validateOptions(code, options) {
  if (options == null) return options;
  if (typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("jsx-dom-expressions-compiler transform() expects options to be an object");
  }

  const wrapperless = options.wrapConditionals === false || options.memoWrapper === false;
  if (wrapperless) {
    if (options.wrapConditionals !== false || options.memoWrapper !== false) {
      throw new Error(
        "jsx-dom-expressions-compiler only supports wrapperless mode when `wrapConditionals: false` and `memoWrapper: false` are used together"
      );
    }
  }

  const nativeOptions = {};
  for (const [key, value] of Object.entries(options)) {
    if (key === "requireImportSource") {
      if (value === false) continue;
      if (typeof value !== "string") {
        throw new TypeError(
          "jsx-dom-expressions-compiler `requireImportSource` option must be false or a string"
        );
      }
      if (!hasJsxImportSource(code, value)) {
        return { skip: true };
      }
      continue;
    }
    if (key === "effectWrapper") {
      if (value === "effect") continue;
      if (value === false) {
        nativeOptions.effectWrapper = false;
        continue;
      }
      throw new Error(
        'jsx-dom-expressions-compiler only supports `effectWrapper: false` or the default `"effect"`'
      );
    }
    if (key === "wrapConditionals") {
      if (value === true) continue;
      if (value === false) {
        nativeOptions.wrapConditionals = false;
        continue;
      }
      throw new TypeError("jsx-dom-expressions-compiler `wrapConditionals` option must be boolean");
    }
    if (key === "memoWrapper") {
      if (value === "memo") continue;
      if (value === false) {
        nativeOptions.memoWrapper = false;
        continue;
      }
      throw new Error(
        'jsx-dom-expressions-compiler only supports `memoWrapper: false` or the default `"memo"`'
      );
    }
    if (key === "validate") {
      if (typeof value !== "boolean") {
        throw new TypeError("jsx-dom-expressions-compiler `validate` option must be boolean");
      }
      continue;
    }
    if (nativeOptionKeys.has(key)) {
      if (key === "renderers") validateRenderers(value);
      nativeOptions[key] = value;
      continue;
    }
    if (compatibleBabelDefaults.has(key)) {
      const defaultValue = compatibleBabelDefaults.get(key);
      if (sameOptionValue(value, defaultValue)) continue;
      throw new Error(
        `jsx-dom-expressions-compiler does not support non-default \`${key}\` options yet`
      );
    }
    throw new Error(`jsx-dom-expressions-compiler received unknown option \`${key}\``);
  }
  return nativeOptions;
}

function hasJsxImportSource(code, source) {
  const pattern = /@jsxImportSource\s+([^\s*]+)/g;
  let match;
  while ((match = pattern.exec(code))) {
    if (match[1] === source) return true;
  }
  return false;
}

function validateRenderers(renderers) {
  if (renderers == null) return;
  if (!Array.isArray(renderers)) {
    throw new TypeError("jsx-dom-expressions-compiler `renderers` option must be an array");
  }

  for (const renderer of renderers) {
    if (typeof renderer !== "object" || renderer == null || Array.isArray(renderer)) {
      throw new TypeError("jsx-dom-expressions-compiler renderer entries must be objects");
    }
    for (const key of Object.keys(renderer)) {
      if (key !== "name" && key !== "moduleName" && key !== "elements") {
        throw new Error(`jsx-dom-expressions-compiler received unknown renderer option \`${key}\``);
      }
    }
    if (renderer.name !== "dom") {
      throw new Error(
        "jsx-dom-expressions-compiler dynamic renderers only support the `dom` renderer override"
      );
    }
  }
}

function sameOptionValue(value, defaultValue) {
  if (Array.isArray(defaultValue)) {
    return Array.isArray(value) && value.length === defaultValue.length;
  }
  return value === defaultValue;
}

function requireNative() {
  const dir = __dirname;
  const explicit = process.env.JSX_DOM_EXPRESSIONS_COMPILER_NATIVE;

  if (explicit) return require(explicit);

  for (const file of fs.readdirSync(dir)) {
    if (file.startsWith("jsx-dom-expressions-compiler") && file.endsWith(".node")) {
      return require(path.join(dir, file));
    }
  }

  throw new Error(
    "Could not find the native jsx-dom-expressions-compiler binary. Run `pnpm run build` in packages/jsx-dom-expressions-compiler."
  );
}

module.exports = {
  transform,
  transformAsync
};
