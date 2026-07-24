use std::{
    fs::{copy, create_dir_all, remove_dir_all},
    path::PathBuf,
    process::Command,
};

use crate::{arch_of, dist_target, run};

pub(crate) fn build_and_package() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?.canonicalize()?;
    // `cross` sets CARGO_TARGET_DIR; read artifacts from wherever cargo placed
    // them. Build the app for the requested triple (`--target` also places
    // artifacts under target/<triple>/release) so a `cross`/cross-compile run
    // does not silently produce host-arch binaries.
    let target = dist_target();
    let target = target.as_str();
    let arch = arch_of(target);
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.join("target"));
    let tmp_dir = target_dir.join("tmp");
    let app_dir = tmp_dir.join("bsargeom");
    let bin_dir = app_dir.join("bin");
    let dist_dir = target_dir.join("dist");
    let release_dir = target_dir.join(target).join("release");
    let install_dir = cwd.join("install/linux");
    let makeself_dir = install_dir.join("makeself");
    let version = env!("APP_VERSION");

    println!("Current working directory: {}", cwd.display());
    println!("Building BSARGeom in release mode...\n$ cargo build --release --target {target}");
    run(Command::new("cargo").args(["build", "--release", "--target", target]))?;

    // Stage the application folder: bin/ (installed to PATH) + the setup script,
    // desktop entry and icon (installed to the desktop menu by setup).
    create_dir_all(&bin_dir)?;
    create_dir_all(&dist_dir)?;
    copy(release_dir.join("bsargeom"), bin_dir.join("bsargeom"))?;
    copy(
        install_dir.join("uninstall-bsargeom"),
        bin_dir.join("uninstall-bsargeom"),
    )?;
    copy(cwd.join("LICENSE"), app_dir.join("LICENSE"))?;
    copy(cwd.join("README.md"), app_dir.join("README.md"))?;
    copy(cwd.join("CHANGELOG.md"), app_dir.join("CHANGELOG.md"))?;
    copy(install_dir.join("setup"), app_dir.join("setup"))?;
    copy(
        install_dir.join("bsargeom.desktop"),
        app_dir.join("bsargeom.desktop"),
    )?;
    copy(
        cwd.join("assets/icon/bsargeom-256.png"),
        app_dir.join("bsargeom.png"),
    )?;
    // makeself scripts into the tmp dir (run from there).
    copy(makeself_dir.join("makeself.sh"), tmp_dir.join("makeself.sh"))?;
    copy(
        makeself_dir.join("makeself-header.sh"),
        tmp_dir.join("makeself-header.sh"),
    )?;

    // makeself runs `./setup` from the extracted archive; ensure it and the
    // installed binaries carry the executable bit regardless of the checkout's
    // recorded git permissions.
    run(Command::new("chmod")
        .arg("+x")
        .arg(app_dir.join("setup"))
        .arg(bin_dir.join("bsargeom"))
        .arg(bin_dir.join("uninstall-bsargeom")))?;

    // Self-extracting .run installer.
    let name_installer = format!("BSARGeom-v{version}-{arch}-linux-installer.run");
    let label = format!("BSARGeom v{version} Installer");
    println!(
        "Building the self-extracting installer...\n\
         $ ./makeself.sh bsargeom {name_installer} \"{label}\" ./setup"
    );
    run(Command::new("bash")
        .current_dir(&tmp_dir)
        .args(["makeself.sh", "bsargeom", &name_installer, &label, "./setup"]))?;
    copy(tmp_dir.join(&name_installer), dist_dir.join(&name_installer))?;

    // Plain tar.gz archive (preserves the executable bit).
    let name_archive = format!("BSARGeom-v{version}-{arch}-linux-archive.tar.gz");
    println!("Building archive...\n$ tar -czf {name_archive} bsargeom");
    run(Command::new("tar")
        .current_dir(&tmp_dir)
        .args(["-czf", &name_archive, "bsargeom"]))?;
    copy(tmp_dir.join(&name_archive), dist_dir.join(&name_archive))?;

    remove_dir_all(&tmp_dir)?;

    println!(
        "\nBSARGeom installer and archive are available in {}",
        dist_dir.display()
    );
    Ok(())
}
