// use bevy::prelude::*;
use bevy_egui::egui;

use crate::entities::{CarrierState, AntennaBeamFootprintState};

pub fn carrier_infos_ui(
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

            // Local incidence min infos
            ui.label("Local incidence min:");
            ui.label(format!("{:.3}°", antenna_beam_footprint_state.loc_incidence_min_deg));
            ui.end_row();
            // Local incidence center infos
            ui.label("Local incidence center:");
            ui.label(format!("{:.3}°", antenna_beam_footprint_state.loc_incidence_center_deg));
            ui.end_row();
            // Local incidence max infos
            ui.label("Local incidence max:");
            ui.label(format!("{:.3}°", antenna_beam_footprint_state.loc_incidence_max_deg));
            ui.end_row();

            // Antenna squint infos
            ui.label("Antenna squint:");
            ui.label(format!("{:.3}°", antenna_beam_footprint_state.antenna_squint_deg));
            ui.end_row();

            // Ground range swath infos
            ui.label("Ground range swath:");
            ui.label(
                if antenna_beam_footprint_state.ground_range_swath_m >= 1e3 {
                    format!("{:.3} km", antenna_beam_footprint_state.ground_range_swath_m * 1e-3)
                } else {
                    format!("{:.3} m", antenna_beam_footprint_state.ground_range_swath_m)
                } 
            );
            ui.end_row();

            // Ground range swath infos
            ui.label("Footprint area:");
            ui.label(
                if antenna_beam_footprint_state.area_m2 >= 1e5 {
                    format!("{:.3} km²", antenna_beam_footprint_state.area_m2 * 1e-6)
                } else {
                    format!("{:.3} m²", antenna_beam_footprint_state.area_m2)
                } 
            );
            ui.end_row();

            // Ground range swath infos
            ui.label("Illumination time:");
            ui.label(
                if antenna_beam_footprint_state.illumination_time_s >= 60.0 {
                    format!("{:.3} min", antenna_beam_footprint_state.illumination_time_s/60.0)
                } else {
                    format!("{:.3} s", antenna_beam_footprint_state.illumination_time_s)
                } 
            );
            ui.end_row();

            // Ground range swath infos
            ui.label("Ground angular velocity:");
            ui.label(format!("{:.3} °/s", antenna_beam_footprint_state.ground_angular_velocity_degps));
            ui.end_row();
        });
}
