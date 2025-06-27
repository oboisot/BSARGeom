use bevy::{
    prelude::*,
    math::DVec3
};

use crate::{
    camera::CameraPlugin,
    world::WorldPlugin,
    entities::{
        AntennaBeamState, AntennaState, CarrierState,
        spawn_carrier
    }
};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TxCarrierState>()
            .init_resource::<TxAntennaState>()
            .init_resource::<TxAntennaBeamState>()
            .init_resource::<RxCarrierState>()
            .init_resource::<RxAntennaState>()
            .init_resource::<RxAntennaBeamState>()
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
}

impl Default for TxCarrierState {
    fn default() -> Self {
        Self {
            inner: CarrierState {
                heading_rad: 0.0,
                elevation_rad: 0.0,
                bank_rad: 0.0,
                height_m: 3000.0,
                velocity_mps: 120.0,
                position_m: DVec3::ZERO
            }
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
                heading_rad: std::f64::consts::FRAC_PI_2, // +90°, right looking
                elevation_rad: -std::f64::consts::FRAC_PI_4, // 45° of depression
                bank_rad: 0.0
            }
        }
    }
}

/// Resource to keep old state of Transmitter
#[derive(Resource)]
pub struct TxAntennaBeamState {
    pub inner: AntennaBeamState,
}

impl Default for TxAntennaBeamState {
    fn default() -> Self {
        Self {
            inner: AntennaBeamState {
                elevation_beam_width_rad: 5.0f64.to_radians(),
                azimuth_beam_width_rad: 5.0f64.to_radians()
            }
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
}

impl Default for RxCarrierState {
    fn default() -> Self {
        Self {
            inner: CarrierState {
                heading_rad: 0.0,
                elevation_rad: 0.0,
                bank_rad: 0.0,
                height_m: 1000.0,
                velocity_mps: 40.0,
                position_m: DVec3::ZERO
            }
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
                heading_rad: std::f64::consts::FRAC_PI_4, // 0°, forward looking
                elevation_rad: -std::f64::consts::FRAC_PI_6, // 30° of depression
                bank_rad: 0.0
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
                elevation_beam_width_rad: 22.0f64.to_radians(),
                azimuth_beam_width_rad: 20.0f64.to_radians()
            }
        }
    }
}


fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tx_state: (ResMut<TxCarrierState>, Res<TxAntennaState>, Res<TxAntennaBeamState>),
    mut rx_state: (ResMut<RxCarrierState>, Res<RxAntennaState>, Res<RxAntennaBeamState>),
) {
    // Tx carrier entity
    let tx_antenna_beam_material = StandardMaterial {
        base_color: Color::WHITE.with_alpha(0.3),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None, // Disable culling to see the beam from all sides
        unlit: true,
        ..default()
    }; 
    let tx_carrier_entity = spawn_carrier(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut tx_state.0.inner,
        &tx_state.1.inner,
        &tx_state.2.inner,
        tx_antenna_beam_material,
        Some("Tx".into())
    );
    commands
        .entity(tx_carrier_entity)
        .insert(Tx); // Add Tx Component marker to entity

    // Rx carrier entity
    let rx_antenna_beam_material = StandardMaterial {
        base_color: Color::BLACK.with_alpha(0.3),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None, // Disable culling to see the beam from all sides
        unlit: true,
        ..default()
    }; 
    let rx_carrier_entity = spawn_carrier(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut rx_state.0.inner,
        &rx_state.1.inner,
        &rx_state.2.inner,
        rx_antenna_beam_material,
        Some("Rx".into())
    );
    commands
        .entity(rx_carrier_entity)
        .insert(Rx); // Add Tx Component marker to entity
}
