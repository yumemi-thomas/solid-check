"use strict";

const fs = require("fs");
const path = require("path");

const native = requireNative();

function transform(code, options) {
  if (typeof code !== "string") {
    throw new TypeError(
      "@dom-expressions/jsx-compiler transform() expects source code as a string"
    );
  }

  const nativeOptions = validateOptions(code, options);
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
  "requireImportSource",
  "validate",
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
    throw new TypeError(
      "@dom-expressions/jsx-compiler transform() expects options to be an object"
    );
  }

  const nativeOptions = {};
  for (const [key, value] of Object.entries(options)) {
    if (key === "effectWrapper" || key === "memoWrapper") {
      if (typeof value !== "string" && typeof value !== "boolean") {
        throw new TypeError(
          `@dom-expressions/jsx-compiler \`${key}\` option must be a string import name or false`
        );
      }
      nativeOptions[key] = value;
      continue;
    }
    if (key === "requireImportSource") {
      if (value !== false && typeof value !== "string") {
        throw new TypeError(
          "@dom-expressions/jsx-compiler `requireImportSource` option must be false or a string"
        );
      }
      if (value !== false) nativeOptions.requireImportSource = value;
      continue;
    }
    if (key === "wrapConditionals") {
      if (typeof value !== "boolean") {
        throw new TypeError(
          "@dom-expressions/jsx-compiler `wrapConditionals` option must be boolean"
        );
      }
      nativeOptions.wrapConditionals = value;
      continue;
    }
    if (key === "validate") {
      if (typeof value !== "boolean") {
        throw new TypeError("@dom-expressions/jsx-compiler `validate` option must be boolean");
      }
      nativeOptions.validate = value;
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
        `@dom-expressions/jsx-compiler does not support non-default \`${key}\` options yet`
      );
    }
    throw new Error(`@dom-expressions/jsx-compiler received unknown option \`${key}\``);
  }
  return nativeOptions;
}

function validateRenderers(renderers) {
  if (renderers == null) return;
  if (!Array.isArray(renderers)) {
    throw new TypeError("@dom-expressions/jsx-compiler `renderers` option must be an array");
  }

  for (const renderer of renderers) {
    if (typeof renderer !== "object" || renderer == null || Array.isArray(renderer)) {
      throw new TypeError("@dom-expressions/jsx-compiler renderer entries must be objects");
    }
    for (const key of Object.keys(renderer)) {
      if (key !== "name" && key !== "moduleName" && key !== "elements") {
        throw new Error(
          `@dom-expressions/jsx-compiler received unknown renderer option \`${key}\``
        );
      }
    }
    if (renderer.name !== "dom") {
      throw new Error(
        "@dom-expressions/jsx-compiler dynamic renderers only support the `dom` renderer override"
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

function platformArchSuffix() {
  const { platform, arch } = process;
  if (platform === "darwin" && (arch === "x64" || arch === "arm64")) return `darwin-${arch}`;
  if (platform === "linux" && (arch === "x64" || arch === "arm64")) return `linux-${arch}-gnu`;
  if (platform === "win32" && arch === "x64") return "win32-x64-msvc";
  return null;
}

function requireNative() {
  const explicit = process.env.JSX_DOM_EXPRESSIONS_COMPILER_NATIVE;
  if (explicit) return require(explicit);

  const suffix = platformArchSuffix();

  // Local builds (napi build output) take precedence for development.
  const localCandidates = [];
  if (suffix) localCandidates.push(`jsx-compiler.${suffix}.node`);
  localCandidates.push("jsx-compiler.node");
  for (const file of localCandidates) {
    const full = path.join(__dirname, file);
    if (fs.existsSync(full)) return require(full);
  }

  if (suffix) {
    try {
      return require(`@dom-expressions/jsx-compiler-${suffix}`);
    } catch (error) {
      if (error.code !== "MODULE_NOT_FOUND") throw error;
    }
  }

  throw new Error(
    `Could not find the native @dom-expressions/jsx-compiler binary for ${process.platform}-${process.arch}` +
      (suffix
        ? ` (expected the @dom-expressions/jsx-compiler-${suffix} package or a local build)`
        : " (no prebuilt binary is published for this platform)") +
      ". Run `pnpm run build` in packages/jsx-compiler to build from source."
  );
}

module.exports = {
  transform,
  transformAsync
};
