#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors
#
# Puts midi.oxp, the MIDI plugin this one needs (oxiguitar.midi), at
# target/midi/midi.oxp for the system this runs on, and prints its path.
# The tests (tests/host.rs) look for it there, or at $OXIGUITAR_MIDI_OXP.
#
#   scripts/midi.sh            the latest release of mateusrdp/oxiguitar-midi:
#                              its zip for this system, checked against the
#                              release's SHA256SUMS (a test downloads our own
#                              release; the plugin itself never downloads)
#   scripts/midi.sh REF        built from that tag or branch of the repository,
#                              for CI before a release has the fix it needs
#
# Needs curl, python3 (and Rust for a REF).
set -euo pipefail
trap 'echo "::error::midi.sh line $LINENO: $BASH_COMMAND" >&2' ERR

ref="${1:-}"
here="$(cd "$(dirname "$0")/.." && pwd)"
dest="$here/target/midi"
mkdir -p "$dest"
rm -f "$dest/midi.oxp"

case "$(uname -s)" in
    Linux) system=linux ;;
    Darwin) system=macos ;;
    *) system=windows ;;
esac
arch="$(uname -m)"
case "$system-$arch" in
    macos-*) target=macos-universal ;;
    *-aarch64 | *-arm64) target="$system-arm64"; [[ "$system" == linux ]] && target=linux-aarch64 ;;
    *) target="$system-x86_64" ;;
esac

if [[ -n "$ref" ]]; then
    work="$(mktemp -d)"
    trap 'rm -rf "$work"' EXIT
    git clone --quiet --depth 1 --branch "$ref" https://github.com/mateusrdp/oxiguitar-midi "$work/midi"
    # scripts/test-all.py --ignore-rust-version sets OXIGUITAR_IGNORE_RUST_VERSION.
    flags=()
    [[ -n "${OXIGUITAR_IGNORE_RUST_VERSION:-}" ]] && flags+=(--ignore-rust-version)
    (cd "$work/midi" && cargo rustc --locked --release --features oxp --crate-type cdylib ${flags[@]+"${flags[@]}"} > /dev/null)
    case "$system" in
        linux) cp "$work/midi/target/release/liboxiguitar_midi_plugin.so" "$dest/midi.oxp" ;;
        macos) cp "$work/midi/target/release/liboxiguitar_midi_plugin.dylib" "$dest/midi.oxp" ;;
        *) cp "$work/midi/target/release/oxiguitar_midi_plugin.dll" "$dest/midi.oxp" ;;
    esac
else
    # Every release is a pre-release until v1.0, and releases/latest
    # answers 404 for those: take the newest published one.
    releases="$(curl -fsSL --retry 5 "https://api.github.com/repos/mateusrdp/oxiguitar-midi/releases?per_page=20")"
    tag="$(python3 -c '
import json, re, sys
tags = [r["tag_name"] for r in json.load(sys.stdin) if not r.get("draft")]
print(max(tags, key=lambda t: tuple(int(n) for n in re.findall(r"\d+", t)[:3])))' <<< "$releases")"
    version="${tag#v}"
    zip="oxiguitar-midi-$version-$target.zip"
    base="https://github.com/mateusrdp/oxiguitar-midi/releases/download/$tag"
    curl -fsSL --retry 5 -o "$dest/$zip" "$base/$zip"
    curl -fsSL --retry 5 -o "$dest/SHA256SUMS" "$base/SHA256SUMS"
    want="$(awk -v n="$zip" '$2 == n || $2 == "*" n { print $1 }' "$dest/SHA256SUMS")"
    have="$(python3 -c 'import hashlib,sys; print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$dest/$zip")"
    [[ -n "$want" && "$want" == "$have" ]] || { echo "$zip does not match SHA256SUMS" >&2; exit 1; }
    python3 - "$dest/$zip" "$dest/midi.oxp" <<'PY'
import sys, zipfile
with zipfile.ZipFile(sys.argv[1]) as z:
    # The zip holds one plugin file, named midi-VERSION-SYSTEM-ARCH.oxp.
    (name,) = [n for n in z.namelist() if n.endswith(".oxp")]
    open(sys.argv[2], "wb").write(z.read(name))
PY
    echo "oxiguitar-midi $version" >&2
fi
echo "$dest/midi.oxp"
