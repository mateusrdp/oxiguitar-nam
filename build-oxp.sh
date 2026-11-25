#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors
#
# Builds the plugin's release file for the system this runs on:
#
#   dist/nam-VERSION-SYSTEM-ARCH.oxp
#   dist/nam-VERSION-linux-x86_64-v3.oxp   (Linux on x86_64 only)
#
# such as dist/nam-0.1.0-linux-x86_64.oxp, named the way OxiGuitar's plugins
# are (docs/plugins/template in github.com/mateusrdp/oxiguitar). On macOS it
# builds both processors and joins them into one universal file (needs both
# Rust targets: rustup target add aarch64-apple-darwin x86_64-apple-darwin).
# The Apple Silicon half plays through NeuralAmpModeler-rs, the Intel half
# through nam-rs (build.rs). On Windows, run it from Git Bash.
# .github/workflows/release.yml runs it on each system.

set -euo pipefail

# The plugin's file name, and the crate's library name (the package's name
# with _ for -).
NAME=nam
LIBRARY=oxiguitar_nam_plugin

cd "$(dirname "$0")"
version="$(cargo pkgid)"
version="${version##*[#@]}"
host="$(rustc -vV | sed -n 's/^host: //p')"
mkdir -p dist

# pack LIBRARY OUT [MANIFEST]: the plugin file OUT (mateusrdp/oxiguitar#347),
# an uncompressed tar archive holding MANIFEST (default manifest.json) as
# manifest.json first, the library second as plugin.so, plugin.dll or
# plugin.dylib, then the ui and assets folders beside the manifest, if
# there are any. COPYFILE_DISABLE keeps macOS's tar from adding ._ files.
pack() {
    local ext="${1##*.}" manifest="${3:-manifest.json}" stage folder
    local more=()
    stage="$(mktemp -d)"
    cp "$manifest" "$stage/manifest.json"
    cp "$1" "$stage/plugin.$ext"
    for folder in ui assets; do
        if [[ -d "$(dirname "$manifest")/$folder" ]]; then
            cp -R "$(dirname "$manifest")/$folder" "$stage/"
            more+=("$folder")
        fi
    done
    (cd "$stage" && COPYFILE_DISABLE=1 tar --format=ustar -cf - manifest.json "plugin.$ext" ${more[@]+"${more[@]}"}) > "$2"
    rm -rf "$stage"
}

build() {
    cargo rustc --locked --release --features oxp --crate-type cdylib "$@"
}

case "$host" in
    *apple*)
        out="dist/$NAME-$version-macos-universal.oxp"
        for target in aarch64-apple-darwin x86_64-apple-darwin; do
            build --target "$target"
        done
        lipo -create -output "target/$(basename "${out%.oxp}").dylib" \
            "target/aarch64-apple-darwin/release/lib$LIBRARY.dylib" \
            "target/x86_64-apple-darwin/release/lib$LIBRARY.dylib"
        pack "target/$(basename "${out%.oxp}").dylib" "$out"
        ;;
    *windows*)
        arch="${host%%-*}"
        [[ "$arch" == aarch64 ]] && arch=arm64
        out="dist/$NAME-$version-windows-$arch.oxp"
        build
        pack "target/release/$LIBRARY.dll" "$out"
        ;;
    *)
        out="dist/$NAME-$version-linux-${host%%-*}.oxp"
        build
        pack "target/release/lib$LIBRARY.so" "$out"
        # On aarch64 the plain build already plays through NeuralAmpModeler-rs
        # (build.rs). Linux on x86_64 also gets the build for processors with
        # AVX2 and FMA (x86-64-v3): NeuralAmpModeler-rs in place of nam-rs,
        # with its own manifest. OxiGuitar takes it over nam.oxp where the processor has
        # those features (manifest-x86-64-v3.json). The release workflow
        # names it nam-x86-64-v3.oxp from the -v3 at the end of the file name.
        if [[ "${host%%-*}" == x86_64 ]]; then
            v3="dist/$NAME-$version-linux-x86_64-v3.oxp"
            RUSTFLAGS="${RUSTFLAGS:-} -C target-cpu=x86-64-v3" \
                cargo rustc --locked --release --features oxp,x86-64-v3 --crate-type cdylib \
                --target-dir target/v3
            pack "target/v3/release/lib$LIBRARY.so" "$v3" manifest-x86-64-v3.json
            out="$out $v3"
        fi
        ;;
esac
echo "$out"
