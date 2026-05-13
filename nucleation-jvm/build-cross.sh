#!/usr/bin/env bash
#
# build-cross.sh — cross-compile nucleation-jvm cdylibs for one or more
# JVM targets and stage them into the Gradle resources tree.
#
# Usage:
#   ./build-cross.sh                       # build all 5 targets
#   ./build-cross.sh linux-arm64           # build only linux-arm64
#   ./build-cross.sh linux-x64 linux-arm64 # build a subset
#
# Targets:
#   linux-x64     → x86_64-unknown-linux-gnu
#   linux-arm64   → aarch64-unknown-linux-gnu
#   macos-x64     → x86_64-apple-darwin
#   macos-arm64   → aarch64-apple-darwin
#   windows-x64   → x86_64-pc-windows-msvc
#
# Requirements:
#   - cargo (always)
#   - For non-host Linux/Windows targets: `cross` (https://github.com/cross-rs/cross)
#     installed via `cargo install cross --git https://github.com/cross-rs/cross`
#     `cross` uses Docker under the hood, so Docker must be running.
#   - macOS host can build macos-* targets natively. Cross-compiling
#     macos-x64 from arm64 just needs `rustup target add x86_64-apple-darwin`.
#   - Windows-from-Linux: cross handles it. Windows-from-macOS: not supported
#     here; build the Windows cdylib in CI or a Windows machine.
#
# After this script finishes, run `./gradlew jar` from the `jvm/` subdir
# and the produced JAR will contain every cdylib you built.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JVM_DIR="${SCRIPT_DIR}/jvm"
RESOURCES_DIR="${JVM_DIR}/src/main/resources/native"

# Map platform-label → (cargo target, lib filename, lib prefix)
declare -A TARGET_FOR=(
    [linux-x64]="x86_64-unknown-linux-gnu"
    [linux-arm64]="aarch64-unknown-linux-gnu"
    [macos-x64]="x86_64-apple-darwin"
    [macos-arm64]="aarch64-apple-darwin"
    [windows-x64]="x86_64-pc-windows-msvc"
)
declare -A LIBNAME_FOR=(
    [linux-x64]="libnucleation_jvm.so"
    [linux-arm64]="libnucleation_jvm.so"
    [macos-x64]="libnucleation_jvm.dylib"
    [macos-arm64]="libnucleation_jvm.dylib"
    [windows-x64]="nucleation_jvm.dll"
)

if [[ $# -eq 0 ]]; then
    PLATFORMS=(linux-x64 linux-arm64 macos-x64 macos-arm64 windows-x64)
else
    PLATFORMS=("$@")
fi

# Pick the builder: `cross` for non-host or when Docker is preferred,
# `cargo` for host-native macOS targets.
host_os="$(uname -s)"
host_arch="$(uname -m)"

is_host_macos=false
[[ "$host_os" == "Darwin" ]] && is_host_macos=true

pick_builder() {
    local platform="$1"
    case "$platform" in
        macos-arm64|macos-x64)
            if $is_host_macos; then echo cargo; else echo unsupported; fi
            ;;
        *)
            echo cross
            ;;
    esac
}

ensure_cross() {
    if ! command -v cross >/dev/null 2>&1; then
        cat >&2 <<'EOS'
Error: `cross` is not installed but is required for cross-compilation.
Install with:
    cargo install cross --git https://github.com/cross-rs/cross

`cross` uses Docker, so Docker must be running.
EOS
        exit 1
    fi
}

build_one() {
    local platform="$1"
    local target="${TARGET_FOR[$platform]:-}"
    local libname="${LIBNAME_FOR[$platform]:-}"

    if [[ -z "$target" ]]; then
        echo "Unknown platform: $platform" >&2
        echo "Supported: ${!TARGET_FOR[*]}" >&2
        exit 1
    fi

    local builder
    builder="$(pick_builder "$platform")"
    if [[ "$builder" == "unsupported" ]]; then
        echo "Skipping $platform — building this target requires a $platform host." >&2
        return 0
    fi

    echo "▶ Building $platform ($target) via $builder"

    if [[ "$builder" == "cargo" ]]; then
        # Native build (or cross-arch on the same OS family).
        rustup target add "$target" >/dev/null 2>&1 || true
        ( cd "$SCRIPT_DIR" && cargo build --release --target "$target" )
    else
        ensure_cross
        ( cd "$SCRIPT_DIR" && cross build --release --target "$target" )
    fi

    local src="${SCRIPT_DIR}/target/${target}/release/${libname}"
    if [[ ! -f "$src" ]]; then
        echo "Build succeeded but artifact not found: $src" >&2
        exit 1
    fi

    local dest_dir="${RESOURCES_DIR}/${platform}"
    mkdir -p "$dest_dir"
    cp -f "$src" "${dest_dir}/${libname}"
    local sz
    sz=$(wc -c <"${dest_dir}/${libname}" | tr -d ' ')
    echo "  → staged ${dest_dir}/${libname} (${sz} bytes)"
}

echo "Cross-build targets: ${PLATFORMS[*]}"
for p in "${PLATFORMS[@]}"; do
    build_one "$p"
done

echo
echo "Done. Run \`./gradlew jar\` from $JVM_DIR to assemble the fat JAR."
