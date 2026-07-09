/**
 * @jest-environment jsdom
 */
import * as r from "../../src/client";
import * as r2 from "../../src/server";

globalThis._$HY = { events: [], completed: new WeakSet() };

// Regression for dynamic spread props shifting the element's own hydration id.
// `mergeProps` with a function source creates a memo, which consumes a
// hydration child id. The client claims the element (getNextElement) before
// applying the spread, so the memo's id comes after the element's. The ssr
// generate used to evaluate `mergeProps(...)` in ssrElement's argument
// position — before the element's id was allocated — so the element's own id
// shifted by one and it was left unclaimed (later siblings re-synced, hiding
// the drift). Compiled shape (both generates) for:
//
//   <title ref={fn} {...props.attrs}>{c()}</title> followed by <span>tail</span>
//
// The ssr generate now defers the merge behind a thunk and ssrElement
// allocates the hydration key before resolving it.
describe("hydrating a spread element with dynamic (merged) props", () => {
  function run(serverProps, clientProps) {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const _tmpl$ = r.template(`<title>`);
    const _tmpl$2 = r.template(`<span>tail`);
    const child = () => "hi";

    const rendered = r2.renderToString(() => [
      r2.ssrElement("title", serverProps, () => r2.scope(() => r2.escape(child())), true),
      r2.ssr(["<span", ">tail</span>"], r2.ssrHydrationKey())
    ]);
    container.innerHTML = rendered;
    const serverTitle = container.firstChild;
    const serverSpan = serverTitle.nextSibling;

    let el;
    let clientTitle, clientSpan;
    r.hydrate(() => {
      const fragment = [
        (() => {
          const _el$ = r.getNextElement(_tmpl$);
          r.ref(() => e => (el = e), _el$);
          r.spread(_el$, clientProps(), true);
          r.insert(
            _el$,
            r.scope(() => child())
          );
          return (clientTitle = _el$);
        })(),
        (() => (clientSpan = r.getNextElement(_tmpl$2)))()
      ];
      r.insert(container, fragment, undefined, [...container.childNodes]);
      r.runHydrationEvents();
    }, container);

    const result = {
      titleClaimed: clientTitle === serverTitle,
      spanClaimed: clientSpan === serverSpan,
      refAssigned: el === clientTitle
    };
    container.remove();
    return result;
  }

  it("claims the element when merged props are deferred behind a thunk", () => {
    const attrs = { class: "x" };
    expect(
      run(
        // server: `() => _$mergeProps(() => props.attrs)` (hydratable output)
        () => r2.mergeProps(() => attrs),
        // client: `_$spread(_el$, _$mergeProps(() => props.attrs), true)`
        () => r.mergeProps(() => attrs)
      )
    ).toEqual({ titleClaimed: true, spanClaimed: true, refAssigned: true });
  });

  it("claims the element with plain object props", () => {
    const attrs = { class: "x" };
    expect(run(attrs, () => attrs)).toEqual({
      titleClaimed: true,
      spanClaimed: true,
      refAssigned: true
    });
  });
});
