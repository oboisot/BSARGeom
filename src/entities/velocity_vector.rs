use bevy::prelude::*;
use bevy::render::mesh::{ConeAnchor, ConeMeshBuilder, CylinderAnchor, CylinderMeshBuilder};

use crate::{
    constants::{POS_YAXIS_TO_XAXIS, YELLOW_MATERIAL},
    entities::CarrierState,
};

/// Spawns a velocity vector entity following the X-axis with unit length.
pub fn spawn_velocity_vector(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // Create the cylinder and cone meshes
    let cylinder_mesh = CylinderMeshBuilder {
        cylinder: Cylinder {
            radius: 1.0,
            half_height: 0.45,
        },
        resolution: 64,
        segments: 1,
        caps: true,
        anchor: CylinderAnchor::Bottom,
    };

    let cone_mesh = ConeMeshBuilder {
        cone: Cone {
            radius: 15.0,
            height: 0.1,
        },
        resolution: 64,
        anchor: ConeAnchor::Base,
    };

    // Spawn the arrow
    commands.spawn((
        Mesh3d(meshes.add(cylinder_mesh)),
        MeshMaterial3d(materials.add(YELLOW_MATERIAL.clone())),
        Transform::from_rotation(POS_YAXIS_TO_XAXIS) // Rotate to align with X-axis
    )).with_child((
        Mesh3d(meshes.add(cone_mesh)),
        MeshMaterial3d(materials.add(YELLOW_MATERIAL.clone())),
        Transform::from_translation(0.9 * Vec3::Y),
    )).id()
}

/// Computes velocity vector transform from the carrier state.
pub fn velocity_vector_transform_from_state(
    carrier_state: &CarrierState
) -> Transform {
    let length = (5.0 * carrier_state.velocity_mps) as f32;
    Transform {
        translation: Vec3::ZERO,
        rotation: POS_YAXIS_TO_XAXIS, // Rotate to align with X-axis
        scale: Vec3::new(1.0, length, 1.0)
    }
}
