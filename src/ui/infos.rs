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
            ui.label("Slant range min:");
            ui.label(
                if let Some(range_min_m) = bsar_infos.range_min_m {
                    if range_min_m >= 1e3 {
                        format!("{:.3} km", range_min_m * 1e-3)
                    } else {
                        format!("{:.3} m", range_min_m)
                    }
                } else {
                    "N/A m".to_string()
                }
            );
            ui.end_row();
            // Slant range center infos
            ui.label("Slant range center:");
            ui.label(
                if let Some(range_center_m) = bsar_infos.range_center_m {
                    if range_center_m >= 1e3 {
                        format!("{:.3} km", range_center_m * 1e-3)
                    } else {
                        format!("{:.3} m", range_center_m)
                    }
                } else {
                    "N/A m".to_string()
                }
            );
            ui.end_row();
            // Slant range max infos
            ui.label("Slant range max:");
            ui.label(
                if let Some(range_max_m) = bsar_infos.range_max_m {
                    if range_max_m >= 1e3 {
                        format!("{:.3} km", range_max_m * 1e-3)
                    } else {
                        format!("{:.3} m", range_max_m)
                    }
                } else {
                    "N/A m".to_string()
                }
            );
            ui.end_row();
            // Tx/Rx direct range infos
            ui.label("Tx/Rx direct range:");
            ui.label(
                if let Some(direct_range_m) = bsar_infos.direct_range_m {
                    if direct_range_m >= 1e3 {
                        format!("{:.3} km", direct_range_m * 1e-3)
                    } else {
                        format!("{:.3} m", direct_range_m)
                    }
                } else {
                    "N/A m".to_string()
                }
            );
            ui.end_row();
            // Bistatic angle infos
            ui.label("Bistatic angle:");
            ui.label(
                if let Some(bistatic_angle_deg) = bsar_infos.bistatic_angle_deg {
                    format!("{:.3} °", bistatic_angle_deg)
                } else {
                    "N/A °".to_string()
                }
            );
            ui.end_row();
            // Slant range res infos
            ui.label("Slant range res.:");
            ui.label(
                if let Some(slant_range_resolution_m) = bsar_infos.slant_range_resolution_m {
                    format!("{:.3} m", slant_range_resolution_m)
                } else {
                    "N/A m".to_string()
                }
            );
            ui.end_row();
            // Ground range res infos
            ui.label("Ground range res.:");
            ui.label(
                if let Some(ground_range_resolution_m) = bsar_infos.ground_range_resolution_m {
                    format!("{:.3} m", ground_range_resolution_m)
                } else {
                    "N/A m".to_string()
                }
            );
            ui.end_row();
            // Slant lateral res infos
            ui.label("Slant lateral res.:");
            ui.label(
                if let Some(slant_lateral_resolution_m) = bsar_infos.slant_lateral_resolution_m {
                    format!("{:.3} m", slant_lateral_resolution_m)
                } else {
                    "N/A m".to_string()
                }
            );
            ui.end_row();
            // Ground lateral res infos
            ui.label("Ground lateral res.:");
            ui.label(
                if let Some(ground_lateral_resolution_m) = bsar_infos.ground_lateral_resolution_m {
                    format!("{:.3} m", ground_lateral_resolution_m)
                } else {
                    "N/A m".to_string()
                }
            );
            ui.end_row();
            // Resolution area infos
            ui.label("Resolution area:");
            ui.label(
                if let Some(resolution_area_m2) = bsar_infos.resolution_area_m2 {
                    if resolution_area_m2 >= 1e5 {
                        format!("{:.3} km²", resolution_area_m2 * 1e-6)
                    } else {
                        format!("{:.3} m²", resolution_area_m2)
                    }
                } else {
                    "N/A m²".to_string()
                }
            );
            ui.end_row();
            // Doppler frequency infos
            ui.label("Doppler frequency:");
            ui.label(
                if let Some(doppler_frequency_hz) = bsar_infos.doppler_frequency_hz {
                    if doppler_frequency_hz >= 1e3 {
                        format!("{:.3} kHz", doppler_frequency_hz * 1e-3)
                    } else {
                        format!("{:.3} Hz", doppler_frequency_hz)
                    }
                } else {
                    "N/A Hz".to_string()
                }
            );
            ui.end_row();
            // Doppler rate infos
            ui.label("Doppler rate:");
            ui.label(
                if let Some(doppler_rate_hzps) = bsar_infos.doppler_rate_hzps {
                    if doppler_rate_hzps.abs() >= 1e3 {
                        format!("{:.3} kHz/s", doppler_rate_hzps * 1e-3)
                    } else {
                        format!("{:.3} Hz/s", doppler_rate_hzps)
                    }
                } else {
                    "N/A Hz/s".to_string()
                }
            );
            ui.end_row();
            // Integration time infos
            ui.label("Integration time:");
            ui.label(
                if let Some(integration_time_s) = bsar_infos.integration_time_s {
                    format!("{:.3} s", integration_time_s)
                } else {
                    "N/A s".to_string()
                }
            );
            ui.end_row();
            // Processed Doppler bandwidth infos
            ui.label("Processed Dop. band.:");
            ui.label(
                if let Some(processed_doppler_bandwidth_hz) = bsar_infos.processed_doppler_bandwidth_hz {
                    if processed_doppler_bandwidth_hz >= 1e3 {
                        format!("{:.3} kHz", processed_doppler_bandwidth_hz * 1e-3)
                    } else {
                        format!("{:.3} Hz", processed_doppler_bandwidth_hz)
                    }
                } else {
                    "N/A Hz".to_string()
                }
            );
            ui.end_row();
            // NESZ infos
            ui.label("NESZ:");
            ui.label(
                if let Some(nesz) = bsar_infos.nesz {
                    format!("{:.3} dBm²/m", 10.0*nesz.log10())
                } else {
                    "N/A dBm²/m²".to_string()
                }
            );
            ui.end_row();
        });
}