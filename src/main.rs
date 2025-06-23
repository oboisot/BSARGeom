use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use camera::CameraPlugin;
use carrier::CarriersPlugin;
use world::WorldPlugin;

mod camera;
mod carrier;
mod world;

pub mod constants;
pub mod entities;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK)) 
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        position: WindowPosition::Automatic,
                        title: "RustSAR Geometry visualizer".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
            // MeshPickingPlugin,
            PanOrbitCameraPlugin,
            CameraPlugin,
            WorldPlugin,
            CarriersPlugin,
        ))
        .run();
}
