use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use egui_extras;

use crate::{
    scene::{
        TxCarrierState, TxAntennaState, TxAntennaBeamState, TxAntennaBeamFootprintState,
        RxCarrierState, RxAntennaState, RxAntennaBeamState, RxAntennaBeamFootprintState,
        BsarInfosState
    },
    ui::{
        bsar_infos_ui, carrier_infos_ui,
        MenuPlugin, MenuWidget, TxPanelPlugin, TxPanelWidget, RxPanelPlugin, RxPanelWidget
    }
};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SidePanelRects>()
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
/// read by the camera plugin to keep [`bevy_panorbit_camera`] from reacting to
/// drags/scrolls performed over the panels (egui cannot report panels shown on
/// the background layer via `Context::is_pointer_over_area`, so the camera
/// cannot rely on egui's own focus signals for them).
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
    // Panel extents for camera input blocking (see camera.rs)
    mut side_panel_rects: ResMut<SidePanelRects>
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Root Ui covering the whole viewport: the side panels are laid out inside it
    // (egui 0.34 deprecates showing panels directly on the Context)
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    // Side panel global menu
    let menu_response = egui::Panel::left("menu")
        .resizable(false)
        .default_size(48.0)
        .max_size(50.0)
        .show_separator_line(true)
        .show_inside(&mut viewport_ui, |ui| {
            menu_widget.ui(ui);
            // Register the remaining empty panel area so that egui reports the pointer
            // as being over the panel (keeps the camera from reacting, see camera.rs)
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        }
    );

        // Receiver panel
    let rx_panel_response = egui::Panel::right("Receiver")
        .resizable(false)
        .default_size(300.0)
        .max_size(300.0)
        .show_separator_line(true)
        .show_animated_inside(&mut viewport_ui, menu_widget.is_rx_panel_opened, |ui| {
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
    let tx_panel_response = egui::Panel::left("Transmitter")
        .resizable(false)
        .default_size(300.0)
        .max_size(300.0)
        .show_separator_line(true)
        .show_animated_inside(&mut viewport_ui, menu_widget.is_tx_panel_opened, |ui| {
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
    // a panel (includes the open/close animation since the actual rects are used)
    side_panel_rects.left_max_x = menu_response.response.rect.max.x.max(
        tx_panel_response.as_ref().map_or(0.0, |r| r.response.rect.max.x)
    );
    side_panel_rects.right_min_x = rx_panel_response
        .as_ref()
        .map_or(f32::INFINITY, |r| r.response.rect.min.x);
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
    
    // Tx Infos
    let tx_infos_window = egui::Window::new("Tx Infos")
        .resizable(false)
        .constrain(false)
        .collapsible(true)
        .title_bar(true)
        .max_width(320.0)
        .enabled(true)
        .default_open(false)
        .anchor(
            egui::Align2::LEFT_TOP,
            if menu_widget.is_tx_panel_opened {
                egui::Vec2::new(348.0, 0.0)
            } else {
                egui::Vec2::new(48.0, 0.0)
            }
        );
    tx_infos_window.show(ctx, |ui| {
        carrier_infos_ui(
            ui,
            &tx_carrier_state.inner,
            &tx_antenna_beam_footprint_state.inner,
            "tx"
        );
    });

    // Rx Infos
    let rx_infos_window = egui::Window::new("Rx Infos")
        .resizable(false)
        .constrain(false)
        .collapsible(true)
        .title_bar(true)
        .max_width(320.0)
        .enabled(true)
        .default_open(false)
        .anchor(
            egui::Align2::RIGHT_TOP,
            if menu_widget.is_rx_panel_opened {
                egui::Vec2::new(-300.0, 0.0)
            } else {
                egui::Vec2::new(0.0, 0.0)
            }            
        );
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
    
    Ok(())
}