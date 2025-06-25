use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

pub mod camera;
pub mod constants;
pub mod entities;
pub mod scene;
pub mod ui;
pub mod world;

use scene::ScenePlugin;
use ui::UiPlugin;

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
        .add_plugins(UiPlugin)    
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(ScenePlugin)
        .run();
}


// fn tx_ui(
//     mut contexts: EguiContexts,
// ) {
//     let ctx = contexts.ctx_mut();

//     egui::SidePanel::left("Transmitter")
//         .resizable(true)
//         .show(ctx, |ui| {
//             ui.separator();
//             ui.label("CARRIER SETTINGS");
            
//             ui.separator();

//             ui.label("ANTENNA SETTINGS");

//             ui.label("Orientation");

//             ui.label("Beamwidth");
//             ui.separator();

//         });
// }
