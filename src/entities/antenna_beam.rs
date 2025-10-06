use bevy::{
    prelude::*,
    mesh::{ConeAnchor, ConeMeshBuilder}
};

use crate::constants::CONE_LENGTH;

/// Spawns an antenna beam entity ine NED referential
/// pointing towards X-axis (N) with Elevation in the (Axz) plane
/// and Azimuth in the (Axy) plane.
pub fn spawn_antenna_beam(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    material: StandardMaterial,
) -> Entity {
    
    const CONE_MESH: ConeMeshBuilder = ConeMeshBuilder {
        cone: Cone {
            radius: 1.0,
            height: CONE_LENGTH as f32
        },
        resolution: 256,
        anchor: ConeAnchor::Tip
    };

    commands.spawn((
        Mesh3d(meshes.add(CONE_MESH)),
        MeshMaterial3d(materials.add(material)),
    )).id()
}
