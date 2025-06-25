use std::sync::LazyLock;
use bevy::{
    math::DQuat,
    prelude::{Quat, StandardMaterial, Srgba, Transform}
};

/// Geometric constants
/// Length of the antenna beam in meters
pub const CONE_LENGTH: f64 = 1e6;
/// Carrier "size", i.e. length of arrows of its referential in meters
pub const CARRIER_SIZE: f32 = 150.0; // Size of the carrier
/// Antenna "size", i.e. length of arrows of its referential in meters
pub const ANTENNA_SIZE: f32 = 100.0;  // Size of the antenna

/// ENU to NED rotation quaternion
pub const ENU_TO_NED: Quat = Quat::from_xyzw(
    0.707106781186547524400844362104884, // x = sqrt(2) / 2
    0.707106781186547524400844362104884, // y = sqrt(2) / 2
    0.0,                // z
    0.0                 // w
);

pub const NED_TO_ENU: Quat = ENU_TO_NED;

/// ENU to NED rotation quaternion but with f64 accuracy
pub const ENU_TO_NED_F64: DQuat = DQuat::from_xyzw(
    0.707106781186547524400844362104884, // x = sqrt(2) / 2
    0.707106781186547524400844362104884, // y = sqrt(2) / 2
    0.0,                                 // z
    0.0                                  // w
);

/// Rotation constants to convert from Z-up (Physics) direction to Y-up (Bevy) direction coordinate systems.
pub const TO_Y_UP: Quat = Quat::from_xyzw(
    0.5, // x
    0.5, // y
    0.5, // z
    -0.5 // w
);

/// Transform relative to TO_Y_UP rotation.
pub const TRANSFORM_TO_Y_UP: Transform = Transform::from_rotation(TO_Y_UP);

/// Rotation constants to convert from Y-up (Bevy) direction to Z-up (Physics) direction coordinate systems.
pub const TO_Z_UP: Quat = Quat::from_xyzw(
    0.5, // x
    0.5, // y
    0.5, // z
    0.5 // w
);

/// Transform relative to TO_Z_UP rotation.
pub const TRANSFORM_TO_Z_UP: Transform = Transform::from_rotation(TO_Z_UP);

/// Rotation to align negative Y-axis with X-axis
/// note: this is used to align antenna cone following -y-axis to x-axis
pub const NEG_YAXIS_TO_XAXIS: Quat = Quat::from_xyzw( // = Quat::from_rotation_z(FRAC_PI_2)
    0.0,
    0.0,
    0.707106781186547524400844362104884, // z = sqrt(2) / 2 = sin((pi/2)/2)
    0.707106781186547524400844362104884  // w = sqrt(2) / 2 = cos((pi/2)/2)
);

// Default materials
pub static RED_MATERIAL: LazyLock<StandardMaterial> = LazyLock::new(|| {
    StandardMaterial {
        base_color: Srgba::RED.into(),
        cull_mode: None,
        unlit: true,
        ..Default::default()
    }
});

pub static GREEN_MATERIAL: LazyLock<StandardMaterial> = LazyLock::new(|| {
    StandardMaterial {
        base_color: Srgba::GREEN.into(),
        // cull_mode: None,
        unlit: true,
        ..Default::default()
    }
});

pub static BLUE_MATERIAL: LazyLock<StandardMaterial> = LazyLock::new(|| {
    StandardMaterial {
        base_color: Srgba::BLUE.into(),
        cull_mode: None,
        unlit: true,
        ..Default::default()
    }
});

pub static YELLOW_MATERIAL: LazyLock<StandardMaterial> = LazyLock::new(|| {
    StandardMaterial {
        base_color: Srgba::new(1.0, 1.0, 0.0, 1.0).into(),
        cull_mode: None,
        unlit: true,
        ..Default::default()
    }
});
