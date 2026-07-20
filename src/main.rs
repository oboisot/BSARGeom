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
    App::new()
        .insert_resource(ClearColor(Color::BLACK)) 
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    position: WindowPosition::Automatic,
                    title: "BSAR Geometry visualizer".to_string(),
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
        .add_plugins(ScenePlugin)
        .run();
}
