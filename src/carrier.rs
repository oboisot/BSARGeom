use bevy::prelude::*;

pub struct CarrierPlugin;

impl Plugin for CarrierPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_carrier);
    }
}

fn spawn_carrier(mut commands: Commands) {
    let carrier = (
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Name::new("Carrier"),
    );

    commands.spawn(carrier);
}