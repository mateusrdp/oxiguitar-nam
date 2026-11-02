# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
