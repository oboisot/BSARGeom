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
        BsarInfosState, IsoRangeDopplerPlane, IsoRangeEllipsoid, PixelResolution,
        Rx, RxAntennaBeamFootprintState, RxAntennaBeamState, RxAntennaState, RxCarrierState,
        TxAntennaBeamFootprintState, TxAntennaBeamState, TxCarrierState
    },
    ui::{carrier_ui, heading_with_reset, MenuWidget},
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
#[derive(Default)]
pub struct RxPanelWidget {
    pub transform_needs_update: bool,
    pub velocity_vector_needs_update: bool,
    pub system_needs_update: bool,
}


impl RxPanelWidget {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        menu_widget: &MenuWidget,
        rx_carrier_state: &mut RxCarrierState,
        rx_antenna_state: &mut RxAntennaState,
        rx_antenna_beam_state: &mut RxAntennaBeamState,
        bsar_infos_state: &mut BsarInfosState,
    ) {
        // Handle update of parameters, meshes, textures, etc...
        self.transform_needs_update = false;
        self.velocity_vector_needs_update = false;
        self.system_needs_update = false;

        // Rx Carrier UI
        ui.add_enabled_ui(
            !menu_widget.is_monostatic,
            |ui| {
                carrier_ui(
                    ui,
                    "rx",
                    "RECEIVER SETTINGS",
                    &mut rx_carrier_state.inner,
                    &mut rx_antenna_state.inner,
                    &mut rx_antenna_beam_state.inner,
                    &RxCarrierState::default().inner,
                    &RxAntennaState::default().inner,
                    &RxAntennaBeamState::default().inner,
                    &mut self.transform_needs_update,
                    &mut self.velocity_vector_needs_update
                );
            }
        );

        // Rx System UI
        rx_system_ui(
            ui,
            rx_carrier_state,
            rx_antenna_beam_state,
            menu_widget.is_monostatic,
            bsar_infos_state,
            &mut self.system_needs_update
        );
    }
}

// see: https://github.com/bevyengine/bevy/issues/4864
pub(super) fn update_rx(
    res: ( // Resources
        Res<RxAntennaState>,              // rx_antenna_state
        Res<RxAntennaBeamState>,          // rx_antenna_beam_state
        Res<TxCarrierState>,              // tx_carrier_state
        Res<TxAntennaBeamState>,          // tx_antenna_beam_state
        Res<TxAntennaBeamFootprintState>, // tx_antenna_beam_footprint_state
    ),
    resmut: ( // Mutable resources
        ResMut<RxPanelWidget>,               // rx_panel_widget
        ResMut<Assets<StandardMaterial>>,    // materials
        ResMut<Assets<Mesh>>,                // meshes
        ResMut<Assets<Image>>,               // images
        ResMut<MenuWidget>,                  // menu_widget // For monostatic case
        ResMut<RxCarrierState>,              // rx_carrier_state
        ResMut<RxAntennaBeamFootprintState>, // rx_antenna_beam_footprint_state
        ResMut<BsarInfosState>,              // bsar_infos_state
        ResMut<IsoRangeDopplerPlaneState>,   // iso_range_doppler_plane_state
    ),
    // Queries
    rx_antenna_beam_footprint_q: Query<&Mesh3d, (With<Rx>, With<AntennaBeamFootprint>)>,
    rx_antenna_beam_elevation_line_q: Query<&Mesh3d, (With<Rx>, With<AntennaBeamElevationLine>)>,
    rx_antenna_beam_azimuth_line_q: Query<&Mesh3d, (With<Rx>, With<AntennaBeamAzimuthLine>)>,
    iso_range_doppler_material_q: Query<&MeshMaterial3d<StandardMaterial>, With<IsoRangeDopplerPlane>>,
    // Mutable queries
    mut rx_carrier_q: Query<(&mut Transform, &Children), (With<Rx>, With<Carrier>)>,
    mut rx_antenna_q: Query<(&mut Transform, &Children), (Without<Rx>, With<Antenna>)>,
    mut rx_antenna_beam_q: Query<&mut Transform, (Without<Rx>, Without<Antenna>, With<AntennaBeam>)>,
    mut rx_velocity_indicator_q: Query<&mut Transform, (Without<Rx>, Without<Antenna>, Without<AntennaBeam>, With<VelocityVector>)>,
    mut iso_range_ellipsoid_q: Query<&mut Transform, (Without<Rx>, Without<Antenna>, Without<AntennaBeam>, Without<VelocityVector>, With<IsoRangeEllipsoid>)>,
    mut iso_range_doppler_q: Query<&mut Transform, (Without<Rx>, Without<Antenna>, Without<AntennaBeam>, Without<VelocityVector>, Without<IsoRangeEllipsoid>, With<IsoRangeDopplerPlane>)>,
) {
    // Extracts resources
    let (
        rx_antenna_state,
        rx_antenna_beam_state,
        tx_carrier_state,
        tx_antenna_beam_state,
        tx_antenna_beam_footprint_state
    ) = res;
    // Extracts mutable resources
    let (
        mut rx_panel_widget,
        mut materials,
        mut meshes,
        mut images,
        mut menu_widget,
        mut rx_carrier_state,
        mut rx_antenna_beam_footprint_state,
        mut bsar_infos_state,
        mut iso_range_doppler_plane_state,
    ) = resmut;
    // Checks if nothing needs to be done
    if !(rx_panel_widget.transform_needs_update  ||
         rx_panel_widget.velocity_vector_needs_update ||
         rx_panel_widget.system_needs_update) {
        return; // No need to update transforms if no changes were made
    }
    for (mut carrier_transform, carrier_children) in rx_carrier_q.iter_mut() {
        for carrier_child in carrier_children.iter() {
            if rx_panel_widget.transform_needs_update
                && let Ok((mut antenna_transform, antenna_children)) = rx_antenna_q.get_mut(carrier_child) {
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
                    *carrier_transform = carrier_transform_from_state(
                        &mut rx_carrier_state.inner,
                        &rx_antenna_state.inner
                    );
                    // Update antenna beam footprint mesh in the same time
                    for mesh_handle in rx_antenna_beam_footprint_q.iter() {
                        if let Some(mut mesh) = meshes.get_mut(mesh_handle) {
                            update_antenna_beam_footprint_mesh_from_state(
                                &rx_carrier_state.inner,
                                &rx_antenna_state.inner,
                                &rx_antenna_beam_state.inner,
                                &mut rx_antenna_beam_footprint_state.inner,
                                &mut mesh
                            );
                        }
                    }
                    // Update antenna beam elevation line mesh in the same time
                    for mesh_handle in rx_antenna_beam_elevation_line_q.iter() {
                        if let Some(mut mesh) = meshes.get_mut(mesh_handle) {
                            update_antenna_beam_footprint_elevation_line_mesh_from_state(
                                &rx_antenna_beam_footprint_state.inner,
                                &mut mesh
                            );
                        }
                    }
                    // Update antenna beam azimuth line mesh in the same time
                    for mesh_handle in rx_antenna_beam_azimuth_line_q.iter() {
                        if let Some(mut mesh) = meshes.get_mut(mesh_handle) {
                            update_antenna_beam_footprint_azimuth_line_mesh_from_state(
                                &rx_antenna_beam_footprint_state.inner,
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
            if rx_panel_widget.velocity_vector_needs_update
                && let Ok(mut velocity_indicator_transform) = rx_velocity_indicator_q.get_mut(carrier_child) {
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
    // Monostatic case
    if menu_widget.is_monostatic {
        if rx_panel_widget.system_needs_update {
            bsar_infos_state.inner.update_from_state(
                &tx_carrier_state,
                &rx_carrier_state,
                &tx_antenna_beam_state.inner,
                &rx_antenna_beam_state.inner,
                &tx_antenna_beam_footprint_state.inner,
                &rx_antenna_beam_footprint_state.inner,
            );
        }
        if menu_widget.force_rx_system_update {
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
            menu_widget.force_rx_system_update = false;
        }
    } else if rx_panel_widget.transform_needs_update  ||
              rx_panel_widget.velocity_vector_needs_update ||
              rx_panel_widget.system_needs_update {
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
    // them here so they cannot linger when the Rx panel (which resets its own
    // flags only while it is open) is closed.
    rx_panel_widget.transform_needs_update = false;
    rx_panel_widget.velocity_vector_needs_update = false;
    rx_panel_widget.system_needs_update = false;
}


fn rx_system_ui(
    ui: &mut egui::Ui,
    rx_carrier_state: &mut RxCarrierState,
    rx_antenna_beam_state: &mut RxAntennaBeamState,
    is_monostatic: bool,
    bsar_infos_state: &mut BsarInfosState,
    system_needs_update: &mut bool,
) {
    let mut old_state = 0.0f64;

    ui.separator();
    if heading_with_reset(
        ui,
        egui::RichText::new("SYSTEM").strong(),
        "Resets the System settings to their defaults"
    ) {
        let default_state = RxCarrierState::default();
        rx_carrier_state.noise_temperature_k = default_state.noise_temperature_k;
        rx_carrier_state.noise_factor_db = default_state.noise_factor_db;
        rx_carrier_state.integration_time_s = default_state.integration_time_s;
        rx_carrier_state.squared_pixels = default_state.squared_pixels;
        rx_carrier_state.pixel_resolution = default_state.pixel_resolution;
        // In monostatic mode this is re-mirrored from Tx in the same frame
        rx_antenna_beam_state.inner.one_way_gain_dbi =
            RxAntennaBeamState::default().inner.one_way_gain_dbi;
        *system_needs_update = true;
    }
    ui.separator();
    // Rx system settings
    egui::Grid::new("rx_system_grid")
        .num_columns(2)
        .striped(false)
        .spacing([1.0, 5.0])
        .show(ui, |ui| {
            // ***** Antenna gain ***** //
            let hover_text = egui::RichText::new("Sets the reception antenna one-way power gain (0 - 100 dBi); mirrors the Tx antenna gain in monostatic mode")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Antenna gain: ").on_hover_text(hover_text.clone());
            old_state = rx_antenna_beam_state.inner.one_way_gain_dbi;
            ui.add_enabled(
                !is_monostatic, // The Rx antenna mirrors the Tx antenna in monostatic mode
                egui::DragValue::new(&mut rx_antenna_beam_state.inner.one_way_gain_dbi)
                    .update_while_editing(false)
                    .speed(0.1)
                    .range(0.0..=100.0)
                    .fixed_decimals(1)
                    .suffix(" dBi")
            )
            .on_hover_text(hover_text);
            if old_state != rx_antenna_beam_state.inner.one_way_gain_dbi {
                *system_needs_update = true;
            }
            ui.end_row();

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
                *system_needs_update = true;
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
                *system_needs_update = true;
            }
            ui.end_row();

            // ***** Integration time ***** //
            let hover_text = egui::RichText::new("Sets the receiver's integration time (0 - 100 s)")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Integration time: ").on_hover_text(hover_text.clone());
            if rx_carrier_state.squared_pixels {
                rx_carrier_state.integration_time_s =
                    bsar_infos_state.inner.integration_time_s;
            }
            old_state = rx_carrier_state.integration_time_s;
            ui.vertical(|ui| {
                let old_state = rx_carrier_state.squared_pixels;
                ui.checkbox(
                    &mut rx_carrier_state.squared_pixels,
                    "Squared pixels",
                );
                if rx_carrier_state.squared_pixels != old_state {
                    *system_needs_update = true;
                }
                ui.add_enabled_ui(
                    rx_carrier_state.squared_pixels,
                    |ui| {
                        ui.horizontal(|ui| {
                            let old_state = rx_carrier_state.pixel_resolution.clone();
                            ui.selectable_value(
                                &mut rx_carrier_state.pixel_resolution,
                                PixelResolution::Ground,
                                "Ground res."
                            );
                            ui.selectable_value(
                                &mut rx_carrier_state.pixel_resolution,
                                PixelResolution::Slant,
                                "Slant res."
                            );
                            if rx_carrier_state.pixel_resolution != old_state {
                                *system_needs_update = true;
                            }
                        });
                    }
                );
                ui.add_enabled(
                    !rx_carrier_state.squared_pixels,
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
                *system_needs_update = true;
            }
            ui.end_row();
        });
}