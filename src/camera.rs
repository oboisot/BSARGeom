use std::f32::consts::FRAC_PI_4;

use bevy::{
    prelude::*,
    window::PrimaryWindow
};
use bevy_egui::EguiStartupSet;
use bevy_panorbit_camera::{EguiFocusIncludesHover, PanOrbitCamera};

use crate::ui::SidePanelRects;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EguiFocusIncludesHover(true)) // blocks the camera on hover over egui *windows*
                                                          // (see: https://github.com/Plonq/bevy_panorbit_camera/issues/75)
            .add_systems( // see: https://github.com/vladbat00/bevy_egui/blob/main/examples/ui.rs
                PreStartup,
                spawn_camera.before(EguiStartupSet::InitContexts),
            )
            .add_systems(Update, block_camera_over_panels);
    }
}

/// Disables the camera while the pointer is over a side panel.
///
/// egui cannot report panels laid out on the background layer through
/// `Context::is_pointer_over_area` (only floating areas like windows register
/// there), so `bevy_panorbit_camera`'s built-in egui focus check does not see
/// them: the camera would orbit/zoom while dragging or scrolling inside a
/// panel. The panel extents are exported by the UI system ([`SidePanelRects`])
/// and compared against the cursor position, both in logical points.
fn block_camera_over_panels(
    window_q: Query<&Window, With<PrimaryWindow>>,
    side_panel_rects: Res<SidePanelRects>,
    mut pan_orbit_camera_q: Query<&mut PanOrbitCamera>,
) {
    let Ok(window) = window_q.single() else { return; };
    let over_panel = window.cursor_position().is_some_and(|pos|
        pos.x <= side_panel_rects.left_max_x ||
        pos.x >= side_panel_rects.right_min_x
    );
    for mut pan_orbit_camera in pan_orbit_camera_q.iter_mut() {
        if pan_orbit_camera.enabled == over_panel { // Avoids triggering change detection every frame
            pan_orbit_camera.enabled = !over_panel;
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    // Camera
    commands.spawn((
        // Note we're setting the initial position below with yaw, pitch, and radius, hence
        // we don't set transform on the camera.
        PanOrbitCamera {
            // Set focal point (what the camera should look at)
            focus: Vec3::ZERO,
            // Set the starting position, relative to focus (overrides camera's transform).
            yaw: Some(-FRAC_PI_4),
            pitch: Some(FRAC_PI_4),
            radius: Some(25980.76211353316), // = sqrt(HALF_PLANE_SIZE**2 * 3)
            // Set limits on rotation and zoom
            yaw_upper_limit: None,
            yaw_lower_limit: None,
            pitch_upper_limit: Some(89f32.to_radians()),
            pitch_lower_limit: Some(-10f32.to_radians()),
            // Adjust sensitivity of controls
            orbit_sensitivity: 1.5,
            pan_sensitivity: 1.0, // = 0.0 Disable panning
            zoom_sensitivity: 1.0,
            // Allow the camera to go upside down
            allow_upside_down: false,
            ..default()
        },
        Msaa::default(), // MSAA,
    ));
}
