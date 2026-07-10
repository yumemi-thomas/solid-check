// Option-matrix sweep: recompiles the whole Babel fixture corpus under
// non-default option combinations (one flag flipped at a time per mode) and
// requires normalized output parity, including error parity — a flag that
// only one compiler honors shows up here as a diff.

const {
  modes,
  fixtureNames,
  readFixtureSource,
  compileBabel,
  compileOxc,
  normalize,
  unifiedDiff
} = require("./parity/harness");

// Applied one at a time on top of each mode's base options so a failure
// points at a single flag. Flags without meaning for a generate target are
// still passed to both compilers — parity includes ignoring them identically.
const variants = {
  "omitQuotes:false": { omitQuotes: false },
  "omitAttributeSpacing:false": { omitAttributeSpacing: false },
  "delegateEvents:false": { delegateEvents: false },
  "omitNestedClosingTags:true": { omitNestedClosingTags: true },
  "omitLastClosingTag:false": { omitLastClosingTag: false },
  "wrapConditionals:false": { wrapConditionals: false },
  "effectWrapper:false": { effectWrapper: false },
  customWrappers: { effectWrapper: "createRenderEffect", memoWrapper: "createMemo" },
  "staticMarker:@once": { staticMarker: "@once" },
  "delegatedEvents:custom": { delegatedEvents: ["custom", "keyup"] },
  "contextToCustomElements:flip": mode => ({
    contextToCustomElements: !mode.options.contextToCustomElements
  }),
  "inlineStyles:false": { inlineStyles: false },
  "dev:true": { dev: true },
  "validate:false": { validate: false }
};

// Known Babel bug, not replicated: with `inlineStyles: false`, a static
// `style` on a non-root element whose position doesn't trigger
// `detectExpressions` is silently dropped — Babel's style-to-IIFE preprocess
// runs after the child id-allocation decision, so the element gets no id and
// the style write is discarded (repro: `<svg><rect style="fill:red"/><g/>
// </svg>` loses the style entirely). Oxc emits the style effect.
const knownBabelBugs = new Set([
  "dom/inlineStyles:false/SVG",
  "dom-hydratable/inlineStyles:false/SVG",
  "dynamic/inlineStyles:false/SVG"
]);

describe("option-matrix parity", () => {
  for (const [modeName, mode] of Object.entries(modes)) {
    describe(modeName, () => {
      for (const [variantName, patch] of Object.entries(variants)) {
        const extra = typeof patch === "function" ? patch(mode) : patch;
        const options = { ...mode.options, ...extra };

        test(variantName, () => {
          const failures = [];
          for (const fixture of fixtureNames(modeName)) {
            if (knownBabelBugs.has(`${modeName}/${variantName}/${fixture}`)) continue;
            const source = readFixtureSource(modeName, fixture);
            let babelRaw, oxcRaw, babelErr, oxcErr;
            try {
              babelRaw = compileBabel(source, options);
            } catch (err) {
              babelErr = err.message.split("\n")[0];
            }
            try {
              oxcRaw = compileOxc(source, fixture, options);
            } catch (err) {
              oxcErr = err.message.split("\n")[0];
            }
            if (babelErr || oxcErr) {
              if (babelErr && oxcErr) continue; // error parity
              failures.push(
                `${fixture}: babel error: ${babelErr ?? "-"} | oxc error: ${oxcErr ?? "-"}`
              );
              continue;
            }
            const babelNorm = normalize(babelRaw);
            const oxcNorm = normalize(oxcRaw);
            if (babelNorm !== oxcNorm) {
              failures.push(`${fixture}:\n${unifiedDiff(babelNorm, oxcNorm)}`);
            }
          }
          if (failures.length) {
            throw new Error(
              `${failures.length} fixture(s) diverged under ${modeName} + ${variantName}:\n\n` +
                failures.join("\n")
            );
          }
        });
      }
    });
  }
});
