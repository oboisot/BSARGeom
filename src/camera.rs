use std::f32::consts::FRAC_PI_4;

use bevy::{
    prelude::*,
    render::view::NoIndirectDrawing
};
use bevy_panorbit_camera::PanOrbitCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
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
            // Set the camera's up direction to Z-up. See: https://github.com/Plonq/bevy_panorbit_camera/blob/master/examples/swapped_axis.rs
            ..default()
        },
        Msaa::default(), // MSAA,
        NoIndirectDrawing, // disable indirect mode to allow correct rendering on integrated Intel GPU (see: https://github.com/bevyengine/bevy/issues/19000)
                           // TODO: remove this when bug will be corrected/handled
    ));
}
