use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    entities::{
        antenna_beam_transform_from_state, antenna_transform_from_state,
        carrier_transform_from_state,
        iso_range_ellipsoid_transform_from_state,
        refresh_iso_range_doppler_plane,
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
        BsarInfosState, IsoRangeDopplerPlane, IsoRangeEllipsoid, RxAntennaBeamFootprintState, RxAntennaBeamState, RxAntennaState, RxCarrierState, Tx, TxAntennaBeamFootprintState, TxAntennaBeamState, TxAntennaState, TxCarrierState
    },
    ui::{carrier_ui, MenuWidget, RxPanelWidget},
};

pub struct TxPanelPlugin;

impl Plugin for TxPanelPlugin {
    fn build(&self, app: &mut App) {
        // update_tx must run after update_rx: in monostatic mode the mirrored
        // Rx state only recomputes its derived fields (position, velocity
        // vector, footprint) inside update_rx, and update_tx reads them when
        // refreshing the BSAR infos and the iso-range/Doppler plane.
        app
            .init_resource::<TxPanelWidget>()
            .add_systems(Update, update_tx.after(super::rx_panel::update_rx));
    }
}

#[derive(Resource)]
#[derive(Default)]
pub struct TxPanelWidget {
    pub transform_needs_update: bool,
    pub velocity_vector_needs_update: bool,
    pub system_needs_update: bool,
}


impl TxPanelWidget {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        menu_widget: &mut MenuWidget,
        rx_panel_widget: &mut RxPanelWidget,
        tx_carrier_state: &mut TxCarrierState,
        tx_antenna_state: &mut TxAntennaState,
        tx_antenna_beam_state: &mut TxAntennaBeamState,
        rx_carrier_state: &mut RxCarrierState,
        rx_antenna_state: &mut RxAntennaState,
        rx_antenna_beam_state: &mut RxAntennaBeamState,
    ) {
        self.transform_needs_update = false;
        self.velocity_vector_needs_update = false;
        self.system_needs_update = false;

        // Tx Carrier UI
        carrier_ui(
            ui,
            "tx",
            "TRANSMITTER SETTINGS",
            &mut tx_carrier_state.inner,
            &mut tx_antenna_state.inner,
            &mut tx_antenna_beam_state.inner,
            &mut self.transform_needs_update,
            &mut self.velocity_vector_needs_update
        );

        // Tx System UI
        tx_system_ui(
            ui,
            tx_carrier_state,
            tx_antenna_beam_state,
            &mut self.system_needs_update
        );

        // Monostatic case
        if menu_widget.is_monostatic {
            rx_carrier_state.inner = tx_carrier_state.inner.clone();
            rx_antenna_state.inner = tx_antenna_state.inner.clone();
            rx_antenna_beam_state.inner = tx_antenna_beam_state.inner.clone();
            if menu_widget.was_monostatic {
                rx_panel_widget.transform_needs_update = self.transform_needs_update;
                rx_panel_widget.velocity_vector_needs_update = self.velocity_vector_needs_update;
            } else {
                rx_panel_widget.transform_needs_update = true;
                rx_panel_widget.velocity_vector_needs_update = true;
                // Make update_rx refresh the BSAR infos for the new (mirrored)
                // geometry: no Tx flag is set by the toggle itself, so
                // update_tx alone would leave them stale.
                rx_panel_widget.system_needs_update = true;
                menu_widget.force_rx_system_update = true;
                menu_widget.was_monostatic = true;
            }
        } else {
            menu_widget.was_monostatic = false;
        }
    }
}

// see: https://github.com/bevyengine/bevy/issues/4864
fn update_tx(
    res: ( // Resources
        Res<TxAntennaState>,              // tx_antenna_state
        Res<TxAntennaBeamState>,          // tx_antenna_beam_state
        Res<RxCarrierState>,              // rx_carrier_state
        Res<RxAntennaBeamState>,          // rx_antenna_beam_state
        Res<RxAntennaBeamFootprintState>, // rx_antenna_beam_footprint_state
    ),
    resmut: ( // Mutable resources
        ResMut<TxPanelWidget>,               // tx_panel_widget
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
        tx_antenna_state,
        tx_antenna_beam_state,
        rx_carrier_state,
        rx_antenna_beam_state,
        rx_antenna_beam_footprint_state
    ) = res;
    // Extracts mutable resources
    let (
        mut tx_panel_widget,
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
    for (mut carrier_transform, carrier_children) in tx_carrier_q.iter_mut() {
        for carrier_child in carrier_children.iter() {
            if tx_panel_widget.transform_needs_update
                && let Ok((mut antenna_transform, antenna_children)) = tx_antenna_q.get_mut(carrier_child) {
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
                    *carrier_transform = carrier_transform_from_state(
                        &mut tx_carrier_state.inner,
                        &tx_antenna_state.inner
                    );
                    // Update antenna beam footprint mesh in the same time
                    for mesh_handle in tx_antenna_beam_footprint_q.iter() {
                        if let Some(mut mesh) = meshes.get_mut(mesh_handle) {
                            update_antenna_beam_footprint_mesh_from_state(
                                &tx_carrier_state.inner,
                                &tx_antenna_state.inner,
                                &tx_antenna_beam_state.inner,
                                &mut tx_antenna_beam_footprint_state.inner,
                                &mut mesh
                            );
                        }
                    }
                    // Update antenna beam elevation line mesh in the same time
                    for mesh_handle in tx_antenna_beam_elevation_line_q.iter() {
                        if let Some(mut mesh) = meshes.get_mut(mesh_handle) {
                            update_antenna_beam_footprint_elevation_line_mesh_from_state(
                                &tx_antenna_beam_footprint_state.inner,
                                &mut mesh
                            );
                        }
                    }
                    // Update antenna beam azimuth line mesh in the same time
                    for mesh_handle in tx_antenna_beam_azimuth_line_q.iter() {
                        if let Some(mut mesh) = meshes.get_mut(mesh_handle) {
                            update_antenna_beam_footprint_azimuth_line_mesh_from_state(
                                &tx_antenna_beam_footprint_state.inner,
                                &mut mesh
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
            if tx_panel_widget.velocity_vector_needs_update
                && let Ok(mut velocity_indicator_transform) = tx_velocity_indicator_q.get_mut(carrier_child) {
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
    // Update BSAR infos state
    if tx_panel_widget.transform_needs_update  ||
       tx_panel_widget.velocity_vector_needs_update ||
       tx_panel_widget.system_needs_update {
        // Update BSAR infos 
        bsar_infos_state.inner.update_from_state(
            &tx_carrier_state,
            &rx_carrier_state,
            &tx_antenna_beam_state.inner,
            &rx_antenna_beam_state.inner,
            &tx_antenna_beam_footprint_state.inner,
            &rx_antenna_beam_footprint_state.inner,
        );
        // Update iso-range doppler plane transform and texture
        refresh_iso_range_doppler_plane(
            &mut materials,
            &mut images,
            &tx_carrier_state,
            &rx_carrier_state,
            &tx_antenna_beam_footprint_state.inner,
            &rx_antenna_beam_footprint_state.inner,
            &mut iso_range_doppler_plane_state,
            &mut iso_range_doppler_q,
            &iso_range_doppler_material_q,
        );
    }
    // The panel flags are one-shot commands consumed by this system: clear
    // them here so they cannot linger when the Tx panel (which resets its own
    // flags only while it is open) is closed.
    tx_panel_widget.transform_needs_update = false;
    tx_panel_widget.velocity_vector_needs_update = false;
    tx_panel_widget.system_needs_update = false;
}


fn tx_system_ui(
    ui: &mut egui::Ui,
    tx_carrier_state: &mut TxCarrierState,
    tx_antenna_beam_state: &mut TxAntennaBeamState,
    system_needs_update: &mut bool,
) {
    let mut old_state = 0.0f64;

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
                *system_needs_update = true;
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
                *system_needs_update = true;
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
                *system_needs_update = true;
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
                *system_needs_update = true;
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
                *system_needs_update = true;
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
                *system_needs_update = true;
            }
            ui.end_row();

            // ***** Antenna gain ***** //
            let hover_text = egui::RichText::new("Sets the transmission antenna one-way power gain (0 - 100 dBi)")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Antenna gain: ").on_hover_text(hover_text.clone());
            old_state = tx_antenna_beam_state.inner.one_way_gain_dbi;
            ui.add(
                egui::DragValue::new(&mut tx_antenna_beam_state.inner.one_way_gain_dbi)
                    .update_while_editing(false)
                    .speed(0.1)
                    .range(0.0..=100.0)
                    .fixed_decimals(1)
                    .suffix(" dBi")
            )
            .on_hover_text(hover_text);
            if old_state != tx_antenna_beam_state.inner.one_way_gain_dbi {
                *system_needs_update = true;
            }
            ui.end_row();
        });
}
