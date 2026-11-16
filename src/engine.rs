// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! What reads a model file and plays a track's amp model and cabinet.
//!
//! - `cfg(fast)` (build.rs): NeuralAmpModeler-rs 2.0 reads the file and
//!   plays both, about ten times faster than nam-rs. That is every build
//!   for aarch64 under Linux or macOS (`nam.oxp` there, and the Apple
//!   Silicon half of the universal file) and, on Linux x86_64, the build
//!   for x86-64-v3 processors (`nam-x86-64-v3.oxp`). Its model plays what
//!   nam-rs plays (both match NAM's reference and each other to under
//!   1e-5), and its cabinet what our convolution plays, `Cabinet::latency`
//!   frames later (one partition), which the effect reports so the player
//!   lines the track up with the others.
//! - Everywhere else, where NeuralAmpModeler-rs can't build (Windows,
//!   Intel Macs, `nam.oxp` on Linux x86_64 for processors without AVX2 and
//!   FMA): nam-rs 0.4.0 for the model and `convolve` for the cabinet, with
//!   no latency.
//!
//! The rest of the plugin (the model file's level and rate, whether it
//! holds a cabinet, the response's rate and level, the conversion to the
//! model's rate) is the same code in every build, so they all sound the
//! same.

use std::path::Path;

/// Which library plays the models, for messages and tests.
#[cfg(not(fast))]
pub const ENGINE: &str = "nam-rs";
/// Which library plays the models, for messages and tests.
#[cfg(fast)]
pub const ENGINE: &str = "NeuralAmpModeler-rs";

/// What the plugin takes from a model file besides the model itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Facts {
    /// Whether the capture holds a cabinet already (`includes_cab`).
    pub(crate) includes_cab: Option<bool>,
    /// How loud the model plays, in dBFS, where the file says.
    pub(crate) loudness: Option<f32>,
    /// The rate the model was captured at (48 kHz where the file doesn't
    /// say).
    pub(crate) rate: f64,
}

/// A model file, read once and kept while it is unchanged: each track
/// builds its own model from it.
pub(crate) struct File {
    #[cfg(not(fast))]
    file: nam_rs::NamModel,
    /// The file's bytes, which NeuralAmpModeler-rs builds each model from.
    #[cfg(fast)]
    bytes: Vec<u8>,
    /// The name the library's messages give the file.
    #[cfg(fast)]
    label: String,
    facts: Facts,
}

impl File {
    /// The model file at `path`, read whole and checked.
    #[cfg(not(fast))]
    pub(crate) fn read(path: &Path) -> Result<File, String> {
        let file = nam_rs::NamModel::from_file(path).map_err(|e| e.to_string())?;
        let facts = Facts {
            includes_cab: includes_cab(file.metadata_typed().gear_type.as_deref()),
            loudness: file.loudness(),
            rate: file.expected_sample_rate(),
        };
        Ok(File { file, facts })
    }

    /// The model file at `path`, read whole and checked: the facts come
    /// from a first build of the model, so a file that can't play is
    /// refused here, once.
    #[cfg(fast)]
    pub(crate) fn read(path: &Path) -> Result<File, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let label = path.display().to_string();
        let pair = build(&bytes, &label)?;
        let facts = Facts {
            includes_cab: includes_cab(pair.metadata.as_ref().and_then(|m| m.gear_type.as_deref())),
            loudness: pair.loudness(),
            rate: f64::from(pair.sample_rate),
        };
        Ok(File {
            bytes,
            label,
            facts,
        })
    }

    /// What the plugin takes from the file besides the model.
    pub(crate) fn facts(&self) -> Facts {
        self.facts
    }
}

/// NeuralAmpModeler-rs's model in `bytes`, built and warmed up (its
/// prewarm: silence through the whole receptive field). Its level
/// adjustments (`input_mult_adj`, `output_mult_adj`) are left unused: the
/// plugin sets the level from the file's loudness itself, as for nam-rs.
#[cfg(fast)]
fn build(
    bytes: &[u8],
    label: &str,
) -> Result<neural_amp_modeler_rs::prelude::LoadedModelPair, String> {
    use neural_amp_modeler_rs::prelude::{
        LoadOptions, SystemSnapshot, load_and_build_model_from_bytes_named,
    };
    let pair = load_and_build_model_from_bytes_named(
        bytes,
        label,
        &SystemSnapshot::capture(),
        false,
        LoadOptions::default(),
    )
    .map_err(|e| e.to_string())?;
    if pair.model_l.is_none() {
        return Err("the file holds no model".to_owned());
    }
    Ok(pair)
}

/// Whether a capture of the gear type `gear_type` (its metadata) holds a
/// cabinet already: the same reading in every build. The vocabulary is
/// NAM's and TONE3000's, as nam-rs 0.4.0 reads it (`Metadata::includes_cab`,
/// MIT, github.com/OpenSauce/nam-rs): case, spaces around and `-` against
/// `_` don't matter; `cab` counts as a whole `_`-separated word only
/// (never `cable`); NAM's undocumented `studio` and TONE3000's
/// `experimental` are unknown.
pub(crate) fn includes_cab(gear_type: Option<&str>) -> Option<bool> {
    let normalized = gear_type?.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        // Everything that stops at or before the power amp's output.
        "amp" | "preamp" | "pedal" | "pedal_amp" | "outboard" | "space" => Some(false),
        // A whole recorded chain, or a cabinet's response.
        "full_rig" | "ir" => Some(true),
        other if other.split('_').any(|word| word == "cab") => Some(true),
        _ => None,
    }
}

/// A model ready to play, warmed up.
pub(crate) struct Model {
    #[cfg(not(fast))]
    model: nam_rs::Model,
    #[cfg(fast)]
    model: Box<neural_amp_modeler_rs::prelude::StaticModel>,
    /// The input, copied: NeuralAmpModeler-rs reads one buffer and writes
    /// another.
    #[cfg(fast)]
    scratch: Vec<f32>,
}

/// The most frames NeuralAmpModeler-rs is given at once: under the buffer
/// size it prepares its models for (8192).
#[cfg(fast)]
const MOST: usize = 4096;

impl Model {
    /// A model of its own from `file`.
    #[cfg(not(fast))]
    pub(crate) fn new(file: &File) -> Result<Model, String> {
        let mut model = nam_rs::Model::from_nam(&file.file).map_err(|e| e.to_string())?;
        // A fresh model's first sound is a transient against silence:
        // spent now, not on the first note.
        let mut warm = vec![0.0f32; model.receptive_field().max(1)];
        model.process_buffer(&mut warm);
        Ok(Model { model })
    }

    /// A model of its own from `file`, warmed up as it is built.
    #[cfg(fast)]
    pub(crate) fn new(file: &File) -> Result<Model, String> {
        let model = build(&file.bytes, &file.label)?
            .model_l
            .ok_or_else(|| "the file holds no model".to_owned())?;
        Ok(Model {
            model,
            scratch: vec![0.0; MOST],
        })
    }

    /// Plays `io` through the model, in place.
    pub(crate) fn process(&mut self, io: &mut [f32]) {
        #[cfg(not(fast))]
        self.model.process_buffer(io);
        #[cfg(fast)]
        {
            use neural_amp_modeler_rs::prelude::NamModel as _;
            for piece in io.chunks_mut(MOST) {
                let input = &mut self.scratch[..piece.len()];
                input.copy_from_slice(piece);
                self.model.process(input, piece);
            }
        }
    }
}

/// A cabinet's response, ready to play.
pub(crate) struct Cabinet {
    #[cfg(not(fast))]
    convolver: crate::convolve::Convolver,
    /// Since NeuralAmpModeler-rs 2.0 it takes pieces of any size and
    /// hands each frame back exactly one partition later: its output queue
    /// starts with a partition of silence.
    #[cfg(fast)]
    adapter: neural_amp_modeler_rs::prelude::CabSimAdapter,
}

/// NeuralAmpModeler-rs's partition, in frames: the cabinet's latency
/// (1.3 ms at 48 kHz).
#[cfg(fast)]
pub(crate) const PARTITION: usize = 64;

impl Cabinet {
    /// The response `taps`, at the track's rate and level already.
    pub(crate) fn new(taps: &[f32]) -> Result<Cabinet, String> {
        #[cfg(not(fast))]
        {
            Ok(Cabinet {
                convolver: crate::convolve::Convolver::new(taps),
            })
        }
        #[cfg(fast)]
        {
            use neural_amp_modeler_rs::prelude::{CabSimAdapter, ConvEngine};
            let engine = ConvEngine::new(taps, PARTITION).map_err(|e| e.to_string())?;
            let adapter = CabSimAdapter::new(Box::new(engine)).map_err(|e| e.to_string())?;
            Ok(Cabinet { adapter })
        }
    }

    /// Plays `io` through the cabinet, in place.
    pub(crate) fn process(&mut self, io: &mut [f32]) {
        #[cfg(not(fast))]
        self.convolver.process(io);
        #[cfg(fast)]
        self.adapter.process_block(io, None);
    }

    /// How many frames late the cabinet's sound comes.
    pub(crate) fn latency(&self) -> usize {
        #[cfg(not(fast))]
        return 0;
        #[cfg(fast)]
        return self.adapter.latency_samples();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/nam")
            .join(name)
    }

    /// Plucked notes, as a DI guitar gives them: decaying partials.
    fn plucks(frames: usize, rate: f32) -> Vec<f32> {
        (0..frames)
            .map(|i| {
                let t = (i % 12_000) as f32 / rate;
                let f = [110.0, 146.8, 196.0, 246.9][(i / 12_000) % 4];
                let decay = (-t * 6.0).exp();
                0.3 * decay
                    * ((2.0 * std::f32::consts::PI * f * t).sin()
                        + 0.5 * (4.0 * std::f32::consts::PI * f * t).sin())
            })
            .collect()
    }

    /// Plays `x` through `run` in pieces of uneven sizes, as the player's
    /// blocks come.
    fn in_pieces(x: &[f32], mut run: impl FnMut(&mut [f32])) -> Vec<f32> {
        let mut out = x.to_vec();
        let sizes = [64, 128, 96, 256, 1, 17, 512, 4800, 10_000];
        let mut at = 0;
        let mut i = 0;
        while at < out.len() {
            let n = sizes[i % sizes.len()].min(out.len() - at);
            run(&mut out[at..at + n]);
            at += n;
            i += 1;
        }
        out
    }

    #[test]
    fn models_play_whatever_the_pieces_are() {
        for name in ["wavenet.nam", "lstm.nam"] {
            let file = File::read(&fixture(name)).unwrap();
            let x = plucks(48_000, 48_000.0);
            let mut a = Model::new(&file).unwrap();
            let mut b = Model::new(&file).unwrap();
            let whole = {
                let mut y = x.clone();
                a.process(&mut y);
                y
            };
            let pieces = in_pieces(&x, |p| b.process(p));
            let peak = whole.iter().fold(0.0f32, |p, v| p.max(v.abs()));
            assert!(peak > 0.01, "{name} ({ENGINE}) is silent");
            let diff = whole
                .iter()
                .zip(&pieces)
                .fold(0.0f32, |p, (u, v)| p.max((u - v).abs()));
            assert!(diff < 1e-6, "{name} ({ENGINE}): pieces differ by {diff}");
        }
    }

    /// Every build reads a file's facts as nam-rs does.
    #[test]
    fn the_facts_are_nam_rs_s() {
        for name in ["wavenet.nam", "lstm.nam"] {
            let path = fixture(name);
            let theirs = nam_rs::NamModel::from_file(&path).unwrap();
            let facts = File::read(&path).unwrap().facts();
            assert_eq!(facts.includes_cab, theirs.includes_cab(), "{name}");
            assert_eq!(facts.loudness, theirs.loudness(), "{name}");
            assert_eq!(facts.rate, theirs.expected_sample_rate(), "{name}");
        }
    }

    /// Our reading of the gear types is nam-rs's, word for word.
    #[test]
    fn a_cabinet_is_read_as_nam_rs_reads_it() {
        for gear in [
            "amp",
            "Preamp",
            " pedal ",
            "pedal-amp",
            "outboard",
            "space",
            "full-rig",
            "ir",
            "amp_cab",
            "amp-pedal-cab",
            "cab",
            "cable",
            "cabless",
            "studio",
            "experimental",
            "",
        ] {
            let metadata: nam_rs::Metadata =
                oxiguitar_plugin::serde_json::from_value(oxiguitar_plugin::serde_json::json!({
                    "gear_type": gear
                }))
                .unwrap();
            assert_eq!(
                includes_cab(Some(gear)),
                metadata.includes_cab(),
                "{gear:?}"
            );
        }
        assert_eq!(includes_cab(None), None);
    }

    /// The fast build's model and cabinet play what nam-rs and our
    /// convolution play: every build sounds the same.
    #[cfg(fast)]
    #[test]
    fn the_fast_build_sounds_like_the_other() {
        let x = plucks(48_000, 48_000.0);
        for name in ["wavenet.nam", "lstm.nam"] {
            let path = fixture(name);
            let file = nam_rs::NamModel::from_file(&path).unwrap();
            let mut ours = nam_rs::Model::from_nam(&file).unwrap();
            let mut warm = vec![0.0f32; ours.receptive_field().max(1)];
            ours.process_buffer(&mut warm);
            let mut theirs = Model::new(&File::read(&path).unwrap()).unwrap();
            let a = in_pieces(&x, |p| ours.process_buffer(p));
            let b = in_pieces(&x, |p| theirs.process(p));
            let peak = a.iter().fold(0.0f32, |p, v| p.max(v.abs()));
            let diff = a
                .iter()
                .zip(&b)
                .fold(0.0f32, |p, (u, v)| p.max((u - v).abs()));
            println!("{name}: largest difference {diff:.2e}, peak {peak:.3}");
            assert!(
                diff < 1e-4 * peak.max(1.0),
                "{name}: the builds differ by {diff}"
            );
        }

        for (name, data) in crate::BUILT_IN {
            let response = crate::wav::read(data).unwrap();
            let taps = crate::convolve::normalise(
                crate::resample::convert(&response.taps, response.rate, 48_000),
                48_000,
            );
            let mut ours = crate::convolve::Convolver::new(&taps);
            let mut theirs = Cabinet::new(&taps).unwrap();
            let late = theirs.latency();
            assert_eq!(late, PARTITION);
            let a = in_pieces(&x, |p| ours.process(p));
            let b = in_pieces(&x, |p| theirs.process(p));
            let peak = a.iter().fold(0.0f32, |p, v| p.max(v.abs()));
            // Their output against ours at each lag up to two partitions:
            // the best one must be the latency it declares. (Their
            // denormal guard adds a dither near 1e-11, so silence isn't
            // exactly zero.)
            let differ = |lag: usize| {
                a[..a.len() - lag]
                    .iter()
                    .zip(&b[lag..])
                    .fold(0.0f32, |p, (u, v)| p.max((u - v).abs()))
            };
            let (best, diff) = (0..=2 * PARTITION)
                .map(|lag| (lag, differ(lag)))
                .fold((0, f32::MAX), |m, d| if d.1 < m.1 { d } else { m });
            let before = b[..late].iter().fold(0.0f32, |p, v| p.max(v.abs()));
            println!(
                "{name}: best lag {best} (declared {late}), largest difference {diff:.2e}, \
                 before the latency {before:.2e}, peak {peak:.3}"
            );
            assert_eq!(
                best, late,
                "{name}: the sound comes {best} frames late, not {late}"
            );
            assert!(before < 1e-6, "{name}: sound before its latency: {before}");
            assert!(
                diff < 1e-4 * peak.max(1.0),
                "{name}: the cabinets differ by {diff}"
            );
        }
    }
}
