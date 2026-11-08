# oxiguitar-nam

The NAM plugin for [OxiGuitar](https://github.com/mateusrdp/oxiguitar), a free, open-source guitar tablature viewer, editor and player: [Neural Amp Modeler](https://www.neuralampmodeler.com/) amp models and cabinet impulse responses on guitar and bass tracks, set in the track's Amp (NAM) section. Models (`.nam`) and impulse responses (`.wav`) are files you put in the `nam` folder; four cabinets are built in. The plugin needs the [MIDI plugin](https://github.com/mateusrdp/oxiguitar-midi) (`oxiguitar.midi`), which plays the track the amp is applied to.

On Linux x86_64 the release also holds a second file, `nam-x86-64-v3.oxp`, built for processors with AVX2 and FMA (about ten times faster, with NeuralAmpModeler-rs in place of nam-rs); OxiGuitar picks it over `nam.oxp` where the processor has those features.

## Install

Every [release](https://github.com/mateusrdp/oxiguitar-nam/releases) holds a zip per system (Linux x86_64 and ARM64, Windows x86_64 and ARM64, macOS), a `.deb` and an `.rpm` for Linux, a per-user Windows installer (no administrator needed), a macOS `.pkg`, and a `SHA256SUMS` file. The OxiGuitar manual's [install page](https://mateusrdp.github.io/oxiguitar/installing.html) says which one to use and how to keep the plugin updated through your package manager. The plugin needs OxiGuitar 0.14 or later (plugin interface 1.5; its 0.2 releases work with 0.11 and later) and the MIDI plugin. Its Input and Output work while the song plays.

## Build and test

Rust 1.99 or later from [rustup](https://rustup.rs). The plugin SDK comes from OxiGuitar's repository by git tag (`v0.14.0`); nothing of OxiGuitar's is on crates.io.

```sh
cargo test          # unit tests, and the contract suite on the built .oxp file
./build-oxp.sh      # dist/nam-VERSION-SYSTEM-ARCH.oxp
```

The tests need the MIDI plugin's file. `scripts/midi.sh` puts the latest release's at `target/midi/midi.oxp` (the release's zip, checked against its `SHA256SUMS`; `scripts/midi.sh REF` builds a tag or branch instead), or point `OXIGUITAR_MIDI_OXP` at one:

```sh
scripts/midi.sh
cargo test
```

The build for x86-64-v3 processors needs Rust 1.99 and `RUSTFLAGS="-C target-cpu=x86-64-v3"`; `build-oxp.sh` makes it on Linux x86_64, and CI's "NAM for x86-64-v3" job tests it.

## History

This plugin lived in the [OxiGuitar repository](https://github.com/mateusrdp/oxiguitar) until its version 0.11.0. Its history came along; an issue number such as `mateusrdp/oxiguitar#123` in an old commit message is in that repository.

## Licence

GPL-3.0-or-later. See LICENSE. Contributions are welcome under the same terms; see [CONTRIBUTING.md](CONTRIBUTING.md).
