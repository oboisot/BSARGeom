//! Build script: on Windows targets, embed the application icon and a
//! `VERSIONINFO` resource into `bsargeom.exe` so the icon appears in Explorer
//! and the taskbar and the version shows under Properties > Details. It is a
//! no-op on every other target (Linux/macOS carry their icon through the
//! `.desktop` entry and the `.app` bundle respectively — see `install/` and
//! `xtask/src/macos.rs`).

use std::{fs, path::PathBuf};

fn main() {
    // `CARGO_CFG_TARGET_OS` reflects the *target* being built (correct under
    // cross-compilation), unlike a host `cfg!(windows)`.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Absolute path to the committed multi-resolution icon, forward-slashed so
    // it is valid for both rc.exe (MSVC) and windres (GNU).
    let ico_path = manifest_dir.join("assets/icon/bsargeom.ico");
    let ico_path_str = ico_path.to_string_lossy().replace('\\', "/");

    // Version parts from Cargo.toml (e.g. "1.2.0" -> 1,2,0).
    let version = env!("CARGO_PKG_VERSION");
    let parts: Vec<u32> = version.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    let major = parts.first().copied().unwrap_or(0);
    let minor = parts.get(1).copied().unwrap_or(0);
    let patch = parts.get(2).copied().unwrap_or(0);
    let version_str = format!("{version}.0"); // "1.2.0.0"

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
            VALUE "FileDescription", "BSAR Geometry visualizer"
            VALUE "ProductVersion",  "{version_str}"
            VALUE "FileVersion",     "{version_str}"
        }}
    }}
    BLOCK "VarFileInfo"
    {{
        VALUE "Translation", 0x0, 0x04b0
    }}
}}
"#
    );

    let rc_path = out_dir.join("bsargeom.rc");
    fs::write(&rc_path, rc_content).expect("Failed to write bsargeom.rc");

    // embed-resource uses windres for *-pc-windows-gnu and rc.exe for
    // *-pc-windows-msvc. Override the default GNU cross-prefixed windres with the
    // plain binary on MSYS2/ucrt64 toolchains.
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "gnu" && std::env::var_os("WINDRES").is_none() {
        // SAFETY: build scripts are single-threaded.
        unsafe {
            std::env::set_var("WINDRES", "windres");
        }
    }

    embed_resource::compile(&rc_path, embed_resource::NONE)
        .manifest_optional()
        .expect("Failed to embed the Windows icon/version resource");

    println!("cargo:rerun-if-changed=assets/icon/bsargeom.ico");
    println!("cargo:rerun-if-changed=build.rs");
}
