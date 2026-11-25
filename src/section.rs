// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! The Track tab's section "Amp (NAM)", for guitars and basses: NAM on
//! or off, the model, the cabinet, and the levels before and after. While
//! the amp is on, a hint under the model says how to feed it a clean
//! guitar (mateusrdp/oxiguitar#152): the MIDI plugin's section Sound,
//! "Like clean guitars". The section can't see that section's settings
//! (each plugin gets only its own), so the hint says what to do rather
//! than checking it. Drawn by `ui/nam.slint` (plugin interface 1.16); this
//! side sets its properties and changes the track's settings.

use std::sync::Mutex;

use oxiguitar_plugin::serde_json::{Map, Value as Json};
use oxiguitar_plugin::{Error, Host, TrackView, Ui, Value};

use crate::files::{self, Found};
use crate::settings::{Cabinet, LEVELS, Track};

/// The section's side in the plugin: the track's settings as the Track
/// tab holds them.
#[derive(Debug)]
pub(crate) struct Section {
    host: Host,
    settings: Mutex<Map<String, Json>>,
}

impl Section {
    pub(crate) fn new(host: Host) -> Section {
        Section {
            host,
            settings: Mutex::new(Map::new()),
        }
    }

    fn found(&self) -> (Option<std::path::PathBuf>, Found) {
        let folder = crate::folder(&self.host);
        let found = folder.as_deref().map(files::find).unwrap_or_default();
        (folder, found)
    }
}

/// The model choice's lines: "None", the model the track names when it
/// isn't there any more, then every model found; the line picked; and
/// where the found models start.
fn models(found: &Found, track: &Track) -> (Vec<String>, u32, usize) {
    let mut options = vec!["None".to_owned()];
    let picked = match &track.model {
        None => 0,
        Some(m) => match found.models.iter().position(|o| o == m) {
            Some(i) => i + 1,
            None => {
                options.push(format!("{m} (not in the nam folder)"));
                1
            }
        },
    };
    let start = options.len();
    options.extend(found.models.iter().cloned());
    (options, picked as u32, start)
}

/// The hint shown under the model while the amp is on: the MIDI plugin's
/// section and choice, by name.
pub(crate) const CLEAN_HINT: &str =
    "For a clean guitar into the amp, set this track's Sound to \"Like clean guitars\".";

/// The cabinet choice's lines and what each picks.
fn cabinets(found: &Found) -> Vec<(String, Cabinet)> {
    let mut out = vec![
        ("From the model".to_owned(), Cabinet::Auto),
        ("None".to_owned(), Cabinet::None),
    ];
    for (name, _) in crate::BUILT_IN {
        out.push((
            format!("{} (built in)", crate::built_in_title(name)),
            Cabinet::BuiltIn(name.into()),
        ));
    }
    for r in &found.responses {
        out.push((r.clone(), Cabinet::File(r.clone())));
    }
    out
}

fn strings(lines: Vec<String>) -> Value {
    Value::List(lines.into_iter().map(Value::String).collect())
}

/// The section's properties for a track whose settings are `settings`,
/// with the models and responses `found` in `folder`.
pub(crate) fn properties(
    settings: &Map<String, Json>,
    folder: Option<&std::path::Path>,
    found: &Found,
) -> Vec<(String, Value)> {
    let t = Track::from_map(settings);
    let (model_options, model_picked, _) = models(found, &t);
    let cabs = cabinets(found);
    // A file no longer there (or a built-in one a later version dropped)
    // is shown as picked, at the end.
    let cab_picked = cabs
        .iter()
        .position(|(_, c)| *c == t.cabinet)
        .unwrap_or(cabs.len());
    let mut cab_options: Vec<String> = cabs.into_iter().map(|(n, _)| n).collect();
    if cab_picked == cab_options.len() {
        cab_options.push(match &t.cabinet {
            Cabinet::File(f) | Cabinet::BuiltIn(f) => format!("{f} (not found)"),
            Cabinet::Auto | Cabinet::None => "?".into(),
        });
    }
    vec![
        ("on".into(), Value::Bool(t.on)),
        ("models".into(), strings(model_options)),
        ("model".into(), Value::Number(f64::from(model_picked))),
        (
            "can-pick-model".into(),
            Value::Bool(!found.models.is_empty() || t.model.is_some()),
        ),
        ("hint".into(), Value::String(CLEAN_HINT.into())),
        ("cabinets".into(), strings(cab_options)),
        ("cabinet".into(), Value::Number(cab_picked as f64)),
        ("input".into(), Value::Number(f64::from(t.input))),
        ("output".into(), Value::Number(f64::from(t.output))),
        ("minimum".into(), Value::Number(f64::from(LEVELS.0))),
        ("maximum".into(), Value::Number(f64::from(LEVELS.1))),
        ("no-models".into(), Value::Bool(found.models.is_empty())),
        (
            "folder".into(),
            Value::String(
                folder.map_or_else(|| "the nam folder".to_owned(), |f| f.display().to_string()),
            ),
        ),
    ]
}

fn number(args: &[Value]) -> Option<f64> {
    match args.first() {
        Some(Value::Number(n)) if n.is_finite() => Some(*n),
        _ => None,
    }
}

fn index(args: &[Value]) -> Option<usize> {
    number(args)
        .filter(|i| *i >= 0.0 && i.fract() == 0.0)
        .map(|i| i as usize)
}

/// Changes `settings` for the section's callback `callback` with `args`,
/// with the models and responses `found`; false when it changes nothing.
pub(crate) fn change(
    settings: &mut Map<String, Json>,
    found: &Found,
    callback: &str,
    args: &[Value],
) -> bool {
    let mut t = Track::from_map(settings);
    match (callback, args.first()) {
        ("switched", Some(Value::Bool(on))) => t.on = *on,
        ("model-chosen", _) => {
            let Some(i) = index(args) else {
                return false;
            };
            let (_, _, start) = models(found, &t);
            if i == 0 {
                t.model = None;
            } else if let Some(m) = i.checked_sub(start).and_then(|k| found.models.get(k)) {
                t.model = Some(m.clone());
                // Picking a model is wanting to hear it.
                t.on = true;
            } else {
                // The missing file's line: left as it is.
                return false;
            }
        }
        ("cabinet-chosen", _) => {
            match index(args).and_then(|i| cabinets(found).into_iter().nth(i)) {
                Some((_, c)) => t.cabinet = c,
                None => return false,
            }
        }
        ("input-slid", _) | ("output-slid", _) => {
            let Some(v) = number(args) else {
                return false;
            };
            let v = (((v * 2.0).round() / 2.0) as f32).clamp(LEVELS.0, LEVELS.1);
            if callback == "input-slid" {
                t.input = v;
            } else {
                t.output = v;
            }
        }
        _ => return false,
    }
    t.input = t.input.clamp(LEVELS.0, LEVELS.1);
    t.output = t.output.clamp(LEVELS.0, LEVELS.1);
    t.write(settings);
    true
}

impl Ui for Section {
    fn shown(&self, view: &str, track: Option<&TrackView>) -> Result<(), Error> {
        if view != crate::SECTION {
            return Ok(());
        }
        let settings: Map<String, Json> = track
            .and_then(|t| oxiguitar_plugin::serde_json::from_str(&t.settings).ok())
            .unwrap_or_default();
        let (folder, found) = self.found();
        let properties = properties(&settings, folder.as_deref(), &found);
        *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = settings;
        self.host.set_view(crate::SECTION, properties)
    }

    fn callback(&self, view: &str, callback: &str, args: Vec<Value>) -> Result<(), Error> {
        if view != crate::SECTION {
            return Ok(());
        }
        if callback == "manual" {
            return crate::links::open(crate::MANUAL)
                .map_err(|why| Error::new(oxiguitar_plugin::ErrorKind::Failed, why));
        }
        let (folder, found) = self.found();
        let settings = {
            let mut settings = self.settings.lock().unwrap_or_else(|e| e.into_inner());
            if !change(&mut settings, &found, callback, &args) {
                return Ok(());
            }
            settings.clone()
        };
        self.host.set_track_settings(crate::SECTION, &settings)?;
        self.host.set_view(
            crate::SECTION,
            properties(&settings, folder.as_deref(), &found),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_model_stays_shown_and_none_comes_first() {
        let found = Found {
            models: vec!["a.nam".into(), "b.nam".into()],
            responses: vec!["4x12.wav".into()],
        };
        let none = models(&found, &Track::default());
        assert_eq!(
            none,
            (vec!["None".into(), "a.nam".into(), "b.nam".into()], 0, 1)
        );
        let b = Track {
            model: Some("b.nam".into()),
            ..Track::default()
        };
        assert_eq!(models(&found, &b).1, 2);
        let gone = Track {
            model: Some("gone.nam".into()),
            ..Track::default()
        };
        let (lines, picked, start) = models(&found, &gone);
        assert_eq!(lines[1], "gone.nam (not in the nam folder)");
        assert_eq!((picked, start), (1, 2));
        let cabs = cabinets(&found);
        assert_eq!(cabs.len(), 2 + crate::BUILT_IN.len() + 1);
        assert_eq!(cabs.last().unwrap().1, Cabinet::File("4x12.wav".into()));
    }

    fn get<'a>(p: &'a [(String, Value)], name: &str) -> &'a Value {
        &p.iter().find(|(n, _)| n == name).unwrap().1
    }

    /// The hint shows while the amp is on (the file shows it then).
    #[test]
    fn the_clean_guitar_hint_shows_while_the_amp_is_on() {
        let found = Found::default();
        let mut settings = Map::new();
        assert_eq!(
            get(&properties(&settings, None, &found), "on"),
            &Value::Bool(false)
        );
        assert!(change(
            &mut settings,
            &found,
            "switched",
            &[Value::Bool(true)]
        ));
        let p = properties(&settings, None, &found);
        assert_eq!(get(&p, "on"), &Value::Bool(true));
        assert!(matches!(get(&p, "hint"), Value::String(h) if h.contains("Like clean guitars")));
        assert_eq!(get(&p, "no-models"), &Value::Bool(true));
        assert_eq!(get(&p, "can-pick-model"), &Value::Bool(false));
    }

    /// Picking a model turns the amp on; the levels go by half decibels
    /// within their range; what is out of range changes nothing.
    #[test]
    fn the_section_picks_the_model_the_cabinet_and_the_levels() {
        let found = Found {
            models: vec!["a.nam".into(), "b.nam".into()],
            responses: vec!["4x12.wav".into()],
        };
        let mut settings = Map::new();
        assert!(change(
            &mut settings,
            &found,
            "model-chosen",
            &[Value::Number(2.0)]
        ));
        let t = Track::from_map(&settings);
        assert_eq!((t.on, t.model.as_deref()), (true, Some("b.nam")));
        let last = (cabinets(&found).len() - 1) as f64;
        assert!(change(
            &mut settings,
            &found,
            "cabinet-chosen",
            &[Value::Number(last)]
        ));
        assert_eq!(
            Track::from_map(&settings).cabinet,
            Cabinet::File("4x12.wav".into())
        );
        assert!(change(
            &mut settings,
            &found,
            "input-slid",
            &[Value::Number(3.3)]
        ));
        assert!(change(
            &mut settings,
            &found,
            "output-slid",
            &[Value::Number(-99.0)]
        ));
        let t = Track::from_map(&settings);
        assert_eq!((t.input, t.output), (3.5, LEVELS.0));
        let before = settings.clone();
        assert!(!change(
            &mut settings,
            &found,
            "cabinet-chosen",
            &[Value::Number(99.0)]
        ));
        assert!(!change(
            &mut settings,
            &found,
            "model-chosen",
            &[Value::Number(-1.0)]
        ));
        assert!(!change(&mut settings, &found, "other", &[]));
        assert_eq!(settings, before);
        let p = properties(&settings, None, &found);
        assert_eq!(get(&p, "model"), &Value::Number(2.0));
        assert_eq!(get(&p, "cabinet"), &Value::Number(last));
    }

    /// The file compiles as OxiGuitar compiles it, with every property and
    /// callback this side uses.
    #[test]
    fn the_slint_file_has_what_the_section_sets_and_hears() {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/nam.slint");
        let compiled = oxiguitar_plugin_test::compile_ui_file(&file).unwrap();
        let c = compiled
            .components
            .iter()
            .find(|c| c.name == "NamSection")
            .unwrap();
        for (name, _) in properties(&Map::new(), None, &Found::default()) {
            assert!(c.properties.iter().any(|p| *p == name), "{name}");
        }
        for name in [
            "switched",
            "model-chosen",
            "cabinet-chosen",
            "input-slid",
            "output-slid",
            "manual",
        ] {
            assert!(c.callbacks.iter().any(|p| *p == name), "{name}");
        }
    }
}
