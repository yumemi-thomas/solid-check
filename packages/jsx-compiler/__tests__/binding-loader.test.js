"use strict";

const fs = require("fs");

describe("binding loader", () => {
  afterEach(() => {
    jest.restoreAllMocks();
    jest.resetModules();
  });

  test("falls back to WASI when native addons cannot load", () => {
    const existsSync = fs.existsSync;
    jest
      .spyOn(fs, "existsSync")
      .mockImplementation(file => !String(file).endsWith(".node") && existsSync(file));

    const nativePackage =
      process.platform === "darwin"
        ? `@dom-expressions/jsx-compiler-darwin-${process.arch}`
        : process.platform === "linux"
          ? `@dom-expressions/jsx-compiler-linux-${process.arch}-gnu`
          : "@dom-expressions/jsx-compiler-win32-x64-msvc";

    jest.doMock(
      nativePackage,
      () => {
        throw new Error("Cannot load native addon because loading addons is disabled");
      },
      { virtual: true }
    );
    jest.doMock(
      "@dom-expressions/jsx-compiler-wasm32-wasi",
      () => ({
        transform() {
          return { code: "wasm", map: null };
        }
      }),
      { virtual: true }
    );

    const compiler = require("..");
    expect(compiler.transform("const value = 1")).toEqual({ code: "wasm", map: null });
  });
});
