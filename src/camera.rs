use std::f32::consts::{FRAC_PI_3, FRAC_PI_6};

use bevy::prelude::*;
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
            yaw: Some(-FRAC_PI_3),
            pitch: Some(FRAC_PI_6),
            radius: Some(15_000.0),
            // // Set limits on rotation and zoom
            // yaw_upper_limit: Some(TAU / 4.0),
            // yaw_lower_limit: Some(-TAU / 4.0),
            // pitch_upper_limit: Some(TAU / 3.0),
            // pitch_lower_limit: Some(-TAU / 3.0),
            // zoom_upper_limit: Some(5.0),
            // zoom_lower_limit: 1.0,
            // // Adjust sensitivity of controls
            orbit_sensitivity: 1.5,
            pan_sensitivity: 1.5,
            zoom_sensitivity: 0.75,
            // // Allow the camera to go upside down
            // allow_upside_down: true,
            // // Change the controls (these match Blender)
            // button_orbit: MouseButton::Middle,
            // button_pan: MouseButton::Middle,
            // modifier_pan: Some(KeyCode::ShiftLeft),
            // // Reverse the zoom direction
            // reversed_zoom: true,
            // // Use alternate touch controls
            // touch_controls: TouchControls::TwoFingerOrbit,
            ..default()
        },
    ));
}
