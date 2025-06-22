use bevy::{
    color::palettes::css::{DARK_SLATE_GRAY, GREEN, GREY, RED},
    prelude::*
};

use crate::entities::{spawn_axes_helper, spawn_grid_helper};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_world, insert_ambient_light));
    }
}

fn insert_ambient_light(mut commands: Commands) {
    let ambient_light = AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: false,
    };

    commands.insert_resource(ambient_light);
}

// const PLANE_LENGTH: f32 = 30_000.0;
const HALF_PLANE_LENGTH: f32 = 15_000.0;
const GRID_SPACING: f32 = 500.0;

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Set up the materials.
    let floor_material = materials.add(
        StandardMaterial {
            base_color: GREY.into(),
            cull_mode: None, // Turning off culling keeps the plane visible when viewed from beneath.
            ..default()
        }
    );

    // Spawn Grid
    let grid_helper_entity = spawn_grid_helper(
        &mut commands,
        &mut meshes,
        &mut materials,
        2.0 * HALF_PLANE_LENGTH, // Size of the grid
        GRID_SPACING, // Spacing of the grid lines
        DARK_SLATE_GRAY.into(), // Color of the grid lines
        RED.into(),   // Color of the center X-line
        GREEN.into(), // Color of the center Y-line
    );

    // Spawn Axes Helper
    let axes_helper_entity = spawn_axes_helper(
        &mut commands,
        &mut meshes,
        &mut materials,
        GRID_SPACING // Size of the axes
    );

    // Floor bundle
    let floor = (
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::splat(HALF_PLANE_LENGTH)))),
        MeshMaterial3d(floor_material),
        Pickable::IGNORE, // Disable picking for the ground plane.
    );

    commands
        .spawn(floor)
        .add_children(&[
            grid_helper_entity,
            axes_helper_entity
        ]);
}
