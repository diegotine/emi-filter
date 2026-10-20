#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")"
task_cargo="${CARGO:-cargo}"
if [[ -n "${CARGO_ENCODED_RUSTFLAGS:-}" ]]; then
    export CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS}"$'\x1f'"--remap-path-prefix=$HOME=/build"
else
    export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$HOME=/build"
fi
"$task_cargo" build --release --locked
task_target="${CARGO_TARGET_DIR:-target}"
mkdir -p dist
install -m 755 "$task_target/release/emi-filter" dist/emi-filter-linux-x86_64
printf 'Executável: %s/dist/emi-filter-linux-x86_64\n' "$PWD"
