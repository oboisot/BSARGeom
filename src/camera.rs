use std::f32::consts::FRAC_PI_4;

use bevy::{
    prelude::*,
    window::PrimaryWindow
};
use bevy_egui::{input::EguiWantsInput, EguiPostUpdateSet, EguiStartupSet};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraSystemSet};

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
        app.add_systems( // see: https://github.com/vladbat00/bevy_egui/blob/main/examples/ui.rs
                PreStartup,
                spawn_camera.before(EguiStartupSet::InitContexts),
            )
            .add_systems(Update, update_camera_focus)
            .add_systems(
                PostUpdate,
                // once egui has processed this frame's input, before the camera reads it
                block_camera_over_ui
                    .after(EguiPostUpdateSet::ProcessOutput)
                    .before(PanOrbitCameraSystemSet),
            );
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

/// Disables the camera for gestures that start over the UI.
///
/// The decision is taken when a gesture starts (first mouse button or touch
/// down) and kept until every button and touch is released: a drag started in
/// the scene keeps orbiting when it crosses a panel or a window, and a drag
/// started on the UI (a slider, a window title bar) never moves the camera.
/// Between gestures it is re-evaluated every frame, so the mouse wheel does
/// not zoom while the pointer is over the UI.
///
/// Runs after egui has processed this frame's input, so a press or touch that
/// lands on a window is already reported by [`EguiWantsInput`] — a touch has
/// no hover beforehand to rely on.
fn block_camera_over_ui(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    side_panel_rects: Res<SidePanelRects>,
    egui_wants_input: Res<EguiWantsInput>,
    mut pan_orbit_camera_q: Query<&mut PanOrbitCamera>,
) {
    if gesture_held(&mouse_buttons, &touches) {
        return; // the decision taken when the gesture started stands
    }
    let Ok(window) = window_q.single() else { return; };
    // A touch that just started takes precedence over a (possibly stale) cursor
    let pointer = touches
        .iter_just_pressed()
        .next()
        .map(|touch| touch.position())
        .or_else(|| window.cursor_position());
    let blocked = camera_input_blocked(
        pointer,
        &side_panel_rects,
        egui_wants_input.wants_any_pointer_input(),
        egui_wants_input.wants_keyboard_input(),
    );
    for mut pan_orbit_camera in pan_orbit_camera_q.iter_mut() {
        if pan_orbit_camera.enabled == blocked { // Avoids triggering change detection every frame
            pan_orbit_camera.enabled = !blocked;
        }
    }
}

/// Whether the camera must ignore a gesture starting with the pointer at
/// `pointer` (logical points, `None` when outside the window).
///
/// egui cannot report panels laid out on the background layer through
/// `Context::is_pointer_over_egui` (only floating areas like windows register
/// there), so the panel extents exported by the UI system ([`SidePanelRects`])
/// are compared against the pointer position. Everything else comes from
/// egui: the pointer over or used by one of its areas (windows, the modal save
/// dialog), or a text field holding keyboard focus.
fn camera_input_blocked(
    pointer: Option<Vec2>,
    side_panel_rects: &SidePanelRects,
    egui_wants_pointer: bool,
    egui_wants_keyboard: bool,
) -> bool {
    egui_wants_pointer
        || egui_wants_keyboard
        || pointer.is_some_and(|pos|
            pos.x <= side_panel_rects.left_max_x ||
            pos.x >= side_panel_rects.right_min_x
        )
}

/// Whether a gesture that started on an earlier frame is still in progress:
/// some mouse button or touch is down that did not just go down this frame.
fn gesture_held(mouse_buttons: &ButtonInput<MouseButton>, touches: &Touches) -> bool {
    mouse_buttons.get_pressed().any(|button| !mouse_buttons.just_pressed(*button))
        || touches.iter().any(|touch| !touches.just_pressed(touch.id()))
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

#[cfg(test)]
mod tests {
    use bevy::input::{mouse::MouseButtonInput, touch::TouchPhase, ButtonState, InputPlugin};

    use super::*;

    // Pointer positions in a `Window::default()` (1280×720 logical points) with
    // the menu and the Tx panel open on the left, the Rx panel on the right
    const SCENE: Vec2 = Vec2::new(640.0, 360.0);
    const LEFT_PANEL: Vec2 = Vec2::new(100.0, 360.0);
    const RIGHT_PANEL: Vec2 = Vec2::new(1100.0, 360.0);

    fn side_panel_rects() -> SidePanelRects {
        SidePanelRects { left_max_x: 350.0, right_min_x: 980.0 }
    }

    #[test]
    fn camera_input_blocked_over_side_panels_or_when_egui_wants_input() {
        let rects = side_panel_rects();
        assert!(!camera_input_blocked(Some(SCENE), &rects, false, false));
        assert!(camera_input_blocked(Some(LEFT_PANEL), &rects, false, false));
        assert!(camera_input_blocked(Some(RIGHT_PANEL), &rects, false, false));
        // Over the scene, egui still takes precedence: pointer over/using one
        // of its areas (a window), or a text field holding keyboard focus
        assert!(camera_input_blocked(Some(SCENE), &rects, true, false));
        assert!(camera_input_blocked(Some(SCENE), &rects, false, true));
        // Pointer outside the window: only egui can block
        assert!(!camera_input_blocked(None, &rects, false, false));
        assert!(camera_input_blocked(None, &rects, false, true));
    }

    /// Headless App running the gating system on a primary window with the
    /// panels above. Bevy's input plugin turns the messages written by the
    /// tests into `ButtonInput`/`Touches` state frame by frame; egui reports
    /// nothing (default `EguiWantsInput`), so only the panels block.
    fn gating_app() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(InputPlugin)
            .insert_resource(side_panel_rects())
            .init_resource::<EguiWantsInput>()
            .add_systems(PostUpdate, block_camera_over_ui);
        let window = app.world_mut().spawn((Window::default(), PrimaryWindow)).id();
        let camera = app.world_mut().spawn(PanOrbitCamera::default()).id();
        (app, window, camera)
    }

    fn move_cursor(app: &mut App, window: Entity, position: Vec2) {
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(position));
    }

    fn mouse_button(app: &mut App, window: Entity, button: MouseButton, state: ButtonState) {
        app.world_mut().write_message(MouseButtonInput { button, state, window });
    }

    fn touch(app: &mut App, window: Entity, phase: TouchPhase, position: Vec2) {
        app.world_mut().write_message(TouchInput { phase, position, window, force: None, id: 0 });
    }

    fn camera_enabled(app: &App, camera: Entity) -> bool {
        app.world().get::<PanOrbitCamera>(camera).unwrap().enabled
    }

    /// A drag started in the scene keeps the camera while it crosses the
    /// panels; once released over a panel, the camera ignores the mouse wheel.
    #[test]
    fn drag_started_in_scene_keeps_the_camera_over_panels() {
        let (mut app, window, camera) = gating_app();
        move_cursor(&mut app, window, SCENE);
        mouse_button(&mut app, window, MouseButton::Left, ButtonState::Pressed);
        app.update();
        assert!(camera_enabled(&app, camera));

        for position in [LEFT_PANEL, RIGHT_PANEL] {
            move_cursor(&mut app, window, position);
            app.update();
            assert!(camera_enabled(&app, camera), "drag lost the camera at {position}");
        }

        mouse_button(&mut app, window, MouseButton::Left, ButtonState::Released);
        app.update();
        assert!(!camera_enabled(&app, camera), "released over a panel");
    }

    /// Regression test: the camera used to be re-enabled from the pointer
    /// position every frame, so a drag started on a panel orbited the scene as
    /// soon as it left the panel.
    #[test]
    fn drag_started_on_panel_never_moves_the_camera() {
        let (mut app, window, camera) = gating_app();
        move_cursor(&mut app, window, LEFT_PANEL);
        mouse_button(&mut app, window, MouseButton::Left, ButtonState::Pressed);
        app.update();
        assert!(!camera_enabled(&app, camera));

        move_cursor(&mut app, window, SCENE);
        app.update();
        assert!(!camera_enabled(&app, camera), "drag leaving the panel");

        // A second button pressed mid-drag does not start a new gesture
        mouse_button(&mut app, window, MouseButton::Right, ButtonState::Pressed);
        app.update();
        assert!(!camera_enabled(&app, camera), "second button over the scene");

        mouse_button(&mut app, window, MouseButton::Left, ButtonState::Released);
        mouse_button(&mut app, window, MouseButton::Right, ButtonState::Released);
        app.update();
        assert!(camera_enabled(&app, camera), "released over the scene");
    }

    /// Touch gestures are gated where the first touch lands (there is no
    /// cursor hovering the UI beforehand), then held like mouse drags.
    #[test]
    fn touch_gestures_are_gated_where_they_start() {
        let (mut app, window, camera) = gating_app();
        touch(&mut app, window, TouchPhase::Started, RIGHT_PANEL);
        app.update();
        assert!(!camera_enabled(&app, camera));
        touch(&mut app, window, TouchPhase::Moved, SCENE);
        app.update();
        assert!(!camera_enabled(&app, camera), "touch drag leaving the panel");
        touch(&mut app, window, TouchPhase::Ended, SCENE);
        app.update();
        assert!(camera_enabled(&app, camera), "no touch and no cursor: nothing to block");

        touch(&mut app, window, TouchPhase::Started, SCENE);
        app.update();
        touch(&mut app, window, TouchPhase::Moved, LEFT_PANEL);
        app.update();
        assert!(camera_enabled(&app, camera), "touch drag crossing a panel");
    }
}
