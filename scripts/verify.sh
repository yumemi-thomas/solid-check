#!/bin/sh
set -eu

manifest=third_party/dom-expressions/packages/jsx-compiler/Cargo.toml
zed_manifest=packages/zed-solid-check/Cargo.toml

go test -race ./...
go vet ./...
test -z "$(gofmt -l cmd internal pkg)"
node --check packages/eslint-plugin-solid-check/index.cjs
node --test packages/eslint-plugin-solid-check/test/*.test.cjs
npm ci --ignore-scripts --prefix packages/cli
npm test --prefix packages/cli

cargo +1.93 fmt --manifest-path "$manifest" -- --check
cargo +1.93 clippy \
  --manifest-path "$manifest" \
  --no-default-features \
  --features sidecar \
  --all-targets \
  -- -D warnings
cargo +1.93 test \
  --manifest-path "$manifest" \
  --no-default-features \
  --features sidecar

cargo fmt --manifest-path "$zed_manifest" -- --check
cargo test --manifest-path "$zed_manifest"

RUSTUP_TOOLCHAIN=1.93 pnpm --dir third_party/dom-expressions \
  --filter @dom-expressions/jsx-compiler run build:debug
node scripts/compiler-conformance.mjs \
  third_party/dom-expressions/packages/jsx-compiler \
  third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts
SOLID_COMPILER_FACTS_BIN="$PWD/third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts" \
  node scripts/eslint-migration-conformance.mjs
SOLID_COMPILER_FACTS_BIN="$PWD/third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts" \
  node scripts/oxlint-conformance.mjs
SOLID_COMPILER_FACTS_BIN="$PWD/third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts" \
  node scripts/oxlint-fix-conformance.mjs
node scripts/package-cli-conformance.mjs
node scripts/zed-config-conformance.mjs

sh -n scripts/*.sh
jq empty schema/*.json pkg/contracts/bundled/*.json \
  .zed/settings.json \
  .zed/tasks.json \
  examples/solid-2-dev-app/.oxlintrc.editor.json \
  examples/solid-2-dev-app/.zed/settings.json \
  examples/solid-2-dev-app/.zed/tasks.json
