use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy::render::mesh::{ConeAnchor, ConeMeshBuilder};

/// Spawns an antenna beam entity ine NED referential
/// pointing towards X-axis (N) with Elevation in the (Axz) plane
/// and Azimuth in the (Axy) plane.
pub fn spawn_antenna_beam(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    elevation_beam_width_rad: f64,
    azimuth_beam_width_rad: f64,
    color: Color,
) -> Entity {
    const CONE_LENGTH: f64 = 1e6; // Height of the antenna beam
    const CONE_MESH: ConeMeshBuilder = ConeMeshBuilder {
        cone: Cone {
            radius: 1.0,
            height: CONE_LENGTH as f32
        },
        resolution: 128,
        anchor: ConeAnchor::Tip
    };
    // Compute scale factors for cone base, based on beam widths
    let scale_azi = 2.0 * CONE_LENGTH * (0.5 * azimuth_beam_width_rad).tan();
    let scale_elv = 2.0 * CONE_LENGTH * (0.5 * elevation_beam_width_rad).tan();

    commands.spawn((
        Mesh3d(meshes.add(CONE_MESH)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_scale(
            Vec3::new(scale_azi as f32, 1.0, scale_elv as f32)
        ).with_rotation(
            Quat::from_rotation_z(FRAC_PI_2) // Rotate to align Y-axis with X-axis
        )
    )).id()
}