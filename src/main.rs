#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
// Bevy ECS queries and system/spawn functions are inherently wide; these two
// pedantic thresholds fight the engine's idioms (Bevy itself allows them).
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

pub mod bsar;
pub mod camera;
pub mod constants;
pub mod contour;
pub mod coordinates;
pub mod download;
pub mod entities;
pub mod raster;
pub mod scene;
pub mod textdraw;
pub mod ui;
pub mod world;

use scene::ScenePlugin;
use ui::AppPlugin;

fn main() {
    let mut app = App::new();
    app
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    position: WindowPosition::Automatic,
                    title: "BSAR Geometry visualizer".to_string(),
                    // Wayland app_id / X11 WM_CLASS. Matches the installed
                    // `bsargeom.desktop` (StartupWMClass) so the desktop shell
                    // associates the launcher icon with the running window.
                    name: Some("bsargeom".to_string()),
                    // Tells Wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }))           
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(AppPlugin)
        .add_plugins(ScenePlugin);

    // Set the native window icon (X11 title bar / taskbar when running the bare
    // binary; Windows already gets it from the exe-embedded resource and macOS
    // from the .app bundle). No-op on the web, which uses the page favicon.
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(window_icon::WindowIconPlugin);

    app.run();
}

/// Applies the embedded application icon to the primary winit window.
///
/// `bevy_winit` 0.19 keeps its windows in a main-thread `WINIT_WINDOWS`
/// thread-local (no longer a `NonSend` resource), so the system must run on the
/// main thread — a `NonSend` parameter forces that even with the multithreaded
/// executor. It runs each `Update` until it finds the window (created during the
/// first winit iteration) and applies the icon exactly once.
#[cfg(not(target_arch = "wasm32"))]
mod window_icon {
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;
    use bevy::winit::WINIT_WINDOWS;

    /// Zero-sized `NonSend` marker whose only purpose is to pin [`set_window_icon`]
    /// to the main thread.
    struct MainThreadPin;

    pub struct WindowIconPlugin;

    impl Plugin for WindowIconPlugin {
        fn build(&self, app: &mut App) {
            app.insert_non_send(MainThreadPin);
            app.add_systems(Update, set_window_icon);
        }
    }

    fn set_window_icon(
        _pin: NonSend<MainThreadPin>,
        primary: Query<Entity, With<PrimaryWindow>>,
        mut done: Local<bool>,
    ) {
        if *done {
            return;
        }
        let Ok(entity) = primary.single() else {
            return;
        };

        // Decode the embedded PNG (the `image` crate is already a dependency).
        let image = match image::load_from_memory(include_bytes!(
            "../assets/icon/bsargeom-256.png"
        )) {
            Ok(image) => image.into_rgba8(),
            Err(err) => {
                eprintln!("failed to decode window icon: {err}");
                *done = true; // don't retry a permanent failure every frame
                return;
            }
        };
        let (width, height) = image.dimensions();
        let Ok(icon) = winit::window::Icon::from_rgba(image.into_raw(), width, height) else {
            *done = true;
            return;
        };

        WINIT_WINDOWS.with_borrow(|winit_windows| {
            if let Some(window) = winit_windows.get_window(entity) {
                window.set_window_icon(Some(icon));
                *done = true;
            }
        });
    }
}
