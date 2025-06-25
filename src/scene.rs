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
            .init_resource::<TxState>()
            .init_resource::<RxState>()
            .add_plugins((CameraPlugin, WorldPlugin))
            .add_systems(Startup, spawn_scene);
    }
}

/// Transmitter marker component
#[derive(Component)]
struct Tx;

/// Resource to keep old state of Transmitter
#[derive(Resource)]
pub struct TxState {
    pub carrier_state: CarrierState,
    pub antenna_state: AntennaState,
    pub antenna_beam_state: AntennaBeamState
}

impl Default for TxState {
    fn default() -> Self {
        Self {
            carrier_state: CarrierState {
                heading_rad: 0.0,
                elevation_rad: 0.0, // -45°
                bank_rad: 0.0,
                height_m: 3000.0,
                velocity_m_s: 120.0,
                position_m: DVec3::ZERO
            },
            antenna_state: AntennaState {
                heading_rad: std::f64::consts::FRAC_PI_2, // +90°, right looking
                elevation_rad: -std::f64::consts::FRAC_PI_4, // -45° of depression
                bank_rad: 0.0
            },
            antenna_beam_state: AntennaBeamState {
                elevation_beam_width_rad: 5.0f64.to_radians(),
                azimuth_beam_width_rad: 5.0f64.to_radians()
            }
        }
    }
}

/// Receiver marker component
#[derive(Component)]
struct Rx;

/// Resource to keep old state of Transmitter
#[derive(Resource)]
pub struct RxState {
    pub carrier_state: CarrierState,
    pub antenna_state: AntennaState,
    pub antenna_beam_state: AntennaBeamState
}

impl Default for RxState {
    fn default() -> Self {
        Self {
            carrier_state: CarrierState {
                heading_rad: 0.0,
                elevation_rad: 0.0,
                bank_rad: 0.0,
                height_m: 1000.0,
                velocity_m_s: 40.0,
                position_m: DVec3::ZERO
            },
            antenna_state: AntennaState {
                heading_rad: std::f64::consts::FRAC_PI_4, // 0°, forward looking
                elevation_rad: -std::f64::consts::FRAC_PI_6, // 30° of depression
                bank_rad: 0.0
            },
            antenna_beam_state: AntennaBeamState {
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
    tx_state: Res<TxState>,
    rx_state: Res<RxState>,
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
        tx_state.carrier_state.clone(),
        tx_state.antenna_state.clone(),
        tx_state.antenna_beam_state.clone(),
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
        rx_state.carrier_state.clone(),
        rx_state.antenna_state.clone(),
        rx_state.antenna_beam_state.clone(),
        rx_antenna_beam_material,
        Some("Rx".into())
    );
    commands
        .entity(rx_carrier_entity)
        .insert(Rx); // Add Tx Component marker to entity
}

// // see: https://github.com/bevyengine/bevy/issues/4864
// fn update_tx_carrier(
//     mut tx_carrier_q: Query<(&mut Transform, &mut CarrierState, &Children), With<Tx>>,
//     tx_antenna_q: Query<(&AntennaState, &Children)>,
//     mut tx_antenna_beam_q: Query<(&mut Transform, &mut AntennaBeamState), Without<Tx>>,
//     time: Res<Time>,
// ) {
//     for (mut carrier_tranform, mut carrier_state, carrier_children) in tx_carrier_q.iter_mut() {
//         for carrier_child in carrier_children.iter() {
//             if let Ok((antenna_state, antenna_children)) = tx_antenna_q.get(carrier_child) {
//                 // Update antenna beam width
//                 for antenna_beam in antenna_children.iter() {
//                     if let Ok((mut antenna_beam_transform, mut antenna_beam_state)) = tx_antenna_beam_q.get_mut(antenna_beam) {
//                         antenna_beam_state.elevation_beam_width_rad += 0.1f64.to_radians();
//                         *antenna_beam_transform = antenna_beam_transform_from_state(&antenna_beam_state);
//                     }
//                 }
//                 // Update carrier heading
//                 carrier_state.heading_rad += 0.1 * time.delta_secs() as f64; // Rotate at 0.1 rad/s
//                 *carrier_tranform = carrier_transform_from_state(&mut carrier_state, &antenna_state);
//             }
            
//         }
//     }
// }