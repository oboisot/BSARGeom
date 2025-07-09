// use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    bsar::BsarInfos,
    entities::{CarrierState, AntennaBeamFootprintState}
};

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


pub fn bsar_infos_ui(
    ui: &mut egui::Ui,
    bsar_infos: &BsarInfos,
) {
    egui::Grid::new("bsar_infos_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            // Slant range min infos
            ui.label("Slant range min:").on_hover_text(
                egui::RichText::new("The minimum BSAR system slant range between Tx/footprint/Rx.\nnote: the footprint is heuristically determined by choosing the one with the smallest ground range swath.")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace()
                );
            ui.label(
                if bsar_infos.range_min_m >= 1e3 {
                    format!("{:.3} km", bsar_infos.range_min_m * 1e-3)
                } else {
                    format!("{:.3} m", bsar_infos.range_min_m)
                }
            );
            ui.end_row();
            // Slant range center infos
            ui.label("Slant range center:").on_hover_text(
                egui::RichText::new("The BSAR system slant range between Tx/center/Rx.")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace()
                );
            ui.label(
                if bsar_infos.range_center_m >= 1e3 {
                    format!("{:.3} km", bsar_infos.range_center_m * 1e-3)
                } else {
                    format!("{:.3} m", bsar_infos.range_center_m)
                }
            );
            ui.end_row();
            // Slant range max infos
            ui.label("Slant range max:").on_hover_text(
                egui::RichText::new("The maximum BSAR system slant range between Tx/footprint/Rx.\nnote: the footprint is heuristically determined by choosing the one with the smallest ground range swath.")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace()
            );
            ui.label(
                if bsar_infos.range_max_m >= 1e3 {
                    format!("{:.3} km", bsar_infos.range_max_m * 1e-3)
                } else {
                    format!("{:.3} m", bsar_infos.range_max_m)
                }
            );
            ui.end_row();
            // Tx/Rx direct range infos
            ui.label("Tx/Rx direct range:");
            ui.label(
                if bsar_infos.direct_range_m >= 1e3 {
                    format!("{:.3} km", bsar_infos.direct_range_m * 1e-3)
                } else {
                    format!("{:.3} m", bsar_infos.direct_range_m)
                }
            );
            ui.end_row();
            // Bistatic angle infos
            ui.label("Bistatic angle:");
            ui.label(format!("{:.3} °", bsar_infos.bistatic_angle_deg));
            ui.end_row();
            // Slant range res infos
            ui.label("Slant range res.:");
            ui.label(format!("{:.3} m", bsar_infos.slant_range_resolution_m));
            ui.end_row();
            // Ground range res infos
            ui.label("Ground range res.:");
            ui.label(format!("{:.3} m", bsar_infos.ground_range_resolution_m));
            ui.end_row();
            // Slant lateral res infos
            ui.label("Slant lateral res.:");
            ui.label(format!("{:.3} m", bsar_infos.slant_lateral_resolution_m));
            ui.end_row();
            // Ground lateral res infos
            ui.label("Ground lateral res.:");
            ui.label(format!("{:.3} m", bsar_infos.ground_lateral_resolution_m));
            ui.end_row();
            // Resolution area infos
            ui.label("Resolution area:");
            ui.label(
                if bsar_infos.resolution_area_m2 >= 1e5 {
                    format!("{:.3} km²", bsar_infos.resolution_area_m2 * 1e-6)
                } else {
                    format!("{:.3} m²", bsar_infos.resolution_area_m2)
                }
            );
            ui.end_row();
            // Doppler frequency infos
            ui.label("Doppler frequency:");
            ui.label(
                if bsar_infos.doppler_frequency_hz >= 1e3 {
                    format!("{:.3} kHz", bsar_infos.doppler_frequency_hz * 1e-3)
                } else {
                    format!("{:.3} Hz", bsar_infos.doppler_frequency_hz)
                }
            );
            ui.end_row();
            // Doppler rate infos
            ui.label("Doppler rate:");
            ui.label(
                if bsar_infos.doppler_rate_hzps.abs() >= 1e3 {
                    format!("{:.3} kHz/s", bsar_infos.doppler_rate_hzps * 1e-3)
                } else {
                    format!("{:.3} Hz/s", bsar_infos.doppler_rate_hzps)
                }
            );
            ui.end_row();
            // Integration time infos
            ui.label("Integration time:");
            ui.label(format!("{:.3} s", bsar_infos.integration_time_s));
            ui.end_row();
            // Processed Doppler bandwidth infos
            ui.label("Processed Dop. band.:");
            ui.label(
                if bsar_infos.processed_doppler_bandwidth_hz >= 1e3 {
                    format!("{:.3} kHz", bsar_infos.processed_doppler_bandwidth_hz * 1e-3)
                } else {
                    format!("{:.3} Hz", bsar_infos.processed_doppler_bandwidth_hz)
                }
            );
            ui.end_row();
            // NESZ infos
            ui.label("NESZ:");
            ui.label(format!("{:.3} dBm²/m²", 10.0*bsar_infos.nesz.log10()));
            ui.end_row();
        });
}