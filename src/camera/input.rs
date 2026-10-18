//! Mouse and touch input for the camera. Bevy's pan-orbit controller ships
//! without an input plugin: this module turns the mouse and fingers into
//! controller motions — left drag or one finger orbits, right drag or two
//! fingers pan, the wheel or a pinch zooms — and keeps gestures that start
//! over the UI away from the camera.

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

/// Smallest distance between two fingers used to measure a pinch, in pixels:
/// keeps fingers landing on top of each other from zooming wildly.
const MIN_PINCH_SPAN: f32 = 10.0;

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
#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Gesture {
    /// Nothing in progress: a button press, a wheel scroll or a finger landing
    /// starts a gesture.
    #[default]
    Idle,
    /// Orbiting (left button) or panning (right button) until that button is
    /// released, wherever the pointer goes meanwhile — across the UI included.
    Drag(MouseButton),
    /// Zooming with the wheel, until the zoom comes to rest.
    Zoom,
    /// A first finger landed on the previous frame, at this position. Unlike a
    /// mouse, a finger does not hover the UI before landing on it, so the
    /// gesture starts (or is ignored) only once egui has reported on that
    /// frame.
    TouchPending(Vec2),
    /// A touch gesture with this many fingers: one orbits, two or more pan
    /// (with a free focus) and pinch-zoom.
    Touch(usize),
    /// A gesture that started on the UI: the camera ignores everything until
    /// every button and finger is released.
    Ignored,
}

/// One frame of input, as far as the camera is concerned.
struct Frame<'a> {
    buttons: &'a ButtonInput<MouseButton>,
    /// Mouse motion this frame.
    motion: Vec2,
    /// Zoom input from the mouse wheel this frame.
    wheel: f32,
    touch: TouchFrame,
    /// Whether the pointer (the landing finger, for a pending touch gesture)
    /// is over the UI, where a gesture must not start.
    over_ui: bool,
    /// Whether the controller's wheel zoom has come to rest.
    zoom_settled: bool,
    /// Whether the controller accepts pans (refused while a focus is pinned).
    pan_enabled: bool,
}

/// One frame of touch input.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct TouchFrame {
    /// Fingers on the screen.
    fingers: usize,
    /// Where the first finger landed, on the frame it did (none was down before).
    landed_at: Option<Vec2>,
    /// Mean motion of the fingers since the previous frame.
    motion: Vec2,
    /// Zoom input from pinching (spreading the fingers zooms in).
    pinch: f32,
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
    let mouse_input = Input { motion: frame.motion, zoom: frame.wheel };
    let touch = &frame.touch;
    let drag_press = [MouseButton::Left, MouseButton::Right]
        .into_iter()
        .find(|&button| frame.buttons.just_pressed(button));
    let any_held = frame.buttons.get_pressed().next().is_some() || touch.fingers > 0;
    match gesture {
        Gesture::Idle | Gesture::Zoom => {
            // Starting another gesture ends a wheel zoom in progress (starting
            // a motion replaces it anyway)
            let end_zoom = if gesture == Gesture::Zoom { vec![End] } else { Vec::new() };
            if let Some(landed_at) = touch.landed_at {
                (Gesture::TouchPending(landed_at), end_zoom)
            } else if let Some(button) = drag_press {
                if frame.over_ui {
                    (Gesture::Ignored, end_zoom)
                } else {
                    let start = if button == MouseButton::Left { StartOrbit } else { StartPan };
                    (Gesture::Drag(button), vec![start, mouse_input])
                }
            } else if gesture == Gesture::Idle {
                if frame.wheel != 0.0 && !frame.over_ui {
                    (Gesture::Zoom, vec![StartZoom, mouse_input])
                } else {
                    (Gesture::Idle, Vec::new())
                }
            } else if (frame.wheel != 0.0 && frame.over_ui)
                || (frame.wheel == 0.0 && frame.zoom_settled)
            {
                // Scrolling over the UI scrolls the UI, not the scene behind it
                (Gesture::Idle, vec![End])
            } else {
                (Gesture::Zoom, vec![mouse_input])
            }
        }
        Gesture::Drag(button) if frame.buttons.pressed(button) => {
            (Gesture::Drag(button), vec![mouse_input])
        }
        // The drag button was released: wait for any other held button too
        Gesture::Drag(_) if any_held => (Gesture::Ignored, vec![End]),
        Gesture::Drag(_) => (Gesture::Idle, vec![End]),
        // A tap: the finger is already up
        Gesture::TouchPending(_) if touch.fingers == 0 => (Gesture::Idle, Vec::new()),
        Gesture::TouchPending(_) if frame.over_ui => (Gesture::Ignored, Vec::new()),
        Gesture::TouchPending(_) => start_touch_motion(frame),
        Gesture::Touch(_) if touch.fingers == 0 => (Gesture::Idle, vec![End]),
        // Each finger count drives its own motion
        Gesture::Touch(fingers) if fingers != touch.fingers => start_touch_motion(frame),
        Gesture::Touch(fingers) => (
            Gesture::Touch(fingers),
            vec![Input { motion: touch.motion, zoom: touch.pinch }],
        ),
        Gesture::Ignored if any_held => (Gesture::Ignored, Vec::new()),
        Gesture::Ignored => (Gesture::Idle, Vec::new()),
    }
}

/// Starts the motion of the fingers on the screen: one finger orbits, two or
/// more pan and pinch-zoom — or only pinch-zoom while a focus is pinned.
fn start_touch_motion(frame: &Frame) -> (Gesture, Vec<ControllerCommand>) {
    use ControllerCommand::*;
    let touch = &frame.touch;
    let start = match touch.fingers {
        1 => StartOrbit,
        _ if frame.pan_enabled => StartPan,
        _ => StartZoom,
    };
    (
        Gesture::Touch(touch.fingers),
        vec![start, Input { motion: touch.motion, zoom: touch.pinch }],
    )
}

/// Touch input this frame, from the fingers on the screen (id and position,
/// sorted by id) on this frame and on the previous one.
///
/// Positions are compared frame to frame here rather than taken from
/// `Touch::delta`, which only refreshes on frames with touch events: a finger
/// held still would keep reporting its last motion.
fn touch_frame(fingers: &[(u64, Vec2)], last: &[(u64, Vec2)], window_height: f32) -> TouchFrame {
    let last_position = |id: u64| {
        last.iter()
            .find(|&&(last_id, _)| last_id == id)
            .map(|&(_, position)| position)
    };
    // A finger that just landed has not moved yet
    let motion = if fingers.is_empty() {
        Vec2::ZERO
    } else {
        fingers
            .iter()
            .map(|&(id, position)| last_position(id).map_or(Vec2::ZERO, |last| position - last))
            .sum::<Vec2>()
            / fingers.len() as f32
    };
    // Pinching only while the same fingers stay down: one landing or lifting
    // would make the span between the first two jump
    let same_fingers = fingers.len() == last.len()
        && fingers.iter().zip(last).all(|(finger, last)| finger.0 == last.0);
    let pinch = match (fingers, last) {
        ([a, b, ..], [last_a, last_b, ..]) if same_fingers => pinch_zoom(
            last_a.1.distance(last_b.1),
            a.1.distance(b.1),
            window_height,
        ),
        _ => 0.0,
    };
    TouchFrame {
        fingers: fingers.len(),
        landed_at: last.is_empty().then(|| fingers.first().map(|&(_, p)| p)).flatten(),
        motion,
        pinch,
    }
}

/// Zoom input for two fingers moving from `previous_span` to `span` pixels
/// apart. The controller zooms by about (input ÷ viewport height) of the
/// viewing distance per unit (with Bevy's default field of view), so this
/// scales the distance by `previous_span / span`: the pinched scene follows
/// the fingers.
fn pinch_zoom(previous_span: f32, span: f32, window_height: f32) -> f32 {
    (1.0 - previous_span.max(MIN_PINCH_SPAN) / span.max(MIN_PINCH_SPAN)) * window_height
}

/// Drives the camera controller from the mouse and the fingers.
///
/// Motions use no picking anchor: the camera orbits, pans and zooms about the
/// point straight ahead at the current viewing distance — the view centre,
/// which is also the menu's pinned focus point when one is selected.
///
/// Whether a gesture starts over the UI is decided when it starts, then held
/// until it ends: a drag started in the scene keeps orbiting when it crosses a
/// panel or a window, and one started on the UI (a slider, a window title bar)
/// never moves the camera. A mouse gesture is checked against egui's report on
/// the previous frame (the pointer already hovers the UI before a press or
/// scroll lands on it), a touch gesture against egui's report on the frame its
/// first finger landed (see [`Gesture::TouchPending`]). Each wheel scroll is
/// checked, though, so scrolling a panel or a window right after zooming the
/// scene scrolls the UI.
fn drive_camera(
    mut gesture: Local<Gesture>,
    mut last_fingers: Local<Vec<(u64, Vec2)>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    touches: Res<Touches>,
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
    let window = window_q.single().ok();
    let mut fingers: Vec<_> = touches.iter().map(|touch| (touch.id(), touch.position())).collect();
    fingers.sort_unstable_by_key(|&(id, _)| id);
    let touch = touch_frame(&fingers, &last_fingers, window.map_or(0.0, Window::height));
    *last_fingers = fingers;

    let Ok(mut camera) = camera_q.single_mut() else { return; };
    let pointer = match *gesture {
        Gesture::TouchPending(landed_at) => Some(landed_at),
        _ => window.and_then(Window::cursor_position),
    };
    let over_ui = camera_input_blocked(
        pointer,
        &side_panel_rects,
        egui_wants_input.wants_any_pointer_input(),
        egui_wants_input.wants_keyboard_input(),
    );
    // Rest is when no wheel input arrived for twice the zoom smoothing window
    // (as Bevy's example input plugin decides it)
    let zoom_settled = camera.current_motion.inputs().is_none_or(|inputs| {
        inputs.zoom_velocity_abs(camera.smoothing.zoom.mul_f32(2.0)) <= 0.0
    });
    let frame = Frame {
        buttons: &mouse_buttons,
        motion: mouse_motion.delta,
        wheel,
        touch,
        over_ui,
        zoom_settled,
        pan_enabled: camera.enabled_motion.pan,
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

    /// Steps the gesture through one mouse frame, then ages the button state
    /// the way Bevy's input plugin does between frames.
    fn run(
        gesture: &mut Gesture,
        buttons: &mut ButtonInput<MouseButton>,
        over_ui: bool,
        wheel: f32,
        zoom_settled: bool,
    ) -> Vec<ControllerCommand> {
        let frame = Frame {
            buttons,
            motion: Vec2::new(3.0, -2.0),
            wheel,
            touch: TouchFrame::default(),
            over_ui,
            zoom_settled,
            pan_enabled: true,
        };
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

    /// Steps the gesture through one touch frame.
    fn run_touch(gesture: &mut Gesture, touch: TouchFrame, over_ui: bool, pan_enabled: bool) -> Vec<ControllerCommand> {
        let frame = Frame {
            buttons: &ButtonInput::default(),
            motion: Vec2::ZERO,
            wheel: 0.0,
            touch,
            over_ui,
            zoom_settled: true,
            pan_enabled,
        };
        let (next, commands) = step(*gesture, &frame);
        *gesture = next;
        commands
    }

    fn fingers(count: usize) -> TouchFrame {
        TouchFrame { fingers: count, landed_at: None, motion: Vec2::new(4.0, 1.0), pinch: 0.5 }
    }

    const TOUCH_INPUT: ControllerCommand = ControllerCommand::Input { motion: Vec2::new(4.0, 1.0), zoom: 0.5 };

    #[test]
    fn finger_count_picks_the_motion() {
        use ControllerCommand::*;
        let mut gesture = Gesture::Idle;
        let landing = TouchFrame { landed_at: Some(SCENE), ..fingers(1) };
        assert!(run_touch(&mut gesture, landing, false, true).is_empty(), "waits for egui's report");
        assert_eq!(gesture, Gesture::TouchPending(SCENE));
        assert_eq!(run_touch(&mut gesture, fingers(1), false, true), [StartOrbit, TOUCH_INPUT]);
        assert_eq!(run_touch(&mut gesture, fingers(1), false, true), [TOUCH_INPUT]);
        assert_eq!(run_touch(&mut gesture, fingers(2), false, true), [StartPan, TOUCH_INPUT]);
        assert_eq!(run_touch(&mut gesture, fingers(2), false, true), [TOUCH_INPUT]);
        assert_eq!(run_touch(&mut gesture, fingers(1), false, true), [StartOrbit, TOUCH_INPUT], "back to orbit");
        assert_eq!(run_touch(&mut gesture, fingers(0), false, true), [End]);
        assert_eq!(gesture, Gesture::Idle);
    }

    /// With a pinned focus, two fingers only pinch-zoom (no pan).
    #[test]
    fn two_fingers_only_zoom_a_pinned_focus() {
        let mut gesture = Gesture::Touch(1);
        assert_eq!(
            run_touch(&mut gesture, fingers(2), false, false),
            [ControllerCommand::StartZoom, TOUCH_INPUT]
        );
    }

    /// A touch is checked where it landed, on the next frame, and a gesture
    /// started on the UI is ignored until every finger is up — fingers added
    /// meanwhile included.
    #[test]
    fn touch_landing_on_the_ui_is_ignored_until_every_finger_is_up() {
        let mut gesture = Gesture::TouchPending(LEFT_PANEL);
        assert!(run_touch(&mut gesture, fingers(1), true, true).is_empty());
        assert_eq!(gesture, Gesture::Ignored);
        assert!(run_touch(&mut gesture, fingers(2), false, true).is_empty());
        assert!(run_touch(&mut gesture, fingers(0), false, true).is_empty());
        assert_eq!(gesture, Gesture::Idle);
    }

    #[test]
    fn a_tap_moves_nothing() {
        let mut gesture = Gesture::TouchPending(SCENE);
        assert!(run_touch(&mut gesture, fingers(0), false, true).is_empty());
        assert_eq!(gesture, Gesture::Idle);
    }

    #[test]
    fn touch_frame_tracks_fingers_between_frames() {
        let (a, b) = (Vec2::new(500.0, 300.0), Vec2::new(600.0, 300.0));
        // First finger landing
        let landed = touch_frame(&[(7, a)], &[], 720.0);
        assert_eq!(landed, TouchFrame { fingers: 1, landed_at: Some(a), motion: Vec2::ZERO, pinch: 0.0 });
        // A second finger landing: it has not moved yet, and no pinch is
        // measured on the frame the finger set changes
        let moved = Vec2::new(10.0, 0.0);
        let second = touch_frame(&[(7, a + moved), (9, b)], &[(7, a)], 720.0);
        assert_eq!(second.landed_at, None);
        assert_eq!(second.motion, moved / 2.0);
        assert_eq!(second.pinch, 0.0);
        // Both held still: no motion at all (unlike `Touch::delta`)
        let still = touch_frame(&[(7, a), (9, b)], &[(7, a), (9, b)], 720.0);
        assert_eq!((still.motion, still.pinch), (Vec2::ZERO, 0.0));
        // Spreading from 100 to 200 px apart halves the viewing distance
        let spread = touch_frame(
            &[(7, a - Vec2::X * 50.0), (9, b + Vec2::X * 50.0)],
            &[(7, a), (9, b)],
            720.0,
        );
        assert_eq!(spread.motion, Vec2::ZERO);
        assert_eq!(spread.pinch, 0.5 * 720.0);
    }

    #[test]
    fn pinch_zoom_follows_the_fingers() {
        assert_eq!(pinch_zoom(100.0, 200.0, 800.0), 400.0, "spreading zooms in");
        assert_eq!(pinch_zoom(200.0, 100.0, 800.0), -800.0, "pinching zooms out");
        assert_eq!(pinch_zoom(150.0, 150.0, 800.0), 0.0);
        assert!(pinch_zoom(0.0, 30.0, 800.0).is_finite(), "fingers landing on top of each other");
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

    fn touch(app: &mut App, window: Entity, id: u64, phase: TouchPhase, position: Vec2) {
        app.world_mut().write_message(TouchInput { phase, position, window, force: None, id });
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

    #[test]
    fn fingers_drive_the_controller() {
        let (mut app, window, camera) = input_app();
        touch(&mut app, window, 0, TouchPhase::Started, SCENE);
        app.update();
        assert!(!controller(&app, camera).current_motion.is_user_controlled(), "waits a frame");
        app.update();
        assert!(controller(&app, camera).current_motion.is_orbiting());
        touch(&mut app, window, 1, TouchPhase::Started, SCENE + Vec2::new(80.0, 0.0));
        app.update();
        assert!(controller(&app, camera).current_motion.is_panning());
        touch(&mut app, window, 1, TouchPhase::Ended, SCENE + Vec2::new(80.0, 0.0));
        app.update();
        assert!(controller(&app, camera).current_motion.is_orbiting());
        touch(&mut app, window, 0, TouchPhase::Ended, SCENE);
        app.update();
        assert!(!controller(&app, camera).current_motion.is_user_controlled());

        // A finger landing on a panel leaves the camera alone
        touch(&mut app, window, 2, TouchPhase::Started, RIGHT_PANEL);
        app.update();
        touch(&mut app, window, 2, TouchPhase::Moved, SCENE);
        app.update();
        app.update();
        assert!(!controller(&app, camera).current_motion.is_user_controlled());
    }
}
