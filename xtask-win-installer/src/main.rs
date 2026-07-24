//! Windows installer executable: unpacks the embedded BSARGeom folder to a temp
//! directory and launches its `InstallOrUninstall.bat` menu. The folder content
//! is embedded at build time from `../target/tmp/BSARGeom/` (populated by
//! `cargo xtask dist`; see xtask/src/windows.rs).

use rust_embed::Embed;
use tempfile::tempdir;

#[derive(Embed)]
#[folder = "../target/tmp/BSARGeom/"]
struct Asset;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempdir()?;
    let tmp_dir = tempdir.path();

    // Unpack the embedded files.
    for file in Asset::iter() {
        let name = file.as_ref();
        if let Some(embedded) = Asset::get(name) {
            std::fs::write(tmp_dir.join(name), embedded.data)?;
        } else {
            eprintln!("embedded file {name} not found!");
        }
    }

    // Launch the install/uninstall menu (`/K` keeps the console open).
    std::process::Command::new("cmd")
        .current_dir(tmp_dir)
        .arg("/K")
        .arg("InstallOrUninstall.bat")
        .spawn()?
        .wait()?;

    Ok(())
}
