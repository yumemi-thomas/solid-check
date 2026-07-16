/**
 * @jest-environment jsdom
 *
 * Streaming writes `<link rel="stylesheet">` tags into the document *body*
 * when a styled fragment flushes after the shell (see the styled-fragment
 * branch of renderToStream). The browser parser appends them as trailing
 * children of <body>, which places them inside the region a document-level
 * hydration root claims as its initial `current` array.
 *
 * These tests verify what happens to such stream-injected assets during and
 * after hydration — specifically whether a post-hydration re-render at the
 * claiming insert's level removes them (dropping the stylesheet → FOUC).
 */
import * as r from "../../src/client";
import * as r2 from "../../src/server";
import { createSignal, flush } from "@solidjs/signals";
import { sharedConfig } from "../core";

globalThis._$HY = { events: [], completed: new WeakSet() };

describe("stream-injected body stylesheet links vs hydration", () => {
  const container = document.createElement("div");
  const _tmpl$home = r.template(`<div>home</div>`);
  const _tmpl$other = r.template(`<div>other</div>`);
  document.body.appendChild(container);

  function injectStreamLink(href) {
    // Simulates what the streaming buffer leaves behind: a stylesheet link
    // appended at the end of the hydration root's child list.
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = href;
    container.appendChild(link);
    return link;
  }

  it("keeps the link connected through hydration itself", () => {
    const rendered = r2.renderToString(() => r2.ssr(["<div", ">home</div>"], r2.ssrHydrationKey()));
    container.innerHTML = rendered;
    const link = injectStreamLink("/hydrate-only.css");

    r.hydrate(() => {
      const el = (function () {
        const _el$ = r.getNextElement(_tmpl$home);
        r.runHydrationEvents(_el$.getAttribute("_hk"));
        return _el$;
      })();
      r.insert(container, el, undefined, [...container.childNodes]);
      r.runHydrationEvents();
    }, container);

    expect(link.isConnected).toBe(true);
    container.innerHTML = "";
  });

  it("keeps the link connected across a post-hydration root re-render", () => {
    const rendered = r2.renderToString(() => r2.ssr(["<div", ">home</div>"], r2.ssrHydrationKey()));
    container.innerHTML = rendered;
    const link = injectStreamLink("/route.css");

    let setPage;
    r.hydrate(() => {
      const [page, set] = createSignal("home");
      setPage = set;
      r.insert(
        container,
        () => {
          if (page() === "home") {
            const _el$ = r.getNextElement(_tmpl$home);
            r.runHydrationEvents(_el$.getAttribute("_hk"));
            return _el$;
          }
          return _tmpl$other();
        },
        undefined,
        [...container.childNodes]
      );
      r.runHydrationEvents();
    }, container);

    expect(container.querySelector("div").textContent).toBe("home");
    expect(link.isConnected).toBe(true);

    // Simulate a client-side navigation after hydration completes — the
    // root-level insert re-renders and cleans its previous `current` nodes.
    setPage("other");
    flush();

    expect(container.querySelector("div").textContent).toBe("other");
    // The stream-injected stylesheet must survive the content swap.
    expect(link.isConnected).toBe(true);
    container.innerHTML = "";
  });

  it("keeps the link connected when it sits outside a marker-bounded hole", () => {
    // Marker-bounded dynamic regions (<!--#-->...<!--/-->) only clean nodes
    // within their range; a link injected after the end marker is outside it.
    const rendered = r2.renderToString(() =>
      r2.ssr(["<section", "><!--#-->", "<!--/--></section>"], r2.ssrHydrationKey(), r2.escape("A"))
    );
    container.innerHTML = rendered;
    const section = container.firstChild;
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = "/scoped.css";
    section.appendChild(link); // after the <!--/--> end marker

    const _tmpl$section = r.template(`<section><!--#--><!--/--></section>`);
    let setText;
    r.hydrate(() => {
      const [text, set] = createSignal("A");
      setText = set;
      const el = (function () {
        const _el$ = r.getNextElement(_tmpl$section);
        const _el$2 = _el$.firstChild,
          [_el$3, _co$] = r.getNextMarker(_el$2.nextSibling);
        r.insert(_el$, () => text(), _el$3, _co$);
        r.runHydrationEvents(_el$.getAttribute("_hk"));
        return _el$;
      })();
      r.insert(container, el, undefined, [...container.childNodes]);
      r.runHydrationEvents();
    }, container);

    setText("B");
    flush();

    expect(section.textContent).toContain("B");
    expect(link.isConnected).toBe(true);
    container.innerHTML = "";
  });

  it("keeps the link connected when a root fragment empties", () => {
    // cleanChildren's no-marker path used to wipe ALL of parent's children
    // (`parent.textContent = ""`), including foreign nodes like
    // stream-injected links. It now removes only the nodes it tracks when
    // foreign siblings are present, so a root-level array expression that
    // becomes empty leaves the stylesheet in place.
    const rendered = r2.renderToString(() => [
      r2.ssr(["<div", ">a</div>"], r2.ssrHydrationKey()),
      r2.ssr(["<div", ">b</div>"], r2.ssrHydrationKey())
    ]);
    container.innerHTML = rendered;
    const link = injectStreamLink("/fragment-root.css");
    const _tmpl$a = r.template(`<div>a</div>`);
    const _tmpl$b = r.template(`<div>b</div>`);

    let setShow;
    r.hydrate(() => {
      const [show, set] = createSignal(true);
      setShow = set;
      r.insert(
        container,
        () => {
          if (!show()) return [];
          const a = r.getNextElement(_tmpl$a);
          r.runHydrationEvents(a.getAttribute("_hk"));
          const b = r.getNextElement(_tmpl$b);
          r.runHydrationEvents(b.getAttribute("_hk"));
          return [a, b];
        },
        undefined,
        [...container.childNodes]
      );
      r.runHydrationEvents();
    }, container);

    expect(link.isConnected).toBe(true);

    setShow(false);
    flush();

    // The fragment's own nodes are gone...
    expect(container.querySelector("div")).toBe(null);
    // ...but the stream-injected stylesheet survives the clear.
    expect(link.isConnected).toBe(true);
    container.innerHTML = "";
  });

  it("keeps the link connected when a root expression becomes text", () => {
    // The string path of insertExpression also cleared via `textContent =
    // value`; with foreign nodes present it now swaps only its own nodes.
    const rendered = r2.renderToString(() => r2.ssr(["<div", ">a</div>"], r2.ssrHydrationKey()));
    container.innerHTML = rendered;
    const link = injectStreamLink("/text-swap.css");
    const _tmpl$a = r.template(`<div>a</div>`);

    let setShow;
    r.hydrate(() => {
      const [show, set] = createSignal(true);
      setShow = set;
      r.insert(
        container,
        () => {
          if (!show()) return "plain text";
          const a = r.getNextElement(_tmpl$a);
          r.runHydrationEvents(a.getAttribute("_hk"));
          return a;
        },
        undefined,
        [...container.childNodes]
      );
      r.runHydrationEvents();
    }, container);

    expect(link.isConnected).toBe(true);

    setShow(false);
    flush();

    expect(container.textContent).toContain("plain text");
    expect(link.isConnected).toBe(true);
    container.innerHTML = "";
  });
});
