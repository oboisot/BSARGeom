use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    constants::{MAX_HEIGHT_M, MAX_VELOCITY_MPS},
    entities::{
        antenna_beam_transform_from_state, antenna_transform_from_state,
        carrier_transform_from_state,
        iso_range_doppler_plane_transform_from_state,
        iso_range_ellipsoid_transform_from_state,
        update_antenna_beam_footprint_azimuth_line_mesh_from_state,
        update_antenna_beam_footprint_elevation_line_mesh_from_state,
        update_antenna_beam_footprint_mesh_from_state,
        update_ground_angular_velocity,
        update_illumination_time,
        update_velocity_vector,
        velocity_indicator_transform_from_state,
        Antenna, AntennaBeam, AntennaBeamAzimuthLine, AntennaBeamElevationLine, AntennaBeamFootprint,
        Carrier, IsoRangeDopplerPlaneState, VelocityVector
    },
    scene::{
        BsarInfosState, IsoRangeEllipsoid, RxAntennaBeamFootprintState, RxCarrierState,
        Tx, TxAntennaBeamFootprintState, TxAntennaBeamState, TxAntennaState, TxCarrierState,
        IsoRangeDopplerPlane,
    },
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
    pub velocity_vector_needs_update: bool,
    pub system_needs_update: bool,
}

impl Default for TxPanelWidget {
    fn default() -> Self {
        Self {
            transform_needs_update: false,
            velocity_vector_needs_update: false,
            system_needs_update: false,
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
        self.velocity_vector_needs_update = false;
        self.system_needs_update = false;
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

        // Carrier settings
        egui::Grid::new("tx_carrier_grid")
            .num_columns(2)
            .striped(false)
            .spacing([20.0, 5.0])
            .show(ui, |ui| {
                // ***** Carrier height ***** //
                let hover_text = egui::RichText::new(format!("Sets the Carrier's height relative to ground (0 - {} m)", MAX_HEIGHT_M))
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Height: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.inner.height_m;
                ui.add(
                    egui::DragValue::new(&mut tx_carrier_state.inner.height_m)
                        .update_while_editing(false)
                        .speed(10.0)
                        .range(0.0..=MAX_HEIGHT_M)
                        .fixed_decimals(3)
                        .suffix(" m")
                ).on_hover_text(hover_text);
                if old_state != tx_carrier_state.inner.height_m {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier velocity ***** //
                let hover_text = egui::RichText::new(format!("Sets the Carrier's velocity (0 - {} m/s)", MAX_VELOCITY_MPS))
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Velocity: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.inner.velocity_mps;
                ui.add(
                    egui::DragValue::new(&mut tx_carrier_state.inner.velocity_mps)
                        .update_while_editing(false)
                        .speed(10.0)
                        .range(0.0..=MAX_VELOCITY_MPS)
                        .fixed_decimals(3)
                        .suffix(" m/s")
                ).on_hover_text(hover_text);
                if old_state != tx_carrier_state.inner.velocity_mps {
                    self.velocity_vector_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier heading ***** //
                let hover_text = egui::RichText::new("Sets the Carrier's heading angle (0 - 360°):\n    0° => North\n   90° => East\n  180° => South\n  270° => West\nnote: rotation along z-axis of Carrier's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Heading: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.inner.heading_deg;
                ui.add(
                    egui::Slider::new(&mut tx_carrier_state.inner.heading_deg, 0.0..=360.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                ).on_hover_text(hover_text);
                if old_state != tx_carrier_state.inner.heading_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier elevation ***** //
                let hover_text = egui::RichText::new("Sets the Carrier's elevation angle (-90 - 90°):\n  -90° => nadir-looking\n    0° => horizontal-looking\n  +90° => sky-looking\nnote: rotation along y-axis of Carrier's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Elevation: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.inner.elevation_deg;
                ui.add(
                    egui::Slider::new(&mut tx_carrier_state.inner.elevation_deg, -90.0..=90.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                ).on_hover_text(hover_text);
                if old_state != tx_carrier_state.inner.elevation_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Carrier bank ***** //
                let hover_text = egui::RichText::new("Sets the Carrier's bank angle (-90 - 90°):\n  -90° => left wing down\n    0° => horizontal wings\n  +90° => right wing down\nnote: rotation along x-axis of Carrier's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Bank: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.inner.bank_deg;
                ui.add(
                    egui::Slider::new(&mut tx_carrier_state.inner.bank_deg, -90.0..=90.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                ).on_hover_text(hover_text);
                if old_state != tx_carrier_state.inner.bank_deg {
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
        
        egui::Grid::new("tx_antenna_orientation_grid")
            .num_columns(2)
            .striped(false)
            .spacing([20.0, 5.0])
            .show(ui, |ui| {
                // ***** Antenna heading ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's heading angle(-180 - 180°):\n  -90° => left-looking\n    0° => forward-looking\n  +90° => right-looking\n ±180° => backward-looking\nnote: rotation along z-axis of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Heading: ").on_hover_text(hover_text.clone());
                old_state = tx_antenna_state.inner.heading_deg;
                ui.add(
                    egui::Slider::new(&mut tx_antenna_state.inner.heading_deg, -180.0..=180.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                )
                .on_hover_text(hover_text);
                if old_state != tx_antenna_state.inner.heading_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Antenna elevation ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's elevation angle(-90 - 0°):\n  -90° => vertical-looking\n    0° => horizontal-looking\nnote: rotation along y-axis of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Elevation: ").on_hover_text(hover_text.clone());
                old_state = tx_antenna_state.inner.elevation_deg;
                ui.add(
                    egui::Slider::new(&mut tx_antenna_state.inner.elevation_deg, -90.0..=0.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                )
                .on_hover_text(hover_text);
                if old_state != tx_antenna_state.inner.elevation_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Antenna bank ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's bank angle (-90 - 90°)\nnote: rotation along x-axis of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Bank: ").on_hover_text(hover_text.clone());
                old_state = tx_antenna_state.inner.bank_deg;
                ui.add(
                    egui::Slider::new(&mut tx_antenna_state.inner.bank_deg, -90.0..=90.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                )
                .on_hover_text(hover_text);
                if old_state != tx_antenna_state.inner.bank_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();
            });

        ui.separator();
        ui.vertical_centered(|ui| ui.label("Beamwidth (half-power)"));
        ui.separator();
        // Antenna beamwidth settings
        egui::Grid::new("tx_antenna_beamwidth_grid")
            .num_columns(2)
            .striped(false)
            .spacing([20.0, 5.0])
            .show(ui, |ui| {
                // ***** Antenna beamwidth elevation ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's elevation half-power beamwidth (0 - 90°)\nnote: elevation beamwidth angle is defined in the x-z plane of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Elevation: ").on_hover_text(hover_text.clone());
                old_state = tx_antenna_beam_state.inner.elevation_beam_width_deg;
                ui.add(
                    egui::Slider::new(&mut tx_antenna_beam_state.inner.elevation_beam_width_deg, 0.0..=90.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                )
                .on_hover_text(hover_text);
                if old_state != tx_antenna_beam_state.inner.elevation_beam_width_deg {
                    self.transform_needs_update = true;
                }
                ui.end_row();

                // ***** Antenna azimuth ***** //
                let hover_text = egui::RichText::new("Sets the Antenna's azimuth half-power beamwidth (0 - 90°)\nnote: azimuth beamwidth angle is defined in the x-y plane of Antenna's NED frame")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Azimuth: ").on_hover_text(hover_text.clone());
                old_state = tx_antenna_beam_state.inner.azimuth_beam_width_deg;
                ui.add(
                    egui::Slider::new(&mut tx_antenna_beam_state.inner.azimuth_beam_width_deg, 0.0..=90.0)
                        .suffix("°")
                        .smart_aim(false)
                        .step_by(0.0)                
                        .drag_value_speed(1.0)
                        .fixed_decimals(3)
                )
                .on_hover_text(hover_text);
                if old_state != tx_antenna_beam_state.inner.azimuth_beam_width_deg {
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
        egui::Grid::new("tx_system_grid")
            .num_columns(2)
            .striped(false)
            .spacing([1.0, 5.0])
            .show(ui, |ui| {
                // ***** Center frequency ***** //
                let hover_text = egui::RichText::new("Sets the transmitted center frequency (0.1 - 100 GHz)")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Center Freq.: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.center_frequency_ghz;
                ui.add(
                    egui::DragValue::new(&mut tx_carrier_state.center_frequency_ghz)
                        .update_while_editing(false)
                        .speed(0.1)
                        .range(0.1..=100.0)
                        .fixed_decimals(3)
                        .suffix(" GHz")
                )
                .on_hover_text(hover_text);
                if old_state != tx_carrier_state.center_frequency_ghz {
                    self.system_needs_update = true;
                }
                ui.end_row();

                // ***** Bandwidth ***** //
                let hover_text = egui::RichText::new("Sets the transmitted bandwidth (1 - 10000 MHz)")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Bandwidth: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.bandwidth_mhz;
                ui.add(
                    egui::DragValue::new(&mut tx_carrier_state.bandwidth_mhz)
                        .update_while_editing(false)
                        .speed(1.0)
                        .range(1.0..=10000.0)
                        .fixed_decimals(1)
                        .suffix(" MHz")
                )
                .on_hover_text(hover_text);
                if old_state != tx_carrier_state.bandwidth_mhz {
                    self.system_needs_update = true;
                }
                ui.end_row();

                // ***** Pulse duration ***** //
                let hover_text = egui::RichText::new("Sets the transmitted pulse duration (0 - 1000000 µs)")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Pulse Dur.: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.pulse_duration_us;
                ui.add(
                    egui::DragValue::new(&mut tx_carrier_state.pulse_duration_us)
                        .update_while_editing(false)
                        .speed(10.0)
                        .range(0.0..=1000000.0)
                        .fixed_decimals(1)
                        .suffix(" µs")
                )
                .on_hover_text(hover_text);
                if old_state != tx_carrier_state.pulse_duration_us {
                    self.system_needs_update = true;
                }
                ui.end_row();

                // ***** PRF ***** //
                let hover_text = egui::RichText::new("Sets the Pulse Repetition Frequency (PRF) of the transmitter (1 - 1000000 Hz)")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("PRF: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.prf_hz;
                ui.add(
                    egui::DragValue::new(&mut tx_carrier_state.prf_hz)
                        .update_while_editing(false)
                        .speed(1.0)
                        .range(1.0..=1000000.0)
                        .fixed_decimals(1)
                        .suffix(" Hz")
                )
                .on_hover_text(hover_text);
                if old_state != tx_carrier_state.prf_hz {
                    self.system_needs_update = true;
                }
                ui.end_row();

                // ***** Peak power ***** //
                let hover_text = egui::RichText::new("Sets the transmitted peak power (0 - 10000 W)")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Peak Power: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.peak_power_w;
                ui.add(
                    egui::DragValue::new(&mut tx_carrier_state.peak_power_w)
                        .update_while_editing(false)
                        .speed(1.0)
                        .range(1.0..=10000.0)
                        .fixed_decimals(1)
                        .suffix(" W")
                )
                .on_hover_text(hover_text);
                if old_state != tx_carrier_state.peak_power_w {
                    self.system_needs_update = true;
                }
                ui.end_row();

                // ***** Loss factor ***** //
                let hover_text = egui::RichText::new("Sets the transmission loss factor (0 - 100 dB)")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .monospace();
                ui.label("Loss Factor: ").on_hover_text(hover_text.clone());
                old_state = tx_carrier_state.loss_factor_db;
                ui.add(
                    egui::DragValue::new(&mut tx_carrier_state.loss_factor_db)
                        .update_while_editing(false)
                        .speed(0.1)
                        .range(0.0..=100.0)
                        .fixed_decimals(1)
                        .suffix(" dB")
                )
                .on_hover_text(hover_text);
                if old_state != tx_carrier_state.loss_factor_db {
                    self.system_needs_update = true;
                }
                ui.end_row();
            });
    }
}

// see: https://github.com/bevyengine/bevy/issues/4864
fn update_tx(
    res: ( // Resources
        Res<TxPanelWidget>,               // tx_panel_widget
        Res<TxAntennaState>,              // tx_antenna_state
        Res<TxAntennaBeamState>,          // tx_antenna_beam_state
        Res<RxCarrierState>,              // rx_carrier_state
        Res<RxAntennaBeamFootprintState>, // rx_antenna_beam_footprint_state
    ),
    resmut: ( // Mutable resources
        ResMut<Assets<StandardMaterial>>,    // materials
        ResMut<Assets<Mesh>>,                // meshes
        ResMut<Assets<Image>>,               // images
        ResMut<TxCarrierState>,              // tx_carrier_state
        ResMut<TxAntennaBeamFootprintState>, // tx_antenna_beam_footprint_state
        ResMut<BsarInfosState>,              // bsar_infos_state
        ResMut<IsoRangeDopplerPlaneState>,   // iso_range_doppler_plane_state
    ),
    // Queries,
    tx_antenna_beam_footprint_q: Query<&Mesh3d, (With<Tx>, With<AntennaBeamFootprint>)>,
    tx_antenna_beam_elevation_line_q: Query<&Mesh3d, (With<Tx>, With<AntennaBeamElevationLine>)>,
    tx_antenna_beam_azimuth_line_q: Query<&Mesh3d, (With<Tx>, With<AntennaBeamAzimuthLine>)>,
    iso_range_doppler_material_q: Query<&MeshMaterial3d<StandardMaterial>, With<IsoRangeDopplerPlane>>,
    // Mutable queries
    mut tx_carrier_q: Query<(&mut Transform, &Children), (With<Tx>, With<Carrier>)>,
    mut tx_antenna_q: Query<(&mut Transform, &Children), (Without<Tx>, With<Antenna>)>,
    mut tx_antenna_beam_q: Query<&mut Transform, (Without<Tx>, Without<Antenna>, With<AntennaBeam>)>,
    mut tx_velocity_indicator_q: Query<&mut Transform, (Without<Tx>, Without<Antenna>, Without<AntennaBeam>, With<VelocityVector>)>,
    mut iso_range_ellipsoid_q: Query<&mut Transform, (Without<Tx>, Without<Antenna>, Without<AntennaBeam>, Without<VelocityVector>, With<IsoRangeEllipsoid>)>,
    mut iso_range_doppler_q: Query<&mut Transform, (Without<Tx>, Without<Antenna>, Without<AntennaBeam>, Without<VelocityVector>, Without<IsoRangeEllipsoid>, With<IsoRangeDopplerPlane>)>,
    
) {
    // Extracts resources
    let (
        tx_panel_widget,
        tx_antenna_state,
        tx_antenna_beam_state,
        rx_carrier_state,
        rx_antenna_beam_footprint_state
    ) = res;
    // Extracts mutable resources
    let (
        mut materials,
        mut meshes,
        mut images,
        mut tx_carrier_state,
        mut tx_antenna_beam_footprint_state,
        mut bsar_infos_state,
        mut iso_range_doppler_plane_state,
    ) = resmut;
    // Checks if nothing needs to be done
    if !(tx_panel_widget.transform_needs_update  ||
         tx_panel_widget.velocity_vector_needs_update ||
         tx_panel_widget.system_needs_update) {
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
                    // Update antenna beam footprint mesh in the same time
                    for mesh_handle in tx_antenna_beam_footprint_q.iter() {
                        if let Some(mesh) = meshes.get_mut(mesh_handle) {
                            update_antenna_beam_footprint_mesh_from_state(
                                &tx_carrier_state.inner,
                                &tx_antenna_state.inner,
                                &tx_antenna_beam_state.inner,
                                &mut tx_antenna_beam_footprint_state.inner,
                                mesh
                            );
                        }
                    }
                    // Update antenna beam elevation line mesh in the same time
                    for mesh_handle in tx_antenna_beam_elevation_line_q.iter() {
                        if let Some(mesh) = meshes.get_mut(mesh_handle) {
                            update_antenna_beam_footprint_elevation_line_mesh_from_state(
                                &tx_antenna_beam_footprint_state.inner,
                                mesh
                            );
                        }
                    }
                    // Update antenna beam azimuth line mesh in the same time
                    for mesh_handle in tx_antenna_beam_azimuth_line_q.iter() {
                        if let Some(mesh) = meshes.get_mut(mesh_handle) {
                            update_antenna_beam_footprint_azimuth_line_mesh_from_state(
                                &tx_antenna_beam_footprint_state.inner,
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
            }
            if tx_panel_widget.velocity_vector_needs_update {
                if let Ok(mut velocity_indicator_transform) = tx_velocity_indicator_q.get_mut(carrier_child) {
                    // Update velocity vector transform
                    *velocity_indicator_transform = velocity_indicator_transform_from_state(
                        &tx_carrier_state.inner
                    );
                    // Update carrier velocity vector in the same time (here direction does not change, only magnitude)
                    update_velocity_vector(&mut tx_carrier_state.inner);
                    // Update ground angular velocity only
                    update_ground_angular_velocity(
                        &tx_carrier_state.inner,
                        &mut tx_antenna_beam_footprint_state.inner,
                    );
                    // Update illumination time
                    update_illumination_time(
                        &tx_carrier_state.inner,
                        &mut tx_antenna_beam_footprint_state.inner,
                    );
                }
            }
        }
    }
    // Update BSAR infos state
    if tx_panel_widget.transform_needs_update  ||
       tx_panel_widget.velocity_vector_needs_update ||
       tx_panel_widget.system_needs_update {
        // Update BSAR infos 
        bsar_infos_state.inner.update_from_state(
            &tx_carrier_state,
            &rx_carrier_state,
            &tx_antenna_beam_footprint_state.inner,
            &rx_antenna_beam_footprint_state.inner,
        );
        // Update iso-range doppler plane transform and texture
        for mut iso_range_doppler_plane_tranform in iso_range_doppler_q.iter_mut() {
            for material_handle in iso_range_doppler_material_q.iter() {
                if let Some(material) = materials.get_mut(material_handle) {
                    if let Some(ref image_handle) = material.base_color_texture {
                        if let Some(image) = images.get_mut(image_handle) {
                            if let Ok(transform) = iso_range_doppler_plane_transform_from_state(
                                &tx_carrier_state,
                                &rx_carrier_state,
                                &tx_antenna_beam_footprint_state.inner,
                                &rx_antenna_beam_footprint_state.inner,
                                image,
                                &mut iso_range_doppler_plane_state
                            ) {
                                // Update iso-range doppler plane transform
                                *iso_range_doppler_plane_tranform = transform;
                            };
                        }
                        // Update iso-range doppler plane texture with newly caluclated image
                        material.base_color_texture = Some(image_handle.clone());
                    }
                }
            }
        }
    }
}
