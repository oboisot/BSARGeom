use std::f64::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    constants::UI_STEP_RAD,
    entities::{
        Antenna, AntennaBeam, Carrier,
        antenna_beam_transform_from_state,
        antenna_transform_from_state,
        carrier_transform_from_state
    },
    scene::{Tx, TxCarrierState, TxAntennaState, TxAntennaBeamState},
};

pub fn tx_ui(
    mut contexts: EguiContexts,
    mut tx_state: (ResMut<TxCarrierState>, ResMut<TxAntennaState>, ResMut<TxAntennaBeamState>),
    tx_carrier_q: Query<(&mut Transform, &Children), (With<Tx>, With<Carrier>)>,
    tx_antenna_q: Query<(&mut Transform, &Children), (Without<Tx>, With<Antenna>)>,
    tx_antenna_beam_q: Query<&mut Transform, (Without<Tx>, Without<Antenna>, With<AntennaBeam>)>,
) {
    let ctx = contexts.ctx_mut();

    let mut old_state = 0.0f64;
    let mut needs_update = false;

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
                    // ***** Carrier height ***** //
                    ui.label("Height: ")
                      .on_hover_text("Set the height of the carrier relative to ground.");
                    old_state = tx_state.0.inner.height_m;
                    ui.add(
                        egui::DragValue::new(&mut tx_state.0.inner.height_m)
                            .speed(10.0)
                            .range(0.0..=1e6)
                            .fixed_decimals(3)
                            .suffix(" m")
                    )
                    .on_hover_text("Set the heading angle of the carrier.");
                    if old_state != tx_state.0.inner.height_m {
                        needs_update = true;
                    }
                    ui.end_row();

                    // ***** Carrier heading ***** //
                    ui.label("Heading: ")
                      .on_hover_text("Set the heading angle of the carrier.");
                    old_state = tx_state.0.inner.heading_rad;
                    ui.add(
                        egui::Slider::new(&mut tx_state.0.inner.heading_rad, 0.0..=TAU)
                            .step_by(UI_STEP_RAD)
                            .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                            .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                    )
                    .on_hover_text("Set the heading angle of the carrier.");
                    if old_state != tx_state.0.inner.heading_rad {
                        needs_update = true;
                    }
                    ui.end_row();

                    // ***** Carrier elevation ***** //
                    ui.label("Elevation: ")
                      .on_hover_text("Set the elevation angle of the carrier.");
                    old_state = tx_state.0.inner.elevation_rad;
                    ui.add(
                        egui::Slider::new(&mut tx_state.0.inner.elevation_rad, -FRAC_PI_2..=FRAC_PI_2)
                            .step_by(UI_STEP_RAD)
                            .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                            .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                    )
                    .on_hover_text("Set the elevation angle of the carrier.");
                    if old_state != tx_state.0.inner.elevation_rad {
                        needs_update = true;
                    }
                    ui.end_row();

                    // ***** Carrier bank ***** //
                    ui.label("Bank: ")
                      .on_hover_text("Set the bank angle of the carrier.");
                    old_state = tx_state.0.inner.bank_rad;
                    ui.add(
                        egui::Slider::new(&mut tx_state.0.inner.bank_rad, -FRAC_PI_2..=FRAC_PI_2)
                            .step_by(UI_STEP_RAD)
                            .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                            .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                    )
                    .on_hover_text("Set the bank angle of the carrier.");
                    if old_state != tx_state.0.inner.bank_rad {
                        needs_update = true;
                    }
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
                    // ***** Antenna heading ***** //
                    let hover_text = egui::RichText::new(
                        "Set the heading angle of the antenna:\n  -90° => left-looking\n    0° => forward-looking\n  +90° => right-looking\n ±180° => backward-looking"
                    )
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                    ui.label("Heading: ").on_hover_text(hover_text.clone());
                    old_state = tx_state.1.inner.heading_rad;
                    ui.add(
                        egui::Slider::new(&mut tx_state.1.inner.heading_rad, -PI..=PI)
                            .step_by(UI_STEP_RAD)
                            .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                            .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                    )
                    .on_hover_text(hover_text);
                    if old_state != tx_state.1.inner.heading_rad {
                        needs_update = true;
                    }
                    ui.end_row();

                    // ***** Antenna elevation ***** //
                    let hover_text = egui::RichText::new(
                        "Set the elevation angle of the antenna:\n  -90° => vertical-looking\n    0° => horizontal-looking"
                    )
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                    ui.label("Elevation: ").on_hover_text(hover_text.clone());
                    old_state = tx_state.1.inner.elevation_rad;
                    ui.add(
                        egui::Slider::new(&mut tx_state.1.inner.elevation_rad, -FRAC_PI_2..=0.0)
                            .step_by(UI_STEP_RAD)
                            .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                            .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                    )
                    .on_hover_text(hover_text);
                    if old_state != tx_state.1.inner.elevation_rad {
                        needs_update = true;
                    }
                    ui.end_row();

                    // ***** Antenna bank ***** //
                    ui.label("Bank: ")
                      .on_hover_text("Set the bank angle of the antenna.");
                    old_state = tx_state.1.inner.bank_rad;
                    ui.add(
                        egui::Slider::new(&mut tx_state.1.inner.bank_rad, -FRAC_PI_2..=FRAC_PI_2)
                            .step_by(UI_STEP_RAD)
                            .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                            .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                    )
                    .on_hover_text("Set the bank angle of the antenna.");
                    if old_state != tx_state.1.inner.bank_rad {
                        needs_update = true;
                    }
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
                    // ***** Antenna beamwidth elevation ***** //
                    ui.label("Elevation: ")
                      .on_hover_text("Set the elevation half-power beamwidth of the antenna.");
                    old_state = tx_state.2.inner.elevation_beam_width_rad;
                    ui.add(
                        egui::Slider::new(&mut tx_state.2.inner.elevation_beam_width_rad, 0.0..=FRAC_PI_2)
                            .step_by(UI_STEP_RAD)
                            .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                            .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                    )
                    .on_hover_text("Set the elevation half-power beamwidth of the antenna.");
                    if old_state != tx_state.2.inner.elevation_beam_width_rad {
                        needs_update = true;
                    }
                    ui.end_row();

                    // ***** Antenna azimuth ***** //
                    ui.label("Azimuth: ")
                      .on_hover_text("Set the azimuth half-power beamwidth of the antenna.");
                    old_state = tx_state.2.inner.azimuth_beam_width_rad;
                    ui.add(
                        egui::Slider::new(&mut tx_state.2.inner.azimuth_beam_width_rad, 0.0..=FRAC_PI_2)
                            .step_by(UI_STEP_RAD)
                            .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                            .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                    )
                    .on_hover_text("Set the azimuth half-power beamwidth of the antenna.");
                    if old_state != tx_state.2.inner.azimuth_beam_width_rad {
                        needs_update = true;
                    }
                    ui.end_row();
                });
        });
    
    // Update transforms based on the state
    if needs_update { // Used to avoid recomputing transforms if no changes were made
        update_tx_transforms(
            tx_carrier_q,
            tx_antenna_q,
            tx_antenna_beam_q,
            tx_state
        );
    }
}


// see: https://github.com/bevyengine/bevy/issues/4864
pub fn update_tx_transforms(
    mut tx_carrier_q: Query<(&mut Transform, &Children), (With<Tx>, With<Carrier>)>,
    mut tx_antenna_q: Query<(&mut Transform, &Children), (Without<Tx>, With<Antenna>)>,
    mut tx_antenna_beam_q: Query<&mut Transform, (Without<Tx>, Without<Antenna>, With<AntennaBeam>)>,
    mut tx_state: (ResMut<TxCarrierState>, ResMut<TxAntennaState>, ResMut<TxAntennaBeamState>),
) {
    for (mut carrier_tranform, carrier_children) in tx_carrier_q.iter_mut() {
        for carrier_child in carrier_children.iter() {
            if let Ok((mut antenna_transform, antenna_children)) = tx_antenna_q.get_mut(carrier_child) {
                // Update antenna beam width
                for antenna_beam in antenna_children.iter() {
                    if let Ok(mut antenna_beam_transform) = tx_antenna_beam_q.get_mut(antenna_beam) {
                        // Update antenna beam width
                        *antenna_beam_transform = antenna_beam_transform_from_state(&tx_state.2.inner);
                    }
                }
                // Update antenna transform
                *antenna_transform = antenna_transform_from_state(&tx_state.1.inner);
                // Update carrier transform                
                *carrier_tranform = carrier_transform_from_state(&mut tx_state.0.inner, &tx_state.1.inner);
            }
        }
    }
}