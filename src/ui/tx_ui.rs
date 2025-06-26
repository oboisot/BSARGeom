use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::scene::TxState;

// use crate::entities::{AntennaBeamState, AntennaState, CarrierState};

pub fn tx_ui(
    mut contexts: EguiContexts,
    mut tx_state: ResMut<TxState>,
) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("Transmitter")
        .resizable(false)
        .default_width(200.0)
        .max_width(300.0)
        .show_separator_line(true)
        .show(ctx, |ui| {
            ui.separator();
            ui.vertical_centered(|ui| ui.label(
                egui::RichText::new("TRANSMITTER SETTINGS")
                    .size(15.0)
                    .strong()
            ));
            ui.separator();

            ui.separator();
            ui.vertical_centered(|ui| ui.label(
                egui::RichText::new("CARRIER").strong()
            ));
            ui.separator();

            // Carrier heading angle
            egui::Grid::new("tx_carrier_grid")
                .num_columns(2)
                .striped(true)
                .spacing([10.0, 5.0])
                .show(ui, |ui| {
                    ui.label("Heading: ")
                      .on_hover_text("Set the heading angle of the carrier.");
                    ui.add(
                        egui::Slider::new(&mut tx_state.carrier_state.heading_rad, 0.0..=360.0)
                            .suffix("°")
                            .custom_formatter(|n, _| format!("{:.2}", n.to_degrees()))
                    )
                    .on_hover_text("Set the heading angle of the carrier.");
                    ui.end_row();

                    ui.label("Elevation: ")
                      .on_hover_text("Set the elevation angle of the carrier.");
                    ui.add(
                        egui::Slider::new(&mut tx_state.carrier_state.elevation_rad, -90.0..=90.0)
                            .suffix("°")
                            .fixed_decimals(2)
                    )
                    .on_hover_text("Set the elevation angle of the carrier.");
                    ui.end_row();

                    ui.label("Bank: ")
                      .on_hover_text("Set the bank angle of the carrier.");
                    ui.add(
                        egui::Slider::new(&mut tx_state.carrier_state.bank_rad, -90.0..=90.0)
                            .suffix("°")
                            .fixed_decimals(2)
                    )
                    .on_hover_text("Set the bank angle of the carrier.");
                    ui.end_row();
                });
            
            ui.separator();
            ui.vertical_centered(|ui| ui.label(
                egui::RichText::new("ANTENNA").strong()
            ));
            ui.separator();

            // Carrier heading angle
            ui.label("Orientation");
            ui.separator();
            // Antenna orientation settings
            egui::Grid::new("tx_antenna_orientation_grid")
                .num_columns(2)
                .striped(true)
                .spacing([10.0, 5.0])
                .show(ui, |ui| {
                    ui.label("Heading: ")
                      .on_hover_text("Set the heading angle of the antenna.");
                    ui.add(
                        egui::Slider::new(&mut tx_state.antenna_state.heading_rad, -180.0..=180.0)
                            .suffix("°")
                            .fixed_decimals(2)
                    )
                    .on_hover_text("Set the heading angle of the antenna.");
                    ui.end_row();

                    ui.label("Elevation: ")
                      .on_hover_text("Set the elevation angle of the antenna.");
                    ui.add(
                        egui::Slider::new(&mut tx_state.antenna_state.elevation_rad, -90.0..=90.0)
                            .suffix("°")
                            .fixed_decimals(2)
                    )
                    .on_hover_text("Set the elevation angle of the antenna.");
                    ui.end_row();

                    ui.label("Bank: ")
                      .on_hover_text("Set the bank angle of the antenna.");
                    ui.add(
                        egui::Slider::new(&mut tx_state.antenna_state.bank_rad, -90.0..=90.0)
                            .suffix("°")
                            .fixed_decimals(2)
                    )
                    .on_hover_text("Set the bank angle of the antenna.");
                    ui.end_row();
                });

            ui.label("Beamwidth (half-power)");
            ui.separator();
            // Antenna beamwidth settings
            egui::Grid::new("tx_antenna_beamwidth_grid")
                .num_columns(2)
                .striped(true)
                .spacing([10.0, 5.0])
                .show(ui, |ui| {
                    ui.label("Elevation: ")
                      .on_hover_text("Set the elevation half-power beamwidth of the antenna.");
                    ui.add(
                        egui::Slider::new(&mut tx_state.antenna_beam_state.elevation_beam_width_rad, 0.0..=90.0)
                            .suffix("°")
                            .fixed_decimals(2)
                    )
                    .on_hover_text("Set the elevation half-power beamwidth of the antenna.");
                    ui.end_row();

                    ui.label("Azimuth: ")
                      .on_hover_text("Set the azimuth half-power beamwidth of the antenna.");
                    ui.add(
                        egui::Slider::new(&mut tx_state.antenna_beam_state.azimuth_beam_width_rad, 0.0..=90.0)
                            .suffix("°")
                            .step_by(0.01)
                            .custom_formatter(|n, _| format!("{:.2}", n.to_degrees()))
                            .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                    )
                    .on_hover_text("Set the azimuth half-power beamwidth of the antenna.");
                    ui.end_row();
                });
        });
}
