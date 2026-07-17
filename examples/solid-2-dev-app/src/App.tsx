import { createSignal } from "solid-js";

export function App() {
  const [count, setCount] = createSignal(0);

  return (
    <main class="card">
      <p class="eyebrow">Solid 2 + solid-check + Oxlint</p>
      <h1>Reactive counter</h1>
      <p class="count">{count()}</p>
      <button type="button" onClick={() => setCount(value => value + 1)}>
        Increment
      </button>
      <p class={{ hint: true, active: count() > 2 }}>
        Signal reads stay inside tracked JSX.
      </p>
    </main>
  );
}
