use std::f64::consts::TAU;
use bevy::{
    asset::RenderAssetUsages,
    math::{DQuat, DVec3},
    prelude::*,
    render::mesh::{PrimitiveTopology, VertexAttributeValues},
};


use crate::{
    constants::{ENU_TO_NED_F64, TO_Y_UP_F64, BLUE_MATERIAL, GREEN_MATERIAL},
    entities::{AntennaBeamState, AntennaState, CarrierState}
};

const ANTENNA_BEAM_FOOTPRINT_SIZE: usize = 2501; // Size of the antenna beam footprint mesh
const ANTENNA_ELV_AZI_LINES_INDEX: usize = 625; // = (ANTENNA_BEAM_FOOTPRINT_SIZE - 1) / 4
const STEP_THETA: f64 = TAU / (ANTENNA_BEAM_FOOTPRINT_SIZE - 1) as f64; // Step size for the antenna beam footprint mesh

pub struct AntennaBeamFootprintState {
    // Antenna Footprint line coordinates in World frame
    pub points: Vec<DVec3>
}

impl Default for AntennaBeamFootprintState {
    fn default() -> Self {
        Self {
            points: vec![DVec3::ZERO; ANTENNA_BEAM_FOOTPRINT_SIZE] // Preallocate points for the antenna beam footprint
        }
    }
}

impl AntennaBeamFootprintState {
    /// Computes the area of the antenna beam footprint.
    /// TO DO: Implement the actual area computation based on the footprint geometry.
    pub fn area(&self) -> f64 {
        0.0
    }
}

pub fn spawn_antenna_beam_footprint(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    carrier_state: &CarrierState,
    antenna_state: &AntennaState,
    antenna_beam_state: &AntennaBeamState,
    antenna_beam_footprint_state: &mut AntennaBeamFootprintState,
    material: StandardMaterial
) -> Entity {
    // Initialize the antenna beam footprint mesh
    let mut footprint_mesh = Mesh::new(
            PrimitiveTopology::LineStrip, // This tells wgpu that the positions are a list of points where a line will be drawn between each consecutive point
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![Vec3::ZERO; antenna_beam_footprint_state.points.len()]
        );
    // Update the mesh with the initial state
    update_antenna_beam_footprint_mesh_from_state(
        carrier_state,
        antenna_state,
        antenna_beam_state,
        antenna_beam_footprint_state,
        &mut footprint_mesh
    );

    commands.spawn((
        Mesh3d(meshes.add(footprint_mesh)),
        MeshMaterial3d(materials.add(material))
    )).id()
}

pub fn update_antenna_beam_footprint_mesh_from_state(
    carrier_state: &CarrierState,
    antenna_state: &AntennaState,
    antenna_beam_state: &AntennaBeamState,
    antenna_beam_footprint_state: &mut AntennaBeamFootprintState,
    mesh: &mut Mesh // Should be the mesh of the antenna beam footprint entity
)  {
    if let Some(VertexAttributeValues::Float32x3(mesh_pos)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        // Rotation to transform ground plane origin and normal into Antena referential
        // World to Antenna: R = R_enu_to_ned * R_carrier * R_antenna
        // => Antenna to World: R^-1 = R_antenna^-1 * R_carrier^-1 * R_enu_to_ned^-1
        let carrier_rotation = ENU_TO_NED_F64 * DQuat::from_euler(
            EulerRot::ZYX,
            carrier_state.heading_deg.to_radians(),
            carrier_state.elevation_deg.to_radians(),
            carrier_state.bank_deg.to_radians()
        );
        let antenna_rotation = DQuat::from_euler(
            EulerRot::ZYX,
            antenna_state.heading_deg.to_radians(),
            antenna_state.elevation_deg.to_radians(),
            antenna_state.bank_deg.to_radians()
        );
        let mut rot_antenna_to_world = carrier_rotation * antenna_rotation;
        let rot_world_to_antenna = rot_antenna_to_world.inverse(); // Inverse rotation to transform from World frame to Antenna frame
        rot_antenna_to_world = TO_Y_UP_F64 * rot_antenna_to_world; // Convert from Z-up to Y-up frame
        let carrier_position_y_up = TO_Y_UP_F64 * carrier_state.position_m;
        // Parameters for the plane/cone intersection computation
        let n = rot_world_to_antenna * DVec3::Z; // Normal vector of the ground plane in Antenna referential
        let o = rot_world_to_antenna * carrier_state.position_m; // Origin of the ground plane in Antenna referential
        let d =  -n.dot(o); // Distance from the origin to the ground plane in Antenna referential
        let ty = (0.5 * antenna_beam_state.azimuth_beam_width_deg.to_radians()).tan(); // Half of the azimuth beam width in radians
        let tz = (0.5 * antenna_beam_state.elevation_beam_width_deg.to_radians()).tan(); // Half of the elevation beam width in radians
        let nyty = n.y * ty; // Normal vector component in the Y direction scaled by the azimuth beam width
        let nztz = n.z * tz; // Normal vector component in the Z direction
        // Compute the intersection points and update corresponding mesh positions
        let (mut s, mut c): (f64, f64); // (sin(theta), cos(theta))
        for (i, point) in antenna_beam_footprint_state.points.iter_mut().enumerate() {
            (s, c) = (i as f64 * STEP_THETA).sin_cos(); // Angle in radians
            // Update resource with the new point in Antenna referential
            point.x = d / (n.x + nyty * c + nztz * s);
            point.y = ty * c * point.x;
            point.z = tz * s * point.x;
            // Transform point to World frame
            *point = rot_antenna_to_world * *point + carrier_position_y_up; // Transform point to World frame and Y-up frame
            point.y = 0.0; // Ensure to have a real zero in Z-up frame (which is here Y axis)
            // Update mesh with the new point
            mesh_pos[i] = [
                point.x as f32,
                point.y as f32,
                point.z as f32
            ];
        }
    }
}

///
/// note: this should always be called after the antenna beam footprint mesh has been spawned
pub fn spawn_antenna_beam_footprint_elevation_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    antenna_beam_footprint_state: &AntennaBeamFootprintState
) -> Entity {
    // Initialize the antenna beam footprint mesh
    let mut elv_line_mesh = Mesh::new(
            PrimitiveTopology::LineStrip, // This tells wgpu that the positions are a list of points where a line will be drawn between each consecutive point
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![Vec3::ZERO; 2]
        );
    // Update the mesh with the initial state
    update_antenna_beam_footprint_elevation_line_mesh_from_state(
        antenna_beam_footprint_state,
        &mut elv_line_mesh
    );

    commands.spawn((
        Mesh3d(meshes.add(elv_line_mesh)),
        MeshMaterial3d(materials.add(BLUE_MATERIAL.clone()))
    )).id()
}

pub fn update_antenna_beam_footprint_elevation_line_mesh_from_state(
    antenna_beam_footprint_state: &AntennaBeamFootprintState,
    mesh: &mut Mesh // Should be the mesh of the antenna elevation line entity
)  {
    if let Some(VertexAttributeValues::Float32x3(mesh_pos)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {

        let p0 = antenna_beam_footprint_state.points[ANTENNA_ELV_AZI_LINES_INDEX]; // Elevation line first point (pi/2)
        mesh_pos[0] = [p0.x as f32, 0.05, p0.z as f32]; // note: 0.05 in z-direction to be slightly above the ground plane

        let p1 = antenna_beam_footprint_state.points[3*ANTENNA_ELV_AZI_LINES_INDEX]; // Elevation line last point (3*pi/2)
        mesh_pos[1] = [p1.x as f32, 0.05, p1.z as f32]; // note: 0.05 in z-direction to be slightly above the ground plane
    }
}

///
/// note: this should always be called after the antenna beam footprint mesh has been spawned
pub fn spawn_antenna_beam_footprint_azimuth_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    antenna_beam_footprint_state: &AntennaBeamFootprintState
) -> Entity {
    // Initialize the antenna beam footprint mesh
    let mut azi_line_mesh = Mesh::new(
            PrimitiveTopology::LineStrip, // This tells wgpu that the positions are a list of points where a line will be drawn between each consecutive point
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![Vec3::ZERO; 2]
        );
    // Update the mesh with the initial state
    update_antenna_beam_footprint_azimuth_line_mesh_from_state(
        antenna_beam_footprint_state,
        &mut azi_line_mesh
    );

    commands.spawn((
        Mesh3d(meshes.add(azi_line_mesh)),
        MeshMaterial3d(materials.add(GREEN_MATERIAL.clone()))
    )).id()
}

pub fn update_antenna_beam_footprint_azimuth_line_mesh_from_state(
    antenna_beam_footprint_state: &AntennaBeamFootprintState,
    mesh: &mut Mesh // Should be the mesh of the antenna azimuth line entity
)  {
    if let Some(VertexAttributeValues::Float32x3(mesh_pos)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {

        let p0 = antenna_beam_footprint_state.points[0]; // Azimuth line first point (0)
        mesh_pos[0] = [p0.x as f32, 0.05, p0.z as f32]; // note: 0.05 in z-direction to be slightly above the ground plane

        let p1 = antenna_beam_footprint_state.points[2*ANTENNA_ELV_AZI_LINES_INDEX]; // Azimuth line last point (pi)
        mesh_pos[1] = [p1.x as f32, 0.05, p1.z as f32]; // note: 0.05 in z-direction to be slightly above the ground plane
    }
}
