use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
// use bevy_inspector_egui::{
//     bevy_egui::EguiPlugin,
//     quick::WorldInspectorPlugin
// };

pub mod camera;
pub mod world;
pub mod constants;
pub mod entities;

// use camera::CameraPlugin;
mod scene;
use scene::ScenePlugin;
// use world::WorldPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK)) 
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    position: WindowPosition::Automatic,
                    title: "BSAR Geometry visualizer [based on RustSAR]".to_string(),
                    ..default()
                }),
                ..default()
            }))
        // .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
        // .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(ScenePlugin)
        .run();
}
