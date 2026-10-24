#!/usr/bin/env bash
set -euo pipefail

# Cross-compilação Linux -> Windows 10/11 x86_64. O resultado não exige
# DLLs distribuídas com o programa: usa apenas as bibliotecas do Windows.
app_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
toolchain_version="20261006"
archive_name="llvm-mingw-${toolchain_version}-ucrt-ubuntu-22.04-x86_64"
archive_sha256="5f9c6ed95b2d4bdb2869a488c5fd5857fbdabcf288a0aa3eb1da43f6a08d8ab4"
toolchain_cache="${EMI_TOOLCHAIN_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/emi-filter-toolchains}"
toolchain_dir="$toolchain_cache/$archive_name"
compat_dir="$toolchain_cache/llvm-mingw-rust-compat"
build_dir="${EMI_WINDOWS_TARGET_DIR:-/tmp/emi-rust-win-target}"
cargo_bin="${CARGO:-$HOME/.cargo/bin/cargo}"
rustup_bin="${RUSTUP:-$HOME/.cargo/bin/rustup}"

if [[ ! -x "$toolchain_dir/bin/x86_64-w64-mingw32-clang" ]]; then
    mkdir -p "$toolchain_cache"
    archive_file="$(mktemp /tmp/emi-llvm-mingw-XXXXXXXX.tar.xz)"
    trap 'if [[ -f "$archive_file" ]]; then rm -- "$archive_file"; fi' EXIT
    curl --location --fail --show-error \
        "https://github.com/mstorsjo/llvm-mingw/releases/download/$toolchain_version/$archive_name.tar.xz" \
        --output "$archive_file"
    printf '%s  %s\n' "$archive_sha256" "$archive_file" | sha256sum --check --status
    tar -xJf "$archive_file" -C "$toolchain_cache"
fi

# O alvo GNU do Rust pede os nomes libgcc/libgcc_eh. Usamos os equivalentes
# estáticos do LLVM, sem copiar runtimes de terceiros para junto do executável.
mkdir -p "$compat_dir"
ln -sfn "$toolchain_dir/lib/clang/23/lib/windows/libclang_rt.builtins-x86_64.a" "$compat_dir/libgcc.a"
ln -sfn "$toolchain_dir/x86_64-w64-mingw32/lib/libunwind.a" "$compat_dir/libgcc_eh.a"
"$rustup_bin" target add x86_64-pc-windows-gnu

export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="$toolchain_dir/bin/x86_64-w64-mingw32-clang"
export PATH="$toolchain_dir/bin:$PATH"
export CC_x86_64_pc_windows_gnu="$toolchain_dir/bin/x86_64-w64-mingw32-clang"
export CXX_x86_64_pc_windows_gnu="$toolchain_dir/bin/x86_64-w64-mingw32-clang++"
export AR_x86_64_pc_windows_gnu="$toolchain_dir/bin/llvm-ar"
export CARGO_TARGET_DIR="$build_dir"
export CARGO_ENCODED_RUSTFLAGS="-Clink-self-contained=no"$'\x1f'"-Ctarget-feature=+crt-static"$'\x1f'"-Lnative=$compat_dir"$'\x1f'"--remap-path-prefix=$HOME=/build"

"$cargo_bin" build --manifest-path "$app_dir/Cargo.toml" --locked \
    --release --target x86_64-pc-windows-gnu
app_version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$app_dir/Cargo.toml" | head -n 1)"
mkdir -p "$app_dir/dist"
cp "$build_dir/x86_64-pc-windows-gnu/release/emi-filter.exe" \
    "$app_dir/dist/emi-filter-v${app_version}-windows-x86_64.exe"
printf 'Executável: %s\n' "$app_dir/dist/emi-filter-v${app_version}-windows-x86_64.exe"
