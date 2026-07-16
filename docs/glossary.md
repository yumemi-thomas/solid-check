# Canonical glossary

Use these terms in code, diagnostics, contracts, and design discussions.

| Term | Meaning |
| --- | --- |
| Reactive provenance | Accepted proof that a value or property is reactive and the binding flow by which it reached an operation. |
| Execution region | Original-source code executed with a compiler/runtime-defined role such as tracked, untracked, owned, event-driven, deferred, cleanup, effect-compute, or effect-apply. |
| Reactive operation | A read, write, call, callback invocation, primitive creation, cleanup registration, async read, or ownership operation in the Reactive IR. |
| Effect summary | An interprocedural description of a function's reactive behavior. |
| Proof obligation | A fact that must be established before a project can be certified. |
| Package contract | A static description of the reactive effects of a package's exported interface. |
| Violation | A proven breach of a Solid reactive rule. |
| Uncertifiable | An unresolved proof obligation at an explicitly unsupported or unavailable boundary. |
| Certification snapshot | A deterministic project result containing status, findings, explanations, fixes, package summaries, and metrics. |

## Naming constraints

- Do not call a heuristic a proof.
- Do not call an unresolved obligation safe.
- Do not expose TypeScript, Oxc, or solver-specific node terminology through
  the certification package.
- Use “adapter” for CLI, LSP, and ESLint integrations; none owns reactive
  analysis.
