
use std::f64::consts::TAU;

const ANTENNA_BEAM_FOOTPRINT_SIZE: usize = 2501; // Size of the antenna beam footprint mesh
const STEP_THETA: f64 = TAU / (ANTENNA_BEAM_FOOTPRINT_SIZE - 1); // Step size for the antenna beam footprint mesh

/// Resource to keep old state of Transmitter
#[derive(Resource)]
pub struct TxAntennaBeamFootprintState {
    pub inner: AntennaBeamFootprintState,
}

impl Default for TxAntennaBeamFootprintState {
    fn default() -> Self {
        Self {
            inner: AntennaBeamFootprintState {
                points: vec![DVec3::ZERO; ANTENNA_BEAM_FOOTPRINT_SIZE], // Preallocate points for the antenna beam footprint
            }
        }
    }
}

impl TxAntennaBeamFootprintState {
    pub fn update_meshes(&mut self, meshes: &mut ResMut<Assets<Mesh>>) {
        // Create a new mesh for the antenna beam footprint
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.inner.points.clone());
        mesh.set_indices(Some(Indices::U32((0..ANTENNA_BEAM_FOOTPRINT_SIZE as u32).collect())));
        meshes.set_untracked(MeshId::new(), mesh);

    }
}
pub fn antenna_beam_footprint_mesh_from_state(
    carrier_state: &CarrierState,
    antenna_state: &AntennaState,
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
    let ty = (0.5 * antenna_state.azimuth_beam_width_rad).tan(); // Half of the azimuth beam width in radians
    let tz = (0.5 * antenna_state.elevation_beam_width_rad).tan(); // Half of the elevation beam width in radians
    let nyty = n.y * ty; // Normal vector component in the Y direction scaled by the azimuth beam width
    let nztz = n.z * tz; // Normal vector component in the Z direction

    // let mut points = Vec::with_capacity(ANTENNA_BEAM_FOOTPRINT_SIZE);
    let mut points = vec![DVec3::ZERO; ANTENNA_BEAM_FOOTPRINT_SIZE]; // Preallocate points for the antenna beam footprint
    let (mut s, mut c): (f64, f64); // (sin(theta), cos(theta))
    for i in 0..ANTENNA_BEAM_FOOTPRINT_SIZE {
        (s, c) = (i as f64 * STEP_THETA).sin_cos(); // Angle in radians
        points.x = d / (n.x + nyty * c + nztz * s);
        points.y = ty * c * points.x;
        points.z = tz * s * points.x;
    }
}