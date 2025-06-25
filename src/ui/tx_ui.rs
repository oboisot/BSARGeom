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
        .default_width(250.0)
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
            ui.horizontal(|ui| {
                ui.spacing_mut().slider_width = 200.0;
                ui.add(
                    egui::Slider::new(&mut tx_state.carrier_state.heading_rad, 0.0..=360.0)
                        .step_by(1.0)
                        .custom_formatter(|n, _| {
                            format!("{:.1}°", n as f32 * 0.1)
                        })
                        .custom_parser(|s| {
                            if let Ok(n) = s.parse::<f64>() {
                                Some((n * 10.0).to_radians())
                            } else {
                                None
                            }
                        })
                )
                .on_hover_text("Minimum contrast percentile. Use the text editor to set tenth of %");
            });

            ui.label("heading");

            ui.label("elevation");

            ui.label("bank");
            
            ui.separator();
            ui.vertical_centered(|ui| ui.label(
                egui::RichText::new("ANTENNA").strong()
            ));
            ui.separator();

            ui.label("Orientation");

            ui.label("Beamwidth");
            ui.separator();
        });
}
