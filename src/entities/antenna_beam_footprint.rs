use std::f64::consts::TAU;
use bevy::{
    asset::RenderAssetUsages,
    math::{DQuat, DVec3},
    prelude::*,
    mesh::{PrimitiveTopology, VertexAttributeValues},
};


use crate::{
    constants::{ENU_TO_NED_F64, TO_Y_UP_F64, BLUE_MATERIAL, GREEN_MATERIAL},
    entities::{AntennaBeamState, AntennaState, CarrierState}
};

const ANTENNA_BEAM_FOOTPRINT_SIZE: usize = 2501; // Size of the antenna beam footprint mesh
const ANTENNA_ELV_AZI_LINES_INDEX: usize = 625; // = (ANTENNA_BEAM_FOOTPRINT_SIZE - 1) / 4
const STEP_THETA: f64 = TAU / (ANTENNA_BEAM_FOOTPRINT_SIZE - 1) as f64; // Step size for the antenna beam footprint mesh

pub struct AntennaBeamFootprintState {
    pub points: Vec<DVec3>, // Antenna Footprint line coordinates in World frame (Y-up)
    pub range_center_m: f64, // Slant range from antenna to antenna beam footprint center in meters
    pub range_min_m: f64, // Minimum slant range from antenna to antenna beam footprint in meters
    pub range_max_m: f64, // Maximum slant range from antenna to antenna beam footprint in meters
    pub loc_incidence_center_deg: f64, // Local incidence angle at the antenna beam footprint center in degrees
    pub loc_incidence_min_deg: f64, // Local incidence angle at the minimum range point in degrees
    pub loc_incidence_max_deg: f64, // Local incidence angle at the maximum range point in degrees
    pub ground_range_swath_m: f64, // Ground range swath in meters (i.e., the width of the antenna beam footprint on the ground between range_min_m and range_max_m)
    // pub ground_max_coord_m: f64, // Ground maximum coordinates of the antenna beam footprint in meters
    pub ground_max_extent_m: f64, // Ground maximum extent of the antenna beam footprint in meters (between scene center and 3d footpint)
    pub area_m2: f64, // half-power antenna beam footprint area in meters squared
    pub antenna_squint_deg: f64, // Antenna squint angle in degrees
    pub illumination_time_s: f64, // Illumination time in seconds
    pub ground_angular_velocity_degps: f64, // Ground angular velocity in degrees per second
}

impl Default for AntennaBeamFootprintState {
    fn default() -> Self {
        Self {
            points: vec![DVec3::ZERO; ANTENNA_BEAM_FOOTPRINT_SIZE], // Preallocate points for the antenna beam footprint
            range_center_m: 0.0, // Default slant range from antenna to antenna beam footprint center
            range_min_m: 0.0, // Default minimum slant range
            range_max_m: 0.0, // Default maximum slant range
            loc_incidence_center_deg: 0.0, // Default local incidence angle at the antenna beam footprint center
            loc_incidence_min_deg: 0.0, // Default local incidence angle at the minimum range point
            loc_incidence_max_deg: 0.0, // Default local incidence angle at the maximum range point
            ground_range_swath_m: 0.0, // Default ground range swath
            ground_max_extent_m: 0.0, // Default maximum extent of the antenna beam footprint in the ground plane
            area_m2: 0.0, // Default area of the antenna beam footprint
            antenna_squint_deg: 0.0, // Default antenna squint angle
            illumination_time_s: 0.0, // Default illumination time
            ground_angular_velocity_degps: 0.0, // Default ground angular velocity
        }
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
    // Closures definitions
    let area = |points: &[DVec3]| -> f64 { // Computes the half-power antenna beam footprint area using the "Shoelace" formula.
        points.iter()
            .zip(points.iter().skip(1))
            .fold(DVec3::ZERO, |acc, (p0, &p1)| acc + p0.cross(p1))
            .length() * 0.5
    };
    // Computes the local incidence angle in degrees at a given point in the antenna beam footprint
    let incidence = |neg_axis: &DVec3| -> f64 {        
        neg_axis.dot(DVec3::Y)
                .acos()
                .to_degrees()
    };
    // Computes this antenna squint angle in degrees from the antenna beam axis (normalized) and the carrier velocity vector
    let squint = |axis: &DVec3, vel: &DVec3| -> f64 {
        let vel_norm = vel.length_squared();
        if vel_norm > 0.0 {
            axis.dot(vel / vel_norm.sqrt())
                .asin()
                .to_degrees()
        } else {
            0.0
        }
    };


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
        let carrier_position_y_up = TO_Y_UP_F64 * carrier_state.position_m; // Carrier position vector in World frame (Y-up)
        // Parameters for the plane/cone intersection computation
        let n = rot_world_to_antenna * DVec3::Z; // Normal vector of the ground plane in Antenna referential
        let o = rot_world_to_antenna * carrier_state.position_m; // Origin of the ground plane in Antenna referential
        let d =  -n.dot(o); // Distance from the origin to the ground plane in Antenna referential
        let ty = (0.5 * antenna_beam_state.azimuth_beam_width_deg.to_radians()).tan(); // Half of the azimuth beam width in radians
        let tz = (0.5 * antenna_beam_state.elevation_beam_width_deg.to_radians()).tan(); // Half of the elevation beam width in radians
        let nyty = n.y * ty; // Normal vector component in the Y direction scaled by the azimuth beam width
        let nztz = n.z * tz; // Normal vector component in the Z direction
        // Parameters for ranges and extent computation
        let mut ground_max_extent_m = 0.0f64;
        let mut range_min_m = f64::MAX;
        let mut range_max_m = 0.0;
        let mut range_m: f64; // Temporary range variable
        let mut index_min_range: usize = 0; // Index of the minimum range point in the antenna beam footprint
        let mut index_max_range: usize = 0; // Index of the maximum range point in the
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
            mesh_pos[i] = [point.x as f32, 0.05, point.z as f32];// note: 0.05 in z-direction to be slightly above the ground plane (here Y axis)                
            // Update ranges and extent computation
            ground_max_extent_m = ground_max_extent_m.max(
                (point.x * point.x + point.z * point.z).sqrt() // Update maximum extent in the ground plane (x and z coordinates in Y-up frame)
            ); // Update maximum extent in the ground plane
            range_m =  carrier_position_y_up.distance(*point); // Compute the slant range from the antenna to the point
            if range_m < range_min_m {
                range_min_m = range_m; // Update minimum range
                index_min_range = i; // Update index of the minimum range point
            }
            if range_m > range_max_m {
                range_max_m = range_m; // Update maximum range
                index_max_range = i; // Update index of the maximum range point
            }
        }

        // Update the antenna beam footprint ranges
        antenna_beam_footprint_state.range_center_m = carrier_position_y_up.length();
        antenna_beam_footprint_state.range_min_m = range_min_m;
        antenna_beam_footprint_state.range_max_m = range_max_m;
        antenna_beam_footprint_state.ground_max_extent_m = ground_max_extent_m;

        // Update the ground range swath and local incidences
        let point_min_range = antenna_beam_footprint_state.points[index_min_range];
        let point_max_range = antenna_beam_footprint_state.points[index_max_range];
            // Ground range swath
        antenna_beam_footprint_state.ground_range_swath_m = point_min_range.distance(point_max_range);
            // Local incidence angle at the antenna beam footprint center
        let neg_antenna_beam_axis = if antenna_beam_footprint_state.range_center_m > 0.0 {  // Antenna beam (negative) axis in World frame (Y-up)    
            carrier_position_y_up / antenna_beam_footprint_state.range_center_m
        } else {
            DVec3::ZERO
        };
        antenna_beam_footprint_state.loc_incidence_center_deg = incidence(&neg_antenna_beam_axis);
            // Local incidence angle at the minimum range point
        let neg_antenna_beam_axis_min = (
            carrier_position_y_up - point_min_range
        ).normalize_or_zero(); // Antenna beam (negative) axis at minimum range in World frame (Y-up)
        antenna_beam_footprint_state.loc_incidence_min_deg = incidence(&neg_antenna_beam_axis_min);
            // Local incidence angle at the maximum range point
        let neg_antenna_beam_axis_max = (
            carrier_position_y_up - point_max_range
        ).normalize_or_zero(); // Antenna beam (negative) axis at minimum range in World frame (Y-up)
        antenna_beam_footprint_state.loc_incidence_max_deg = incidence(&neg_antenna_beam_axis_max);

        // Computes the half-power antenna beam footprint area using the "Shoelace" formula.
        antenna_beam_footprint_state.area_m2 = area(&antenna_beam_footprint_state.points);

        // Update the antenna squint angle
        antenna_beam_footprint_state.antenna_squint_deg = -squint(
            &carrier_state.position_m.normalize_or_zero(), // Antenna beam axis in World frame (Z-up)
            &carrier_state.velocity_vector_mps // Carrier velocity vector in World frame (Z-up)
        );

        // Update the illumination time
        update_illumination_time(
            carrier_state,
            antenna_beam_footprint_state
        );

        // Update the ground angular velocity
        update_ground_angular_velocity(
            carrier_state,
            antenna_beam_footprint_state
        );
    }
}

/// Computes the antenna ground angular velocity in degrees per second
/// note: it has its own function to be called if velocity value is update,
///       without the need to update the whole antenna beam footprint mesh and so on.
pub fn update_ground_angular_velocity(
    carrier_state: &CarrierState,
    antenna_beam_footprint_state: &mut AntennaBeamFootprintState,
) {
    let mut pos_ground = carrier_state.position_m; // Carrier position vector in World frame (Z-up)
    pos_ground.z = 0.0; // Rejection of the position vector in the ground plane (X-Y plane) for Z-up frame
    let pos_ground_length_squared = pos_ground.length_squared();
    // Update the ground angular velocity in degrees per second
    antenna_beam_footprint_state.ground_angular_velocity_degps = if pos_ground_length_squared > 0.0 {
        let mut vel_ground = carrier_state.velocity_vector_mps; // Carrier velocity vector in World frame (Z-up)
        vel_ground.z = 0.0; // Rejection of the velocity vector in the ground plane (X-Y plane) for Z-up frame
        pos_ground.cross(vel_ground)
            .length()
            .to_degrees() /
            pos_ground_length_squared
    } else {
        0.0
    };
}

/// Computes the illumination time from the intersection of the footprint with
/// the velocity vector projection on the world plane (uses line/segment intersection)
pub fn update_illumination_time(
    carrier_state: &CarrierState,
    antenna_beam_footprint_state: &mut AntennaBeamFootprintState,
) {
    if carrier_state.velocity_mps > 0.0 {
        // Carrier velocity in World frame (Y-up)
        let carrier_velocity_y_up = TO_Y_UP_F64 * carrier_state.velocity_vector_mps; // Carrier velocity vector in World frame (Y-up)
        let vx = carrier_velocity_y_up.x; 
        let vz = carrier_velocity_y_up.z;
        // Temporary variables
        let mut intersections = [DVec3::ZERO; 2]; // Intersections of the antenna beam footprint with the ground plane
        let mut count: usize = 0; // Number of intersection points
        let mut e1x: f64;
        let mut e1z: f64;
        let mut e2x: f64;
        let mut e2z: f64;
        let mut v: f64;
        for (e1, e2) in antenna_beam_footprint_state.points.iter()
                            .zip(antenna_beam_footprint_state.points.iter().skip(1)) {
            e1x = e1.x; // X coordinate of the first point
            e1z = e1.z; // Z coordinate of the first point
            e2x = e2.x; // X coordinate of the second point
            e2z = e2.z; // Z coordinate of the second point
            v = (vz * e1x - vx * e1z) / (vx * (e2z - e1z) - vz * (e2x - e1x));
            if (v >= 0.0) && (v < 1.0) {
                intersections[count] = DVec3::new(
                    e1x + v * (e2x - e1x),
                    0.0,
                    e1z + v * (e2z - e1z)
                );
                count += 1;
                if count == 2 { break; } // We only need two intersection points to compute the illumination time
            }
        }
        antenna_beam_footprint_state.illumination_time_s = 
            intersections[0].distance(intersections[1]) /
                carrier_state.velocity_mps; // Illumination time in seconds
    } else {
        antenna_beam_footprint_state.illumination_time_s = 0.0; // No illumination if velocity is zero
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
