use bevy::prelude::*;

use crate::{
    camera::CameraPlugin,
    world::WorldPlugin,
        entities::{
        AntennaBeamState, AntennaState, CarrierState,
        spawn_carrier, carrier_transform_from_state
    }
};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((CameraPlugin, WorldPlugin))
            .add_systems(Startup, spawn_scene)
            .add_systems(Update, update_tx_carrier);
    }
}

/// Transmitter marker component
#[derive(Component)]
struct Tx;

/// Receiver marker component
#[derive(Component)]
struct Rx;

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        CarrierState::default_tx(),
        AntennaState::default_tx(),
        AntennaBeamState::default_tx(),
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
        CarrierState::default_rx(),
        AntennaState::default_rx(),
        AntennaBeamState::default_rx(),
        rx_antenna_beam_material,
        Some("Rx".into())
    );
    commands
        .entity(rx_carrier_entity)
        .insert(Rx); // Add Tx Component marker to entity
}

fn update_tx_carrier(
    mut tx_carrier_q: Query<(&mut Transform, &mut CarrierState, &Children), With<Tx>>,
    tx_antenna_q: Query<&AntennaState>,
    time: Res<Time>,
) {
    for (mut carrier_tranform, mut carrier_state, carrier_children) in tx_carrier_q.iter_mut() {
        for carrier_child in carrier_children.iter() {
            if let Ok(antenna_state) = tx_antenna_q.get(carrier_child) {
                // Update carrier heading
                carrier_state.heading_rad += 0.1 * time.delta_secs() as f64; // Rotate at 0.1 rad/s
                *carrier_tranform = carrier_transform_from_state(&mut carrier_state, &antenna_state);
            }
        }
    }
    // if let Ok((mut transform, mut carrier_state)) = tx_carrier_q.single_mut() {
    //     if let Ok(antenna_state) = tx_antenna_q.single() {
    //         // Update carrier heading
    //         carrier_state.heading_rad += 0.1 * time.delta_secs() as f64; // Rotate at 0.1 rad/s
    //         *transform = carrier_transform_from_state(&mut carrier_state, &antenna_state);
    //     }
    // }
}