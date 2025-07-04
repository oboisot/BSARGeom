use bevy::{
    prelude::*,
    math::DVec3
};

use crate::{
    camera::CameraPlugin,
    world::WorldPlugin,
    entities::{
        AntennaBeamState, AntennaBeamFootprintState, AntennaState, CarrierState,
        spawn_carrier, spawn_iso_range_ellipsoid, iso_range_ellipsoid_transform_from_state
    }
};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TxCarrierState>()
            .init_resource::<TxAntennaState>()
            .init_resource::<TxAntennaBeamState>()
            .init_resource::<TxAntennaBeamFootprintState>()
            .init_resource::<RxCarrierState>()
            .init_resource::<RxAntennaState>()
            .init_resource::<RxAntennaBeamState>()
            .init_resource::<RxAntennaBeamFootprintState>()
            .add_plugins((CameraPlugin, WorldPlugin))
            .add_systems(Startup, spawn_scene);
    }
}

/// Transmitter marker component
#[derive(Component)]
pub struct Tx;

/// Resource to keep old state of Transmitter
#[derive(Resource)]
pub struct TxCarrierState {
    pub inner: CarrierState,
    pub center_frequency_ghz: f64, // Center frequency of the carrier
    pub bandwidth_mhz: f64, // Bandwidth of the carrier
    pub pulse_duration_us: f64, // Pulse duration of the carrier
    pub prf_hz: f64, // Pulse repetition frequency of the carrier
    pub peak_power_w: f64, // Peak power of the carrier
    pub loss_factor_db: f64, // Loss factor of the carrier
}

impl Default for TxCarrierState {
    fn default() -> Self {
        Self {
            inner: CarrierState {
                heading_deg: 0.0,
                elevation_deg: 0.0,
                bank_deg: 0.0,
                height_m: 3000.0,
                velocity_mps: 120.0,
                position_m: DVec3::ZERO,
                velocity_vector_mps: DVec3::ZERO,
            },
            center_frequency_ghz: 10.0,
            bandwidth_mhz: 800.0,
            pulse_duration_us: 10.0,
            prf_hz: 10000.0,
            peak_power_w: 250.0,
            loss_factor_db: 3.0,
        }
    }
}

/// Resource to keep old state of Transmitter
#[derive(Resource)]
pub struct TxAntennaState {
    pub inner: AntennaState,
}

impl Default for TxAntennaState {
    fn default() -> Self {
        Self {
            inner: AntennaState {
                heading_deg: 90.0,
                elevation_deg: -30.0,
                bank_deg: 0.0
            }
        }
    }
}

/// Resource to keep old state of Transmitter Antenna Beam
#[derive(Resource)]
pub struct TxAntennaBeamState {
    pub inner: AntennaBeamState,
}

impl Default for TxAntennaBeamState {
    fn default() -> Self {
        Self {
            inner: AntennaBeamState {
                elevation_beam_width_deg: 20.0f64,
                azimuth_beam_width_deg: 20.0f64
            }
        }
    }
}

/// Resource to keep old state of Transmitter Antenna Beam Footprint
#[derive(Resource)]
pub struct TxAntennaBeamFootprintState {
    pub inner: AntennaBeamFootprintState
}

impl Default for TxAntennaBeamFootprintState {
    fn default() -> Self {
        Self {
            inner: AntennaBeamFootprintState::default()
        }
    }
}

/// Receiver marker component
#[derive(Component)]
pub struct Rx;

/// Resource to keep old state of Transmitter
#[derive(Resource)]
pub struct RxCarrierState {
    pub inner: CarrierState,
    pub noise_temperature_k: f64,
    pub noise_factor_db: f64,
    pub integration_time_s: f64,
    pub integration_time_for_squared_ground_pixels: bool, // true if integration time is set to have "squared ground pixels"
}

impl Default for RxCarrierState {
    fn default() -> Self {
        Self {
            inner: CarrierState {
                heading_deg: 0.0,
                elevation_deg: 0.0,
                bank_deg: 0.0,
                height_m: 1000.0,
                velocity_mps: 36.0,
                position_m: DVec3::ZERO,
                velocity_vector_mps: DVec3::ZERO,
            },
            noise_temperature_k: 290.0,
            noise_factor_db: 5.0,
            integration_time_s: 0.0,
            integration_time_for_squared_ground_pixels: true
        }
    }
}

/// Resource to keep old state of Transmitter
#[derive(Resource)]
pub struct RxAntennaState {
    pub inner: AntennaState,
}

impl Default for RxAntennaState {
    fn default() -> Self {
        Self {
            inner: AntennaState {
                heading_deg: 90.0, // 0°, right-looking
                elevation_deg: -45.0, // 45° of depression
                bank_deg: 0.0
            }
        }
    }
}

/// Resource to keep old state of Transmitter
#[derive(Resource)]
pub struct RxAntennaBeamState {
    pub inner: AntennaBeamState,
}

impl Default for RxAntennaBeamState {
    fn default() -> Self {
        Self {
            inner: AntennaBeamState {
                elevation_beam_width_deg: 16.0f64,
                azimuth_beam_width_deg: 16.0f64
            }
        }
    }
}

/// Resource to keep old state of Transmitter Antenna Beam Footprint
#[derive(Resource)]
pub struct RxAntennaBeamFootprintState {
    pub inner: AntennaBeamFootprintState
}

impl Default for RxAntennaBeamFootprintState {
    fn default() -> Self {
        Self {
            inner: AntennaBeamFootprintState::default()
        }
    }
}

/// Iso-range ellipsoid marker component
#[derive(Component)]
pub struct IsoRangeEllipsoid;

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tx_state: (ResMut<TxCarrierState>, Res<TxAntennaState>, Res<TxAntennaBeamState>, ResMut<TxAntennaBeamFootprintState>),
    mut rx_state: (ResMut<RxCarrierState>, Res<RxAntennaState>, Res<RxAntennaBeamState>, ResMut<RxAntennaBeamFootprintState>),
) {
    // Tx antenna beam material
    let tx_antenna_beam_material = StandardMaterial {
        base_color: Color::linear_rgba(1.0, 1.0, 1.0, 0.15), // White
        alpha_mode: AlphaMode::Blend,
        cull_mode: None, // Disable culling to see the beam from all sides
        unlit: true,
        ..default()
    };
    // Tx antenna beam footprint material
    let tx_antenna_beam_footprint_material = StandardMaterial {
        base_color: Color::linear_rgb(1.0, 1.0, 1.0), // White
        alpha_mode: AlphaMode::Opaque,
        cull_mode: None, // Disable culling to see the beam from all sides
        unlit: true,
        ..default()
    };
    // Tx carrier entity
    let (
        tx_carrier_entity,
        tx_antenna_beam_footprint_entity,
        tx_antenna_beam_elevation_line_entity,
        tx_antenna_beam_azimuth_line_entity
    ) = spawn_carrier(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut tx_state.0.inner,
        &tx_state.1.inner,
        &tx_state.2.inner,
        &mut tx_state.3.inner,
        tx_antenna_beam_material,
        tx_antenna_beam_footprint_material,
        Some("Tx".into())
    );
    commands
        .entity(tx_carrier_entity)
        .insert(Tx); // Add Tx Component marker to entity
    commands
        .entity(tx_antenna_beam_footprint_entity)
        .insert(Tx); // Add Tx Component marker to entity
    commands
        .entity(tx_antenna_beam_elevation_line_entity)
        .insert(Tx); // Add Tx Component marker to entity
    commands
        .entity(tx_antenna_beam_azimuth_line_entity)
        .insert(Tx); // Add Tx Component marker to entity

    // Rx antenna beam material
    let rx_antenna_beam_material = StandardMaterial {
        base_color: Color::linear_rgba(0.0, 0.0, 0.0, 0.15), // White
        alpha_mode: AlphaMode::Blend,
        cull_mode: None, // Disable culling to see the beam from all sides
        unlit: true,
        ..default()
    };
    // Rx antenna beam footprint material
    let rx_antenna_beam_footprint_material = StandardMaterial {
        base_color: Color::linear_rgb(0.0, 0.0, 0.0), // Black
        alpha_mode: AlphaMode::Opaque,
        cull_mode: None, // Disable culling to see the beam from all sides
        unlit: true,
        ..default()
    };
    // Rx carrier entity
    let (
        rx_carrier_entity,
        rx_antenna_beam_footprint_entity,
        rx_antenna_beam_elevation_line_entity,
        rx_antenna_beam_azimuth_line_entity
    ) = spawn_carrier(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut rx_state.0.inner,
        &rx_state.1.inner,
        &rx_state.2.inner,
        &mut rx_state.3.inner,
        rx_antenna_beam_material,
        rx_antenna_beam_footprint_material,
        Some("Rx".into())
    );
    commands
        .entity(rx_carrier_entity)
        .insert(Rx); // Add Rx Component marker to entity
    commands
        .entity(rx_antenna_beam_footprint_entity)
        .insert(Rx); // Add Rx Component marker to entity
    commands
        .entity(rx_antenna_beam_elevation_line_entity)
        .insert(Rx); // Add Rx Component marker to entity
    commands
        .entity(rx_antenna_beam_azimuth_line_entity)
        .insert(Rx); // Add Rx Component marker to entity

    // Iso-range ellipsoid material
    let iso_range_ellipsoid_material = StandardMaterial {
        base_color: Color::linear_rgba(0.839215686, 0.152941176, 0.156862745, 0.15),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None, // Disable culling to see the beam from all sides
        unlit: true,
        ..default()
    };
    // Iso-range ellipsoid entity
    let iso_range_ellipsoid_entity = spawn_iso_range_ellipsoid(
        &mut commands,
        &mut meshes,
        &mut materials,
        iso_range_ellipsoid_material
    );
    commands
        .entity(iso_range_ellipsoid_entity)
        .insert(iso_range_ellipsoid_transform_from_state( // Update ellipsoid transform
            &tx_state.0.inner.position_m, // OT in world frame
            &rx_state.0.inner.position_m  // OR in world frame
        ))
        .insert(IsoRangeEllipsoid) // Add IsoRangeEllipsoid Component marker to entity
        .insert(Name::new("Iso Range Ellipsoid"));
}
