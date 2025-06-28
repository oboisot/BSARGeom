use bevy::{
    math::{DQuat, DVec3},
    prelude::*
};

use crate::{
    constants::{ANTENNA_SIZE, CARRIER_SIZE, CONE_LENGTH, ENU_TO_NED_F64, TO_Y_UP, NEG_YAXIS_TO_XAXIS},
    entities::{
        spawn_antenna_beam, spawn_axes_helper, spawn_velocity_indicator,
        velocity_indicator_transform_from_state
    }
};

/// Component marker to identify the Transmitter
#[derive(Component)]
pub struct Carrier;

/// Component marker to identify the Antenna
#[derive(Component)]
pub struct Antenna;

/// Component marker to identify the Antenna Beam
#[derive(Component)]
pub struct AntennaBeam;

/// Component marker to identify the Velocity Vector entity.
#[derive(Component)]
pub struct VelocityVector;

/// Struct to keep the internal state of the Transmitter
#[derive(Clone)]
pub struct CarrierState {
    /// Carrier orientation in World frame (NED referential)
    pub heading_rad: f64,
    pub elevation_rad: f64,
    pub bank_rad: f64,
    // Carrier height
    pub height_m: f64,
    // Carrier velocity
    pub velocity_mps: f64,
    // Carrier position in World frame
    pub position_m: DVec3
}

/// Struct to keep the internal state of the Antenna
#[derive(Clone)]
pub struct AntennaState {
    /// Antenna orientation relative to Carrier
    pub heading_rad: f64,
    pub elevation_rad: f64,
    pub bank_rad: f64,
}

/// Struct to keep the internal state of the Antenna Beam
#[derive(Clone)]
pub struct AntennaBeamState {
    pub elevation_beam_width_rad: f64,
    pub azimuth_beam_width_rad: f64,
}

pub fn spawn_carrier(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    carrier_state: &mut CarrierState,
    antenna_state: &AntennaState,
    antenna_beam_state: &AntennaBeamState,
    antenna_beam_material: StandardMaterial,
    name: Option<String>
) -> Entity {
    // Entity name
    let name = if let Some(name) = name { name } else { "".to_string() };
    // Carrier
    let carrier_entity = spawn_axes_helper(
        commands,
        meshes,
        materials,
        CARRIER_SIZE // Size of the axes
    );
    commands
        .entity(carrier_entity)
        .insert(carrier_transform_from_state(carrier_state, antenna_state)) // update carrier transform
        .insert(Carrier) // Add Carrier component
        .insert(Name::new(format!("{} Carrier", name)));

    // Antenna
    let antenna_entity = spawn_axes_helper(
        commands,
        meshes,
        materials,
        ANTENNA_SIZE // Size of the axes
    );
    commands
        .entity(antenna_entity)
        .insert(antenna_transform_from_state(antenna_state)) // Update antenna transform
        .insert(Antenna) // Add Antenna component
        .insert(Name::new(format!("{} Antenna", name)));

    // Antenna beam
    let antenna_beam_entity = spawn_antenna_beam(
        commands,
        meshes,
        materials,
        antenna_beam_material
    );
    commands
        .entity(antenna_beam_entity)
        .insert(antenna_beam_transform_from_state(antenna_beam_state))
        .insert(AntennaBeam) // Add AntennaBeam component
        .insert(Name::new(format!("{} Antenna Beam", name)));

    // Velocity vector
    let velocity_indicator_entity = spawn_velocity_indicator(
        commands,
        meshes,
        materials
    );
    commands
        .entity(velocity_indicator_entity) // Update base transform and adds corresponding component
        .insert(velocity_indicator_transform_from_state(carrier_state)) // Update velocity vector transform
        .insert(VelocityVector) // Add VelocityVector component
        .insert(Name::new(format!("{} Velocity Vector", name)));

    // Concatenate entities (parent -> child): Carrier -> Antenna -> AntennaBeam
    commands // Adds antenna beam as child of antenna entity
        .entity(antenna_entity)
        .add_child(antenna_beam_entity);    
    commands // Adds antenna and velocity vector as children of carrier entity
        .entity(carrier_entity)
        .add_children(&[
            antenna_entity,
            velocity_indicator_entity,
        ]).id()
}

pub fn carrier_transform_from_state(
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
pub fn antenna_transform_from_state(
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

pub fn antenna_beam_transform_from_state(
    antenna_beam_state: &AntennaBeamState
) -> Transform {
    // Compute scale factors for cone base, based on beam widths
    let scale_azi = 2.0 * CONE_LENGTH * (
        0.5 * antenna_beam_state.azimuth_beam_width_rad
    ).tan();
    let scale_elv = 2.0 * CONE_LENGTH * (
        0.5 * antenna_beam_state.elevation_beam_width_rad
    ).tan();

    Transform {
        translation: Vec3::ZERO,
        rotation: NEG_YAXIS_TO_XAXIS,
        scale: Vec3::new(scale_azi as f32, 1.0, scale_elv as f32)
    }
}
