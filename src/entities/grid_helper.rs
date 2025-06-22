use bevy::prelude::*;

use crate::entities::LineList;

// https://users.rust-lang.org/t/solved-placement-of-mut-in-function-parameters/19891
pub fn spawn_grid_helper(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    grid_size: f32,
    grid_spacing: f32,
    grid_color: Color,
    center_x_line_color: Color,
    center_y_line_color: Color, 
) -> Entity {

    // Create the grid lines
    let half_grid_size = grid_size * 0.5;
    let half_num_lines = (half_grid_size / grid_spacing).floor() as usize;
    let mut lines = Vec::<(Vec3, Vec3)>::with_capacity(4 * half_num_lines);
    let mut offset: f32;
    // X-lines
    for i in 1..=half_num_lines {
        offset = grid_spacing * i as f32; // y-offset
        lines.push(
            (Vec3::new(-half_grid_size, offset, 0.0), Vec3::new(half_grid_size, offset, 0.0))
        );
        lines.push(
            (Vec3::new(-half_grid_size, -offset, 0.0), Vec3::new(half_grid_size, -offset, 0.0))
        );
    }
    // Y-lines
    for i in 1..=half_num_lines {
        offset = grid_spacing * i as f32; // x-offset
        lines.push(
            (Vec3::new(offset, -half_grid_size, 0.0), Vec3::new(offset, half_grid_size, 0.0))
        );
        lines.push(
            (Vec3::new(-offset, -half_grid_size, 0.0), Vec3::new(-offset, half_grid_size, 0.0))
        );
    }

    // Center X-line Entity
    let center_x_line = commands.spawn((
        Mesh3d(meshes.add(LineList {
            lines: vec![
                (Vec3::new(-half_grid_size, 0.0, 0.0), Vec3::new(half_grid_size, 0.0, 0.0)),
            ],
        })),
        MeshMaterial3d(materials.add(
            StandardMaterial {
                base_color: center_x_line_color,
                cull_mode: None,
                unlit: true,
                ..default()
            }
        )),
    )).id();

    // Center Y-line Entity
    let center_y_line = commands.spawn((
        Mesh3d(meshes.add(LineList {
            lines: vec![
                (Vec3::new(0.0, -half_grid_size, 0.0), Vec3::new(0.0, half_grid_size, 0.0)),
            ],
        })),
        MeshMaterial3d(materials.add(
            StandardMaterial {
                base_color: center_y_line_color,
                cull_mode: None,
                unlit: true,
                ..default()
            }
        )),
    )).id();

    let grid_lines = (
        Mesh3d(meshes.add(LineList{ lines })),
        MeshMaterial3d(materials.add(
            StandardMaterial {
                base_color: grid_color,
                cull_mode: None,
                unlit: true,
                ..default()
            }
        ))
    );

    commands
        .spawn(grid_lines)
        .add_children(&[
            center_x_line,
            center_y_line,
        ]).id()
}
