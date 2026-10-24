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
task_version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
mkdir -p dist
install -m 755 "$task_target/release/emi-filter" "dist/emi-filter-v${task_version}-linux-x86_64"
printf 'Executável: %s/dist/emi-filter-v%s-linux-x86_64\n' "$PWD" "$task_version"
