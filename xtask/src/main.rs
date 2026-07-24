//! `cargo xtask dist` — build BSARGeom in release mode and package it into a
//! platform installer plus a plain archive under `target/dist`.
//!
//! Dispatches by host OS: Windows (zip + embedded `.exe` installer), Linux
//! (`makeself` `.run` + tar.gz), macOS (`.app` bundle in a tar.gz).

mod linux;
mod macos;
mod windows;

use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "windows") {
        windows::build_and_package()?;
    } else if cfg!(target_os = "linux") {
        linux::build_and_package()?;
    } else if cfg!(target_os = "macos") {
        macos::build_and_package()?;
    } else {
        eprintln!(
            "`cargo xtask dist` is only supported on Windows, Linux, and macOS.\n\
             On other platforms, build and install BSARGeom manually with \
             `cargo build --release`."
        );
        std::process::exit(1);
    }

    Ok(())
}

/// Run a command inheriting stdio and fail if it exits non-zero.
pub(crate) fn run(cmd: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("command {cmd:?} exited with {status}").into());
    }
    Ok(())
}

/// Triple the app is built for. Defaults to the triple xtask itself was compiled
/// for (`env!("TARGET")`, correct for a plain local `cargo xtask dist`), but can
/// be overridden with `XTASK_TARGET` to cross-compile from a single xtask — e.g.
/// building the `x86_64-apple-darwin` binary on an arm64 CI runner.
pub(crate) fn dist_target() -> String {
    std::env::var("XTASK_TARGET").unwrap_or_else(|_| env!("TARGET").to_string())
}

/// Architecture label for artifact names, derived from the target *triple*
/// (not `cfg!(target_arch)`, which would report the host arch under a
/// cross-compile).
pub(crate) fn arch_of(target: &str) -> &'static str {
    if target.starts_with("x86_64") {
        "x86_64"
    } else if target.starts_with("aarch64") {
        "aarch64"
    } else {
        "unknown"
    }
}
