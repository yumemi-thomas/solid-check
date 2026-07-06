/**
 * @jest-environment jsdom
 *
 * solidjs/solid#2830 — nodes migrating between adjacent expression slots.
 *
 * The compiler used to give adjacent slots the same insertion marker (null at
 * the tail, or a shared following sibling), collapsing them into one ownership
 * region: one slot's cleanup would remove the node its neighbor just claimed.
 * Slots in multi-slot parents now each get their own truthy marker (`<!>`
 * placeholders when no static sibling follows), so the runtime's $$SLOT
 * ownership discriminates and re-tags on adoption — same geometry hydratable
 * output has always produced.
 */
import { createRoot, createSignal, flush } from "@solidjs/signals";

describe("adjacent expression slots (#2830)", () => {
  test("two adjacent slots exchange hoisted elements", () => {
    const el1 = <span>1</span>;
    const el2 = <b>2</b>;
    const [swap, setSwap] = createSignal(false);
    const div = createRoot(() => (
      <div>
        {swap() ? el2 : el1}
        {swap() ? el1 : el2}
      </div>
    ));
    expect(div.textContent).toBe("12");

    setSwap(true);
    flush();
    expect(div.textContent).toBe("21");
    expect(el1.parentNode).toBe(div);
    expect(el2.parentNode).toBe(div);

    setSwap(false);
    flush();
    expect(div.textContent).toBe("12");
    expect(el1.parentNode).toBe(div);
    expect(el2.parentNode).toBe(div);
  });

  test("single node migrating forward to the adjacent slot", () => {
    const el = <b>X</b>;
    const [right, setRight] = createSignal(false);
    const div = createRoot(() => (
      <div>
        {right() ? null : el}
        {right() ? el : null}
      </div>
    ));
    expect(div.textContent).toBe("X");

    setRight(true);
    flush();
    expect(div.textContent).toBe("X");
    expect(el.parentNode).toBe(div);

    setRight(false);
    flush();
    expect(div.textContent).toBe("X");
    expect(el.parentNode).toBe(div);
  });

  test("single node migrating backward to the preceding slot", () => {
    const el = <b>X</b>;
    const [left, setLeft] = createSignal(false);
    const div = createRoot(() => (
      <div>
        {left() ? el : null}
        {left() ? null : el}
      </div>
    ));
    expect(div.textContent).toBe("X");

    setLeft(true);
    flush();
    expect(div.textContent).toBe("X");
    expect(el.parentNode).toBe(div);
  });

  test("arrays exchanging members across adjacent slots", () => {
    const a = <i>a</i>;
    const b = <i>b</i>;
    const c = <i>c</i>;
    const d = <i>d</i>;
    const [swap, setSwap] = createSignal(false);
    const div = createRoot(() => (
      <div>
        {swap() ? [c, d] : [a, b]}
        {swap() ? [a, b] : [c, d]}
      </div>
    ));
    expect(div.textContent).toBe("abcd");

    setSwap(true);
    flush();
    expect(div.textContent).toBe("cdab");

    setSwap(false);
    flush();
    expect(div.textContent).toBe("abcd");
  });

  test("rotation across three adjacent slots", () => {
    const a = <i>a</i>;
    const b = <i>b</i>;
    const c = <i>c</i>;
    const order = [
      [a, b, c],
      [c, a, b],
      [b, c, a]
    ];
    const [step, setStep] = createSignal(0);
    const div = createRoot(() => (
      <div>
        {order[step()][0]}
        {order[step()][1]}
        {order[step()][2]}
      </div>
    ));
    expect(div.textContent).toBe("abc");

    setStep(1);
    flush();
    expect(div.textContent).toBe("cab");

    setStep(2);
    flush();
    expect(div.textContent).toBe("bca");

    setStep(0);
    flush();
    expect(div.textContent).toBe("abc");
  });

  test("interleaved dynamic text slot does not merge neighbors", () => {
    const el1 = <span>1</span>;
    const el2 = <b>2</b>;
    const [swap, setSwap] = createSignal(false);
    const [label, setLabel] = createSignal("-");
    const div = createRoot(() => (
      <div>
        {swap() ? el2 : el1}
        {label()}
        {swap() ? el1 : el2}
      </div>
    ));
    expect(div.textContent).toBe("1-2");

    setSwap(true);
    flush();
    expect(div.textContent).toBe("2-1");

    setLabel("+");
    flush();
    expect(div.textContent).toBe("2+1");
  });

  test("one-sided update after migration leaves no ghost", () => {
    const el = <b>X</b>;
    const [right, setRight] = createSignal(false);
    const [tail, setTail] = createSignal(null);
    const div = createRoot(() => (
      <div>
        {right() ? null : el}
        {right() ? el : tail()}
      </div>
    ));
    expect(div.textContent).toBe("X");

    // el migrates into the second slot
    setRight(true);
    flush();
    expect(div.textContent).toBe("X");

    // second slot alone replaces its content — el must actually leave the DOM
    setRight(false);
    setTail(<u>Y</u>);
    flush();
    expect(div.textContent).toBe("XY");
    setRight(true);
    flush();
    expect(div.textContent).toBe("X");
    expect(div.querySelectorAll("u").length).toBe(0);
  });

  test("emptied slot refills in position, not at the end", () => {
    const x = <i>x</i>;
    const y = <b>y</b>;
    const [full, setFull] = createSignal(true);
    const div = createRoot(() => (
      <div>
        {full() ? [x] : []}
        {y}
      </div>
    ));
    expect(div.textContent).toBe("xy");

    setFull(false);
    flush();
    expect(div.textContent).toBe("y");

    setFull(true);
    flush();
    expect(div.textContent).toBe("xy");
  });

  test("control: static element between slots still works", () => {
    const el1 = <span>1</span>;
    const el2 = <b>2</b>;
    const [swap, setSwap] = createSignal(false);
    const div = createRoot(() => (
      <div>
        {swap() ? el2 : el1}
        <hr />
        {swap() ? el1 : el2}
      </div>
    ));
    expect(div.textContent).toBe("12");

    setSwap(true);
    flush();
    expect(div.textContent).toBe("21");

    setSwap(false);
    flush();
    expect(div.textContent).toBe("12");
  });

  test("control: migration between slots in different parents", () => {
    const el = <b>X</b>;
    const [right, setRight] = createSignal(false);
    const div = createRoot(() => (
      <div>
        <div>{right() ? null : el}</div>
        <div>{right() ? el : null}</div>
      </div>
    ));
    const [left, rightDiv] = div.children;
    expect(left.textContent).toBe("X");

    setRight(true);
    flush();
    expect(rightDiv.textContent).toBe("X");
    expect(left.textContent).toBe("");

    setRight(false);
    flush();
    expect(left.textContent).toBe("X");
    expect(rightDiv.textContent).toBe("");
  });
});
