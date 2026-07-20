# DOM Expressions compiler subset

This directory contains only the upstream DOM Expressions JSX compiler needed
by `solid-checker`. The DOM runtime, Babel plugin, alternate JSX frontends,
publishing configuration, and upstream repository tooling are intentionally
excluded.

The retained Rust crate provides JSX execution semantics in-process and through
the `solid-compiler-facts` sidecar. Its small Node wrapper exists solely for the
differential conformance check.

See the repository's `THIRD_PARTY_NOTICES.md` for provenance and
`docs/monorepo.md` for the selective update procedure.
