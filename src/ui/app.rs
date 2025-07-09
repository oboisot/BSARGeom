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
            .add_plugins(EguiPlugin::default())
            .add_plugins((MenuPlugin, TxPanelPlugin, RxPanelPlugin))
            .add_systems(Startup, ui_setup)
            .add_systems(EguiPrimaryContextPass, ui_system);
    }
}

fn ui_setup(
    mut contexts: EguiContexts
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Install image loaders for egui
    egui_extras::install_image_loaders(&ctx); // This gives us image support

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
    bsar_infos_state: Res<BsarInfosState>
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
        .default_width(300.0)
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
        .default_width(300.0)
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