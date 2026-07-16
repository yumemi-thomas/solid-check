const crypto = require("node:crypto");
const { transform } = require("../index");

function hash(source) {
  return `sha256:${crypto.createHash("sha256").update(source).digest("hex")}`;
}

describe("compiler-native ExecutionMap", () => {
  it("records the original spans chosen by DOM transform branches", () => {
    const source =
      "const 東京 = true; const view = <button title={count()} onClick={() => count()}>{count()}</button>;";
    const result = transform(source, {
      filename: "App.tsx",
      moduleName: "dom",
      generate: "dom",
      compilerFacts: true
    });

    expect(result.code).toContain("_$effect");
    expect(result.code).toContain(".$$click = () => count()");
    expect(result.code).toContain("_$insert");
    expect(result.executionMap).toEqual({
      compilerFactsProtocol: 1,
      sourceHash: hash(source),
      trackedRegions: [
        {
          span: byteSpan(source, source.indexOf("count()")),
          reason: "jsx-attribute"
        },
        {
          span: byteSpan(source, source.lastIndexOf("count()")),
          reason: "jsx-child"
        }
      ],
      untrackedRegions: [],
      ownershipRegions: [],
      callbackRoles: [
        {
          span: byteSpan(source, source.indexOf("() => count()"), "() => count()"),
          role: "event-handler"
        }
      ],
      jsxOperations: [
        {
          span: byteSpan(source, source.indexOf("count()")),
          kind: "dynamic-attribute"
        },
        {
          span: byteSpan(source, source.indexOf("() => count()"), "() => count()"),
          kind: "event-listener"
        },
        {
          span: byteSpan(source, source.lastIndexOf("count()")),
          kind: "insert"
        }
      ]
    });
  });

  it("does not emit facts unless explicitly requested", () => {
    const result = transform("const view = <div>{count()}</div>;", {
      filename: "App.tsx",
      moduleName: "dom"
    });
    expect(result.executionMap).toBeUndefined();
  });

  it("follows compiler options that change whether expressions are tracked", () => {
    const attribute = transform("const view = <div title={count()} />;", {
      filename: "App.tsx",
      moduleName: "dom",
      compilerFacts: true,
      effectWrapper: false
    });
    expect(attribute.code).not.toContain("_$effect");
    expect(attribute.executionMap.trackedRegions).toEqual([]);
    expect(attribute.executionMap.jsxOperations).toEqual([]);

    const child = transform("const view = <div>{/*@once*/ count()}</div>;", {
      filename: "App.tsx",
      moduleName: "dom",
      compilerFacts: true,
      staticMarker: "@once"
    });
    expect(child.executionMap.trackedRegions).toEqual([]);
    expect(child.executionMap.jsxOperations).toEqual([]);
  });

  it("rejects compiler-facts analysis for unsupported output modes", () => {
    expect(() =>
      transform("const view = <div>{count()}</div>;", {
        filename: "App.tsx",
        moduleName: "dom/server",
        generate: "ssr",
        compilerFacts: true
      })
    ).toThrow(/compiler facts.*dom/i);

    expect(() =>
      transform("const view = <div>{count()}</div>;", {
        filename: "App.tsx",
        moduleName: "dom",
        compilerFacts: true,
        requireImportSource: "solid-js"
      })
    ).toThrow(/compiler facts.*requireImportSource/i);
  });
});

function byteSpan(source, characterStart, text = "count()") {
  return {
    start: Buffer.byteLength(source.slice(0, characterStart)),
    end: Buffer.byteLength(source.slice(0, characterStart) + text)
  };
}
