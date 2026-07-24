//! Ensures the rust-embed source folder exists, and (Windows only) embeds the
//! app icon, a `VERSIONINFO` resource, and an `asInvoker` manifest into the
//! installer executable.

use std::{fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    // src/main.rs embeds `../target/tmp/BSARGeom/` via rust-embed, which reads
    // the folder at compile time. On a fresh clone (or any non-dist build such
    // as `cargo clippy --workspace`) the folder does not exist yet, which makes
    // the derive error out. Create it empty; the dist flow fills it before
    // building this crate (see xtask/src/windows.rs). Emit a rerun trigger so a
    // repeated dist re-embeds updated staged files.
    let embed_dir = manifest_dir.join("../target/tmp/BSARGeom");
    fs::create_dir_all(&embed_dir).expect("failed to create the rust-embed source folder");
    println!("cargo:rerun-if-changed=../target/tmp/BSARGeom");

    // The Windows resource is meaningful only for windows targets. On other
    // targets (e.g. the Linux `cargo clippy --workspace` CI leg) skip it so the
    // crate still type-checks.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // App version single-sourced from the root Cargo.toml's [package].version.
    let version = read_root_version(&manifest_dir.join("../Cargo.toml"))
        .expect("failed to read [package].version from the root Cargo.toml");
    let parts: Vec<u32> = version.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    let (major, minor, patch) = (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    );
    let version_str = format!("{version}.0");

    // Icon shared with the app.
    let ico_path = manifest_dir.join("../assets/icon/bsargeom.ico");
    let ico_path_str = ico_path.to_string_lossy().replace('\\', "/");

    // asInvoker manifest: prevents Windows "installer detection" from
    // auto-elevating (the exe name contains "installer"), matching the reference.
    let manifest_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#;
    let manifest_path = out_dir.join("bsargeom.manifest");
    fs::write(&manifest_path, manifest_xml).expect("Failed to write bsargeom.manifest");
    let manifest_path_str = manifest_path.to_string_lossy().replace('\\', "/");

    let rc_content = format!(
        r#"#pragma code_page(65001)
1 ICON "{ico_path_str}"
1 VERSIONINFO
FILETYPE 0x1
PRODUCTVERSION {major}, {minor}, {patch}, 0
FILEVERSION    {major}, {minor}, {patch}, 0
FILEFLAGSMASK 0x3f
FILEFLAGS 0x0
FILEOS 0x40004
FILESUBTYPE 0x0
{{
    BLOCK "StringFileInfo"
    {{
        BLOCK "000004b0"
        {{
            VALUE "ProductName",     "BSARGeom"
            VALUE "FileDescription", "BSARGeom Installer"
            VALUE "ProductVersion",  "{version_str}"
            VALUE "FileVersion",     "{version_str}"
        }}
    }}
    BLOCK "VarFileInfo"
    {{
        VALUE "Translation", 0x0, 0x04b0
    }}
}}
1 24 "{manifest_path_str}"
"#
    );

    let rc_path = out_dir.join("bsargeom-installer.rc");
    fs::write(&rc_path, rc_content).expect("Failed to write bsargeom-installer.rc");

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "gnu" && std::env::var_os("WINDRES").is_none() {
        // SAFETY: build scripts are single-threaded.
        unsafe {
            std::env::set_var("WINDRES", "windres");
        }
    }

    embed_resource::compile(&rc_path, embed_resource::NONE)
        .manifest_required()
        .expect("Failed to embed the Windows resource file");
}

/// Minimal `[package].version` line scan (no TOML dependency).
fn read_root_version(path: &std::path::Path) -> Option<String> {
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
