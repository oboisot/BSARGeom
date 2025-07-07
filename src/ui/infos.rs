// use bevy::prelude::*;
use bevy_egui::egui;

use crate::entities::{CarrierState, AntennaBeamFootprintState};

pub fn infos_ui(
    ui: &mut egui::Ui,
    carrier_state: &CarrierState,
    antenna_beam_footprint_state: &AntennaBeamFootprintState,
    name: &str,
) {
    egui::Grid::new(format!("{}_carrier_infos_grid", name))
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            // Carrier position ENU
            ui.label("Carrier position:")
                .on_hover_text(
                    egui::RichText::new("In East North Up (ENU) coordinates (x, y, z).")
                        .color(egui::Color32::from_rgb(200, 200, 200))
                        .monospace()
                );
            ui.label(format!("({:.1} m, {:.1} m, {:.1} m)", carrier_state.position_m.x, carrier_state.position_m.y, carrier_state.position_m.z));
            ui.end_row();
            // Carrier velocity vector ENU
            ui.label("Carrier velocity vector:")
                .on_hover_text(
                    egui::RichText::new("In East North Up (ENU) coordinates (vx, vy, vz).")
                        .color(egui::Color32::from_rgb(200, 200, 200))
                        .monospace()
                );
            ui.label(format!(
                "({:.1} m/s, {:.1} m/s, {:.1} m/s)",
                carrier_state.velocity_vector_mps.x,
                carrier_state.velocity_vector_mps.y,
                carrier_state.velocity_vector_mps.z
            ));
            ui.end_row();
        });

    ui.separator();

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
