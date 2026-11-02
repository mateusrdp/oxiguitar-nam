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
# On Windows, run it from Git Bash. .github/workflows/release.yml runs it on
# each system.

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

build() {
    cargo rustc --locked --release --features oxp --crate-type cdylib "$@"
}

case "$host" in
    *apple*)
        out="dist/$NAME-$version-macos-universal.oxp"
        for target in aarch64-apple-darwin x86_64-apple-darwin; do
            build --target "$target"
        done
        lipo -create -output "$out" \
            "target/aarch64-apple-darwin/release/lib$LIBRARY.dylib" \
            "target/x86_64-apple-darwin/release/lib$LIBRARY.dylib"
        ;;
    *windows*)
        arch="${host%%-*}"
        [[ "$arch" == aarch64 ]] && arch=arm64
        out="dist/$NAME-$version-windows-$arch.oxp"
        build
        cp "target/release/$LIBRARY.dll" "$out"
        ;;
    *)
        out="dist/$NAME-$version-linux-${host%%-*}.oxp"
        build
        cp "target/release/lib$LIBRARY.so" "$out"
        # Linux on x86_64 also gets the build for processors with AVX2 and FMA
        # (x86-64-v3): NeuralAmpModeler-rs in place of nam-rs, with its own
        # manifest. OxiGuitar takes it over nam.oxp where the processor has
        # those features (manifest-x86-64-v3.json). The release workflow
        # names it nam-x86-64-v3.oxp from the -v3 at the end of the file name.
        if [[ "${host%%-*}" == x86_64 ]]; then
            v3="dist/$NAME-$version-linux-x86_64-v3.oxp"
            RUSTFLAGS="${RUSTFLAGS:-} -C target-cpu=x86-64-v3" \
                cargo rustc --locked --release --features oxp,x86-64-v3 --crate-type cdylib \
                --target-dir target/v3
            cp "target/v3/release/lib$LIBRARY.so" "$v3"
            out="$out $v3"
        fi
        ;;
esac
echo "$out"
