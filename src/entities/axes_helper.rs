use bevy::prelude::*;
use bevy::render::mesh::{ConeAnchor, ConeMeshBuilder, CylinderAnchor, CylinderMeshBuilder};

use std::f32::consts::FRAC_PI_2;

use crate::constants::{BLUE_MATERIAL, GREEN_MATERIAL, RED_MATERIAL, TO_Y_UP, YELLOW_MATERIAL};

// https://users.rust-lang.org/t/solved-placement-of-mut-in-function-parameters/19891
pub fn spawn_axes_helper(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    size: f32 // 
) -> Entity {
    let cylinder_mesh =  CylinderMeshBuilder {
        cylinder: Cylinder {
            radius: 0.005 * size,
            half_height: 0.45 * size
        },
        resolution: 32,
        segments: 1,
        caps: true,
        anchor: CylinderAnchor::Bottom, 
    };

    let cone_mesh = ConeMeshBuilder {
        cone: Cone {
            radius: 0.05 * size,
            height: 0.1 * size
        },
        resolution: 32,
        anchor: ConeAnchor::Base
    };

    // Spawn the X-axis helper
    let xaxis = commands.spawn(( // base
        Mesh3d(meshes.add(cylinder_mesh)),
        MeshMaterial3d(materials.add(RED_MATERIAL.clone())),
        Transform::from_rotation(
            TO_Y_UP *
            Quat::from_rotation_z(-FRAC_PI_2) // Rotate to align with X-axis
        ) 
    )).with_child(( // arrow
        Mesh3d(meshes.add(cone_mesh)),
        MeshMaterial3d(materials.add(RED_MATERIAL.clone())),
        Transform::from_translation(0.9 * size * Vec3::Y)
    )).id();

    // Spawn the Y-axis helper
    let yaxis = commands.spawn(( // base
        Mesh3d(meshes.add(cylinder_mesh)),
        MeshMaterial3d(materials.add(GREEN_MATERIAL.clone())),
        Transform::from_rotation(TO_Y_UP)
    )).with_child(( // arrow
        Mesh3d(meshes.add(cone_mesh)),
        MeshMaterial3d(materials.add(GREEN_MATERIAL.clone())),
        Transform::from_translation(0.9 * size * Vec3::Y)
    )).id();

    // Spawn the Y-axis helper
    let zaxis = commands.spawn(( // base
        Mesh3d(meshes.add(cylinder_mesh)),
        MeshMaterial3d(materials.add(BLUE_MATERIAL.clone())),
        Transform::from_rotation(
            TO_Y_UP *
            Quat::from_rotation_x(FRAC_PI_2) // Rotate to align with Z-axis
        ) 
    )).with_child(( // arrow
        Mesh3d(meshes.add(cone_mesh)),
        MeshMaterial3d(materials.add(BLUE_MATERIAL.clone())),
        Transform::from_translation(0.9 * size * Vec3::Y)
    )).id();

    // Spawn a sphere for the origin
    let mut sphere_base = commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.0125 * size))),
        MeshMaterial3d(materials.add(YELLOW_MATERIAL.clone()))
    ));

    // Axes helper entity
    sphere_base
        .add_children(&[xaxis, yaxis, zaxis])
        .id()
}
