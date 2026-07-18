#!/bin/sh
set -eu

revision=46e038a1554cdac58b0a2f04cde735f010508061
owned_root=false
go_contracts=
cleanup() {
  if [ "$owned_root" = true ]; then rm -rf "$root"; fi
  if [ -n "$go_contracts" ]; then rm -rf "$go_contracts"; fi
}
trap cleanup EXIT
if [ -n "${SOLID_PRIMITIVES_CORPUS:-}" ]; then
  root=$SOLID_PRIMITIVES_CORPUS
else
  root=$(mktemp -d "${TMPDIR:-/tmp}/solid-primitives-corpus.XXXXXX")
  owned_root=true
fi

if [ ! -d "$root/.git" ]; then
  git clone https://github.com/solidjs-community/solid-primitives.git "$root"
fi

if [ -n "$(git -C "$root" status --porcelain)" ]; then
  echo "Solid Primitives corpus checkout has local changes: $root" >&2
  exit 1
fi

git -C "$root" fetch origin next
git -C "$root" checkout --detach "$revision"
pnpm --dir "$root" install --frozen-lockfile
pnpm --dir "$root" build
node scripts/prepare-solid-primitives-corpus.mjs "$root"

SOLID_CHECK_BIN=$(pwd)/bin/solid-check \
SOLID_COMPILER_FACTS_BIN=$(pwd)/third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts \
  scripts/generate-solid-primitives-contracts.sh "$root"
SOLID_CHECK_BIN=$(pwd)/bin/solid-check \
  scripts/validate-solid-primitives-contracts.sh "$root"

if [ -n "${SOLID_CHECK_RUST_BIN:-}" ]; then
  go_contracts=$(mktemp -d "${TMPDIR:-/tmp}/solid-primitives-go-contracts.XXXXXX")
  find "$root/packages" -name solid-reactivity.json -print | while IFS= read -r contract; do
    relative=${contract#"$root/"}
    mkdir -p "$go_contracts/${relative%/*}"
    cp "$contract" "$go_contracts/$relative"
  done

  SOLID_CHECK_BIN=$SOLID_CHECK_RUST_BIN \
    scripts/generate-solid-primitives-contracts.sh "$root"
  SOLID_CHECK_BIN=$SOLID_CHECK_RUST_BIN \
    scripts/validate-solid-primitives-contracts.sh "$root"

  compared=0
  find "$root/packages" -name solid-reactivity.json -print | sort | while IFS= read -r contract; do
    relative=${contract#"$root/"}
    cmp "$go_contracts/$relative" "$contract" || {
      echo "Rust contract differs from Go: $relative" >&2
      exit 1
    }
    compared=$((compared + 1))
  done
  echo "Rust contracts are byte-identical to the Go fixed point"
fi
