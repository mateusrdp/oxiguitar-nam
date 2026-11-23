# Credits

The NAM plugin is a thin layer over other people's remarkable work. Thank you to every project below, and especially to Steven Atkinson and the Neural Amp Modeler community for making amp capture open to everyone.

## Code we use

| Project | Authors | What it does here | Licence |
| --- | --- | --- | --- |
| [OxiGuitar's plugin SDK](https://github.com/mateusrdp/oxiguitar) (`oxiguitar-plugin`) | Mateus Rocha de Paula and the OxiGuitar contributors | The plugin interface | MIT OR Apache-2.0 |
| [NeuralAmpModeler-rs](https://github.com/fabiohl/NeuralAmpModeler-rs) | Fábio Henrique de Lima Silva | Runs NAM models on ARM64 Linux, Apple Silicon and x86-64-v3 Linux | Apache-2.0 |
| [nam-rs](https://github.com/OpenSauce/nam-rs) | OpenSauce and contributors | Runs NAM models everywhere else | MIT |
| [RustFFT](https://github.com/ejmahler/RustFFT) | Elliott Mahler and contributors | The cabinet convolution | MIT OR Apache-2.0 |
| [Rust](https://www.rust-lang.org/) and its standard library | The Rust Project | Everything | MIT OR Apache-2.0 |

Both engines follow [Neural Amp Modeler](https://github.com/sdatkinson/neural-amp-modeler) and [NeuralAmpModelerCore](https://github.com/sdatkinson/NeuralAmpModelerCore) by Steven Atkinson (MIT), the reference that defines what a `.nam` file means, and are checked against it.

## Sounds and data

| Asset | Authors | Use | Licence |
| --- | --- | --- | --- |
| Four cabinet impulse responses of the BestPlugins Mega Pack 2, as [Guitarix](https://github.com/brummer10/guitarix) ships them | David Fau Casquel | The built-in cabinets (`irs/`) | GPL-2.0-or-later |
| Two test models from [NeuralAmpModelerCore](https://github.com/sdatkinson/NeuralAmpModelerCore) | Steven Atkinson | Tests only (`fixtures/nam/`), never shipped | MIT |

## Tools

| Tool | Use |
| --- | --- |
| [mdBook](https://github.com/rust-lang/mdBook) | Builds this manual |
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | Checks the licences of everything we build with |

## Thanks

To [TONE3000](https://www.tone3000.com/) and everyone who shares their captures there for free, to the Guitarix project for keeping those cabinets available under the GPL, and to everyone who tries the plugin and tells us what they hear.
