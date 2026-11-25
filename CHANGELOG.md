# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- **The plugin file is a tar archive** (mateusrdp/oxiguitar#347): `build-oxp.sh` packs `manifest.json` first and the library second (`plugin.so`, `plugin.dll` or `plugin.dylib`), as OxiGuitar 0.23 and later read plugin files, which older OxiGuitars can't load. The SDK is at OxiGuitar v0.23.0 (interface 1.16); the host test names no display system (`NO_DISPLAY`), as the SDK now has it. The x86-64-v3 build is packed with `manifest-x86-64-v3.json`. `scripts/midi.sh` packs the MIDI plugin it builds as an archive too, and refuses a release of the format before; the tests take MIDI 0.2.6 (`MIDI_REF`), since 0.2.4 misses this SDK's real-time check on its notes' start. `scripts/test-all.py` 1.1.0, as in every repository.

### Added

- **Every test with one command** (mateusrdp/oxiguitar#273): `scripts/test-all.py` builds and runs all of this repository's tests, and tests the plugin against the newest OxiGuitar release, with a progress bar, a summary to paste into an issue and a log to diagnose from. The manual has a Features page naming the tests of every feature (CI checks it) and a Testing page.

### Changed

- **CI outlasts a slow Ubuntu mirror**: it installs packages with `scripts/apt-install.sh`, the same file as in every OxiGuitar repository. apt is stopped only when its downloads have not grown for 2 minutes, then tried again on the next mirror; while they grow, it may take as long as it needs (24 Nov 2026: fixed 5-minute tries failed jobs on mirrors at 80 to 185 kB/s).
- **The manual is the plugin's own** (OxiGuitar #325): `docs/manual`, published at https://mateusrdp.github.io/oxiguitar-nam/. The advice dialogs link to its page on where models go instead of a page of OxiGuitar's.
- **The manual explains the ideas behind the plugin** (its overview, with pictures), shows how to use it with screenshots, says in Architecture and strategy what it can't do yet, what is left out on purpose and where help is wanted, and has a Credits page listing every project it uses, with links and thanks. The README has a plain-words introduction and a Credits section.

### Fixed

- `scripts/midi.sh` with no ref fetches the newest MIDI release again: it asked for `releases/latest`, which answers 404 while every release is a pre-release, and looked in the zip for `midi.oxp`, which is named `midi-VERSION-SYSTEM-ARCH.oxp`.

## [0.4.2] - 2026-11-17

### Changed

- The `nam` folder is asked for by its name (`host.folder`, interface 1.8), as OxiGuitar no longer knows it (the core owns nothing of a plugin, OxiGuitar's ADR 0012; mateusrdp/oxiguitar#308 retires the folder names it knew). Nothing changes for users.

## [0.4.1] - 2026-11-16

### Changed

- **NeuralAmpModeler-rs plays wherever it builds**, now that its 2.0
  runs on ARM64: `nam.oxp` on Linux ARM64 and the Apple Silicon half of
  the macOS file use it too, as `nam-x86-64-v3.oxp` does on Linux
  x86_64, about ten times faster than nam-rs (a cabinet sounds 64 frames,
  1.3 ms at 48 kHz, late there, and the other tracks wait for it). nam-rs
  stays where NeuralAmpModeler-rs can't build: Windows, Intel Macs and
  `nam.oxp` on Linux x86_64 (processors without AVX2 and FMA). The
  NeuralAmpModeler-rs builds read the model files with it too (the ARM64
  ones hold no nam-rs at all), and whether a capture holds a cabinet is
  read the same way in every build, as nam-rs reads it.
### Added

- **A Flatpak extension**, `io.github.mateusrdp.OxiGuitar.Plugin.nam`, in
  the release and in OxiGuitar's Flatpak repository, for OxiGuitar
  installed as a Flatpak. 0.4.0 was released without it: its release was
  built just before the packaging gained Flatpak extensions.

## [0.4.0] - 2026-11-16

### Changed

- **`nam-x86-64-v3.oxp` uses NeuralAmpModeler-rs 2.0** (from 1.0.1). Its
  cabinet adapter now takes pieces of any size at a constant latency of one
  partition (64 frames), so the plugin hands it the track's sound as it
  comes and takes the latency from the adapter: a cabinet still sounds
  1.3 ms late at 48 kHz, not 2.7 ms. The amp models sound as before.

- **The amp says where it goes in a track's chain** (mateusrdp/oxiguitar#293):
  `"order": 200` on the effect, plugin interface 1.8. OxiGuitar no longer
  keeps a list of plugin ids for the default order of a track's effects;
  each effect's manifest says it (pedals 100, amp models 200, equalizers
  300), and the Track tab's arrows still change it for a track. The plugin
  now needs OxiGuitar 0.20.0 or later (interface 1.8), whose tag the SDK
  comes from.

## [0.3.0] - 2026-11-08

### Added

- **Knowledge graph analysis** (Understand Anything): the `.ua/knowledge-graph.json` captures the project's architecture — 36 nodes, 52 edges across 7 layers and an 8-step guided tour — for navigation and documentation. Generated by the `/understand` command.

### Changed

- **Input and Output follow the track's section while the song plays**
  (mateusrdp/oxiguitar#232): moving either slider changes the sound at
  once, the level gliding to the new one in about 10 ms so nothing clicks.
  Turning the amp on or off, or another model or cabinet, loads files, so
  it is heard when the song next starts, as before. Needs OxiGuitar 0.14
  (plugin interface 1.5): the SDK is the tag v0.14.0.

## [0.2.0] - 2026-11-04

### Added

- While a track's amp is on, the section Amp (NAM) says under the model how to feed it a clean guitar: set the track's Sound (the MIDI plugin's section) to "Like clean guitars" (mateusrdp/oxiguitar#152). A NAM capture expects a clean guitar; a track whose General MIDI program is overdriven or distortion would otherwise go into the amp already distorted.

### Changed

- The release and publish workflows call oxiguitar's at v0.12.1, whose install check passes on Fedora's rpm (which now refuses unsigned packages without `--nosignature`).

## [0.1.0] - 2026-11-02

First release in a repository of its own (mateusrdp/oxiguitar#165). The
plugin (`oxiguitar.nam`, file `nam.oxp`) is the one of OxiGuitar 0.11.0, with its
history.

### Added

- `build-oxp.sh`, and a contract test that builds the `.oxp` file and checks
  it through OxiGuitar's plugin host.
- `release.yml`: test builds, and on a `v*` tag a draft release with the
  packages for each system; `publish.yml`.

### Changed

- The plugin SDK is a git dependency on the `v0.11.0` tag of
  mateusrdp/oxiguitar (plugin interface 1.4). The plugin sits at the
  repository's root.
