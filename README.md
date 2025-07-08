# BSARGeom
 A new [BSARConf](https://github.com/oboisot/BSARConf) but improved with [Rust](https://www.rust-lang.org/) / [Bevy](https://github.com/bevyengine/bevy) / [egui](https://github.com/emilk/egui)


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