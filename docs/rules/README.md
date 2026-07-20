# Rules

solid-checker certifies the reactive correctness of Solid 2.0 projects. Every finding
carries a stable diagnostic code (`SCxxxx`), a rule name, a message describing what
went wrong, and a hint describing how to fix it. This index links to the full
documentation page for each rule.

Findings come in two kinds:

- **violation** — the analyzer proved the code misbehaves at runtime.
- **uncertifiable** — the analyzer could not prove the code correct; the page for
  each `SC9xxx` rule explains how to make the code provable.

## Tracking & component semantics

| Code | Rule | Severity |
| --- | --- | --- |
| SC1001 | [strict-read-untracked](strict-read-untracked.md) | warning |
| SC1002 | [reactive-read-after-await](reactive-read-after-await.md) | error |
| SC1003 | [component-props-destructure](component-props-destructure.md) | error |
| SC1004 | [component-returns-conditionally](component-returns-conditionally.md) | error |

## Writes & actions

| Code | Rule | Severity |
| --- | --- | --- |
| SC2001 | [reactive-write-in-owned-scope](reactive-write-in-owned-scope.md) | error |
| SC2002 | [action-called-in-owned-scope](action-called-in-owned-scope.md) | error |

## Leaf owners & cleanup

| Code | Rule | Severity |
| --- | --- | --- |
| SC3001 | [cleanup-in-forbidden-scope](cleanup-in-forbidden-scope.md) | error |
| SC3002 | [primitive-in-leaf-owner](primitive-in-leaf-owner.md) | error |
| SC3003 | [flush-in-forbidden-scope](flush-in-forbidden-scope.md) | error |
| SC3004 | [invalid-cleanup-return](invalid-cleanup-return.md) | error |
| SC3005 | [settled-cleanup-unowned](settled-cleanup-unowned.md) | error |

## Ownership

| Code | Rule | Severity |
| --- | --- | --- |
| SC4001 | [no-owner-effect](no-owner-effect.md) | warning |
| SC4002 | [no-owner-cleanup](no-owner-cleanup.md) | warning |
| SC4003 | [no-owner-boundary](no-owner-boundary.md) | warning |

## Async

| Code | Rule | Severity |
| --- | --- | --- |
| SC5001 | [pending-async-untracked-read](pending-async-untracked-read.md) | error |
| SC5002 | [pending-async-forbidden-scope](pending-async-forbidden-scope.md) | warning |
| SC5003 | [async-outside-loading-boundary](async-outside-loading-boundary.md) | warning |

## Directives

| Code | Rule | Severity |
| --- | --- | --- |
| SC6001 | [primitive-in-directive-application](primitive-in-directive-application.md) | error |

## API shapes

| Code | Rule | Severity |
| --- | --- | --- |
| SC7001 | [missing-effect-function](missing-effect-function.md) | error |
| SC7002 | [sync-node-received-async](sync-node-received-async.md) | error |
| SC7003 | [invalid-refresh-target](invalid-refresh-target.md) | error |
| SC7003 | [invalid-affects-target](invalid-affects-target.md) | error |
| SC7004 | [affects-keys-on-accessor](affects-keys-on-accessor.md) | error |

## Uncertifiable (analysis limits)

| Code | Rule | Severity |
| --- | --- | --- |
| SC9001 | [package-contract-export-missing](package-contract-export-missing.md) | error |
| SC9002 | [cleanup-return-unresolved](cleanup-return-unresolved.md) | error |
| SC9003 | [refresh-target-unresolved](refresh-target-unresolved.md) | error |
| SC9003 | [affects-target-unresolved](affects-target-unresolved.md) | error |
| SC9004 | [execution-map-incomplete](execution-map-incomplete.md) | error |
| SC9005 | [package-contract-missing](package-contract-missing.md) | error |
