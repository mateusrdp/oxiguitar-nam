# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
