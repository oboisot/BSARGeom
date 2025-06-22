use std::sync::LazyLock;
use bevy::{
    math::DQuat,
    prelude::{StandardMaterial, Srgba, Quat}
};

/// Geometric constants
// pub static ENU_TO_NED_ROT: LazyLock<Quat> = LazyLock::new(|| {
//     Quat::from_mat3(&Mat3 { // ENU -> NED rotation
//         x_axis: Vec3::Y,
//         y_axis: Vec3::X,
//         z_axis: -Vec3::Z
//     })
// });
pub const ENU_TO_NED: Quat = Quat::from_xyzw(
    0.7071067811865476, // x = sqrt(2) / 2
    0.7071067811865476, // y = sqrt(2) / 2
    0.0,                // z
    0.0                 // w
);

pub const NED_TO_ENU: Quat = ENU_TO_NED;

// Rotation constants to convert from Z-up (Physics) direction to Y-up (Bevy) direction coordinate systems.
pub const TO_Y_UP: Quat = Quat::from_xyzw(
    0.5, // x
    0.5, // y
    0.5, // z
    -0.5 // w
);

// Rotation constants to convert from Y-up (Bevy) direction to Z-up (Physics) direction coordinate systems.
pub const TO_Z_UP: Quat = Quat::from_xyzw(
    0.5, // x
    0.5, // y
    0.5, // z
    0.5 // w
);

pub const ENU_TO_NED_F64: DQuat = DQuat::from_xyzw(
    0.707106781186547524400844362104884, // x = sqrt(2) / 2
    0.707106781186547524400844362104884, // y = sqrt(2) / 2
    0.0,                                 // z
    0.0                                  // w
);

// Default materials
pub static RED_MATERIAL: LazyLock<StandardMaterial> = LazyLock::new(|| {
    StandardMaterial {
        base_color: Srgba::RED.into(),
        cull_mode: None, // Turning off culling keeps the plane visible when viewed from beneath.
        ..Default::default()
    }
});

pub static GREEN_MATERIAL: LazyLock<StandardMaterial> = LazyLock::new(|| {
    StandardMaterial {
        base_color: Srgba::GREEN.into(),
        cull_mode: None, // Turning off culling keeps the plane visible when viewed from beneath.
        ..Default::default()
    }
});

pub static BLUE_MATERIAL: LazyLock<StandardMaterial> = LazyLock::new(|| {
    StandardMaterial {
        base_color: Srgba::BLUE.into(),
        cull_mode: None, // Turning off culling keeps the plane visible when viewed from beneath.
        ..Default::default()
    }
});

pub static YELLOW_MATERIAL: LazyLock<StandardMaterial> = LazyLock::new(|| {
    StandardMaterial {
        base_color: Srgba::new(1.0, 1.0, 0.0, 1.0).into(),
        cull_mode: None, // Turning off culling keeps the plane visible when viewed from beneath.
        ..Default::default()
    }
});
