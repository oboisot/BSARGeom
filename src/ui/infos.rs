// use bevy::prelude::*;
use bevy_egui::egui;

use crate::entities::{CarrierState, AntennaBeamFootprintState};

pub fn infos_ui(
    ui: &mut egui::Ui,
    carrier_state: &CarrierState,
    antenna_beam_footprint_state: &AntennaBeamFootprintState,
    name: &str,
) {
    egui::Grid::new(format!("{}_infos_grid", name))
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            // Slant range min infos
            ui.label("Slant range min:");
            ui.label(
                if antenna_beam_footprint_state.range_min_m >= 1e3 {
                    format!("{:.3} km", antenna_beam_footprint_state.range_min_m * 1e-3)
                } else {
                    format!("{:.3} m", antenna_beam_footprint_state.range_min_m)
                }
            );
            ui.end_row();
            // Slant range center infos
            ui.label("Slant range center:");
            ui.label(
                if antenna_beam_footprint_state.range_center_m >= 1e3 {
                    format!("{:.3} km", antenna_beam_footprint_state.range_center_m * 1e-3)
                } else {
                    format!("{:.3} m", antenna_beam_footprint_state.range_center_m)
                } 
            );
            ui.end_row();
            // Slant range max infos
            ui.label("Slant range max:");
            ui.label(
                if antenna_beam_footprint_state.range_max_m >= 1e3 {
                    format!("{:.3} km", antenna_beam_footprint_state.range_max_m * 1e-3)
                } else {
                    format!("{:.3} m", antenna_beam_footprint_state.range_max_m)
                } 
            );
            ui.end_row();
        });

}
