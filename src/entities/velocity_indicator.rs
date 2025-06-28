use bevy::prelude::*;
use bevy::render::mesh::{CylinderAnchor, CylinderMeshBuilder};

use crate::{
    constants::{POS_YAXIS_TO_XAXIS, YELLOW_MATERIAL},
    entities::CarrierState,
};

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

/// Computes velocity indicator transform from the carrier state.
pub fn velocity_indicator_transform_from_state(
    carrier_state: &CarrierState
) -> Transform {
    let x = carrier_state.velocity_mps;
    if x <= 150.0 { // = CARRIER_SIZE
        let length = 150.0 * x.ln_1p() / 150.0f64.ln_1p() ;// logarithmic growth: F(x) = ymax * ln(1 + x) / ln(1 + xmax)
        Transform {
            translation: Vec3::ZERO,
            rotation: POS_YAXIS_TO_XAXIS, // Rotate to align with X-axis
            scale: Vec3::new(1.0, length as f32, 1.0)
        }
    } else {
        let length = 0.08542713567839195 * (x - 150.0) + 150.0; // linear growth (note max length for vmax=10_000m/s is 1000.0)
        let scale = 0.00047058823529411766 * x + 1.0294117647058825; // linear growth of the cylinder radius
        Transform {
            translation: Vec3::ZERO,
            rotation: POS_YAXIS_TO_XAXIS, // Rotate to align with X-axis
            scale: Vec3::new(scale as f32, length as f32, scale as f32)
        }
    }
}
