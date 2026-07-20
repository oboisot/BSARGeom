use bevy_egui::egui;

use crate::{
    constants::{MAX_HEIGHT_M, MAX_VELOCITY_MPS},
    entities::{AntennaBeamState, AntennaState, CarrierState},
    ui::menu::RESET_ICON,
};

/// Section heading row: centered title with a small right-aligned "↺" reset
/// button. Returns `true` when the reset button was clicked.
pub fn heading_with_reset(ui: &mut egui::Ui, title: egui::RichText, hover: &str) -> bool {
    let hover_text = egui::RichText::new(hover)
        .color(egui::Color32::from_rgb(200, 200, 200))
        .monospace();
    let row = ui.vertical_centered(|ui| ui.label(title)).response;
    let button_rect = egui::Rect::from_center_size(
        egui::pos2(row.rect.right() - 12.0, row.rect.center().y),
        egui::vec2(18.0, row.rect.height()),
    );
    ui.put(button_rect, egui::Button::image(RESET_ICON).frame_when_inactive(false))
        .on_hover_text(hover_text)
        .clicked()
}

/// Carrier + antenna geometry settings UI, shared by the Transmitter and
/// Receiver panels.
///
/// `id_salt` ("tx" | "rx") rebuilds the historical egui grid ids
/// ("tx_carrier_grid", ...) so widget memory is preserved; it must not change.
/// The `default_*` states are the side-specific defaults restored by the
/// per-section reset buttons.
///
/// Returns `true` when the title-row reset was clicked, i.e. the whole side
/// must go back to its defaults. The carrier/antenna sections are restored
/// here; the caller additionally restores its own SYSTEM section.
pub fn carrier_ui(
    ui: &mut egui::Ui,
    id_salt: &str,
    title: &str,
    carrier_state: &mut CarrierState,
    antenna_state: &mut AntennaState,
    antenna_beam_state: &mut AntennaBeamState,
    default_carrier_state: &CarrierState,
    default_antenna_state: &AntennaState,
    default_antenna_beam_state: &AntennaBeamState,
    transform_needs_update: &mut bool,
    velocity_vector_needs_update: &mut bool,
) -> bool {
    let mut old_state = 0.0f64;

    ui.separator();
    let reset_all = heading_with_reset(
        ui,
        egui::RichText::new(title).size(15.0).strong(),
        "Resets every setting of this element to its defaults",
    );
    ui.separator();

    ui.separator();
    if heading_with_reset(
        ui,
        egui::RichText::new("CARRIER").strong(),
        "Resets the Carrier settings to their defaults"
    ) || reset_all {
        // Only the fields edited in this section (derived fields are
        // recomputed by the update systems from the flags below)
        carrier_state.height_m = default_carrier_state.height_m;
        carrier_state.velocity_mps = default_carrier_state.velocity_mps;
        carrier_state.heading_deg = default_carrier_state.heading_deg;
        carrier_state.elevation_deg = default_carrier_state.elevation_deg;
        carrier_state.bank_deg = default_carrier_state.bank_deg;
        *transform_needs_update = true;
        *velocity_vector_needs_update = true;
    }
    ui.separator();

    // Carrier settings
    egui::Grid::new(format!("{id_salt}_carrier_grid"))
        .num_columns(2)
        .striped(false)
        .spacing([20.0, 5.0])
        .show(ui, |ui| {
            // ***** Carrier height ***** //
            let hover_text = egui::RichText::new(format!("Sets the Carrier's height relative to ground (0 - {} m)", MAX_HEIGHT_M))
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Height: ").on_hover_text(hover_text.clone());
            old_state = carrier_state.height_m;
            ui.add(
                egui::DragValue::new(&mut carrier_state.height_m)
                    .update_while_editing(false)
                    .speed(10.0)
                    .range(0.0..=MAX_HEIGHT_M)
                    .fixed_decimals(3)
                    .suffix(" m")
            ).on_hover_text(hover_text);
            if old_state != carrier_state.height_m {
                *transform_needs_update = true;
            }
            ui.end_row();

            // ***** Carrier velocity ***** //
            let hover_text = egui::RichText::new(format!("Sets the Carrier's velocity (0 - {} m/s)", MAX_VELOCITY_MPS))
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Velocity: ").on_hover_text(hover_text.clone());
            old_state = carrier_state.velocity_mps;
            ui.add(
                egui::DragValue::new(&mut carrier_state.velocity_mps)
                    .update_while_editing(false)
                    .speed(10.0)
                    .range(0.0..=MAX_VELOCITY_MPS)
                    .fixed_decimals(3)
                    .suffix(" m/s")
            ).on_hover_text(hover_text);
            if old_state != carrier_state.velocity_mps {
                *velocity_vector_needs_update = true;
            }
            ui.end_row();

            // ***** Carrier heading ***** //
            let hover_text = egui::RichText::new("Sets the Carrier's heading angle (0 - 360°):\n    0° => North\n   90° => East\n  180° => South\n  270° => West\nnote: rotation along z-axis of Carrier's NED frame")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Heading: ").on_hover_text(hover_text.clone());
            old_state = carrier_state.heading_deg;
            ui.add(
                egui::Slider::new(&mut carrier_state.heading_deg, 0.0..=360.0)
                    .suffix("°")
                    .smart_aim(false)
                    .step_by(0.0)
                    .drag_value_speed(1.0)
                    .fixed_decimals(3)
            ).on_hover_text(hover_text);
            if old_state != carrier_state.heading_deg {
                *transform_needs_update = true;
            }
            ui.end_row();

            // ***** Carrier elevation ***** //
            let hover_text = egui::RichText::new("Sets the Carrier's elevation angle (-90 - 90°):\n  -90° => nadir-looking\n    0° => horizontal-looking\n  +90° => sky-looking\nnote: rotation along y-axis of Carrier's NED frame")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Elevation: ").on_hover_text(hover_text.clone());
            old_state = carrier_state.elevation_deg;
            ui.add(
                egui::Slider::new(&mut carrier_state.elevation_deg, -90.0..=90.0)
                    .suffix("°")
                    .smart_aim(false)
                    .step_by(0.0)
                    .drag_value_speed(1.0)
                    .fixed_decimals(3)
            ).on_hover_text(hover_text);
            if old_state != carrier_state.elevation_deg {
                *transform_needs_update = true;
            }
            ui.end_row();

            // ***** Carrier bank ***** //
            let hover_text = egui::RichText::new("Sets the Carrier's bank angle (-90 - 90°):\n  -90° => left wing down\n    0° => horizontal wings\n  +90° => right wing down\nnote: rotation along x-axis of Carrier's NED frame")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Bank: ").on_hover_text(hover_text.clone());
            old_state = carrier_state.bank_deg;
            ui.add(
                egui::Slider::new(&mut carrier_state.bank_deg, -90.0..=90.0)
                    .suffix("°")
                    .smart_aim(false)
                    .step_by(0.0)
                    .drag_value_speed(1.0)
                    .fixed_decimals(3)
            ).on_hover_text(hover_text);
            if old_state != carrier_state.bank_deg {
                *transform_needs_update = true;
            }
            ui.end_row();
        });

    ui.separator();
    ui.vertical_centered(|ui| ui.label(
        egui::RichText::new("ANTENNA").strong()
    ));
    ui.separator();

    // Antenna orientation settings
    if heading_with_reset(
        ui,
        egui::RichText::new("Orientation"),
        "Resets the Antenna orientation to its defaults"
    ) || reset_all {
        antenna_state.heading_deg = default_antenna_state.heading_deg;
        antenna_state.elevation_deg = default_antenna_state.elevation_deg;
        antenna_state.bank_deg = default_antenna_state.bank_deg;
        *transform_needs_update = true;
    }
    ui.separator();

    egui::Grid::new(format!("{id_salt}_antenna_orientation_grid"))
        .num_columns(2)
        .striped(false)
        .spacing([20.0, 5.0])
        .show(ui, |ui| {
            // ***** Antenna heading ***** //
            let hover_text = egui::RichText::new("Sets the Antenna's heading angle (-180 - 180°):\n  -90° => left-looking\n    0° => forward-looking\n  +90° => right-looking\n ±180° => backward-looking\nnote: rotation along z-axis of Antenna's NED frame")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Heading: ").on_hover_text(hover_text.clone());
            old_state = antenna_state.heading_deg;
            ui.add(
                egui::Slider::new(&mut antenna_state.heading_deg, -180.0..=180.0)
                    .suffix("°")
                    .smart_aim(false)
                    .step_by(0.0)
                    .drag_value_speed(1.0)
                    .fixed_decimals(3)
            )
            .on_hover_text(hover_text);
            if old_state != antenna_state.heading_deg {
                *transform_needs_update = true;
            }
            ui.end_row();

            // ***** Antenna elevation ***** //
            let hover_text = egui::RichText::new("Sets the Antenna's elevation angle (-90 - 0°):\n  -90° => vertical-looking\n    0° => horizontal-looking\nnote: rotation along y-axis of Antenna's NED frame")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Elevation: ").on_hover_text(hover_text.clone());
            old_state = antenna_state.elevation_deg;
            ui.add(
                egui::Slider::new(&mut antenna_state.elevation_deg, -90.0..=0.0)
                    .suffix("°")
                    .smart_aim(false)
                    .step_by(0.0)
                    .drag_value_speed(1.0)
                    .fixed_decimals(3)
            )
            .on_hover_text(hover_text);
            if old_state != antenna_state.elevation_deg {
                *transform_needs_update = true;
            }
            ui.end_row();

            // ***** Antenna bank ***** //
            let hover_text = egui::RichText::new("Sets the Antenna's bank angle (-90 - 90°)\nnote: rotation along x-axis of Antenna's NED frame")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Bank: ").on_hover_text(hover_text.clone());
            old_state = antenna_state.bank_deg;
            ui.add(
                egui::Slider::new(&mut antenna_state.bank_deg, -90.0..=90.0)
                    .suffix("°")
                    .smart_aim(false)
                    .step_by(0.0)
                    .drag_value_speed(1.0)
                    .fixed_decimals(3)
            )
            .on_hover_text(hover_text);
            if old_state != antenna_state.bank_deg {
                *transform_needs_update = true;
            }
            ui.end_row();
        });

    ui.separator();
    if heading_with_reset(
        ui,
        egui::RichText::new("Beamwidth (half-power)"),
        "Resets the Antenna beamwidths to their defaults"
    ) || reset_all {
        // Only the beamwidths: the antenna gain belongs to the SYSTEM section
        antenna_beam_state.elevation_beam_width_deg = default_antenna_beam_state.elevation_beam_width_deg;
        antenna_beam_state.azimuth_beam_width_deg = default_antenna_beam_state.azimuth_beam_width_deg;
        *transform_needs_update = true;
    }
    ui.separator();
    // Antenna beamwidth settings
    egui::Grid::new(format!("{id_salt}_antenna_beamwidth_grid"))
        .num_columns(2)
        .striped(false)
        .spacing([20.0, 5.0])
        .show(ui, |ui| {
            // ***** Antenna beamwidth elevation ***** //
            let hover_text = egui::RichText::new("Sets the Antenna's elevation half-power beamwidth (0 - 90°)\nnote: elevation beamwidth angle is defined in the x-z plane of Antenna's NED frame")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Elevation: ").on_hover_text(hover_text.clone());
            old_state = antenna_beam_state.elevation_beam_width_deg;
            ui.add(
                egui::Slider::new(&mut antenna_beam_state.elevation_beam_width_deg, 0.0..=90.0)
                    .suffix("°")
                    .smart_aim(false)
                    .step_by(0.0)
                    .drag_value_speed(1.0)
                    .fixed_decimals(3)
            )
            .on_hover_text(hover_text);
            if old_state != antenna_beam_state.elevation_beam_width_deg {
                *transform_needs_update = true;
            }
            ui.end_row();

            // ***** Antenna azimuth ***** //
            let hover_text = egui::RichText::new("Sets the Antenna's azimuth half-power beamwidth (0 - 90°)\nnote: azimuth beamwidth angle is defined in the x-y plane of Antenna's NED frame")
                .color(egui::Color32::from_rgb(200, 200, 200))
                .monospace();
            ui.label("Azimuth: ").on_hover_text(hover_text.clone());
            old_state = antenna_beam_state.azimuth_beam_width_deg;
            ui.add(
                egui::Slider::new(&mut antenna_beam_state.azimuth_beam_width_deg, 0.0..=90.0)
                    .suffix("°")
                    .smart_aim(false)
                    .step_by(0.0)
                    .drag_value_speed(1.0)
                    .fixed_decimals(3)
            )
            .on_hover_text(hover_text);
            if old_state != antenna_beam_state.azimuth_beam_width_deg {
                *transform_needs_update = true;
            }
            ui.end_row();
        });

    reset_all
}
