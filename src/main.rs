use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use camera::CameraPlugin;
use world::WorldPlugin;

mod camera;
mod world;

pub mod constants;
pub mod entities;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        position: WindowPosition::Automatic,
                        resolution: [800.0, 600.0].into(),
                        title: "RustSAR Geometry visualizer".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
            PanOrbitCameraPlugin,
            WorldPlugin,
            CameraPlugin,
        ))
        .run();
}
