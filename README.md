# BSARGeom
 A new [BSARConf](https://github.com/oboisot/BSARConf) but improved with [Rust](https://www.rust-lang.org/) / [Bevy](https://github.com/bevyengine/bevy) / [egui](https://github.com/emilk/egui)

**Live demo:** <https://oboisot.github.io/BSARGeom/> (requires a WebGPU-capable browser)


## Desktop application

Prebuilt installers and archives for Windows, Linux and macOS are attached to
each [release](https://github.com/oboisot/BSARGeom/releases):

- **Windows** — run `BSARGeom-*-windows-installer.exe` (installs to
  `%LOCALAPPDATA%\BSARGeom` with Start-Menu and Desktop shortcuts), or unzip the
  `*-windows-archive.zip` and run `bsargeom.exe` directly.
- **Linux** — run `./BSARGeom-*-linux-installer.run` (installs `bsargeom` to
  `~/.local/bin` or, with `sudo`, `/usr/local/bin`, plus a desktop-menu entry),
  or extract the `*-linux-archive.tar.gz` and run `./setup`.
- **macOS** — extract `BSARGeom-*-macos.tar.gz` and move `BSARGeom.app` to
  `/Applications`. The bundle is **unsigned**, so on first launch Gatekeeper
  blocks it: right-click the app and choose *Open*, or run
  `xattr -dr com.apple.quarantine BSARGeom.app`.

To build the installer/archive for the current platform yourself:
```sh
cargo xtask dist   # artifacts are written to target/dist
```


## Building for the Web
The easiest way to build the application for the Web, i.e. WASM build, is to use the [bevy CLI](https://github.com/TheBevyFlock/bevy_cli)

Installing from latest Github repo (main branch):
```sh
cargo install --git https://github.com/TheBevyFlock/bevy_cli bevy_cli
```

Building and bundling:
```sh
bevy build --yes --release web --bundle --wasm-opt=true
```

Building/Running/Opening in the webbrowser:
```sh
bevy run --release web --open
```