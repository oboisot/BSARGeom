//! The 3D view camera: Bevy's built-in pan-orbit controller, driven by the
//! app's own mouse input layer ([`input`]), with the menu's focus and reset
//! commands and the pitch range applied on top.

mod input;

use std::{f32::consts::FRAC_PI_4, time::Duration};

use bevy::{
    camera_controller::pan_orbit_camera::prelude::{
        momentum::Momentum, motion::CurrentMotion, projections::PerspectiveSettings,
        MinimalPanOrbitCameraPlugin, OrbitConstraint, PanOrbitCamera,
    },
    math::DVec3,
    prelude::*,
    transform::TransformSystems,
};
use bevy_egui::EguiStartupSet;

use crate::{
    entities::Carrier,
    scene::{Rx, Tx},
    ui::{CameraFocus, MenuWidget},
};

/// Initial camera viewpoint, also the target of the menu "reset view" button.
const INITIAL_YAW_RAD: f32 = -FRAC_PI_4;
const INITIAL_PITCH_RAD: f32 = FRAC_PI_4;
const INITIAL_RADIUS_M: f32 = 25_980.762; // = sqrt(HALF_PLANE_SIZE**2 * 3)

/// Pitch range: the camera's elevation above the horizon, seen from the point
/// it orbits — from slightly under the ground plane to almost overhead.
const MIN_PITCH_RAD: f32 = -10f32.to_radians();
const MAX_PITCH_RAD: f32 = 89f32.to_radians();

/// Time constant of the camera's glides to a newly pinned focus and back to
/// the initial view.
const GLIDE_TIME_CONSTANT_S: f32 = 0.1;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MinimalPanOrbitCameraPlugin, input::CameraInputPlugin))
            .add_systems( // see: https://github.com/vladbat00/bevy_egui/blob/main/examples/ui.rs
                PreStartup,
                spawn_camera.before(EguiStartupSet::InitContexts),
            )
            .add_systems(
                PostUpdate,
                // Once the carriers have moved (Update), before the frame's
                // transforms are propagated
                (update_camera_focus, clamp_camera_pitch)
                    .chain()
                    .before(TransformSystems::Propagate),
            );
    }
}

/// Applies the menu's camera commands on top of the user's control:
/// - a pinned focus (ground origin, Tx or Rx carrier) keeps that point at the
///   centre of the view, at the current viewing distance, following a carrier
///   when its parameters move it. Panning is disabled meanwhile; orbiting and
///   zooming then revolve around the pinned point;
/// - the one-shot "reset view" request glides the camera back to its initial
///   viewpoint, unless the user takes over the camera or pins a focus first.
fn update_camera_focus(
    mut menu_widget: ResMut<MenuWidget>,
    tx_carrier_q: Query<&Transform, (With<Tx>, With<Carrier>, Without<PanOrbitCamera>)>,
    rx_carrier_q: Query<&Transform, (With<Rx>, With<Carrier>, Without<PanOrbitCamera>)>,
    mut camera_q: Query<(&mut PanOrbitCamera, &mut Transform)>,
    time: Res<Time<Real>>,
    mut resetting: Local<bool>,
) {
    let target_focus = match menu_widget.camera_focus {
        CameraFocus::Free => None,
        CameraFocus::Ground => Some(Vec3::ZERO),
        CameraFocus::Tx => Some(tx_carrier_q.single().map_or(Vec3::ZERO, |t| t.translation)),
        CameraFocus::Rx => Some(rx_carrier_q.single().map_or(Vec3::ZERO, |t| t.translation)),
    };
    let reset_requested = menu_widget.reset_view_requested;
    if reset_requested {
        menu_widget.reset_view_requested = false; // written only when set: avoids spurious change detection
    }
    let Ok((mut controller, mut transform)) = camera_q.single_mut() else { return; };

    // Panning would fight a pinned focus
    let pan_enabled = target_focus.is_none();
    if controller.enabled_motion.pan != pan_enabled {
        controller.enabled_motion.pan = pan_enabled;
    }

    if reset_requested {
        controller.current_motion = CurrentMotion::Stationary;
        controller.last_anchor_depth = -f64::from(INITIAL_RADIUS_M);
        *resetting = true;
    }
    if *resetting && (controller.current_motion.is_user_controlled() || target_focus.is_some()) {
        *resetting = false; // the user took over
    }

    let goal = if *resetting {
        initial_camera_transform()
    } else if let Some(target) = target_focus {
        let distance = controller.last_anchor_depth().abs() as f32;
        transform.with_translation(target - transform.forward() * distance)
    } else {
        return;
    };
    // Fraction of the remaining way covered this frame, whatever the frame rate
    let fraction = 1.0 - (-time.delta_secs() / GLIDE_TIME_CONSTANT_S).exp();
    match glide(&transform, &goal, fraction) {
        Some(next) => *transform = next,
        None => *resetting = false,
    }
}

/// One frame of a glide of `from` toward the `goal` pose: `fraction` of the
/// remaining way, snapped onto the goal once the difference no longer shows.
/// `None` once there, i.e. nothing to write.
fn glide(from: &Transform, goal: &Transform, fraction: f32) -> Option<Transform> {
    if from.translation == goal.translation && from.rotation == goal.rotation {
        return None;
    }
    // A hundred-thousandth of the distance from the origin, i.e. far below a
    // pixel: also keeps the glide from creeping on in f32 rounding steps
    let tolerance = 1e-5 * goal.translation.length().max(1.0);
    if from.translation.distance(goal.translation) <= tolerance
        && from.rotation.angle_between(goal.rotation) <= 1e-4
    {
        return Some(*goal);
    }
    Some(Transform {
        translation: from.translation.lerp(goal.translation, fraction),
        rotation: from.rotation.slerp(goal.rotation, fraction),
        ..*from
    })
}

/// Keeps the camera's pitch within [`MIN_PITCH_RAD`, `MAX_PITCH_RAD`] by
/// rotating it about the point it orbits: the built-in controller only stops
/// short of the poles.
fn clamp_camera_pitch(mut camera_q: Query<(&PanOrbitCamera, &mut Transform)>) {
    for (controller, mut transform) in &mut camera_q {
        let forward = transform.forward();
        let pitch = (-forward.y).clamp(-1.0, 1.0).asin();
        let clamped = pitch.clamp(MIN_PITCH_RAD, MAX_PITCH_RAD);
        if clamped == pitch {
            continue;
        }
        let distance = controller.last_anchor_depth().abs() as f32;
        let focus = transform.translation + forward * distance;
        // The controller keeps the camera level, so its right axis is horizontal
        let right = transform.right();
        let yaw = (-right.z).atan2(right.x);
        *transform = orbit_transform(focus, yaw, clamped, distance);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        initial_camera_transform(),
        PanOrbitCamera {
            // Level horizon, never flipping over the top (`clamp_camera_pitch`
            // narrows the pitch range further)
            orbit_constraint: OrbitConstraint::Fixed { up: DVec3::Y, can_pass_tdc: false },
            // No momentum: a released drag stops where it is
            momentum: Momentum {
                init_orbit: Duration::ZERO,
                init_pan: Duration::ZERO,
                ..default()
            },
            // Keep Bevy's default near plane: the controller would otherwise
            // move it to 5% of the focus distance (1.3 km in the initial view),
            // clipping whatever comes closer to the camera
            perspective: PerspectiveSettings {
                near_clip_limits: 0.1..0.1,
                ..default()
            },
            ..default()
        }
        .with_initial_anchor_depth(f64::from(INITIAL_RADIUS_M)),
        Msaa::default(), // MSAA
    ));
}

/// The initial viewpoint, also restored by the menu's "reset view" button.
fn initial_camera_transform() -> Transform {
    orbit_transform(Vec3::ZERO, INITIAL_YAW_RAD, INITIAL_PITCH_RAD, INITIAL_RADIUS_M)
}

/// Camera pose orbiting `focus` at `radius`, turned `yaw` about the vertical
/// and raised `pitch` above the horizon. This is `bevy_panorbit_camera`'s
/// convention, which the app used before Bevy's built-in controller, so the
/// initial view is unchanged.
fn orbit_transform(focus: Vec3, yaw: f32, pitch: f32, radius: f32) -> Transform {
    let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(-pitch);
    Transform::from_translation(focus + rotation * Vec3::new(0.0, 0.0, radius))
        .with_rotation(rotation)
}

#[cfg(test)]
mod tests {
    use bevy::time::TimeUpdateStrategy;

    use super::*;

    const TX_POSITION: Vec3 = Vec3::new(3000.0, 6000.0, -8000.0);

    /// Headless App running the focus, reset and pitch systems at 60 frames
    /// per (simulated) second, on a camera at the initial viewpoint and with a
    /// Tx carrier.
    fn camera_app() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)))
            .init_resource::<MenuWidget>()
            .add_systems(PostUpdate, (update_camera_focus, clamp_camera_pitch).chain());
        app.world_mut().spawn((Transform::from_translation(TX_POSITION), Tx, Carrier));
        let camera = app
            .world_mut()
            .spawn((
                initial_camera_transform(),
                PanOrbitCamera::default().with_initial_anchor_depth(f64::from(INITIAL_RADIUS_M)),
            ))
            .id();
        (app, camera)
    }

    fn run_frames(app: &mut App, frames: usize) {
        for _ in 0..frames {
            app.update();
        }
    }

    fn camera_transform(app: &App, camera: Entity) -> Transform {
        *app.world().get::<Transform>(camera).unwrap()
    }

    #[test]
    fn initial_view_is_the_former_panorbit_one() {
        let transform = initial_camera_transform();
        assert!(
            transform.translation.abs_diff_eq(Vec3::new(-12_990.38, 18_371.17, 12_990.38), 0.05),
            "{}",
            transform.translation
        );
        // Looking at the ground origin, horizon level
        assert!(transform.forward().dot(-transform.translation.normalize()) > 1.0 - 1e-6);
        assert!(transform.right().y.abs() < 1e-6);
    }

    #[test]
    fn pinned_focus_centres_its_target_at_the_viewing_distance() {
        let (mut app, camera) = camera_app();
        app.world_mut().resource_mut::<MenuWidget>().camera_focus = CameraFocus::Tx;
        run_frames(&mut app, 120); // 2 s: many glide time constants
        let transform = camera_transform(&app, camera);
        let focus = transform.translation + transform.forward() * INITIAL_RADIUS_M;
        assert!(focus.distance(TX_POSITION) < 1.0, "the view centre is {focus}");
        assert!(
            transform.rotation.angle_between(initial_camera_transform().rotation) < 1e-4,
            "pinning a focus must not turn the camera"
        );
        assert!(!app.world().get::<PanOrbitCamera>(camera).unwrap().enabled_motion.pan);

        // Freed again: panning comes back, the camera stays where it is
        app.world_mut().resource_mut::<MenuWidget>().camera_focus = CameraFocus::Free;
        run_frames(&mut app, 1);
        assert!(app.world().get::<PanOrbitCamera>(camera).unwrap().enabled_motion.pan);
        assert_eq!(camera_transform(&app, camera), transform);
    }

    #[test]
    fn free_focus_leaves_the_camera_where_the_user_put_it() {
        let (mut app, camera) = camera_app();
        let moved = orbit_transform(Vec3::new(500.0, 0.0, -200.0), 1.0, 0.3, 4000.0);
        *app.world_mut().get_mut::<Transform>(camera).unwrap() = moved;
        run_frames(&mut app, 10);
        assert_eq!(camera_transform(&app, camera), moved);
    }

    fn move_camera_away_and_request_reset(app: &mut App, camera: Entity) {
        *app.world_mut().get_mut::<Transform>(camera).unwrap() =
            orbit_transform(Vec3::new(500.0, 0.0, -200.0), 1.0, 0.3, 4000.0);
        app.world_mut().get_mut::<PanOrbitCamera>(camera).unwrap().last_anchor_depth = -4000.0;
        let mut menu_widget = app.world_mut().resource_mut::<MenuWidget>();
        menu_widget.camera_focus = CameraFocus::Free;
        menu_widget.reset_view_requested = true;
    }

    #[test]
    fn reset_view_glides_back_to_the_initial_viewpoint() {
        let (mut app, camera) = camera_app();
        move_camera_away_and_request_reset(&mut app, camera);
        run_frames(&mut app, 1);
        assert!(!app.world().resource::<MenuWidget>().reset_view_requested);
        run_frames(&mut app, 120);
        assert_eq!(camera_transform(&app, camera), initial_camera_transform());
        assert_eq!(
            app.world().get::<PanOrbitCamera>(camera).unwrap().last_anchor_depth(),
            -f64::from(INITIAL_RADIUS_M)
        );
    }

    #[test]
    fn a_user_gesture_interrupts_the_view_reset() {
        let (mut app, camera) = camera_app();
        move_camera_away_and_request_reset(&mut app, camera);
        run_frames(&mut app, 2);
        app.world_mut().get_mut::<PanOrbitCamera>(camera).unwrap().start_orbit(None);
        let taken_over = camera_transform(&app, camera);
        run_frames(&mut app, 10);
        assert_eq!(camera_transform(&app, camera), taken_over);
    }

    #[test]
    fn pitch_stays_in_range_around_the_orbited_point() {
        let (mut app, camera) = camera_app();
        let focus = Vec3::new(100.0, 0.0, 50.0);
        for (pitch_deg, expected_deg) in [(-20.0_f32, -10.0_f32), (89.5, 89.0), (30.0, 30.0)] {
            *app.world_mut().get_mut::<Transform>(camera).unwrap() =
                orbit_transform(focus, 0.7, pitch_deg.to_radians(), 5000.0);
            app.world_mut().get_mut::<PanOrbitCamera>(camera).unwrap().last_anchor_depth = -5000.0;
            run_frames(&mut app, 1);
            let transform = camera_transform(&app, camera);
            let pitch_deg = (-transform.forward().y).asin().to_degrees();
            assert!((pitch_deg - expected_deg).abs() < 0.01, "pitch {pitch_deg}°, expected {expected_deg}°");
            // Same orbited point, distance and heading
            let orbited = transform.translation + transform.forward() * 5000.0;
            assert!(orbited.distance(focus) < 0.05, "orbited point moved to {orbited}");
            let right = transform.right();
            assert!(((-right.z).atan2(right.x) - 0.7).abs() < 1e-4, "heading changed");
        }
    }
}
