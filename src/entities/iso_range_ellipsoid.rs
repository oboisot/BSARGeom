use bevy::{
    math::{DMat3, DQuat, DVec3},
    prelude::*,
    mesh::{SphereKind, SphereMeshBuilder}
};

use crate::constants::TO_Y_UP_F64;

pub fn spawn_iso_range_ellipsoid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    material: StandardMaterial,
) -> Entity {

    const SPHERE_MESH: SphereMeshBuilder = SphereMeshBuilder {
        sphere: Sphere { 
            radius: 1.0
        },
        kind: SphereKind::Uv {
            sectors: 128,
            stacks: 128,
        }
    };

    commands.spawn((
        Mesh3d(meshes.add(SPHERE_MESH)),
        MeshMaterial3d(materials.add(material)),
    )).id()
}


pub fn iso_range_ellipsoid_transform_from_state(
    otx: &DVec3, // OT in world frame
    orx: &DVec3, // OR in world frame
) -> Transform {
    // Center of the ellipsoid
    let txrx = orx - otx; // TR = OR - OT
    let center = otx + 0.5 * txrx; // Center = OT + 0.5 * TR
    // Ellipsoid referential axes
    let (u, v, w) = if txrx.length() < 1e-10 { // Monostatic case
        (DVec3::X, DVec3::Y, DVec3::Z) // Default axes
    } else {
        // Compute orthonormal basis from the direction of txrx
        let u = txrx.normalize();
        let mut v = DVec3::Z.cross(u);
        if v.length_squared() > 0.0 { // If u is not aligned with Z (indeed, z.cross(u) = 0 if u and z are colinear)
            v = v.normalize();
        } else { // If txrx is aligned with Z, we set v as x-axis
            v = DVec3::X;
        }
        let w = u.cross(v).normalize();
        (u, v, w)
    };
    // Ellipsoid radii
    let tx_norm = otx.length();
    let rx_norm = orx.length();
    let x_radius = 0.5 * (tx_norm + rx_norm); // Semi-major axis
    let y_radius = (0.5 * (tx_norm * rx_norm + otx.dot(*orx))).sqrt(); // Semi-minor axis

     // Convert to Y-up coordinate system + set rotation
    let center_y_up = TO_Y_UP_F64 * center;
    let rotation_y_up = TO_Y_UP_F64 * DQuat::from_mat3(&DMat3::from_cols(u, v, w));
    Transform {
        translation: center_y_up.as_vec3(),
        rotation: rotation_y_up.as_quat(),
        scale: Vec3::new(x_radius as f32, y_radius as f32, y_radius as f32),
    }
}