//! Re-export two values so they are readable at run time via `env!(...)`:
//!
//!  - `TARGET` — the triple xtask is compiled for. cargo exposes it to build
//!    scripts only. The Linux/macOS packaging steps build the app for this same
//!    triple and read the binary from `target/<triple>/release`. Under `cross`
//!    this is the requested cross-target, which `cross` does NOT propagate to
//!    xtask's nested `cargo build`, so passing it explicitly is what keeps a
//!    cross build from silently producing host-architecture binaries.
//!  - `APP_VERSION` — the app version, read from the root `Cargo.toml`'s
//!    `[package].version` so installer/archive names always match the app with
//!    nothing to keep in sync (xtask has its own unrelated crate version).

use std::path::Path;

fn main() {
    let target = std::env::var("TARGET").expect("cargo always sets TARGET for build scripts");
    println!("cargo:rustc-env=TARGET={target}");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root_cargo = Path::new(&manifest_dir).join("../Cargo.toml");
    let version = read_package_version(&root_cargo)
        .expect("failed to read [package].version from the root Cargo.toml");
    println!("cargo:rustc-env=APP_VERSION={version}");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../Cargo.toml");
}

/// Extract `version = "..."` from the `[package]` table of a Cargo manifest with
/// a minimal line scan (no TOML dependency).
fn read_package_version(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut in_package = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package
            && let Some(rest) = trimmed.strip_prefix("version")
            && let Some((_, value)) = rest.split_once('=')
        {
            return Some(value.trim().trim_matches('"').to_string());
        }
    }
    None
}
