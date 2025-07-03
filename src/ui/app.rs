use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};
use egui_extras;

use crate::{
    scene::{
        TxCarrierState, TxAntennaState, TxAntennaBeamState,
        RxCarrierState, RxAntennaState, RxAntennaBeamState
    },
    ui::{MenuPlugin, MenuWidget, TxPanelPlugin, TxPanelWidget, RxPanelPlugin, RxPanelWidget}
};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(EguiPlugin { enable_multipass_for_primary_context: false })
            .add_plugins(EguiPlugin::default())
            .add_plugins((MenuPlugin, TxPanelPlugin, RxPanelPlugin))
            .add_systems(Startup, ui_setup)
            .add_systems(EguiPrimaryContextPass, ui_system);
    }
}

fn ui_setup(
    mut contexts: EguiContexts,
    mut global_settings: ResMut<EguiGlobalSettings>
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Install image loaders for egui
    egui_extras::install_image_loaders(&ctx); // This gives us image support

    // Allow egui to absorb Bevy input events
    global_settings.enable_absorb_bevy_input_system = true; // see: https://docs.rs/bevy_egui/latest/bevy_egui/struct.EguiGlobalSettings.html

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
    //
    dark_visuals.slider_trailing_fill = true; // Fill the slider trailing area
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
    // Rx state resources
    mut rx_carrier_state: ResMut<RxCarrierState>,
    mut rx_antenna_state: ResMut<RxAntennaState>,
    mut rx_antenna_beam_state: ResMut<RxAntennaBeamState>
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Side panel global menu
    egui::SidePanel::left("menu")
        .resizable(false)
        .default_width(48.0)
        .max_width(50.0)
        .show_separator_line(true)
        .show(ctx, |ui| {
            menu_widget.ui(ui);
        }
    );

    // Transmitter panel
    egui::SidePanel::left("Transmitter")
        .resizable(false)
        .default_width(260.0)
        .max_width(300.0)
        .show_separator_line(true)
        .show_animated(ctx, menu_widget.is_tx_panel_opened, |ui| {
            tx_panel_widget.ui(
                ui,
                &mut tx_carrier_state,
                &mut tx_antenna_state,
                &mut tx_antenna_beam_state
            );
        });
    
    // Rceiver panel
    egui::SidePanel::right("Receiver")
        .resizable(false)
        .default_width(260.0)
        .max_width(300.0)
        .show_separator_line(true)
        .show_animated(ctx, menu_widget.is_rx_panel_opened, |ui| {
            rx_panel_widget.ui(
                ui,
                &mut rx_carrier_state,
                &mut rx_antenna_state,
                &mut rx_antenna_beam_state
            );
        });
    
    Ok(())
}