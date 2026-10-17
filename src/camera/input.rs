//! Mouse input for the camera. Bevy's pan-orbit controller ships without an
//! input plugin: this module turns the mouse into controller motions — left
//! drag orbits, right drag pans, the wheel zooms — and keeps gestures that
//! start over the UI away from the camera.

use bevy::{
    camera_controller::pan_orbit_camera::{prelude::PanOrbitCamera, SyncCameraPosition},
    input::{
        mouse::{AccumulatedMouseMotion, MouseScrollUnit, MouseWheel},
        InputSystems,
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::input::EguiWantsInput;

use crate::ui::SidePanelRects;

/// Zoom input per mouse-wheel line ("notch"); pixel-precise scrolling (touchpads)
/// counts one per pixel. The controller zooms by a fraction of the focus
/// distance per unit, so this is the zoom speed (Bevy's examples use 150).
const ZOOM_PER_WHEEL_LINE: f32 = 150.0;

pub(super) struct CameraInputPlugin;

impl Plugin for CameraInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            drive_camera.after(InputSystems).before(SyncCameraPosition),
        );
    }
}

/// The camera gesture in progress.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Gesture {
    /// Nothing in progress: a button press or a wheel scroll starts a gesture.
    #[default]
    Idle,
    /// Orbiting (left button) or panning (right button) until that button is
    /// released, wherever the pointer goes meanwhile — across the UI included.
    Drag(MouseButton),
    /// Zooming with the wheel, until the zoom comes to rest.
    Zoom,
    /// A press that landed on the UI: the camera ignores everything until
    /// every button is released.
    Ignored,
}

/// One frame of input, as far as the camera is concerned.
struct Frame<'a> {
    buttons: &'a ButtonInput<MouseButton>,
    /// Mouse motion this frame.
    motion: Vec2,
    /// Zoom input from the mouse wheel this frame.
    wheel: f32,
    /// Whether the pointer is over the UI, where a gesture must not start.
    over_ui: bool,
    /// Whether the controller's wheel zoom has come to rest.
    zoom_settled: bool,
}

/// What to tell the camera controller.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ControllerCommand {
    StartOrbit,
    StartPan,
    StartZoom,
    /// Pointer motion and zoom for the motion in progress. Sent on every frame
    /// of a motion, zeros included: the controller keeps applying its latest
    /// smoothed input until it gets a new one.
    Input { motion: Vec2, zoom: f32 },
    End,
}

/// Advances the gesture by one frame and lists what the controller must do.
fn step(gesture: Gesture, frame: &Frame) -> (Gesture, Vec<ControllerCommand>) {
    use ControllerCommand::*;
    let input = Input { motion: frame.motion, zoom: frame.wheel };
    let drag_press = [MouseButton::Left, MouseButton::Right]
        .into_iter()
        .find(|&button| frame.buttons.just_pressed(button));
    let any_button_held = frame.buttons.get_pressed().next().is_some();
    match (gesture, drag_press) {
        // A press starts a drag, or is ignored when it lands on the UI. A wheel
        // zoom in progress ends either way (starting a motion replaces it).
        (Gesture::Idle | Gesture::Zoom, Some(button)) => {
            if frame.over_ui {
                let end_zoom = (gesture == Gesture::Zoom).then_some(End);
                (Gesture::Ignored, end_zoom.into_iter().collect())
            } else {
                let start = if button == MouseButton::Left { StartOrbit } else { StartPan };
                (Gesture::Drag(button), vec![start, input])
            }
        }
        (Gesture::Idle, None) if frame.wheel != 0.0 && !frame.over_ui => {
            (Gesture::Zoom, vec![StartZoom, input])
        }
        (Gesture::Idle, None) => (Gesture::Idle, Vec::new()),
        (Gesture::Zoom, None) => {
            // Scrolling over the UI scrolls the UI, not the scene behind it
            if (frame.wheel != 0.0 && frame.over_ui) || (frame.wheel == 0.0 && frame.zoom_settled) {
                (Gesture::Idle, vec![End])
            } else {
                (Gesture::Zoom, vec![input])
            }
        }
        (Gesture::Drag(button), _) if frame.buttons.pressed(button) => {
            (Gesture::Drag(button), vec![input])
        }
        // The drag button was released: wait for any other held button too
        (Gesture::Drag(_), _) if any_button_held => (Gesture::Ignored, vec![End]),
        (Gesture::Drag(_), _) => (Gesture::Idle, vec![End]),
        (Gesture::Ignored, _) if any_button_held => (Gesture::Ignored, Vec::new()),
        (Gesture::Ignored, _) => (Gesture::Idle, Vec::new()),
    }
}

/// Drives the camera controller from the mouse.
///
/// Motions use no picking anchor: the camera orbits, pans and zooms about the
/// point straight ahead at the current viewing distance — the view centre,
/// which is also the menu's pinned focus point when one is selected.
///
/// Whether a gesture starts over the UI is decided when it starts, from egui's
/// report on the previous frame (the pointer already hovers the UI before a
/// press or scroll lands on it), then held until the gesture ends: a drag
/// started in the scene keeps orbiting when it crosses a panel or a window,
/// and one started on the UI (a slider, a window title bar) never moves the
/// camera. Each wheel scroll is checked, though, so scrolling a panel or a
/// window right after zooming the scene scrolls the UI.
fn drive_camera(
    mut gesture: Local<Gesture>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    side_panel_rects: Res<SidePanelRects>,
    egui_wants_input: Res<EguiWantsInput>,
    mut camera_q: Query<&mut PanOrbitCamera>,
) {
    let wheel = mouse_wheel
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * ZOOM_PER_WHEEL_LINE,
            MouseScrollUnit::Pixel => wheel.y,
        })
        .sum();
    let Ok(mut camera) = camera_q.single_mut() else { return; };
    let over_ui = window_q.single().is_ok_and(|window| {
        camera_input_blocked(
            window.cursor_position(),
            &side_panel_rects,
            egui_wants_input.wants_any_pointer_input(),
            egui_wants_input.wants_keyboard_input(),
        )
    });
    // Rest is when no wheel input arrived for twice the zoom smoothing window
    // (as Bevy's example input plugin decides it)
    let zoom_settled = camera.current_motion.inputs().is_none_or(|inputs| {
        inputs.zoom_velocity_abs(camera.smoothing.zoom.mul_f32(2.0)) <= 0.0
    });
    let frame = Frame {
        buttons: &mouse_buttons,
        motion: mouse_motion.delta,
        wheel,
        over_ui,
        zoom_settled,
    };
    let (next, commands) = step(*gesture, &frame);
    *gesture = next;
    for command in commands {
        match command {
            ControllerCommand::StartOrbit => camera.start_orbit(None),
            ControllerCommand::StartPan => camera.start_pan(None),
            ControllerCommand::StartZoom => camera.start_zoom(None),
            ControllerCommand::Input { motion, zoom } => {
                camera.send_screenspace_input(motion);
                camera.send_zoom_input(zoom);
            }
            ControllerCommand::End => camera.end_move(),
        }
    }
}

/// Whether a gesture starting with the pointer at `pointer` (logical points,
/// `None` when outside the window) lands on the UI.
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

    /// Steps the gesture through one frame, then ages the button state the
    /// way Bevy's input plugin does between frames.
    fn run(
        gesture: &mut Gesture,
        buttons: &mut ButtonInput<MouseButton>,
        over_ui: bool,
        wheel: f32,
        zoom_settled: bool,
    ) -> Vec<ControllerCommand> {
        let frame = Frame { buttons, motion: Vec2::new(3.0, -2.0), wheel, over_ui, zoom_settled };
        let (next, commands) = step(*gesture, &frame);
        *gesture = next;
        buttons.clear();
        commands
    }

    const MOVE: ControllerCommand = ControllerCommand::Input { motion: Vec2::new(3.0, -2.0), zoom: 0.0 };

    /// "Keep orbiting": the UI is only checked when the drag starts.
    #[test]
    fn left_drag_orbits_until_released_even_across_the_ui() {
        let (mut gesture, mut buttons) = (Gesture::Idle, ButtonInput::default());
        buttons.press(MouseButton::Left);
        assert_eq!(run(&mut gesture, &mut buttons, false, 0.0, true), [ControllerCommand::StartOrbit, MOVE]);
        assert_eq!(run(&mut gesture, &mut buttons, true, 0.0, true), [MOVE], "over the UI");
        assert_eq!(
            run(&mut gesture, &mut buttons, true, 2.0, true),
            [ControllerCommand::Input { motion: Vec2::new(3.0, -2.0), zoom: 2.0 }],
            "the wheel zooms during a drag"
        );
        buttons.release(MouseButton::Left);
        assert_eq!(run(&mut gesture, &mut buttons, true, 0.0, true), [ControllerCommand::End]);
        assert_eq!(gesture, Gesture::Idle);
    }

    #[test]
    fn right_drag_pans() {
        let (mut gesture, mut buttons) = (Gesture::Idle, ButtonInput::default());
        buttons.press(MouseButton::Right);
        assert_eq!(run(&mut gesture, &mut buttons, false, 0.0, true), [ControllerCommand::StartPan, MOVE]);
        assert_eq!(gesture, Gesture::Drag(MouseButton::Right));
    }

    /// A drag started on the UI (a slider, a window title bar) never moves the
    /// camera, even once it leaves the UI, and neither does a second button
    /// pressed meanwhile.
    #[test]
    fn press_on_the_ui_is_ignored_until_every_button_is_released() {
        let (mut gesture, mut buttons) = (Gesture::Idle, ButtonInput::default());
        buttons.press(MouseButton::Left);
        assert!(run(&mut gesture, &mut buttons, true, 0.0, true).is_empty());
        assert!(run(&mut gesture, &mut buttons, false, 0.0, true).is_empty(), "drag leaving the UI");
        buttons.press(MouseButton::Right);
        assert!(run(&mut gesture, &mut buttons, false, 0.0, true).is_empty(), "second button");
        buttons.release(MouseButton::Left);
        assert!(run(&mut gesture, &mut buttons, false, 0.0, true).is_empty());
        assert_eq!(gesture, Gesture::Ignored, "the right button is still held");
        buttons.release(MouseButton::Right);
        assert!(run(&mut gesture, &mut buttons, false, 0.0, true).is_empty());
        assert_eq!(gesture, Gesture::Idle);
    }

    #[test]
    fn releasing_the_drag_button_waits_for_the_others() {
        let (mut gesture, mut buttons) = (Gesture::Idle, ButtonInput::default());
        buttons.press(MouseButton::Left);
        run(&mut gesture, &mut buttons, false, 0.0, true);
        buttons.press(MouseButton::Right);
        assert_eq!(run(&mut gesture, &mut buttons, false, 0.0, true), [MOVE], "no new gesture");
        buttons.release(MouseButton::Left);
        assert_eq!(run(&mut gesture, &mut buttons, false, 0.0, true), [ControllerCommand::End]);
        assert_eq!(gesture, Gesture::Ignored);
        buttons.release(MouseButton::Right);
        run(&mut gesture, &mut buttons, false, 0.0, true);
        assert_eq!(gesture, Gesture::Idle);
    }

    #[test]
    fn wheel_zooms_until_the_zoom_comes_to_rest() {
        let (mut gesture, mut buttons) = (Gesture::Idle, ButtonInput::default());
        let zoom = |zoom| ControllerCommand::Input { motion: Vec2::new(3.0, -2.0), zoom };
        assert_eq!(
            run(&mut gesture, &mut buttons, false, 150.0, false),
            [ControllerCommand::StartZoom, zoom(150.0)]
        );
        // Still smoothing out: inputs keep flowing, zeros included
        assert_eq!(run(&mut gesture, &mut buttons, false, 0.0, false), [zoom(0.0)]);
        assert_eq!(run(&mut gesture, &mut buttons, false, 0.0, true), [ControllerCommand::End]);
        assert_eq!(gesture, Gesture::Idle);
    }

    /// Scrolling over a panel or a window scrolls the UI, not the scene —
    /// also right after zooming the scene.
    #[test]
    fn wheel_over_the_ui_leaves_the_camera_alone() {
        let (mut gesture, mut buttons) = (Gesture::Idle, ButtonInput::default());
        assert!(run(&mut gesture, &mut buttons, true, 150.0, false).is_empty());
        assert_eq!(gesture, Gesture::Idle);
        run(&mut gesture, &mut buttons, false, 150.0, false);
        assert_eq!(gesture, Gesture::Zoom);
        assert_eq!(run(&mut gesture, &mut buttons, true, 150.0, false), [ControllerCommand::End]);
        assert_eq!(gesture, Gesture::Idle);
    }

    #[test]
    fn a_press_during_a_wheel_zoom_starts_a_drag_unless_on_the_ui() {
        let (mut gesture, mut buttons) = (Gesture::Zoom, ButtonInput::default());
        buttons.press(MouseButton::Left);
        assert_eq!(run(&mut gesture, &mut buttons, false, 0.0, false), [ControllerCommand::StartOrbit, MOVE]);

        let (mut gesture, mut buttons) = (Gesture::Zoom, ButtonInput::default());
        buttons.press(MouseButton::Left);
        assert_eq!(run(&mut gesture, &mut buttons, true, 0.0, false), [ControllerCommand::End]);
        assert_eq!(gesture, Gesture::Ignored);
    }

    /// Headless App running the input system against a primary window with
    /// the panels above and a controller component (the controller's own
    /// update needs a rendering camera, so only the motion it is told to start
    /// is checked). Bevy's input plugin turns the messages written by the
    /// tests into input state frame by frame; egui reports nothing.
    fn input_app() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(InputPlugin)
            .insert_resource(side_panel_rects())
            .init_resource::<EguiWantsInput>()
            .add_systems(PreUpdate, drive_camera.after(InputSystems));
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

    fn controller(app: &App, camera: Entity) -> &PanOrbitCamera {
        app.world().get::<PanOrbitCamera>(camera).unwrap()
    }

    #[test]
    fn mouse_drives_the_controller() {
        let (mut app, window, camera) = input_app();
        move_cursor(&mut app, window, SCENE);
        mouse_button(&mut app, window, MouseButton::Left, ButtonState::Pressed);
        app.update();
        assert!(controller(&app, camera).current_motion.is_orbiting());

        // Dragging across a panel keeps orbiting
        move_cursor(&mut app, window, LEFT_PANEL);
        app.update();
        assert!(controller(&app, camera).current_motion.is_orbiting());
        mouse_button(&mut app, window, MouseButton::Left, ButtonState::Released);
        app.update();
        assert!(!controller(&app, camera).current_motion.is_user_controlled());

        // A press on the panel leaves the camera alone
        mouse_button(&mut app, window, MouseButton::Right, ButtonState::Pressed);
        app.update();
        assert!(!controller(&app, camera).current_motion.is_user_controlled());
        mouse_button(&mut app, window, MouseButton::Right, ButtonState::Released);
        app.update();

        // The wheel over the scene zooms
        move_cursor(&mut app, window, SCENE);
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 1.0,
            window,
            phase: TouchPhase::Moved,
        });
        app.update();
        assert!(controller(&app, camera).current_motion.is_zooming_only());
    }

    /// With a pinned focus, the controller refuses to pan: a right drag then
    /// leaves the camera alone.
    #[test]
    fn right_drag_does_not_pan_a_pinned_focus() {
        let (mut app, window, camera) = input_app();
        app.world_mut().get_mut::<PanOrbitCamera>(camera).unwrap().enabled_motion.pan = false;
        move_cursor(&mut app, window, SCENE);
        mouse_button(&mut app, window, MouseButton::Right, ButtonState::Pressed);
        app.update();
        assert!(!controller(&app, camera).current_motion.is_user_controlled());
    }
}
