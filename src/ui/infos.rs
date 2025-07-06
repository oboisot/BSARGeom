// use bevy::prelude::*;
use bevy_egui::egui;

use crate::entities::{CarrierState, AntennaBeamFootprintState};

pub fn infos_ui(
    ui: &mut egui::Ui,
    carrier_state: &CarrierState,
    antenna_beam_footprint_state: &AntennaBeamFootprintState
) {
    // Slant range min infos
    ui.label(format!("Slant range min: {}",
        if antenna_beam_footprint_state.range_min_m > 1e3 {
            format!("{:.3} km", antenna_beam_footprint_state.range_min_m * 1e-3)
        } else {
            format!("{:.3} m", antenna_beam_footprint_state.range_min_m)
        } 
    ));
    // Slant range center infos
    ui.label(format!("Slant range center: {}",
        if antenna_beam_footprint_state.range_center_m > 1e3 {
            format!("{:.3} km", antenna_beam_footprint_state.range_center_m * 1e-3)
        } else {
            format!("{:.3} m", antenna_beam_footprint_state.range_center_m)
        } 
    ));
    // Slant range max infos
    ui.label(format!("Slant range max: {}",
        if antenna_beam_footprint_state.range_max_m > 1e3 {
            format!("{:.3} km", antenna_beam_footprint_state.range_max_m * 1e-3)
        } else {
            format!("{:.3} m", antenna_beam_footprint_state.range_max_m)
        } 
    ));
}
