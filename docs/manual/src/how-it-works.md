# Architecture and strategy

The plugin is an effect that sits on a track after its instrument, in the order the sections stand in the Track tab. Amp first, then the equalizer if you use one, unless you move them.

The engine is NeuralAmpModeler-rs wherever it builds: ARM64 on Linux, Apple Silicon Macs, and x86_64 Linux in a second file, `nam-x86-64-v3.oxp`, for processors with AVX2 and FMA. OxiGuitar picks that file when the processor has those features. Elsewhere, on Windows, Intel Macs and older x86_64 processors, the `nam-rs` library plays instead. Every build sounds the same.

An amp model is a neural network computed for every sample, so it is heavy. A standard model takes about a fifth to two fifths of a processor core per track with the slower engine, and about a tenth of that with the faster one. Models run at 48 kHz. On a device at another rate, the track is converted there and back, and OxiGuitar shifts the other tracks by the same small delay to keep them together.

The four built-in cabinets are impulse responses from the BestPlugins Mega Pack 2 by David Fau Casquel, as Guitarix ships them under the GPL. They are named after the amps they were made to sound like. The makers have nothing to do with them.
