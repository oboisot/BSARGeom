use std::f64::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    constants::{MAX_HEIGHT_M, MAX_VELOCITY_MPS},
    entities::{
        Antenna, AntennaBeam, AntennaBeamFootprint, Carrier, VelocityVector,
        antenna_beam_transform_from_state, antenna_transform_from_state, carrier_transform_from_state,
        velocity_indicator_transform_from_state,
        
    },
    scene::{Tx, TxCarrierState, TxAntennaState, TxAntennaBeamState, TxAntennaBeamFootprintState},
};

pub struct TxPanelPlugin;

impl Plugin for TxPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TxPanelWidget>()
            .add_systems(Update, update_tx);
    }
}

#[derive(Resource)]
pub struct TxPanelWidget {
    pub transform_needs_update: bool,
    pub velocity_indicator_needs_update: bool,
}

impl Default for TxPanelWidget {
    fn default() -> Self {
        Self {
            transform_needs_update: false,
            velocity_indicator_needs_update: false,
        }
    }
}

impl TxPanelWidget {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        tx_carrier_state: &mut TxCarrierState,
        tx_antenna_state: &mut TxAntennaState,
        tx_antenna_beam_state: &mut TxAntennaBeamState
    ) {
        
        self.transform_needs_update = false;
        self.velocity_indicator_needs_update = false;
        let mut old_state = 0.0f64;

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
                let hover_text = egui::RichText::new("Sets the Carrier's height relative to ground")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Height: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.inner.height_m;
                ui.add(
                    egui::DragValue::new(&mut tx_carrier_state.inner.height_m)
                        .speed(10.0)
                        .range(0.0..=MAX_HEIGHT_M)
                        .fixed_decimals(3)
                        .suffix(" m")
                ).on_hover_text(hover_text);
                if old_state != tx_carrier_state.inner.height_m {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier veolcity ***** //
                let hover_text = egui::RichText::new("Sets the Carrier's velocity")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Velocity: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.inner.velocity_mps;
                ui.add(
                    egui::DragValue::new(&mut tx_carrier_state.inner.velocity_mps)
                        .speed(10.0)
                        .range(0.0..=MAX_VELOCITY_MPS)
                        .fixed_decimals(3)
                        .suffix(" m/s")
                ).on_hover_text(hover_text);
                if old_state != tx_carrier_state.inner.velocity_mps {
                    self.velocity_indicator_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier heading ***** //
                let hover_text = egui::RichText::new("Sets the Carrier's heading angle:\n    0° => North\n   90° => East\n  180° => South\n  270° => West\nnote: rotation along z-axis of Carrier's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Heading: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.inner.heading_rad;
                ui.add(
                    egui::Slider::new(&mut tx_carrier_state.inner.heading_rad, 0.0..=TAU)
                        .smart_aim(false)
                        .step_by(0.0)
                        .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                        .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                ).on_hover_text(hover_text);
                if old_state != tx_carrier_state.inner.heading_rad {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier elevation ***** //
                let hover_text = egui::RichText::new("Sets the Carrier's elevation angle:\n  -90° => nadir-looking\n    0° => horizontal-looking\n  +90° => sky-looking\nnote: rotation along y-axis of Carrier's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Elevation: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.inner.elevation_rad;
                ui.add(
                    egui::Slider::new(&mut tx_carrier_state.inner.elevation_rad, -FRAC_PI_2..=FRAC_PI_2)
                        .smart_aim(false)
                        .step_by(0.0)
                        .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                        .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                ).on_hover_text(hover_text);
                if old_state != tx_carrier_state.inner.elevation_rad {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier bank ***** //
                let hover_text = egui::RichText::new("Sets the Carrier's bank angle:\n  -90° => left wing down\n    0° => horizontal wings\n  +90° => right wing down\nnote: rotation along x-axis of Carrier's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Bank: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.inner.bank_rad;
                ui.add(
                    egui::Slider::new(&mut tx_carrier_state.inner.bank_rad, -FRAC_PI_2..=FRAC_PI_2)
                        .smart_aim(false)
                        .step_by(0.0)
                        .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                        .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                ).on_hover_text(hover_text);
                if old_state != tx_carrier_state.inner.bank_rad {
                    self.transform_needs_update = true;
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
                let hover_text = egui::RichText::new("Sets the Antenna's heading angle:\n  -90° => left-looking\n    0° => forward-looking\n  +90° => right-looking\n ±180° => backward-looking\nnote: rotation along z-axis of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Heading: ").on_hover_text(hover_text.clone());
                old_state = tx_antenna_state.inner.heading_rad;
                ui.add(
                    egui::Slider::new(&mut tx_antenna_state.inner.heading_rad, -PI..=PI)
                        .smart_aim(false)
                        .step_by(0.0)
                        .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                        .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                )
                .on_hover_text(hover_text);
                if old_state != tx_antenna_state.inner.heading_rad {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Antenna elevation ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's elevation angle:\n  -90° => vertical-looking\n    0° => horizontal-looking\nnote: rotation along y-axis of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Elevation: ").on_hover_text(hover_text.clone());
                old_state = tx_antenna_state.inner.elevation_rad;
                ui.add(
                    egui::Slider::new(&mut tx_antenna_state.inner.elevation_rad, -FRAC_PI_2..=0.0)
                        .smart_aim(false)
                        .step_by(0.0)
                        .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                        .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                )
                .on_hover_text(hover_text);
                if old_state != tx_antenna_state.inner.elevation_rad {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Antenna bank ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's bank angle\nnote: rotation along x-axis of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Bank: ").on_hover_text(hover_text.clone());
                old_state = tx_antenna_state.inner.bank_rad;
                ui.add(
                    egui::Slider::new(&mut tx_antenna_state.inner.bank_rad, -FRAC_PI_2..=FRAC_PI_2)
                        .smart_aim(false)
                        .step_by(0.0)
                        .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                        .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                )
                .on_hover_text(hover_text);
                if old_state != tx_antenna_state.inner.bank_rad {
                    self.transform_needs_update = true;
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
                let hover_text = egui::RichText::new("Sets the Antenna's elevation half-power beamwidth\nnote: elevation beamwidth angle is defined in the x-z plane of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Elevation: ").on_hover_text(hover_text.clone());
                old_state = tx_antenna_beam_state.inner.elevation_beam_width_rad;
                ui.add(
                    egui::Slider::new(&mut tx_antenna_beam_state.inner.elevation_beam_width_rad, 0.0..=FRAC_PI_2)
                        .smart_aim(false)
                        .step_by(0.0)
                        .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                        .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                )
                .on_hover_text(hover_text);
                if old_state != tx_antenna_beam_state.inner.elevation_beam_width_rad {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Antenna azimuth ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's azimuth half-power beamwidth\nnote: azimuth beamwidth angle is defined in the x-y plane of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Azimuth: ").on_hover_text(hover_text.clone());
                old_state = tx_antenna_beam_state.inner.azimuth_beam_width_rad;
                ui.add(
                    egui::Slider::new(&mut tx_antenna_beam_state.inner.azimuth_beam_width_rad, 0.0..=FRAC_PI_2)
                        .smart_aim(false)
                        .step_by(0.0)
                        .custom_formatter(|n, _| format!("{:.3}°", n.to_degrees()))
                        .custom_parser(|s| if let Ok(s) = s.parse::<f64>() { Some(s.to_radians()) } else { None })
                )
                .on_hover_text(hover_text);
                if old_state != tx_antenna_beam_state.inner.azimuth_beam_width_rad {
                    self.transform_needs_update = true;
                }
                ui.end_row();
            });
    }
}

// see: https://github.com/bevyengine/bevy/issues/4864
// fn update_tx(
//     tx_panel_widget: Res<TxPanelWidget>,
//     mut tx_carrier_q: Query<(&mut Transform, &Children), (With<Tx>, With<Carrier>)>,
//     mut tx_antenna_q: Query<(&mut Transform, &Children), (Without<Tx>, With<Antenna>)>,
//     mut tx_antenna_beam_q: Query<&mut Transform, (Without<Tx>, Without<Antenna>, With<AntennaBeam>)>,
//     mut tx_velocity_indicator_q: Query<&mut Transform, (Without<Tx>, Without<Antenna>, Without<AntennaBeam>, With<VelocityVector>)>,
//     mut tx_carrier_state: ResMut<TxCarrierState>,
//     tx_antenna_state: Res<TxAntennaState>,
//     tx_antenna_beam_state: Res<TxAntennaBeamState>
// ) {
//     if !(tx_panel_widget.transform_needs_update  ||
//          tx_panel_widget.velocity_indicator_needs_update) {
//         return; // No need to update transforms if no changes were made
//     }
//     for (mut carrier_tranform, carrier_children) in tx_carrier_q.iter_mut() {
//         for carrier_child in carrier_children.iter() {
//             if tx_panel_widget.transform_needs_update {
//                 if let Ok((mut antenna_transform, antenna_children)) = tx_antenna_q.get_mut(carrier_child) {
//                     // Update antenna beam width
//                     for antenna_beam in antenna_children.iter() {
//                         if let Ok(mut antenna_beam_transform) = tx_antenna_beam_q.get_mut(antenna_beam) {
//                             // Update antenna beam width
//                             *antenna_beam_transform = antenna_beam_transform_from_state(
//                                 &tx_antenna_beam_state.inner
//                             );
//                         }
//                     }
//                     // Update antenna transform
//                     *antenna_transform = antenna_transform_from_state(
//                         &tx_antenna_state.inner
//                     );
//                     // Update carrier transform                
//                     *carrier_tranform = carrier_transform_from_state(
//                         &mut tx_carrier_state.inner,
//                         &tx_antenna_state.inner
//                     );
//                 }
//             }
//             if tx_panel_widget.velocity_indicator_needs_update {
//                 if let Ok(mut velocity_indicator_transform) = tx_velocity_indicator_q.get_mut(carrier_child) {
//                     // Update velocity vector transform
//                     *velocity_indicator_transform = velocity_indicator_transform_from_state(
//                         &tx_carrier_state.inner
//                     );
//                 }
//             }
//         }
//     }
// }

fn update_tx(
    tx_panel_widget: Res<TxPanelWidget>,
    mut tx_carrier_q: Query<(&mut Transform, &Children), (With<Tx>, With<Carrier>)>,
    mut tx_antenna_q: Query<(&mut Transform, &Children), (Without<Tx>, With<Antenna>)>,
    mut tx_antenna_beam_q: Query<&mut Transform, (Without<Tx>, Without<Antenna>, With<AntennaBeam>)>,
    mut tx_velocity_indicator_q: Query<&mut Transform, (Without<Tx>, Without<Antenna>, Without<AntennaBeam>, With<VelocityVector>)>,
    mut tx_carrier_state: ResMut<TxCarrierState>,
    tx_antenna_state: Res<TxAntennaState>,
    tx_antenna_beam_state: Res<TxAntennaBeamState>,
    mut tx_antenna_beam_footprint_state: ResMut<TxAntennaBeamFootprintState>
) {
    if !(tx_panel_widget.transform_needs_update  ||
         tx_panel_widget.velocity_indicator_needs_update) {
        return; // No need to update transforms if no changes were made
    }
    for (mut carrier_tranform, carrier_children) in tx_carrier_q.iter_mut() {
        for carrier_child in carrier_children.iter() {
            if tx_panel_widget.transform_needs_update {
                if let Ok((mut antenna_transform, antenna_children)) = tx_antenna_q.get_mut(carrier_child) {
                    // Update antenna beam width
                    for antenna_beam in antenna_children.iter() {
                        if let Ok(mut antenna_beam_transform) = tx_antenna_beam_q.get_mut(antenna_beam) {
                            // Update antenna beam width
                            *antenna_beam_transform = antenna_beam_transform_from_state(
                                &tx_antenna_beam_state.inner
                            );
                        }
                    }
                    // Update antenna transform
                    *antenna_transform = antenna_transform_from_state(
                        &tx_antenna_state.inner
                    );
                    // Update carrier transform                
                    *carrier_tranform = carrier_transform_from_state(
                        &mut tx_carrier_state.inner,
                        &tx_antenna_state.inner
                    );
                }
            }
            if tx_panel_widget.velocity_indicator_needs_update {
                if let Ok(mut velocity_indicator_transform) = tx_velocity_indicator_q.get_mut(carrier_child) {
                    // Update velocity vector transform
                    *velocity_indicator_transform = velocity_indicator_transform_from_state(
                        &tx_carrier_state.inner
                    );
                }
            }
        }
    }
}
