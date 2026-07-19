# Solid Check

Solid Check certifies the reactivity and asynchronous behavior of Solid
TypeScript projects without coupling its analysis to one compiler backend.

## Language

**Type Facts**:
Compiler-independent semantic facts about a configured TypeScript project.
_Avoid_: Compiler facts, checker data

**Async function fact**:
A semantic summary of a function-like declaration or expression, including whether it can return asynchronously and which calls are dominated by an await.
_Avoid_: Async scan result, async metadata

**Reference index**:
The generation-scoped mapping from durable symbol identities to their source reference locations.
_Avoid_: Reference cache, usage map

**Type Facts session**:
A retained analysis lifetime for one configured TypeScript project, carrying its current generation and acknowledged demand state across requests.
_Avoid_: Lifecycle responder, retained protocol state

**Semantic lookup**:
The project-wide query surface rule discovery asks for semantic answers — the entity or symbol at or containing a location, the function a symbol names, whether an owner is rendered under a Loading boundary — instead of scanning fact tables.
_Avoid_: Index helpers, fact-table scan, range-query module
