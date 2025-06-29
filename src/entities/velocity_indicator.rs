use bevy::prelude::*;
use bevy::render::mesh::{CylinderAnchor, CylinderMeshBuilder};

use crate::constants::{POS_YAXIS_TO_XAXIS, YELLOW_MATERIAL};

/// Spawns a velocity cylinder entity following the X-axis with unit length.
pub fn spawn_velocity_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // Create the cylinder and cone meshes
    let cylinder_mesh = CylinderMeshBuilder {
        cylinder: Cylinder {
            radius: 1.1,
            half_height: 0.5,
        },
        resolution: 64,
        segments: 1,
        caps: true,
        anchor: CylinderAnchor::Bottom,
    };

    commands.spawn((
        Mesh3d(meshes.add(cylinder_mesh)),
        MeshMaterial3d(materials.add(YELLOW_MATERIAL.clone())),
        Transform::from_rotation(POS_YAXIS_TO_XAXIS) // Rotate to align with X-axis
    )).id()
}
