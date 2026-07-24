// NOTE: this module is compiled on every host (main.rs declares `mod macos`
// unconditionally), so it must stay free of `std::os::unix` imports — the
// executable bit is set at run time with `chmod`, which only ever runs on macOS.

use std::{
    fs::{copy, create_dir_all, remove_dir_all, write},
    path::PathBuf,
    process::Command,
};

use crate::{arch_of, dist_target, run};

pub(crate) fn build_and_package() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?.canonicalize()?;
    let target = dist_target();
    let target = target.as_str();
    let arch = arch_of(target);
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.join("target"));
    let tmp_dir = target_dir.join("tmp");
    let bundle = tmp_dir.join("BSARGeom.app");
    let macos_dir = bundle.join("Contents/MacOS");
    let resources_dir = bundle.join("Contents/Resources");
    let dist_dir = target_dir.join("dist");
    let release_dir = target_dir.join(target).join("release");
    let version = env!("APP_VERSION");

    println!("Current working directory: {}", cwd.display());
    println!("Building BSARGeom in release mode...\n$ cargo build --release --target {target}");
    run(Command::new("cargo").args(["build", "--release", "--target", target]))?;

    // Assemble the .app bundle.
    create_dir_all(&macos_dir)?;
    create_dir_all(&resources_dir)?;
    create_dir_all(&dist_dir)?;
    let exe = macos_dir.join("bsargeom");
    copy(release_dir.join("bsargeom"), &exe)?;
    // `copy` preserves the source's executable bit on macOS; make it explicit.
    run(Command::new("chmod").arg("+x").arg(&exe))?;
    copy(
        cwd.join("assets/icon/bsargeom.icns"),
        resources_dir.join("bsargeom.icns"),
    )?;
    write(bundle.join("Contents/Info.plist"), info_plist(version))?;
    write(bundle.join("Contents/PkgInfo"), "APPL????")?;

    // Archive the bundle (tar.gz preserves bundle structure and the exec bit).
    let name_archive = format!("BSARGeom-v{version}-{arch}-macos.tar.gz");
    println!("Building archive...\n$ tar -czf {name_archive} BSARGeom.app");
    run(Command::new("tar")
        .current_dir(&tmp_dir)
        .args(["-czf", &name_archive, "BSARGeom.app"]))?;
    copy(tmp_dir.join(&name_archive), dist_dir.join(&name_archive))?;

    remove_dir_all(&tmp_dir)?;

    println!(
        "\nBSARGeom .app archive is available in {}\n\
         Note: the bundle is unsigned. On first launch macOS Gatekeeper will\n\
         block it; right-click the app and choose Open, or run\n\
         `xattr -dr com.apple.quarantine BSARGeom.app`.",
        dist_dir.display()
    );
    Ok(())
}

fn info_plist(version: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>BSARGeom</string>
    <key>CFBundleDisplayName</key>
    <string>BSARGeom</string>
    <key>CFBundleIdentifier</key>
    <string>com.oboisot.bsargeom</string>
    <key>CFBundleExecutable</key>
    <string>bsargeom</string>
    <key>CFBundleIconFile</key>
    <string>bsargeom</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>{version}</string>
    <key>CFBundleVersion</key>
    <string>{version}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
</dict>
</plist>
"#
    )
}
