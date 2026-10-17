use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use crate::{
    scene::{
        TxCarrierState, TxAntennaState, TxAntennaBeamState, TxAntennaBeamFootprintState,
        RxCarrierState, RxAntennaState, RxAntennaBeamState, RxAntennaBeamFootprintState,
        BsarInfosState
    },
    ui::{
        bsar_infos_ui, carrier_infos_ui, show_gaf_window, GafState,
        MenuPlugin, MenuWidget, TxPanelPlugin, TxPanelWidget, RxPanelPlugin, RxPanelWidget
    }
};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SidePanelRects>()
            .init_resource::<GafState>()
            .add_plugins(EguiPlugin::default())
            .add_plugins((MenuPlugin, TxPanelPlugin, RxPanelPlugin))
            .add_systems(Startup, ui_setup)
            .add_systems(EguiPrimaryContextPass, ui_system);
    }
}

/// Screen-space horizontal extents of the side panels in logical points
/// (same unit and origin as [`Window::cursor_position`]).
///
/// Updated every frame by [`ui_system`] from the actual panel rectangles and
/// read by the camera plugin to ignore drags/scrolls that start over the
/// panels (egui cannot report panels shown on the background layer via
/// `Context::is_pointer_over_egui`, so the camera cannot rely on egui's own
/// input signals for them).
#[derive(Resource)]
pub struct SidePanelRects {
    /// Right edge of the left panel block (menu + Transmitter panel)
    pub left_max_x: f32,
    /// Left edge of the right (Receiver) panel
    pub right_min_x: f32,
}

impl Default for SidePanelRects {
    fn default() -> Self {
        Self {
            left_max_x: 0.0,
            right_min_x: f32::INFINITY,
        }
    }
}

impl SidePanelRects {
    /// Extents of the laid-out panels: the menu, plus the Transmitter and
    /// Receiver panels while shown (`None` once fully closed).
    fn from_panels(menu: egui::Rect, tx: Option<egui::Rect>, rx: Option<egui::Rect>) -> Self {
        Self {
            left_max_x: menu.max.x.max(tx.map_or(0.0, |rect| rect.max.x)),
            right_min_x: rx.map_or(f32::INFINITY, |rect| rect.min.x),
        }
    }
}

fn ui_setup(
    mut contexts: EguiContexts
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Install image loaders for egui
    egui_extras::install_image_loaders(ctx); // This gives us image support

    // UI style
    let mut dark_visuals = egui::Theme::Dark.default_visuals();
    // Squared corners
    dark_visuals.window_corner_radius = egui::CornerRadius::ZERO;
    dark_visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    dark_visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
    dark_visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
    dark_visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
    dark_visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
    dark_visuals.widgets.open.corner_radius = egui::CornerRadius::ZERO;
    // Fill the slider trailing area
    dark_visuals.slider_trailing_fill = true;
    // Alternate background color for striped tables
    dark_visuals.faint_bg_color = egui::Color32::BLACK; // Use black for faint background color
    ctx.set_visuals_of(egui::Theme::Dark, dark_visuals);

    Ok(())
}


fn ui_system(
    mut contexts: EguiContexts,
    // UI resources
    mut menu_widget: ResMut<MenuWidget>,
    mut tx_panel_widget: ResMut<TxPanelWidget>,
    mut rx_panel_widget: ResMut<RxPanelWidget>,
    // Tx state resources
    mut tx_carrier_state: ResMut<TxCarrierState>,
    mut tx_antenna_state: ResMut<TxAntennaState>,
    mut tx_antenna_beam_state: ResMut<TxAntennaBeamState>,
    tx_antenna_beam_footprint_state: Res<TxAntennaBeamFootprintState>,
    // Rx state resources
    mut rx_carrier_state: ResMut<RxCarrierState>,
    mut rx_antenna_state: ResMut<RxAntennaState>,
    mut rx_antenna_beam_state: ResMut<RxAntennaBeamState>,
    rx_antenna_beam_footprint_state: Res<RxAntennaBeamFootprintState>,
    // BSAR infos resource
    mut bsar_infos_state: ResMut<BsarInfosState>,
    // GAF plot texture cache
    mut gaf_state: ResMut<GafState>,
    // Panel extents for camera input blocking (see camera.rs)
    mut side_panel_rects: ResMut<SidePanelRects>
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Root Ui covering the whole viewport: the side panels are laid out inside it
    // (egui no longer shows panels directly on the Context)
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    // Side panel global menu
    let menu_response = menu_panel()
        .show(&mut viewport_ui, |ui| {
            menu_widget.ui(ui);
            // Register the remaining empty panel area so that egui reports the pointer
            // as being over the panel (keeps the camera from reacting, see camera.rs)
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        }
    );

    // The open/closed flags are passed as copies: `show_collapsible` only writes
    // them back for resizable panels (drag-to-close), which these are not, and
    // the panel contents borrow `menu_widget` — the menu buttons stay the only
    // place that opens or closes a panel.
    let mut is_rx_panel_opened = menu_widget.is_rx_panel_opened;
    let mut is_tx_panel_opened = menu_widget.is_tx_panel_opened;

        // Receiver panel
    let rx_panel_response = rx_panel()
        .show_collapsible(&mut viewport_ui, &mut is_rx_panel_opened, |ui| {
            rx_panel_widget.ui(
                ui,
                &menu_widget,
                &mut rx_carrier_state,
                &mut rx_antenna_state,
                &mut rx_antenna_beam_state,
                &mut bsar_infos_state,
            );
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    // Transmitter panel
    let tx_panel_response = tx_panel()
        .show_collapsible(&mut viewport_ui, &mut is_tx_panel_opened, |ui| {
            tx_panel_widget.ui(
                ui,
                &mut menu_widget,
                &mut rx_panel_widget,
                &mut tx_carrier_state,
                &mut tx_antenna_state,
                &mut tx_antenna_beam_state,
                &mut rx_carrier_state,
                &mut rx_antenna_state,
                &mut rx_antenna_beam_state,
            );
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    // Update the panel extents used to block the camera when the pointer is over
    // a panel and to anchor the Infos windows (includes the open/close animation
    // since the actual rects are used)
    *side_panel_rects = SidePanelRects::from_panels(
        menu_response.response.rect,
        tx_panel_response.as_ref().map(|r| r.response.rect),
        rx_panel_response.as_ref().map(|r| r.response.rect),
    );
    // Forces Rx updates in Monostatic case when Tx panel is closed
    if menu_widget.is_monostatic &&
       !menu_widget.was_monostatic &&
       !menu_widget.is_tx_panel_opened {
        rx_carrier_state.inner = tx_carrier_state.inner.clone();
        rx_antenna_state.inner = tx_antenna_state.inner.clone();
        rx_antenna_beam_state.inner = tx_antenna_beam_state.inner.clone();
        rx_panel_widget.transform_needs_update = true;
        rx_panel_widget.velocity_vector_needs_update = true;
        rx_panel_widget.system_needs_update = true;
        menu_widget.force_rx_system_update = true;
        menu_widget.was_monostatic = true;
    }
    
    // Tx and Rx Infos, flush against the side panels
    let [tx_infos_window, rx_infos_window] =
        infos_windows(&side_panel_rects, ctx.content_rect());
    tx_infos_window.show(ctx, |ui| {
        carrier_infos_ui(
            ui,
            &tx_carrier_state.inner,
            &tx_antenna_beam_footprint_state.inner,
            "tx"
        );
    });
    rx_infos_window.show(ctx, |ui| {
        carrier_infos_ui(
            ui,
            &rx_carrier_state.inner,
            &rx_antenna_beam_footprint_state.inner,
            "rx"
        );
    });

    // BSAR Infos
    let bsar_infos_window = egui::Window::new("BSAR Infos")
        .resizable(false)
        .constrain(false)
        .collapsible(true)
        .title_bar(true)
        .max_width(300.0)
        .enabled(true)
        .default_open(false)
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2::ZERO);
    bsar_infos_window.show(ctx, |ui| {
        bsar_infos_ui(
            ui,
            &bsar_infos_state.inner
        );
    });

    // Generalized Ambiguity Function plot window
    show_gaf_window(
        ctx,
        &mut menu_widget.is_gaf_opened,
        &mut gaf_state,
        &bsar_infos_state.inner,
        tx_carrier_state.bandwidth_mhz * 1e6, // MHz -> Hz
        tx_carrier_state.center_frequency_ghz * 1e9, // GHz -> Hz
    );

    Ok(())
}

/// Global menu panel, along the left edge of the window.
fn menu_panel() -> egui::Panel {
    egui::Panel::left("menu")
        .resizable(false)
        .default_size(48.0)
        .max_size(50.0)
        .show_separator_line(true)
}

/// Receiver panel, along the right edge of the window.
fn rx_panel() -> egui::Panel {
    egui::Panel::right("Receiver")
        .resizable(false)
        .default_size(300.0)
        .max_size(300.0)
        .show_separator_line(true)
}

/// Transmitter panel, right of the menu.
fn tx_panel() -> egui::Panel {
    egui::Panel::left("Transmitter")
        .resizable(false)
        .default_size(300.0)
        .max_size(300.0)
        .show_separator_line(true)
}

/// The Tx and Rx Infos windows, anchored flush against the inner edges of the
/// side panels. The edges come from the panels' actual rects, so the windows
/// follow the panels while they slide open or closed.
fn infos_windows(
    side_panel_rects: &SidePanelRects,
    content_rect: egui::Rect,
) -> [egui::Window<'static>; 2] {
    let infos_window = |title: &'static str| {
        egui::Window::new(title)
            .resizable(false)
            .constrain(false)
            .collapsible(true)
            .title_bar(true)
            .max_width(320.0)
            .enabled(true)
            .default_open(false)
    };
    // Anchor offsets are relative to egui's content rect. Once the Receiver
    // panel is fully closed its edge is infinite, and the Rx window sits against
    // the window border.
    [
        infos_window("Tx Infos").anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(side_panel_rects.left_max_x - content_rect.left(), 0.0),
        ),
        infos_window("Rx Infos").anchor(
            egui::Align2::RIGHT_TOP,
            egui::vec2((side_panel_rects.right_min_x - content_rect.right()).min(0.0), 0.0),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: egui::Vec2 = egui::vec2(1600.0, 1000.0);

    /// Headless replica of the `ui_system` layout: the real side panels (with
    /// placeholder contents, shown in the same order) and the real Infos
    /// windows, one frame per call.
    #[derive(Default)]
    struct Layout {
        ctx: egui::Context,
        frame: u32,
    }

    /// The panel extents and the outer rects of the Infos windows in a frame.
    struct LaidOut {
        panels: SidePanelRects,
        tx_infos: egui::Rect,
        rx_infos: egui::Rect,
    }

    impl Layout {
        fn frame(&mut self, mut is_tx_panel_opened: bool, mut is_rx_panel_opened: bool) -> LaidOut {
            self.frame += 1;
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, SCREEN)),
                // The panels slide over `animation_time`: the clock has to move
                time: Some(f64::from(self.frame) / 60.0),
                ..Default::default()
            };
            let mut laid_out = None;
            let mut output = self.ctx.run_ui(input, |ui| {
                let fill = |ui: &mut egui::Ui| {
                    ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
                };
                let menu = menu_panel().show(ui, fill);
                let rx = rx_panel().show_collapsible(ui, &mut is_rx_panel_opened, fill);
                let tx = tx_panel().show_collapsible(ui, &mut is_tx_panel_opened, fill);
                let panels = SidePanelRects::from_panels(
                    menu.response.rect,
                    tx.map(|r| r.response.rect),
                    rx.map(|r| r.response.rect),
                );
                let ctx = ui.ctx();
                let [tx_infos, rx_infos] = infos_windows(&panels, ctx.content_rect())
                    .map(|window| window.show(ctx, |ui| ui.label("…")).unwrap().response.rect);
                laid_out = Some(LaidOut { panels, tx_infos, rx_infos });
            });
            output.textures_delta.clear(); // nothing renders these frames
            laid_out.unwrap()
        }
    }

    /// The Infos windows' inner edges are at `tx_left` / `rx_right`, within
    /// half a pixel: egui snaps window positions to whole pixels, while a
    /// sliding panel's edge can sit on a half pixel.
    fn assert_infos_at(laid_out: &LaidOut, tx_left: f32, rx_right: f32) {
        assert!(
            (laid_out.tx_infos.left() - tx_left).abs() <= 0.5,
            "Tx Infos starts at x = {}, expected {tx_left}",
            laid_out.tx_infos.left()
        );
        assert!(
            (laid_out.rx_infos.right() - rx_right).abs() <= 0.5,
            "Rx Infos ends at x = {}, expected {rx_right}",
            laid_out.rx_infos.right()
        );
    }

    /// The Infos windows sit flush against the panels' inner edges (the Rx
    /// window against the window border once its panel is closed).
    fn assert_flush(laid_out: &LaidOut) {
        assert_infos_at(
            laid_out,
            laid_out.panels.left_max_x,
            laid_out.panels.right_min_x.min(SCREEN.x),
        );
    }

    /// Regression test: the Infos windows used to jump to their final position
    /// as soon as a panel was toggled, while the panel itself slides for
    /// `animation_time` (0.2 s since egui 0.35). They must stay flush against
    /// the panels on every frame of the slide, and come to rest where the
    /// former hard-coded anchors put them.
    #[test]
    fn infos_windows_follow_the_sliding_side_panels() {
        let mut layout = Layout::default();
        // Both panels closed: the menu alone on the left, nothing on the right
        let mut rest = layout.frame(false, false);
        for _ in 0..3 {
            rest = layout.frame(false, false);
        }
        assert_flush(&rest);
        assert_infos_at(&rest, 48.0, SCREEN.x);

        for (open, rest_tx_left, rest_rx_right) in [
            (true, 348.0, SCREEN.x - 300.0),
            (false, 48.0, SCREEN.x),
        ] {
            let mut sliding_frames = 0;
            for _ in 0..30 {
                let laid_out = layout.frame(open, open);
                assert_flush(&laid_out);
                if laid_out.panels.left_max_x > 48.0 && laid_out.panels.left_max_x < 348.0 {
                    sliding_frames += 1;
                }
            }
            assert!(sliding_frames > 0, "the panels never slid (open = {open})");
            let rest = layout.frame(open, open);
            assert_infos_at(&rest, rest_tx_left, rest_rx_right);
        }
    }
}