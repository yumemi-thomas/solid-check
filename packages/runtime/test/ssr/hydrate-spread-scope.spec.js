/**
 * @jest-environment jsdom
 */
import * as r from "../../src/client";
import * as r2 from "../../src/server";

globalThis._$HY = { events: [], completed: new WeakSet() };

// Regression for the ssrElement (spread-element) children path missing the
// hole id scope wrap. The compiled shapes below mirror the two generates for:
//
//   function Wrapper(props) {
//     return <div {...props.attrs}>{props.children}</div>;
//   }
//   <><Wrapper attrs={{ class: "box" }}>hi</Wrapper><span>tail</span></>
//
// The dom generate scope()s the insert accessor for the dynamic child hole,
// reserving one hydration id. If the ssr generate does not evaluate the
// matching ssrElement children thunk under a scope, the server allocates ids
// without the reservation and every id after the hole drifts — the sibling
// <span> (and everything following it) is left unclaimed on hydration.
describe("hydrating a spread element with a dynamic child hole", () => {
  const container = document.createElement("div");
  document.body.appendChild(container);

  const _tmpl$ = r.template(`<div>`),
    _tmpl$2 = r.template(`<span>tail`);

  it("keeps sibling hydration ids in sync", () => {
    const attrs = { class: "box" };
    const child = () => "hi";

    // server (generate: "ssr", hydratable) — children thunk evaluates the
    // dynamic hole under a scope, mirroring transformChildren's template path
    const rendered = r2.renderToString(() => [
      r2.ssrElement("div", attrs, () => r2.scope(() => r2.escape(child())), true),
      r2.ssr(["<span", ">tail</span>"], r2.ssrHydrationKey())
    ]);
    container.innerHTML = rendered;
    const serverDiv = container.firstChild;
    const serverSpan = serverDiv.nextSibling;
    expect(serverSpan.tagName).toBe("SPAN");

    // client (generate: "dom", hydratable)
    let clientDiv, clientSpan;
    r.hydrate(() => {
      const fragment = [
        (() => {
          const _el$ = r.getNextElement(_tmpl$);
          r.spread(_el$, attrs, true);
          r.insert(
            _el$,
            r.scope(() => child())
          );
          return (clientDiv = _el$);
        })(),
        (() => {
          return (clientSpan = r.getNextElement(_tmpl$2));
        })()
      ];
      r.insert(container, fragment, undefined, [...container.childNodes]);
      r.runHydrationEvents();
    }, container);

    // both server nodes are claimed — the same DOM nodes, not re-created
    expect(clientDiv).toBe(serverDiv);
    expect(clientSpan).toBe(serverSpan);
    expect(container.querySelectorAll("span").length).toBe(1);
  });
});
