// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! Sets the cfgs that pick the engine (src/engine.rs):
//!
//! - `fast`: NeuralAmpModeler-rs plays the models and cabinets and reads
//!   the model files. On aarch64 under Linux or macOS always (Cargo.toml
//!   brings the library in there); on Linux x86_64 with the feature
//!   `x86-64-v3` and the eight features of that level enabled
//!   (RUSTFLAGS="-C target-cpu=x86-64-v3"; without them the library's own
//!   build script stops the build, so this never sees the feature on
//!   without them there).
//! - `x86_64_v3`: the second case, whose file (`nam-x86-64-v3.oxp`) carries
//!   the manifest with the processor conditions.
//!
//! Everywhere else (Windows, Intel Macs, `nam.oxp` on Linux x86_64) nam-rs
//! plays.

fn main() {
    println!("cargo::rustc-check-cfg=cfg(fast)");
    println!("cargo::rustc-check-cfg=cfg(x86_64_v3)");
    let var = |name: &str| std::env::var(name).unwrap_or_default();
    let features = var("CARGO_CFG_TARGET_FEATURE");
    let has = |f: &str| features.split(',').any(|g| g == f);
    let level = [
        "avx", "avx2", "bmi1", "bmi2", "f16c", "fma", "lzcnt", "movbe",
    ];
    let (os, arch) = (var("CARGO_CFG_TARGET_OS"), var("CARGO_CFG_TARGET_ARCH"));
    let x86_64_v3 = std::env::var_os("CARGO_FEATURE_X86_64_V3").is_some()
        && os == "linux"
        && var("CARGO_CFG_TARGET_ENV") == "gnu"
        && arch == "x86_64"
        && level.iter().all(|f| has(f));
    let aarch64 = arch == "aarch64" && (os == "linux" || os == "macos");
    if x86_64_v3 {
        println!("cargo::rustc-cfg=x86_64_v3");
    }
    if x86_64_v3 || aarch64 {
        println!("cargo::rustc-cfg=fast");
    }
}
