//! Normalized Generalized Ambiguity Function (GAF) plot.
//!
//! Renders, over a ground patch around the target, the point-target response
//!
//! ```text
//! GAF(r) = |sinc(B/c0 · (βg·r)) · sinc(Tint/λ · (dβg·r))|,  r = (x, y, 0)
//! ```
//!
//! in dB, following BSARConf's `drawGAFIntensity`. The ground bisector vectors
//! `βg`/`dβg` come from the already-computed [`BsarInfos`].

use bevy::math::DVec3;
use bevy::prelude::Resource;
use bevy_egui::egui;

use crate::bsar::{sinc, BsarInfos, SPEED_OF_LIGHT_IN_VACUUM};
use crate::contour::{march, Field};

/// Ground patch resolution (pixels per side) of the rendered GAF image.
const GAF_RENDER_SIZE: usize = 400;
/// Displayed dynamic range: `GAF_DB_MIN` dB maps to black, 0 dB to white.
const GAF_DB_MIN: f64 = -30.0;
/// Fallback half-extent [m] when a resolution is degenerate (matches BSARConf).
const GAF_FALLBACK_HALF_EXTENT_M: f64 = 1000.0;
/// Overlaid iso-level contours [dB] with their (R, G, B) colors.
const GAF_CONTOURS: [(f64, (u8, u8, u8)); 3] = [
    (-3.0, (255, 60, 60)),   // half-power resolution cell
    (-10.0, (255, 150, 60)),
    (-20.0, (255, 210, 90)),
];

/// Cached GAF texture and iso-dB contours, rebuilt only when the inputs
/// change. The open/close state lives in [`crate::ui::MenuWidget`] (the menu
/// owns the toggle button).
#[derive(Resource, Default)]
pub struct GafState {
    texture: Option<egui::TextureHandle>,
    /// Iso-dB contours as (level, color, polylines in ground metres). Drawn as
    /// vector plot lines so they stay crisp when zooming and can be toggled
    /// from the plot legend.
    contours: Vec<(f64, egui::Color32, Vec<Vec<[f64; 2]>>)>,
    cache_key: Option<GafKey>,
}

/// The GAF dB grid, as a [`Field`] so contours can be extracted with the same
/// marching-squares implementation used by the ground iso-range/Doppler plane.
struct GafField {
    size: usize,
    data: Vec<f64>,
}

impl Field for GafField {
    fn dimensions(&self) -> (usize, usize) {
        (self.size, self.size)
    }

    fn z_at(&self, x: usize, y: usize) -> f64 {
        self.data[y * self.size + x] // y -> row, x -> col
    }
}

/// The inputs that fully determine the rendered image; equality gates the
/// texture rebuild.
#[derive(Clone, Copy, PartialEq)]
struct GafKey {
    betag: DVec3,
    dbetag: DVec3,
    b_over_c0: f64,
    tint_over_lem: f64,
    half_extent_m: f64,
}

/// GAF value in dB at ground point `(x, y)` [m].
fn gaf_db(betag: DVec3, dbetag: DVec3, b_over_c0: f64, tint_over_lem: f64, x: f64, y: f64) -> f64 {
    let range_phase = betag.x * x + betag.y * y; // βg·r, z = 0
    let doppler_phase = dbetag.x * x + dbetag.y * y; // dβg·r, z = 0
    let amplitude = sinc(b_over_c0 * range_phase) * sinc(tint_over_lem * doppler_phase);
    20.0 * amplitude.abs().log10()
}

/// Builds the render inputs from the current BSAR state, or `None` when the
/// geometry is degenerate (NaN bisectors/resolutions/integration time).
fn gaf_key(bsar_infos: &BsarInfos, bandwidth_hz: f64, center_frequency_hz: f64) -> Option<GafKey> {
    let lem = SPEED_OF_LIGHT_IN_VACUUM / center_frequency_hz;
    let tint = bsar_infos.integration_time_s;
    if !bsar_infos.betag.is_finite()
        || !bsar_infos.dbetag.is_finite()
        || !tint.is_finite()
        || !lem.is_finite()
    {
        return None;
    }
    // Half-extent: 5x the coarsest ground resolution, clamped when degenerate,
    // exactly like BSARConf.
    let ground_range = bsar_infos.ground_range_resolution_m;
    let ground_lateral = bsar_infos.ground_lateral_resolution_m;
    let half_extent_m = if !ground_range.is_finite()
        || !ground_lateral.is_finite()
        || ground_range > 1e6
        || ground_lateral > 1e6
    {
        GAF_FALLBACK_HALF_EXTENT_M
    } else {
        5.0 * ground_range.max(ground_lateral)
    };
    if half_extent_m <= 0.0 {
        return None;
    }
    Some(GafKey {
        betag: bsar_infos.betag,
        dbetag: bsar_infos.dbetag,
        b_over_c0: bandwidth_hz / SPEED_OF_LIGHT_IN_VACUUM,
        tint_over_lem: tint / lem,
        half_extent_m,
    })
}

/// Samples the GAF over the ground patch. Row 0 is +North (top of the image),
/// matching how `PlotImage` maps the texture into plot coordinates.
fn compute_gaf_grid(key: &GafKey) -> GafField {
    let size = GAF_RENDER_SIZE;
    let mut data = vec![0.0f64; size * size];
    let step = 2.0 * key.half_extent_m / (size - 1) as f64;
    for row in 0..size {
        let y = key.half_extent_m - step * row as f64;
        for col in 0..size {
            let x = -key.half_extent_m + step * col as f64;
            data[row * size + col] =
                gaf_db(key.betag, key.dbetag, key.b_over_c0, key.tint_over_lem, x, y);
        }
    }
    GafField { size, data }
}

/// Greyscale intensity image of the GAF: `GAF_DB_MIN` dB is black, 0 dB white.
/// Contours are *not* baked in — they are drawn as vector plot lines instead.
fn render_gaf_image(field: &GafField) -> egui::ColorImage {
    let mut rgb = vec![0u8; field.data.len() * 3];
    for (i, &db) in field.data.iter().enumerate() {
        let intensity = ((db - GAF_DB_MIN) / -GAF_DB_MIN).clamp(0.0, 1.0);
        let grey = (intensity * 255.0).round() as u8;
        rgb[i * 3] = grey;
        rgb[i * 3 + 1] = grey;
        rgb[i * 3 + 2] = grey;
    }
    egui::ColorImage::from_rgb([field.size, field.size], &rgb)
}

/// Extracts the iso-dB contours as polylines in ground metres, ready to be
/// drawn as `egui_plot` lines (grid column/row -> Easting/Northing).
fn gaf_contours(
    field: &GafField,
    key: &GafKey,
) -> Vec<(f64, egui::Color32, Vec<Vec<[f64; 2]>>)> {
    let step = 2.0 * key.half_extent_m / (field.size - 1) as f64;
    GAF_CONTOURS
        .iter()
        .map(|&(level, (r, g, b))| {
            let polylines = march(field, level)
                .into_iter()
                .map(|line| {
                    line.into_iter()
                        .map(|(col, row)| {
                            [
                                -key.half_extent_m + col * step, // Easting
                                key.half_extent_m - row * step,  // Northing (row 0 = +North)
                            ]
                        })
                        .collect()
                })
                .collect();
            (level, egui::Color32::from_rgb(r, g, b), polylines)
        })
        .collect()
}

/// Shows the GAF window while `open` is true, (re)building the cached texture
/// only when the geometry changed. `open` is the menu's toggle flag and is set
/// to `false` when the window's close button is clicked.
/// `bandwidth_hz`/`center_frequency_hz` are the Tx system parameters.
pub fn show_gaf_window(
    ctx: &egui::Context,
    open: &mut bool,
    gaf_state: &mut GafState,
    bsar_infos: &BsarInfos,
    bandwidth_hz: f64,
    center_frequency_hz: f64,
) {
    if !*open {
        return;
    }
    let key = gaf_key(bsar_infos, bandwidth_hz, center_frequency_hz);
    match key {
        Some(key) => {
            if gaf_state.cache_key != Some(key) {
                let field = compute_gaf_grid(&key);
                let image = render_gaf_image(&field);
                gaf_state.texture = Some(ctx.load_texture("gaf", image, egui::TextureOptions::LINEAR));
                gaf_state.contours = gaf_contours(&field, &key);
                gaf_state.cache_key = Some(key);
            }
        }
        None => {
            gaf_state.texture = None;
            gaf_state.contours.clear();
            gaf_state.cache_key = None;
        }
    }

    egui::Window::new("Generalized Ambiguity Function")
        .open(open)
        .resizable(true)
        .collapsible(true)
        .default_open(true)
        // No anchor: the window is freely draggable, opening centered the first
        // time and then remembering wherever the user leaves it.
        .default_pos(ctx.content_rect().center() - egui::vec2(240.0, 260.0))
        .show(ctx, |ui| {
            match (&gaf_state.texture, gaf_state.cache_key) {
                (Some(texture), Some(key)) => {
                    ui.label(
                        egui::RichText::new("Point-target response, 0 dB = white")
                            .color(egui::Color32::from_rgb(200, 200, 200)),
                    );
                    // egui_plot frames the image with metric axes and gives
                    // zoom/pan, a cursor coordinate readout and a legend that
                    // toggles the individual iso-dB contours.
                    let extent = 2.0 * key.half_extent_m;
                    egui_plot::Plot::new("gaf_plot")
                        .width(420.0)
                        .height(420.0)
                        .data_aspect(1.0) // Equal Easting/Northing scales
                        .x_axis_label("Easting [m]")
                        .y_axis_label("Northing [m]")
                        .legend(egui_plot::Legend::default())
                        .show(ui, |plot_ui| {
                            plot_ui.image(egui_plot::PlotImage::new(
                                "GAF intensity",
                                texture.id(),
                                egui_plot::PlotPoint::new(0.0, 0.0),
                                egui::vec2(extent as f32, extent as f32),
                            ));
                            // Vector iso-dB contours: crisp at any zoom level.
                            // All chunks of a level share its legend entry.
                            for (level, color, polylines) in &gaf_state.contours {
                                let name = format!("{level:.0} dB");
                                for polyline in polylines {
                                    plot_ui.line(
                                        egui_plot::Line::new(
                                            name.clone(),
                                            polyline.clone(),
                                        )
                                        .color(*color)
                                        .width(1.5_f32),
                                    );
                                }
                            }
                        });
                }
                _ => {
                    ui.label("Invalid geometry: the GAF requires non-zero Tx/Rx velocities.");
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_key() -> GafKey {
        // Non-degenerate bistatic geometry (mirrors bsar::tests reference)
        GafKey {
            betag: DVec3::new(0.3, 0.8, 0.0),
            dbetag: DVec3::new(0.01, -0.02, 0.0),
            b_over_c0: 300.0e6 / SPEED_OF_LIGHT_IN_VACUUM,
            tint_over_lem: 1.0 / 0.031,
            half_extent_m: 20.0,
        }
    }

    #[test]
    fn gaf_is_zero_db_at_origin() {
        let key = reference_key();
        let db = gaf_db(key.betag, key.dbetag, key.b_over_c0, key.tint_over_lem, 0.0, 0.0);
        assert!(db.abs() < 1e-9, "GAF at r=0 must be 0 dB, got {db}");
    }

    #[test]
    fn gaf_is_symmetric_about_origin() {
        let key = reference_key();
        for &(x, y) in &[(3.0, 1.0), (-2.5, 4.0), (7.0, -6.0)] {
            let a = gaf_db(key.betag, key.dbetag, key.b_over_c0, key.tint_over_lem, x, y);
            let b = gaf_db(key.betag, key.dbetag, key.b_over_c0, key.tint_over_lem, -x, -y);
            assert!((a - b).abs() < 1e-9, "GAF must be symmetric: {a} vs {b}");
        }
    }

    #[test]
    fn gaf_peak_is_the_maximum() {
        // No point should exceed the 0 dB main-lobe peak at the origin.
        let key = reference_key();
        let step = 2.0 * key.half_extent_m / 63.0;
        for row in 0..64 {
            for col in 0..64 {
                let x = -key.half_extent_m + step * col as f64;
                let y = -key.half_extent_m + step * row as f64;
                let db = gaf_db(key.betag, key.dbetag, key.b_over_c0, key.tint_over_lem, x, y);
                assert!(db <= 1e-9, "GAF exceeded 0 dB at ({x}, {y}): {db}");
            }
        }
    }

    #[test]
    fn degenerate_geometry_yields_no_key() {
        let mut infos = BsarInfos::default(); // all NaN
        assert!(gaf_key(&infos, 300.0e6, 9.65e9).is_none());
        // Valid bisectors but NaN integration time is still rejected
        infos.betag = DVec3::new(0.3, 0.8, 0.0);
        infos.dbetag = DVec3::new(0.01, -0.02, 0.0);
        assert!(gaf_key(&infos, 300.0e6, 9.65e9).is_none());
    }

    #[test]
    fn render_produces_bright_center_heatmap() {
        let key = reference_key();
        let image = render_gaf_image(&compute_gaf_grid(&key));
        assert_eq!(image.size, [GAF_RENDER_SIZE, GAF_RENDER_SIZE]);
        // Center pixel is the 0 dB peak -> white
        let center = GAF_RENDER_SIZE / 2;
        let center_px = image.pixels[center * GAF_RENDER_SIZE + center];
        assert!(center_px.r() > 240 && center_px.g() > 240 && center_px.b() > 240);
        // The heatmap is pure greyscale: contours are drawn as vector plot
        // lines, not baked into the texture.
        assert!(image
            .pixels
            .iter()
            .all(|p| p.r() == p.g() && p.g() == p.b()));
    }

    /// The vector contours must be closed-ish loops around the main lobe, in
    /// ground metres inside the plotted patch, so egui_plot can draw them
    /// directly on top of the heatmap.
    #[test]
    fn contours_are_extracted_in_ground_metres() {
        let key = reference_key();
        let contours = gaf_contours(&compute_gaf_grid(&key), &key);
        assert_eq!(contours.len(), GAF_CONTOURS.len());
        for (level, _color, polylines) in &contours {
            assert!(
                !polylines.is_empty(),
                "the {level} dB level must produce at least one contour"
            );
            for polyline in polylines {
                for &[x, y] in polyline {
                    assert!(
                        x.abs() <= key.half_extent_m + 1e-9
                            && y.abs() <= key.half_extent_m + 1e-9,
                        "contour point ({x}, {y}) outside the +/-{} m patch",
                        key.half_extent_m
                    );
                }
            }
        }
        // The -3 dB cell must be tighter than the -20 dB one
        let extent_of = |i: usize| {
            contours[i]
                .2
                .iter()
                .flatten()
                .fold(0.0f64, |max, &[x, y]| max.max(x.hypot(y)))
        };
        assert!(
            extent_of(0) < extent_of(2),
            "the -3 dB contour must be inside the -20 dB one"
        );
    }
}



