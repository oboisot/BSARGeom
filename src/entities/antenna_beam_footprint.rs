use std::f64::consts::TAU;
use bevy::{
    prelude::*,
    math::{DQuat, DVec3}
};


use crate::{
    constants::ENU_TO_NED_F64,
    entities::{AntennaState, AntennaBeamState, CarrierState}
};

const ANTENNA_BEAM_FOOTPRINT_SIZE: usize = 2501; // Size of the antenna beam footprint mesh
const STEP_THETA: f64 = TAU / (ANTENNA_BEAM_FOOTPRINT_SIZE - 1) as f64; // Step size for the antenna beam footprint mesh

/// Resource to keep old state of Transmitter
#[derive(Resource)]
pub struct TxAntennaBeamFootprintState {
    pub points: Vec<DVec3>
}

impl Default for TxAntennaBeamFootprintState {
    fn default() -> Self {
        Self {
            points: vec![DVec3::ZERO; ANTENNA_BEAM_FOOTPRINT_SIZE] // Preallocate points for the antenna beam footprint
        }
    }
}

pub fn update_antenna_beam_footprint_mesh_from_state(
    carrier_state: &CarrierState,
    antenna_state: &AntennaState,
    antenna_beam_state: &AntennaBeamState,
    points: &mut Vec<DVec3>,
    meshes: &mut ResMut<Assets<Mesh>>,
)  {
    // Rotation to transform ground plane origin and normal into Antena referential
    // World to Antenna: R = R_enu_to_ned * R_carrier * R_antenna
    // => Antenna to World: R^-1 = R_antenna^-1 * R_carrier^-1 * R_enu_to_ned^-1
    let carrier_rotation = ENU_TO_NED_F64 * DQuat::from_euler(
        EulerRot::ZYX,
        carrier_state.heading_rad,
        carrier_state.elevation_rad,
        carrier_state.bank_rad
    );
    let antenna_rotation = DQuat::from_euler(
        EulerRot::ZYX,
        antenna_state.heading_rad,
        antenna_state.elevation_rad,
        antenna_state.bank_rad
    );
    let rot = (
        carrier_rotation *
        antenna_rotation
    ).inverse();

    //
    let n = rot * DVec3::Z; // Normal vector of the ground plane in Antenna referential
    let o = rot * carrier_state.position_m; // Origin of the ground plane in Antenna referential
    let d =  n.dot(o); // Distance from the origin to the ground plane in Antenna referential
    let ty = (0.5 * antenna_beam_state.azimuth_beam_width_rad).tan(); // Half of the azimuth beam width in radians
    let tz = (0.5 * antenna_beam_state.elevation_beam_width_rad).tan(); // Half of the elevation beam width in radians
    let nyty = n.y * ty; // Normal vector component in the Y direction scaled by the azimuth beam width
    let nztz = n.z * tz; // Normal vector component in the Z direction
    // 
    let (mut s, mut c): (f64, f64); // (sin(theta), cos(theta))
    for (i, point) in points.iter_mut().enumerate() {
        (s, c) = (i as f64 * STEP_THETA).sin_cos(); // Angle in radians
        point.x = d / (n.x + nyty * c + nztz * s);
        point.y = ty * c * point.x;
        point.z = tz * s * point.x;
    }
}