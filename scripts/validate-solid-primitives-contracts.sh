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

validated=0
for package_json in "$root"/packages/*/package.json; do
  package_dir=${package_json%/package.json}
  contract="$package_dir/solid-reactivity.json"
  node -e '
    const manifest = require(process.argv[1]);
    if (!manifest.files?.includes("solid-reactivity.json")) process.exit(1);
  ' "$package_json" || {
    echo "package does not publish solid-reactivity.json: $package_json" >&2
    exit 1
  }
  "$checker" --validate-contract "$contract"
  validated=$((validated + 1))
done

echo "validated $validated published Solid Primitives contracts"
