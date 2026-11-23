# Architecture and strategy

## How it fits into OxiGuitar

The plugin is an *effect*: it sits on a track after the instrument that plays it, the MIDI plugin. Inside, each track that has an amp gets a small chain:

1. **Input** gain, which drives the amp harder or softer, the way your guitar's volume knob does.
2. **The amp model**, the `.nam` file, run for every sample of sound.
3. **The cabinet**, the impulse response, applied by convolution (an FFT, through the `rustfft` library).
4. **Output** gain, the track's level after the amp.

On the Track tab, effects stand in the order they play. The amp comes first, then the equalizer if you use one, and the arrows beside each section swap them. Settings are saved per track in the song's `.oxg` file beside it; the score itself doesn't change.

## The engines

Two Rust libraries run NAM models, and the plugin uses whichever builds on your system. Both are checked against NAM's own reference code, so every build sounds the same.

- **[NeuralAmpModeler-rs](https://github.com/fabiohl/NeuralAmpModeler-rs)**, about ten times faster: on ARM64 Linux, on Apple Silicon Macs, and on x86_64 Linux in a second file of the release, `nam-x86-64-v3.oxp`, for processors with AVX2 and FMA. OxiGuitar picks that file by itself when the processor has those features.
- **[nam-rs](https://github.com/OpenSauce/nam-rs)** everywhere else: Windows, Intel Macs and older x86_64 processors, where NeuralAmpModeler-rs can't build.

A model is a neural network computed for every sample, so it is the heaviest thing OxiGuitar runs. A standard model takes about a fifth to two fifths of a processor core per track with nam-rs, and about a tenth of that with NeuralAmpModeler-rs.

Models are trained at 48 kHz. If your sound device runs at another rate, the track is converted to 48 kHz and back, and OxiGuitar delays the other tracks by the same small amount so everything stays together.

## The built-in cabinets

The four built-in cabinets are impulse responses from the BestPlugins Mega Pack 2 by David Fau Casquel, as Guitarix ships them under the GPL. They are named after the amps they were made to go with (an ENGL Retro Tube, an EVH 5150 III, a modified Marshall JMP 2203, a Mesa/Boogie Mark V). The amps' makers have nothing to do with them.

## Current limitations

- **It is heavy on Windows and Intel Macs.** There, nam-rs is the only engine that builds, and several amp tracks can make an older computer stutter. Lighter models ("lite", "feather", "nano") help.
- **No amp knobs.** A profile is a snapshot of one setting. Only Input and Output are offered; there is no gain, bass, middle or treble that changes the amp itself. The equalizer plugin can shape the sound after it.
- **No noise gate, no stereo.** The chain is mono and has no gate. Synthesised guitars have no hum to gate, so this rarely matters.
- **It can't see the Sound section.** It doesn't know whether the track plays a clean or an already distorted guitar, so it only reminds you to feed it a clean one.
- **No browsing or downloading.** You find models yourself and drop them in the folder; there is no TONE3000 browser.

## Left out on purpose

- **Downloading anything.** OxiGuitar never connects to the internet, and that includes fetching models. That rules out a built-in model browser too.
- **Training models.** Capturing your own amp needs the real amp, a recording setup and NAM's trainer, which is Python and machine-learning tooling. It is better done with [Neural Amp Modeler's own tools](https://github.com/sdatkinson/neural-amp-modeler) or TONE3000, and the result works here unchanged.
- **Bundling models.** Each model has its own licence and author. Shipping any of them would mean choosing for you and taking on their licences; the plugin ships only the four GPL cabinets.
- **A modelling amp.** Writing circuit models amp by amp is a different and much bigger project. NAM covers the need with files anyone can make.
- **Forks of the engines.** The libraries are used exactly as published. A missing feature is handled in our code or asked for upstream.

## Where you can help

- **Speed on Windows and Intel Macs.** Anything that makes nam-rs faster, or lets NeuralAmpModeler-rs build there, helps the most people. That work happens in those libraries' own repositories.
- **A noise gate or a simple tone stack** in the chain, if you can do it without allocating memory on the audio thread (`src/lib.rs`, `src/convolve.rs`).
- **Listening.** Compare a model here with the same model in NAM's own plugin and tell us if anything sounds different.
- **A NAM capture we may publish.** We'd love one sample model whose licence lets us use it in the manual's examples.

Every change needs its tests and a row on the [Features](features.md) page; the [Testing](testing.md) page says how to run them.
