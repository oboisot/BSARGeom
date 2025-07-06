use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    constants::{MAX_HEIGHT_M, MAX_VELOCITY_MPS},
    entities::{
        antenna_beam_transform_from_state, antenna_transform_from_state,
        carrier_transform_from_state,
        iso_range_ellipsoid_transform_from_state,
        update_antenna_beam_footprint_azimuth_line_mesh_from_state,
        update_antenna_beam_footprint_elevation_line_mesh_from_state,
        update_antenna_beam_footprint_mesh_from_state,
        update_ground_angular_velocity,
        update_illumination_time,
        update_velocity_vector,
        velocity_indicator_transform_from_state,
        Antenna, AntennaBeam, AntennaBeamAzimuthLine, AntennaBeamElevationLine, AntennaBeamFootprint,
        Carrier, VelocityVector
    },
    scene::{
        Rx, RxAntennaBeamState, RxAntennaState, RxCarrierState, RxAntennaBeamFootprintState,
        TxCarrierState, IsoRangeEllipsoid
    },
};

pub struct RxPanelPlugin;

impl Plugin for RxPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<RxPanelWidget>()
            .add_systems(Update, update_rx);
    }
}

#[derive(Resource)]
pub struct RxPanelWidget {
    pub transform_needs_update: bool,
    pub velocity_indicator_needs_update: bool,
    pub system_needs_update: bool,
}

impl Default for RxPanelWidget {
    fn default() -> Self {
        Self {
            transform_needs_update: false,
            velocity_indicator_needs_update: false,
            system_needs_update: false,
        }
    }
}

impl RxPanelWidget {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        rx_carrier_state: &mut RxCarrierState,
        rx_antenna_state: &mut RxAntennaState,
        rx_antenna_beam_state: &mut RxAntennaBeamState
    ) {
        
        self.transform_needs_update = false;
        self.velocity_indicator_needs_update = false;
        self.system_needs_update = false;
        let mut old_state = 0.0f64;

        ui.separator();
        ui.vertical_centered(|ui| ui.label(
            egui::RichText::new("RECEIVER SETTINGS")
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
        egui::Grid::new("rx_carrier_grid")
            .num_columns(2)
            .striped(true)
            .spacing([20.0, 5.0])
            .show(ui, |ui| {
                // ***** Carrier height ***** //
                let hover_text = egui::RichText::new(format!("Sets the Carrier's height relative to ground (0 - {} m)", MAX_HEIGHT_M))
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Height: ").on_hover_text(hover_text.clone());
                old_state = rx_carrier_state.inner.height_m;
                ui.add(
                    egui::DragValue::new(&mut rx_carrier_state.inner.height_m)
                        .update_while_editing(false)
                        .speed(10.0)
                        .range(0.0..=MAX_HEIGHT_M)
                        .fixed_decimals(3)
                        .suffix(" m")
                ).on_hover_text(hover_text);
                if old_state != rx_carrier_state.inner.height_m {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier velocity ***** //
                let hover_text = egui::RichText::new(format!("Sets the Carrier's velocity (0 - {} m/s)", MAX_VELOCITY_MPS))
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Velocity: ").on_hover_text(hover_text.clone());
                old_state = rx_carrier_state.inner.velocity_mps;
                ui.add(
                    egui::DragValue::new(&mut rx_carrier_state.inner.velocity_mps)
                        .update_while_editing(false)
                        .speed(10.0)
                        .range(0.0..=MAX_VELOCITY_MPS)
                        .fixed_decimals(3)
                        .suffix(" m/s")
                ).on_hover_text(hover_text);
                if old_state != rx_carrier_state.inner.velocity_mps {
                    self.velocity_indicator_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier heading ***** //
                let hover_text = egui::RichText::new("Sets the Carrier's heading angle (0 - 360°):\n    0° => North\n   90° => East\n  180° => South\n  270° => West\nnote: rotation along z-axis of Carrier's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Heading: ").on_hover_text(hover_text.clone());
                old_state = rx_carrier_state.inner.heading_deg;
                ui.add(
                    egui::Slider::new(&mut rx_carrier_state.inner.heading_deg, 0.0..=360.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                ).on_hover_text(hover_text);
                if old_state != rx_carrier_state.inner.heading_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier elevation ***** //
                let hover_text = egui::RichText::new("Sets the Carrier's elevation angle (-90 - 90°):\n  -90° => nadir-looking\n    0° => horizontal-looking\n  +90° => sky-looking\nnote: rotation along y-axis of Carrier's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Elevation: ").on_hover_text(hover_text.clone());
                old_state = rx_carrier_state.inner.elevation_deg;
                ui.add(
                    egui::Slider::new(&mut rx_carrier_state.inner.elevation_deg, -90.0..=90.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                ).on_hover_text(hover_text);
                if old_state != rx_carrier_state.inner.elevation_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier bank ***** //
                let hover_text = egui::RichText::new("Sets the Carrier's bank angle (-90 - 90°):\n  -90° => left wing down\n    0° => horizontal wings\n  +90° => right wing down\nnote: rotation along x-axis of Carrier's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Bank: ").on_hover_text(hover_text.clone());
                old_state = rx_carrier_state.inner.bank_deg;
                ui.add(
                    egui::Slider::new(&mut rx_carrier_state.inner.bank_deg, -90.0..=90.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                ).on_hover_text(hover_text);
                if old_state != rx_carrier_state.inner.bank_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();
            });

        ui.separator();
        ui.vertical_centered(|ui| ui.label(
            egui::RichText::new("ANTENNA").strong()
        ));
        ui.separator();

        // Antenna orientation settings
        ui.vertical_centered(|ui| ui.label("Orientation"));
        ui.separator();

        egui::Grid::new("rx_antenna_orientation_grid")
            .num_columns(2)
            .striped(true)
            .spacing([20.0, 5.0])
            .show(ui, |ui| {
                // ***** Antenna heading ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's heading angle (-180 - 180°):\n  -90° => left-looking\n    0° => forward-looking\n  +90° => right-looking\n ±180° => backward-looking\nnote: rotation along z-axis of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Heading: ").on_hover_text(hover_text.clone());
                old_state = rx_antenna_state.inner.heading_deg;
                ui.add(
                    egui::Slider::new(&mut rx_antenna_state.inner.heading_deg, -180.0..=180.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                )
                .on_hover_text(hover_text);
                if old_state != rx_antenna_state.inner.heading_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Antenna elevation ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's elevation angle (-90 - 0°):\n  -90° => vertical-looking\n    0° => horizontal-looking\nnote: rotation along y-axis of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Elevation: ").on_hover_text(hover_text.clone());
                old_state = rx_antenna_state.inner.elevation_deg;
                ui.add(
                    egui::Slider::new(&mut rx_antenna_state.inner.elevation_deg, -90.0..=0.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                )
                .on_hover_text(hover_text);
                if old_state != rx_antenna_state.inner.elevation_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Antenna bank ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's bank angle (-90 - 90°)\nnote: rotation along x-axis of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Bank: ").on_hover_text(hover_text.clone());
                old_state = rx_antenna_state.inner.bank_deg;
                ui.add(
                    egui::Slider::new(&mut rx_antenna_state.inner.bank_deg, -90.0..=90.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                )
                .on_hover_text(hover_text);
                if old_state != rx_antenna_state.inner.bank_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();
            });

        ui.separator();
        ui.vertical_centered(|ui| ui.label("Beamwidth (half-power)"));
        ui.separator();
        // Antenna beamwidth settings
        egui::Grid::new("rx_antenna_beamwidth_grid")
            .num_columns(2)
            .striped(true)
            .spacing([20.0, 5.0])
            .show(ui, |ui| {
                // ***** Antenna beamwidth elevation ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's elevation half-power beamwidth (0 - 90°)\nnote: elevation beamwidth angle is defined in the x-z plane of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Elevation: ").on_hover_text(hover_text.clone());
                old_state = rx_antenna_beam_state.inner.elevation_beam_width_deg;
                ui.add(
                    egui::Slider::new(&mut rx_antenna_beam_state.inner.elevation_beam_width_deg, 0.0..=90.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                )
                .on_hover_text(hover_text);
                if old_state != rx_antenna_beam_state.inner.elevation_beam_width_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Antenna azimuth ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's azimuth half-power beamwidth (0 - 90°)\nnote: azimuth beamwidth angle is defined in the x-y plane of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Azimuth: ").on_hover_text(hover_text.clone());
                old_state = rx_antenna_beam_state.inner.azimuth_beam_width_deg;
                ui.add(
                    egui::Slider::new(&mut rx_antenna_beam_state.inner.azimuth_beam_width_deg, 0.0..=90.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                )
                .on_hover_text(hover_text);
                if old_state != rx_antenna_beam_state.inner.azimuth_beam_width_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();
            });
        
        ui.separator();
        ui.vertical_centered(|ui| ui.label(
            egui::RichText::new("SYSTEM").strong()
        ));
        ui.separator();
        // Tx system settings
        egui::Grid::new("rx_system_grid")
            .num_columns(2)
            .striped(true)
            .spacing([1.0, 5.0])
            .show(ui, |ui| {
                // ***** Noise temperature ***** //
                let hover_text = egui::RichText::new("Sets the noise temperature of the Receiver's system (0 - 1000 K)")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Noise temp.: ").on_hover_text(hover_text.clone());
                old_state = rx_carrier_state.noise_temperature_k;
                ui.add(
                    egui::DragValue::new(&mut rx_carrier_state.noise_temperature_k)
                        .update_while_editing(false)
                        .speed(1.0)
                        .range(0.0..=1000.0)
                        .fixed_decimals(1)
                        .suffix(" K")
                )
                .on_hover_text(hover_text);
                if old_state != rx_carrier_state.noise_temperature_k {
                    self.system_needs_update = true;
                }
                ui.end_row();

                // ***** Noise factor ***** //
                let hover_text = egui::RichText::new("Sets the receiver's noise factor (0 - 100 dB)")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Noise factor: ").on_hover_text(hover_text.clone());
                old_state = rx_carrier_state.noise_factor_db;
                ui.add(
                    egui::DragValue::new(&mut rx_carrier_state.noise_factor_db)
                        .update_while_editing(false)
                        .speed(1.0)
                        .range(0.0..=100.0)
                        .fixed_decimals(1)
                        .suffix(" dB")
                )
                .on_hover_text(hover_text);
                if old_state != rx_carrier_state.noise_factor_db {
                    self.system_needs_update = true;
                }
                ui.end_row();

                // ***** Integration time ***** //
                let hover_text = egui::RichText::new("Sets the receiver's integration time (0 - 100 s)")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Integration time: ").on_hover_text(hover_text.clone());
                old_state = rx_carrier_state.integration_time_s;
                ui.vertical(|ui| {
                    ui.checkbox(
                        &mut rx_carrier_state.integration_time_for_squared_ground_pixels,
                        "Squared ground pixels",
                    );
                    ui.add_enabled(
                        !rx_carrier_state.integration_time_for_squared_ground_pixels,
                        egui::DragValue::new(&mut rx_carrier_state.integration_time_s)
                            .update_while_editing(false)
                            .speed(1.0)
                            .range(0.0..=100.0)
                            .fixed_decimals(3)
                            .suffix(" s")
                    )
                    .on_hover_text(hover_text);
                });
                if old_state != rx_carrier_state.integration_time_s {
                    self.system_needs_update = true;
                }
                ui.end_row();
            });
    }
}

// see: https://github.com/bevyengine/bevy/issues/4864
fn update_rx(
    // Resources
    rx_panel_widget: Res<RxPanelWidget>,
    rx_antenna_state: Res<RxAntennaState>,
    rx_antenna_beam_state: Res<RxAntennaBeamState>,
    tx_carrier_state: Res<TxCarrierState>,
    // Mutable resources
    mut meshes: ResMut<Assets<Mesh>>,
    mut rx_carrier_state: ResMut<RxCarrierState>,
    mut rx_antenna_beam_footprint_state: ResMut<RxAntennaBeamFootprintState>,
    // Queries
    rx_antenna_beam_footprint_q: Query<&Mesh3d, (With<Rx>, With<AntennaBeamFootprint>)>,
    rx_antenna_beam_elevation_line_q: Query<&Mesh3d, (With<Rx>, With<AntennaBeamElevationLine>)>,
    rx_antenna_beam_azimuth_line_q: Query<&Mesh3d, (With<Rx>, With<AntennaBeamAzimuthLine>)>,
    // Mutable queries
    mut rx_carrier_q: Query<(&mut Transform, &Children), (With<Rx>, With<Carrier>)>,
    mut rx_antenna_q: Query<(&mut Transform, &Children), (Without<Rx>, With<Antenna>)>,
    mut rx_antenna_beam_q: Query<&mut Transform, (Without<Rx>, Without<Antenna>, With<AntennaBeam>)>,
    mut rx_velocity_indicator_q: Query<&mut Transform, (Without<Rx>, Without<Antenna>, Without<AntennaBeam>, With<VelocityVector>)>,
    mut iso_range_ellipsoid_q: Query<&mut Transform, (Without<Rx>, Without<Antenna>, Without<AntennaBeam>, Without<VelocityVector>, With<IsoRangeEllipsoid>)>,
) {
    if !(rx_panel_widget.transform_needs_update  ||
         rx_panel_widget.velocity_indicator_needs_update) {
        return; // No need to update transforms if no changes were made
    }
    for (mut carrier_tranform, carrier_children) in rx_carrier_q.iter_mut() {
        for carrier_child in carrier_children.iter() {
            if rx_panel_widget.transform_needs_update {
                if let Ok((mut antenna_transform, antenna_children)) = rx_antenna_q.get_mut(carrier_child) {
                    // Update antenna beam width
                    for antenna_beam in antenna_children.iter() {
                        if let Ok(mut antenna_beam_transform) = rx_antenna_beam_q.get_mut(antenna_beam) {
                            // Update antenna beam width
                            *antenna_beam_transform = antenna_beam_transform_from_state(
                                &rx_antenna_beam_state.inner
                            );
                        }
                    }
                    // Update antenna transform
                    *antenna_transform = antenna_transform_from_state(
                        &rx_antenna_state.inner
                    );
                    // Update carrier transform                
                    *carrier_tranform = carrier_transform_from_state(
                        &mut rx_carrier_state.inner,
                        &rx_antenna_state.inner
                    );
                }
                // Update antenna beam footprint mesh in the same time
                for mesh_handle in rx_antenna_beam_footprint_q.iter() {
                    if let Some(mesh) = meshes.get_mut(mesh_handle) {
                        update_antenna_beam_footprint_mesh_from_state(
                            &rx_carrier_state.inner,
                            &rx_antenna_state.inner,
                            &rx_antenna_beam_state.inner,
                            &mut rx_antenna_beam_footprint_state.inner,
                            mesh
                        );
                    }
                }
                // Update antenna beam elevation line mesh in the same time
                for mesh_handle in rx_antenna_beam_elevation_line_q.iter() {
                    if let Some(mesh) = meshes.get_mut(mesh_handle) {
                        update_antenna_beam_footprint_elevation_line_mesh_from_state(
                            &rx_antenna_beam_footprint_state.inner,
                            mesh
                        );
                    }
                }
                // Update antenna beam azimuth line mesh in the same time
                for mesh_handle in rx_antenna_beam_azimuth_line_q.iter() {
                    if let Some(mesh) = meshes.get_mut(mesh_handle) {
                        update_antenna_beam_footprint_azimuth_line_mesh_from_state(
                            &rx_antenna_beam_footprint_state.inner,
                            mesh
                        );
                    }
                }
                //Update iso-range ellipsoid transform
                for mut iso_range_ellipsoid_transform in iso_range_ellipsoid_q.iter_mut() {
                    *iso_range_ellipsoid_transform = iso_range_ellipsoid_transform_from_state(
                        &tx_carrier_state.inner.position_m, // OT in world frame
                        &rx_carrier_state.inner.position_m  // OR in world frame
                    );
                }
            }
            if rx_panel_widget.velocity_indicator_needs_update {
                if let Ok(mut velocity_indicator_transform) = rx_velocity_indicator_q.get_mut(carrier_child) {
                    // Update velocity vector transform
                    *velocity_indicator_transform = velocity_indicator_transform_from_state(
                        &rx_carrier_state.inner
                    );
                    // Update carrier velocity vector in the same time (here direction does not change, only magnitude)
                    update_velocity_vector(&mut rx_carrier_state.inner);
                    // Update ground angular velocity only
                    update_ground_angular_velocity(
                        &rx_carrier_state.inner,
                        &mut rx_antenna_beam_footprint_state.inner,
                    );
                    // Update illumination time
                    update_illumination_time(
                        &rx_carrier_state.inner,
                        &mut rx_antenna_beam_footprint_state.inner,
                    );
                }
            }
        }
    }
}
