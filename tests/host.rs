// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! The plugin through OxiGuitar's plugin host, as the app loads it: a model
//! played in real time, the rate conversion, and (on Linux x86_64, when the
//! file is given) the build for x86-64-v3 processors. Moved from OxiGuitar's
//! `oxiguitar-plugin-test` tests (#165).
//!
//! The plugin needs the MIDI plugin (`oxiguitar.midi`): `midi.oxp` comes from
//! `$OXIGUITAR_MIDI_OXP`, or `scripts/midi.sh` puts the latest release's at
//! `target/midi/midi.oxp`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use oxiguitar_host::api::message::{PrepareEffect, PreparedTrack, TrackInfo};
use oxiguitar_host::{Plugins, SharedSettings, Unheard};
use oxiguitar_plugin_test::{TEST_SET, check_with};

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_owned()
}

/// This plugin as the `.oxp` file we ship (a debug build), built once.
fn ours() -> &'static Path {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(|| {
        oxiguitar_plugin_test::build(&workspace(), env!("CARGO_PKG_NAME"), "nam", false)
            .expect("the .oxp file builds")
    })
}

/// The MIDI plugin this one needs.
fn midi_oxp() -> &'static Path {
    static MIDI: OnceLock<PathBuf> = OnceLock::new();
    MIDI.get_or_init(|| {
        if let Some(path) = std::env::var_os("OXIGUITAR_MIDI_OXP") {
            return PathBuf::from(path);
        }
        let path = workspace().join("target/midi/midi.oxp");
        assert!(
            path.is_file(),
            "midi.oxp isn't at {}: run scripts/midi.sh, or set OXIGUITAR_MIDI_OXP",
            path.display()
        );
        path
    })
}

/// The NAM plugin with a model in its `nam` folder: a host over a copy of
/// it (a library loads once per process), and that folder.
fn nam_plugins(tag: &str) -> (Plugins, PathBuf) {
    nam_plugins_of(
        &[
            (ours().to_owned(), format!("nam-{tag}.oxp")),
            (midi_oxp().to_owned(), "midi.oxp".into()),
        ],
        tag,
    )
}

/// Like [`nam_plugins`], with the plugin files `files` (each copied under
/// its new name) in the folder.
fn nam_plugins_of(files: &[(PathBuf, String)], tag: &str) -> (Plugins, PathBuf) {
    let scratch = std::env::temp_dir().join(format!("oxiguitar-nam-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let folder = scratch.join("plugins");
    std::fs::create_dir_all(&folder).unwrap();
    for (file, name) in files {
        std::fs::copy(file, folder.join(name)).unwrap();
    }
    let home = scratch.join("home");
    std::fs::create_dir_all(home.join("nam/Amps")).unwrap();
    // The MIDI plugin NAM needs is checked too, and plays from a set.
    std::fs::create_dir_all(home.join("soundfonts")).unwrap();
    std::fs::write(home.join("soundfonts/test-sine.sf2"), TEST_SET).unwrap();
    std::fs::copy(
        workspace().join("fixtures/nam/wavenet.nam"),
        home.join("nam/Amps/wavenet.nam"),
    )
    .unwrap();
    let plugins = Plugins::discover(
        &[folder],
        SharedSettings::default(),
        oxiguitar_host::display_system(),
        Arc::new(Unheard),
    );
    let folders = oxiguitar_host::Folders::resolve(
        oxiguitar_host::System::this(),
        |name| (name == oxiguitar_host::HOME_VARIABLE).then(|| home.clone().into()),
        None,
    )
    .unwrap();
    plugins.use_folders(folders);
    (plugins, scratch)
}

/// The NAM plugin passes the contract, and plays a model and a built-in
/// cabinet through the suite's real-time checks: nothing allocated and
/// (on Linux) no system call made while it runs (M4 slice 6).
#[test]
fn our_nam_plugin_runs_a_model_in_real_time() {
    let report = check_with(ours(), &[midi_oxp()]);
    assert!(report.ok(), "nam.oxp: {:#?}", report.failed);
    let (plugins, scratch) = nam_plugins("realtime");
    let settings =
        r#"{"on": true, "model": "Amps/wavenet.nam", "cabinet": "built-in:evh-5150-iii"}"#;
    let report = oxiguitar_plugin_test::check_realtime(&plugins, settings);
    assert!(report.ok(), "{:#?}", report.failed);
    for wanted in [
        "effect nam: it prepares",
        "effect nam: it allocates nothing",
    ] {
        assert!(
            report.passed.iter().any(|p| p.contains(wanted)),
            "{wanted}: {:#?}",
            report.passed
        );
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// At a rate other than the model's, the track goes to 48 kHz and back:
/// it says its latency, makes sound, and allocates nothing while running;
/// a model that isn't there is refused with its name.
#[test]
fn our_nam_plugin_converts_the_rate_for_its_model() {
    use oxiguitar_host::api::class::InstrumentClass;

    let (plugins, scratch) = nam_plugins("rate");
    let effect = plugins.effects_for(InstrumentClass::Fretted)[0];
    let prepare = |settings: &str| PrepareEffect {
        effect: "nam".into(),
        rate: 44_100,
        frames: 512,
        track: PreparedTrack {
            info: TrackInfo {
                index: 0,
                name: "Lead".into(),
                class: InstrumentClass::Fretted,
            },
            program: 29,
            percussion: false,
            tuning: vec![64, 59, 55, 50, 45, 40],
            capo: 0,
            settings: settings.into(),
        },
    };
    let mut amp = plugins
        .prepare_effect(
            effect,
            prepare(r#"{"on": true, "model": "Amps/wavenet.nam"}"#),
        )
        .unwrap();
    assert!(
        amp.latency() > 16,
        "the conversion's latency: {}",
        amp.latency()
    );
    let mut peak = 0.0f32;
    let mut allocated = 0;
    for k in 0..200 {
        let mut l: Vec<f32> = (0..300)
            .map(|n| (((k * 300 + n) as f32) * 0.06).sin() * 0.3)
            .collect();
        let mut r = l.clone();
        let before = amp.allocations();
        assert!(amp.process(&mut l, &mut r));
        if k > 0 {
            allocated += amp.allocations().saturating_sub(before);
        }
        peak = l.iter().fold(peak, |p, v| p.max(v.abs()));
        assert_eq!(l, r, "the same on both sides");
    }
    assert!(peak > 0.01, "it makes sound: {peak}");
    assert_eq!(allocated, 0, "nothing allocated while running");
    let off = plugins
        .prepare_effect(
            effect,
            prepare(r#"{"on": false, "model": "Amps/wavenet.nam"}"#),
        )
        .unwrap();
    assert_eq!(off.latency(), 0);
    let missing = plugins
        .prepare_effect(effect, prepare(r#"{"on": true, "model": "gone.nam"}"#))
        .unwrap_err();
    assert!(missing.to_string().contains("gone.nam"), "{missing}");
    let escaping = plugins
        .prepare_effect(effect, prepare(r#"{"on": true, "model": "../../x.nam"}"#))
        .unwrap_err();
    assert!(
        escaping.to_string().contains("isn't in the nam folder"),
        "{escaping}"
    );
    let _ = std::fs::remove_dir_all(&scratch);
}

/// The NAM plugin's build for x86-64-v3 processors (nam-x86-64-v3.oxp, made
/// by build-oxp.sh on Linux x86_64), when
/// `OXIGUITAR_TEST_NAM_X86_64_V3` names it (CI's "NAM for x86-64-v3" job):
/// it passes the contract and the real-time checks as nam.oxp does, comes a
/// cabinet's partition (64 frames) later than it, and the host uses it
/// rather than nam.oxp, whatever the order of their names (the
/// processor-features step of the choice: docs/design/plugins.md). Without
/// the variable there is nothing to test.
#[test]
fn the_nam_build_for_x86_64_v3_processors_runs_in_real_time_and_is_chosen() {
    use oxiguitar_host::State;
    use oxiguitar_host::api::class::InstrumentClass;

    let Some(v3) = std::env::var_os("OXIGUITAR_TEST_NAM_X86_64_V3") else {
        println!("OXIGUITAR_TEST_NAM_X86_64_V3 isn't set: nothing to test");
        return;
    };
    let v3 = PathBuf::from(v3);
    let report = check_with(&v3, &[midi_oxp()]);
    assert!(report.ok(), "{}: {:#?}", v3.display(), report.failed);

    // Named to come after nam.oxp: the host must still take it.
    let (plugins, scratch) = nam_plugins_of(
        &[
            (ours().to_owned(), "a-nam.oxp".into()),
            (v3.clone(), "b-nam.oxp".into()),
            (midi_oxp().to_owned(), "midi.oxp".into()),
        ],
        "v3",
    );
    let states: Vec<(String, &State)> = plugins
        .found()
        .iter()
        .filter(|f| f.path.file_name().is_some_and(|n| n != "midi.oxp"))
        .map(|f| {
            (
                f.path.file_name().unwrap().to_string_lossy().into_owned(),
                &f.state,
            )
        })
        .collect();
    assert_eq!(states.len(), 2, "{states:#?}");
    assert!(
        matches!(states[0].1, State::Replaced(p) if p.ends_with("b-nam.oxp")),
        "{states:#?}"
    );
    assert_eq!(states[1].1, &State::Ready, "{states:#?}");

    let settings =
        r#"{"on": true, "model": "Amps/wavenet.nam", "cabinet": "built-in:evh-5150-iii"}"#;
    let report = oxiguitar_plugin_test::check_realtime(&plugins, settings);
    assert!(report.ok(), "{:#?}", report.failed);
    assert!(
        report
            .passed
            .iter()
            .any(|p| p.contains("effect nam: it allocates nothing")),
        "{:#?}",
        report.passed
    );

    let effect = plugins.effects_for(InstrumentClass::Fretted)[0];
    let prepare = PrepareEffect {
        effect: "nam".into(),
        rate: 48_000,
        frames: 512,
        track: PreparedTrack {
            info: TrackInfo {
                index: 0,
                name: "Lead".into(),
                class: InstrumentClass::Fretted,
            },
            program: 29,
            percussion: false,
            tuning: vec![64, 59, 55, 50, 45, 40],
            capo: 0,
            settings: settings.into(),
        },
    };
    let amp = plugins.prepare_effect(effect, prepare).unwrap();
    assert_eq!(amp.latency(), 64, "the cabinet's partition");
    let _ = std::fs::remove_dir_all(&scratch);
}
