#!/bin/sh
set -eu

compiler_manifest=third_party/dom-expressions/packages/jsx-compiler/Cargo.toml
rust_manifest=rust/Cargo.toml
zed_manifest=packages/zed-solid-check/Cargo.toml

go test -race ./cmd/solid-typefacts ./internal/typefacts/... ./internal/wirecbor
go vet ./cmd/solid-typefacts ./internal/typefacts/... ./internal/wirecbor
test -z "$(gofmt -l cmd/solid-typefacts internal/typefacts internal/wirecbor)"

cargo +1.97 fmt --manifest-path "$rust_manifest" --all -- --check
cargo +1.97 clippy --manifest-path "$rust_manifest" --workspace --all-targets

mkdir -p bin
go build -o bin/solid-typefacts ./cmd/solid-typefacts
SOLID_TYPEFACTS_BIN="$PWD/bin/solid-typefacts" \
  cargo +1.97 test --manifest-path "$rust_manifest" --workspace

cargo +1.97 fmt --manifest-path "$compiler_manifest" -- --check
cargo +1.97 clippy \
  --manifest-path "$compiler_manifest" \
  --no-default-features \
  --features sidecar \
  --all-targets
cargo +1.97 test \
  --manifest-path "$compiler_manifest" \
  --no-default-features \
  --features sidecar

npm ci --ignore-scripts --prefix packages/cli
npm test --prefix packages/cli

cargo +1.97 fmt --manifest-path "$zed_manifest" -- --check
cargo +1.97 test --manifest-path "$zed_manifest"

sh -n scripts/*.sh
jq empty schema/*.json pkg/contracts/bundled/*.json
