use std::f32::consts::FRAC_PI_4;

use bevy::{
    prelude::*,
    window::PrimaryWindow
};
use bevy_egui::EguiStartupSet;
use bevy_panorbit_camera::{EguiFocusIncludesHover, PanOrbitCamera};

use crate::{
    entities::Carrier,
    scene::{Rx, Tx},
    ui::{CameraFocus, MenuWidget, SidePanelRects},
};

/// Initial camera viewpoint, also the target of the menu "reset view" button.
const INITIAL_YAW_RAD: f32 = -FRAC_PI_4;
const INITIAL_PITCH_RAD: f32 = FRAC_PI_4;
const INITIAL_RADIUS_M: f32 = 25_980.762; // = sqrt(HALF_PLANE_SIZE**2 * 3)

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EguiFocusIncludesHover(true)) // blocks the camera on hover over egui *windows*
                                                          // (see: https://github.com/Plonq/bevy_panorbit_camera/issues/75)
            .add_systems( // see: https://github.com/vladbat00/bevy_egui/blob/main/examples/ui.rs
                PreStartup,
                spawn_camera.before(EguiStartupSet::InitContexts),
            )
            .add_systems(Update, (block_camera_over_panels, update_camera_focus));
    }
}

/// Keeps the camera focused on the point selected in the menu (ground origin
/// or one of the carriers — following it when its parameters move it), and
/// consumes the one-shot "reset view" request by restoring the initial
/// viewpoint. `force_update` makes the plugin animate the change even while
/// camera input is disabled (e.g. the pointer still hovering the menu).
pub(crate) fn update_camera_focus(
    mut menu_widget: ResMut<MenuWidget>,
    tx_carrier_q: Query<&Transform, (With<Tx>, With<Carrier>)>,
    rx_carrier_q: Query<&Transform, (With<Rx>, With<Carrier>)>,
    mut pan_orbit_camera_q: Query<&mut PanOrbitCamera>,
) {
    // `Free` leaves the focus point alone so panning keeps working; the other
    // variants pin it (and therefore override any pan).
    let target_focus = match menu_widget.camera_focus {
        CameraFocus::Free => None,
        CameraFocus::Ground => Some(Vec3::ZERO),
        CameraFocus::Tx => Some(tx_carrier_q.single().map_or(Vec3::ZERO, |t| t.translation)),
        CameraFocus::Rx => Some(rx_carrier_q.single().map_or(Vec3::ZERO, |t| t.translation)),
    };
    let reset_view = menu_widget.reset_view_requested;
    if reset_view {
        menu_widget.reset_view_requested = false; // written only when set: avoids spurious change detection
    }
    for mut pan_orbit_camera in pan_orbit_camera_q.iter_mut() {
        if let Some(target_focus) = target_focus
            && pan_orbit_camera.target_focus != target_focus {
                pan_orbit_camera.target_focus = target_focus;
                pan_orbit_camera.force_update = true;
            }
        if reset_view {
            pan_orbit_camera.target_focus = Vec3::ZERO;
            pan_orbit_camera.target_yaw = INITIAL_YAW_RAD;
            pan_orbit_camera.target_pitch = INITIAL_PITCH_RAD;
            pan_orbit_camera.target_radius = INITIAL_RADIUS_M;
            pan_orbit_camera.force_update = true;
        }
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
            yaw: Some(INITIAL_YAW_RAD),
            pitch: Some(INITIAL_PITCH_RAD),
            radius: Some(INITIAL_RADIUS_M),
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
