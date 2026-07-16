#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
  echo "usage: $0 /path/to/solid-primitives" >&2
  exit 2
fi

root=$1
checker=${SOLID_CHECK_BIN:-}
if [ -z "$checker" ]; then
  echo "SOLID_CHECK_BIN must point to a built solid-check binary" >&2
  exit 2
fi
if [ -z "${SOLID_COMPILER_FACTS_BIN:-}" ]; then
  echo "SOLID_COMPILER_FACTS_BIN must point to solid-compiler-facts" >&2
  exit 2
fi

failures=$(mktemp)
trap 'rm -f "$failures"' EXIT

contract_digest() {
  find "$root/packages" -name solid-reactivity.json -print | sort | xargs shasum | shasum | awk '{print $1}'
}

pass=1
while [ "$pass" -le 12 ]; do
  before=$(contract_digest)
  : > "$failures"
  generated=0

  for package_json in "$root"/packages/*/package.json; do
    package_dir=${package_json%/package.json}
    name=$(node -e 'process.stdout.write(require(process.argv[1]).name)' "$package_json")
    version=$(node -e 'process.stdout.write(require(process.argv[1]).version)' "$package_json")
    output="$package_dir/solid-reactivity.json"
    declaration="$package_dir/dist/index.d.ts"
    implementation="$package_dir/dist/index.js"

    if "$checker" \
      --project "$package_dir/tsconfig.json" \
      --emit-contract "$output" \
      --package-name "$name" \
      --package-version "$version" \
      --declaration-artifact "$declaration" \
      --implementation-artifact "$implementation"; then
      generated=$((generated + 1))
    else
      printf '%s\n' "$name" >> "$failures"
    fi
  done

  after=$(contract_digest)
  if [ ! -s "$failures" ] && [ "$pass" -gt 1 ] && [ "$before" = "$after" ]; then
    echo "generated $generated Solid Primitives contracts to a fixed point in $pass passes"
    exit 0
  fi
  pass=$((pass + 1))
done

if [ -s "$failures" ]; then
  echo "failed package contracts:" >&2
  sed 's/^/  /' "$failures" >&2
else
  echo "package contracts did not reach a fixed point" >&2
fi
exit 1
