use std::{
    fs::{copy, create_dir_all, remove_dir_all},
    path::PathBuf,
    process::Command,
};

use crate::run;

pub(crate) fn build_and_package() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?.canonicalize()?;
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.join("target"));
    let tmp_dir = target_dir.join("tmp");
    let app_dir = tmp_dir.join("BSARGeom");
    let dist_dir = target_dir.join("dist");
    let release_dir = target_dir.join("release");
    let install_dir = cwd.join("install/windows");
    let version = env!("APP_VERSION");

    println!("Current working directory: {}", cwd.display());

    // Build the app (default-members = ["."], so this is only bsargeom).
    println!("Building BSARGeom in release mode...\n$ cargo build --release");
    run(Command::new("cargo").args(["build", "--release"]))?;

    // Stage the application folder.
    create_dir_all(&app_dir)?;
    create_dir_all(&dist_dir)?;
    copy(release_dir.join("bsargeom.exe"), app_dir.join("bsargeom.exe"))?;
    copy(cwd.join("LICENSE"), app_dir.join("LICENSE"))?;
    copy(cwd.join("README.md"), app_dir.join("README.md"))?;
    copy(cwd.join("CHANGELOG.md"), app_dir.join("CHANGELOG.md"))?;
    copy(install_dir.join("Install.ps1"), app_dir.join("Install.ps1"))?;
    copy(install_dir.join("Uninstall.ps1"), app_dir.join("Uninstall.ps1"))?;
    copy(
        install_dir.join("InstallOrUninstall.bat"),
        app_dir.join("InstallOrUninstall.bat"),
    )?;

    // Plain zip archive (Windows `tar` is bsdtar; `-a` picks zip from the .zip
    // extension).
    let name_archive = format!("BSARGeom-v{version}-x86_64-windows-archive.zip");
    println!("Building archive...\n$ tar -acf {name_archive} BSARGeom");
    run(Command::new("tar")
        .current_dir(&tmp_dir)
        .args(["-a", "-c", "-f", &name_archive, "BSARGeom"]))?;
    copy(tmp_dir.join(&name_archive), dist_dir.join(&name_archive))?;

    // Self-contained .exe installer (embeds the staged BSARGeom folder via
    // rust-embed — see xtask-win-installer).
    let name_installer = format!("BSARGeom-v{version}-x86_64-windows-installer.exe");
    println!(
        "Building the executable installer...\n\
         $ cargo build --release --package xtask-win-installer"
    );
    run(Command::new("cargo").args(["build", "--release", "--package", "xtask-win-installer"]))?;
    copy(
        release_dir.join("xtask-win-installer.exe"),
        dist_dir.join(&name_installer),
    )?;

    remove_dir_all(&tmp_dir)?;

    println!(
        "\nBSARGeom archive and installer are available in {}",
        dist_dir.display()
    );
    Ok(())
}
