use bevy::prelude::*;
// use bevy_egui::{EguiContextPass, EguiPlugin};
use bevy_egui::EguiPlugin;

use crate::ui::{tx_ui, rx_ui};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true }) // For now crash when closing in
            // .add_systems(EguiContextPass, (tx_ui, rx_ui)); 
            .add_plugins(EguiPlugin { enable_multipass_for_primary_context: false })
            .add_systems(Update, (tx_ui, rx_ui));
    }
}
