#!/bin/sh
set -eu

revision=46e038a1554cdac58b0a2f04cde735f010508061
if [ -n "${SOLID_PRIMITIVES_CORPUS:-}" ]; then
  root=$SOLID_PRIMITIVES_CORPUS
else
  root=$(mktemp -d "${TMPDIR:-/tmp}/solid-primitives-corpus.XXXXXX")
  trap 'rm -rf "$root"' EXIT
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
