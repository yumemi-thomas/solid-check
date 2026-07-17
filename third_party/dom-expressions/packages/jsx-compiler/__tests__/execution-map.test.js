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
        },
        {
          span: byteSpan(source, source.lastIndexOf("count()")),
          kind: "jsx-expression"
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
    const attributeSource = "const view = <div title={count()} />;";
    const attribute = transform(attributeSource, {
      filename: "App.tsx",
      moduleName: "dom",
      compilerFacts: true,
      effectWrapper: false
    });
    expect(attribute.code).not.toContain("_$effect");
    expect(attribute.executionMap.trackedRegions).toEqual([]);
    expect(attribute.executionMap.untrackedRegions).toEqual([
      {
        span: byteSpan(attributeSource, attributeSource.indexOf("count()")),
        reason: "jsx-attribute"
      }
    ]);
    expect(attribute.executionMap.jsxOperations).toEqual([]);

    const childSource = "const view = <div>{/*@once*/ count()}</div>;";
    const child = transform(childSource, {
      filename: "App.tsx",
      moduleName: "dom",
      compilerFacts: true,
      staticMarker: "@once"
    });
    expect(child.executionMap.trackedRegions).toEqual([]);
    expect(child.executionMap.untrackedRegions).toEqual([
      {
        span: byteSpan(childSource, childSource.indexOf("count()")),
        reason: "jsx-child"
      }
    ]);
    expect(child.executionMap.jsxOperations).toEqual([
      {
        span: byteSpan(childSource, childSource.indexOf("count()")),
        kind: "jsx-expression"
      }
    ]);
  });

  it("records untracked regions for holes the compiler renders once", () => {
    const staticChild = 'const size = 4; const view = <div>{"static"}{size}</div>;';
    const inlined = transform(staticChild, {
      filename: "App.tsx",
      moduleName: "dom",
      generate: "dom",
      compilerFacts: true
    }).executionMap;
    expect(inlined.trackedRegions).toEqual([]);
    expect(inlined.untrackedRegions).toEqual([
      {
        span: byteSpan(staticChild, staticChild.indexOf('"static"'), '"static"'),
        reason: "jsx-child"
      },
      {
        span: byteSpan(staticChild, staticChild.lastIndexOf("size"), "size"),
        reason: "jsx-child"
      }
    ]);
    expect(inlined.jsxOperations).toEqual([
      {
        span: byteSpan(staticChild, staticChild.indexOf('"static"'), '"static"'),
        kind: "jsx-expression"
      },
      {
        span: byteSpan(staticChild, staticChild.lastIndexOf("size"), "size"),
        kind: "jsx-expression"
      }
    ]);

    const runSource = "const view = <div>{first}{second()}</div>;";
    const run = transform(runSource, {
      filename: "App.tsx",
      moduleName: "dom",
      generate: "dom",
      compilerFacts: true
    }).executionMap;
    expect(run.trackedRegions).toEqual([
      {
        span: byteSpan(runSource, runSource.indexOf("second()"), "second()"),
        reason: "jsx-child"
      }
    ]);
    expect(run.untrackedRegions).toEqual([
      {
        span: byteSpan(runSource, runSource.indexOf("first"), "first"),
        reason: "jsx-child"
      }
    ]);
    expect(run.jsxOperations).toEqual([
      {
        span: byteSpan(runSource, runSource.indexOf("first"), "first"),
        kind: "jsx-expression"
      },
      {
        span: byteSpan(runSource, runSource.indexOf("second()"), "second()"),
        kind: "insert"
      },
      {
        span: byteSpan(runSource, runSource.indexOf("second()"), "second()"),
        kind: "jsx-expression"
      }
    ]);

    const componentSource = 'const view = <Comp label={"static"}>{items}</Comp>;';
    const component = transform(componentSource, {
      filename: "App.tsx",
      moduleName: "dom",
      generate: "dom",
      compilerFacts: true
    }).executionMap;
    expect(component.callbackRoles).toEqual([]);
    expect(component.untrackedRegions).toEqual([
      {
        span: byteSpan(componentSource, componentSource.indexOf('"static"'), '"static"'),
        reason: "component-getter"
      },
      {
        span: byteSpan(componentSource, componentSource.indexOf("items"), "items"),
        reason: "component-getter"
      }
    ]);
  });

  it("is deterministic across Unicode, CRLF, hydratable, and dev configurations", () => {
    const source = "const emoji = '😀';\r\nconst view = <div title={東京()}>{東京()}</div>;";
    const maps = [];
    for (const hydratable of [false, true]) {
      for (const dev of [false, true]) {
        const first = transform(source, {
          filename: "東京.tsx",
          moduleName: "dom",
          generate: "dom",
          hydratable,
          dev,
          compilerFacts: true
        }).executionMap;
        const second = transform(source, {
          filename: "東京.tsx",
          moduleName: "dom",
          generate: "dom",
          hydratable,
          dev,
          compilerFacts: true
        }).executionMap;
        expect(second).toEqual(first);
        expect(first.trackedRegions).toEqual([
          {
            span: byteSpan(source, source.indexOf("東京()"), "東京()"),
            reason: "jsx-attribute"
          },
          {
            span: byteSpan(source, source.lastIndexOf("東京()"), "東京()"),
            reason: "jsx-child"
          }
        ]);
        maps.push(first);
      }
    }
    for (const map of maps.slice(1)) expect(map).toEqual(maps[0]);
  });

  it("records component invocations, deferred getters, and built-in render callbacks", () => {
    const customSource = "const view = <Comp value={count()} />;";
    const custom = transform(customSource, {
      filename: "App.tsx",
      moduleName: "dom",
      generate: "dom",
      compilerFacts: true
    }).executionMap;
    expect(custom.callbackRoles).toEqual([
      {
        span: byteSpan(customSource, customSource.indexOf("count()")),
        role: "deferred"
      }
    ]);
    expect(custom.jsxOperations).toEqual([
      {
        span: byteSpan(customSource, customSource.indexOf("<Comp"), "<Comp value={count()} />"),
        kind: "component-invocation"
      },
      {
        span: byteSpan(customSource, customSource.indexOf("count()")),
        kind: "component-property"
      }
    ]);

    const forSource =
      "const view = <For each={items()}>{item => <span>{item()}</span>}</For>;";
    const controlFlow = transform(forSource, {
      filename: "App.tsx",
      moduleName: "dom",
      generate: "dom",
      builtIns: ["For"],
      compilerFacts: true
    }).executionMap;
    expect(controlFlow.callbackRoles).toEqual([
      {
        span: byteSpan(forSource, forSource.indexOf("items()"), "items()"),
        role: "deferred"
      },
      {
        span: byteSpan(
          forSource,
          forSource.indexOf("item =>"),
          "item => <span>{item()}</span>"
        ),
        role: "render"
      }
    ]);
  });

  it("records directive factory setup and returned ref application phases", () => {
    const source = "const view = <button ref={tooltip(options())}>Save</button>;";
    const facts = transform(source, {
      filename: "App.tsx",
      moduleName: "dom",
      generate: "dom",
      compilerFacts: true
    }).executionMap;
    const expression = "tooltip(options())";
    expect(facts.callbackRoles).toEqual([
      {
        span: byteSpan(source, source.indexOf(expression), expression),
        role: "directive-apply"
      }
    ]);
    expect(facts.jsxOperations).toEqual([
      {
        span: byteSpan(source, source.indexOf(expression), expression),
        kind: "directive-apply"
      },
      {
        span: byteSpan(source, source.indexOf(expression), expression),
        kind: "directive-setup"
      }
    ]);

    const directSource = "const view = <button ref={element => focus(element)}>Save</button>;";
    const direct = transform(directSource, {
      filename: "App.tsx",
      moduleName: "dom",
      generate: "dom",
      compilerFacts: true
    }).executionMap;
    const callback = "element => focus(element)";
    expect(direct.callbackRoles).toEqual([
      {
        span: byteSpan(directSource, directSource.indexOf(callback), callback),
        role: "directive-apply"
      }
    ]);
    expect(direct.jsxOperations).toEqual([
      {
        span: byteSpan(directSource, directSource.indexOf(callback), callback),
        kind: "directive-apply"
      }
    ]);
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
