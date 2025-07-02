#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

pub mod camera;
pub mod constants;
pub mod entities;
pub mod scene;
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
                    ..default()
                }),
                ..default()
            }))           
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(AppPlugin) 
        .add_plugins(ScenePlugin)
        .run();
}
