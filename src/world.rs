use bevy::{
    color::palettes::css::{DARK_SLATE_GRAY, GREEN, GREY, RED},
    prelude::*,
    render::render_resource::Face,
};

use crate::{
    constants::{GRID_SPACING, HALF_PLANE_LENGTH},
    entities::{spawn_axes_helper, spawn_grid_helper},
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (insert_ambient_light, spawn_world));
    }
}

/// Component marker for floor plane
#[derive(Component)]
struct WorldFloor;

/// Component marker for world grid
#[derive(Component)]
struct WorldGridHelper;

/// Component marker for world referential
#[derive(Component)]
struct WorldAxesHelper;

fn insert_ambient_light(mut commands: Commands) {
    let ambient_light = AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: false,
    };

    commands.insert_resource(ambient_light);
}

// const HALF_PLANE_LENGTH: f32 = 15_000.0;
// const GRID_SPACING: f32 = 500.0;

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Grid helper
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

    commands
        .entity(grid_helper_entity)
        .insert(WorldGridHelper)
        .insert(Name::new("World grid helper"));

    // Axes Helper
    let axes_helper_entity = spawn_axes_helper(
        &mut commands,
        &mut meshes,
        &mut materials,
        GRID_SPACING // Size of the axes
    );

    commands
        .entity(axes_helper_entity)
        .insert(WorldAxesHelper)
        .insert(Name::new("World axes helper"));

    // Floor bundle
    let floor_material = materials.add(
        StandardMaterial {
            base_color: GREY.into(),
            cull_mode: Some(Face::Back),
            unlit: true,
            ..default()
        }
    );

    let floor = (
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(HALF_PLANE_LENGTH)))),
        MeshMaterial3d(floor_material)
    );

    commands
        .spawn(floor)
        .insert(WorldFloor)
        .insert(Name::new("World floor"))
        .add_children(&[
            grid_helper_entity,
            axes_helper_entity
        ]);
}


// fn force_init_world_transform(
//     mut floor_q: Query<&mut Transform, With<Floor>>,
// ) {
//     let mut transform = floor_q
//         .single_mut()
//         .expect("Can't get `Floor` transform");
//     *transform = Transform::IDENTITY;
// }
