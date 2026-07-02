/**
 * @jest-environment jsdom
 */
import { createRoot } from "@solidjs/signals";

describe("static initialization props", () => {
  it("applies static props passed during element creation", () => {
    let div, dispose;

    createRoot(disposer => {
      dispose = disposer;
      div = <div id="main" class="box" style={{ color: "red" }} textContent="Ready" />;
    });

    expect(div.id).toBe("main");
    expect(div.className).toBe("box");
    expect(div.style.color).toBe("red");
    expect(div.textContent).toBe("Ready");
    dispose();
  });
});
