use bevy::{
    math::{DQuat, DVec3},
    prelude::*
};

use crate::{
    constants::{ENU_TO_NED_F64, TO_Y_UP},
    entities::{spawn_antenna_beam, spawn_axes_helper}
};
pub struct CarriersPlugin;

impl Plugin for CarriersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            spawn_tx_carrier,
            spawn_rx_carrier
        ));
    }
}

// The internal state of the Carrier
#[derive(Component)]
pub struct CarrierState {
    /// Carrier orientation in World frame (NED referential)
    pub heading_rad: f64,
    pub elevation_rad: f64,
    pub bank_rad: f64,
    // Carrier height
    pub height_m: f64,
    // Carrier velocity
    pub velocity_m_s: f64,
    // Carrier position in World frame
    pub position_m: DVec3
}

// The internal state of the Antenna
#[derive(Component)]
pub struct AntennaState {
    /// Antenna orientation relative to Carrier
    pub heading_rad: f64,
    pub elevation_rad: f64,
    pub bank_rad: f64,
}

// The internal state of the Antenna
#[derive(Component)]
pub struct AntennaBeamState {
    /// Antenna 3d beam widths
    pub elevation_beam_width_rad: f64,
    pub azimuth_beam_width_rad: f64,
}

// Transmitter Component
#[derive(Component)]
struct Tx;

// Receiver Component
#[derive(Component)]
struct Rx;

const CARRIER_SIZE: f32 = 150.0; // Size of the carrier
const ANTENNA_SIZE: f32 = 100.0;  // Size of the antenna

fn spawn_tx_carrier(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // States
    // Carrier state
    let mut tx_carrier_state = CarrierState {
        heading_rad: 0.0, // Heading in radians
        elevation_rad: 0.0, // Elevation in radians
        bank_rad: 0.0, // Bank in radians
        height_m: 3000.0, // Height in meters
        velocity_m_s: 120.0, // Velocity in meters per second
        position_m: DVec3::ZERO, // Position in meters
    };
    // Antenna state (relative to carrier NED frame)
    let tx_antenna_state = AntennaState {
        heading_rad: 90.0f64.to_radians(), // Heading in radians
        elevation_rad: -30.0f64.to_radians(), // Elevation in radians
        bank_rad: 0.0, // Bank in radians
    };
    // Antenna beam state (relative to antenna NED frame)
    let tx_antenna_beam_state = AntennaBeamState {
        elevation_beam_width_rad: 15.0f64.to_radians(), // Elevation beam width in radians
        azimuth_beam_width_rad: 15.0f64.to_radians(),   // Azimuth beam width in radians
    };

    let tx_carrier_entity = spawn_axes_helper(
        &mut commands,
        &mut meshes,
        &mut materials,
        CARRIER_SIZE // Size of the axes
    );
    commands
        .entity(tx_carrier_entity)
        .insert(
            carrier_transform_from_state(
                &mut tx_carrier_state, &tx_antenna_state
            )
        )
        .insert(tx_carrier_state)
        .insert(Tx) // Mark as a transmitter
        .insert(Name::new("Tx Carrier"));

    // Antenna entity
    let tx_antenna_entity = spawn_axes_helper(
        &mut commands,
        &mut meshes,
        &mut materials,
        ANTENNA_SIZE // Size of the axes
    );
    commands
        .entity(tx_antenna_entity)
        .insert( // Update antenna transform
            antenna_transform_from_state(&tx_antenna_state)
        )
        .insert(tx_antenna_state)
        .insert(Tx) // Mark as a transmitter
        .insert(Name::new("Tx Antenna"));

    // Antenna beam entity
    let antenna_material = StandardMaterial {
        base_color: Color::WHITE.with_alpha(0.25),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None, // Disable culling to see the beam from all sides
        unlit: true,
        ..default()
    }; 
    let tx_antenna_beam_entity = spawn_antenna_beam(
        &mut commands,
        &mut meshes,
        &mut materials,
        tx_antenna_beam_state.elevation_beam_width_rad, // Elevation beam width in radians
        tx_antenna_beam_state.azimuth_beam_width_rad,   // Azimuth beam width in radians
        antenna_material,
    );
    commands
        .entity(tx_antenna_beam_entity)
        .insert(tx_antenna_beam_state)
        .insert(Tx) // Mark as a transmitter
        .insert(Name::new("Tx Antenna Beam"));

    // Parent/child relationship    
    commands // Adds antenna beam as child of antenna entity
        .entity(tx_antenna_entity)
        .add_child(tx_antenna_beam_entity);    
    commands // Adds antenna beam as child of carrier entity
        .entity(tx_carrier_entity)
        .add_child(tx_antenna_entity);
}

fn spawn_rx_carrier(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // States
    // Carrier state
    let mut rx_carrier_state = CarrierState {
        heading_rad: 0.0, // Heading in radians
        elevation_rad: 0.0, // Elevation in radians
        bank_rad: 0.0, // Bank in radians
        height_m: 1000.0, // Height in meters
        velocity_m_s: 40.0, // Velocity in meters per second
        position_m: DVec3::ZERO, // Position in meters
    };
    // Antenna state (relative to carrier NED frame)
    let rx_antenna_state = AntennaState {
        heading_rad: 80.0f64.to_radians(), // Heading in radians
        elevation_rad: -15.0f64.to_radians(), // Elevation in radians
        bank_rad: 0.0, // Bank in radians
    };
    // Antenna beam state (relative to antenna NED frame)
    let rx_antenna_beam_state = AntennaBeamState {
        elevation_beam_width_rad: 5.0f64.to_radians(), // Elevation beam width in radians
        azimuth_beam_width_rad: 5.0f64.to_radians(),   // Azimuth beam width in radians
    };

    // Entities
    // Carrier entity
    let rx_carrier_entity = spawn_axes_helper(
        &mut commands,
        &mut meshes,
        &mut materials,
        CARRIER_SIZE // Size of the axes
    );
    commands
        .entity(rx_carrier_entity)
        .insert(
            carrier_transform_from_state(
                &mut rx_carrier_state, &rx_antenna_state
            )
        )
        .insert(rx_carrier_state)
        .insert(Rx) // Mark as a receiver
        .insert(Name::new("Rx Carrier"));

    // Antenna entity
    let rx_antenna_entity = spawn_axes_helper(
        &mut commands,
        &mut meshes,
        &mut materials,
        ANTENNA_SIZE // Size of the axes
    );
    commands
        .entity(rx_antenna_entity)
        .insert( // Update antenna transform
            antenna_transform_from_state(&rx_antenna_state)
        )
        .insert(rx_antenna_state)
        .insert(Rx) // Mark as a receiver
        .insert(Name::new("Rx Antenna"));

    // Antenna beam entity
    let antenna_material = StandardMaterial {
        base_color: Color::BLACK.with_alpha(0.25),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None, // Disable culling to see the beam from all sides
        unlit: true,
        ..default()
    };
    let rx_antenna_beam_entity = spawn_antenna_beam(
        &mut commands,
        &mut meshes,
        &mut materials,
        rx_antenna_beam_state.elevation_beam_width_rad, // Elevation beam width in radians
        rx_antenna_beam_state.azimuth_beam_width_rad,   // Azimuth beam width in radians
        antenna_material,
    );
    commands
        .entity(rx_antenna_beam_entity)        
        .insert(rx_antenna_beam_state)
        .insert(Rx) // Mark as a receiver
        .insert(Name::new("Rx Antenna Beam"));

    // Parent/child relationship
    commands // Adds antenna beam as child of antenna entity
        .entity(rx_antenna_entity)
        .add_child(rx_antenna_beam_entity);
    commands // Adds antenna beam as child as child of carrier entity
        .entity(rx_carrier_entity)
        .add_child(rx_antenna_entity);
}


// /// Returns an observer that updates the entity's material to the one specified.
// fn update_material_on<E>(
//     new_material: Handle<StandardMaterial>,
// ) -> impl Fn(Trigger<E>, Query<&mut MeshMaterial3d<StandardMaterial>>) {
//     // An observer closure that captures `new_material`. We do this to avoid needing to write four
//     // versions of this observer, each triggered by a different event and with a different hardcoded
//     // material. Instead, the event type is a generic, and the material is passed in.
//     move |trigger, mut query| {
//         if let Ok(mut material) = query.get_mut(trigger.target()) {
//             material.0 = new_material.clone();
//         }
//     }
// }



fn carrier_transform_from_state(
    carrier_state: &mut CarrierState,
    antenna_state: &AntennaState,
) -> Transform {
    // Carrier rotation from ENU to NED frame + orientation
    let carrier_rotation = ENU_TO_NED_F64 * DQuat::from_euler(
        EulerRot::ZYX,
        carrier_state.heading_rad,
        carrier_state.elevation_rad,
        carrier_state.bank_rad
    );

    // Carrier position in World frame
    // We compute the intersection of Carrier at position (0, 0, height_m) with antenna pointing direction
    // with the ground plane (z = 0) then we apply the inverse translation to get the position
    // of the carrier in the World frame.
    // Antenna pointing direction
    let antenna_rotation = DQuat::from_euler(
        EulerRot::ZYX,
        antenna_state.heading_rad,
        antenna_state.elevation_rad,
        antenna_state.bank_rad
    );
    let ax = (
        carrier_rotation *
        antenna_rotation *
        DVec3::X // Antenna points towards X-axis in its local frame
    ).normalize();

    let t = if carrier_state.height_m > 0.0 {
        carrier_state.height_m / ax.z
    } else {
        0.0
    };

    // Update carrier position in CarrierState
    carrier_state.position_m = DVec3::new(
        t * ax.x,
        t * ax.y,
        carrier_state.height_m
    );

    Transform {
        translation: TO_Y_UP * Vec3::new( // Transforms from Z-up to Y-up
            carrier_state.position_m.x as f32,
            carrier_state.position_m.y as f32,
            carrier_state.position_m.z as f32
        ),
        rotation: TO_Y_UP * Quat::from_xyzw( // Transforms from Z-up to Y-up
            carrier_rotation.x as f32,
            carrier_rotation.y as f32,
            carrier_rotation.z as f32,
            carrier_rotation.w as f32
        ),
        scale: Vec3::ONE
    }
}

/// Computes antenna transform from antenna state
/// related to carrier NED frame
fn antenna_transform_from_state(
    antenna_state: &AntennaState,
) -> Transform {
    let rotation = Quat::from_euler(
        EulerRot::ZYX,
        antenna_state.heading_rad as f32,
        antenna_state.elevation_rad as f32,
        antenna_state.bank_rad as f32
    );
    // Note: we don't apply ENU_TO_NED here because the antenna is already in the NED frame
    Transform::from_rotation(rotation)
}
