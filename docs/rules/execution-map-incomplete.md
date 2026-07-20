# execution-map-incomplete

`SC9004` · **error** · uncertifiable

The Solid compiler did not classify a JSX expression as tracked, untracked, or a
callback.

## What it does

Flags JSX expression positions for which the compiler facts carry no execution
role. Every reactive-read rule depends on knowing whether a position tracks;
without a classification, reads inside the expression can be neither certified nor
proven wrong.

## Why is this analysis-limiting?

solid-check's read analysis is anchored on the compiler's execution map: each JSX
region is tracked (subscribes), untracked (runs once), or a callback (runs later
under its own rules). A gap in that map usually means the expression's shape falls
outside what the JSX compiler recognizes — or that the compiler facts on disk are
stale relative to the source.

## How to fix

Two things to try, in order:

1. **Simplify the expression.** Hoist complex logic into a `createMemo` and
   interpolate the accessor — simple interpolations always classify:

   ```tsx
   // Instead of an exotic inline expression:
   const label = createMemo(() => buildLabel(user(), locale()));
   return <span>{label()}</span>;
   ```

2. **Refresh compiler facts.** If the flagged expression is plain JSX, the facts
   may be stale — re-run the analysis cold. If the finding persists, please report
   the JSX pattern as a solid-check issue so the compiler-facts extraction can
   learn it.

## Related

- [strict-read-untracked](strict-read-untracked.md) — what the execution map feeds
