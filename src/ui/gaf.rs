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
use crate::contour::{march_levels, Field};
use crate::download::SaveRequest;
use crate::raster::draw_polyline_bgrx;

/// Ground patch resolution (pixels per side) of the rendered GAF image.
const GAF_RENDER_SIZE: usize = 400;
/// Displayed dynamic range: `GAF_DB_MIN` dB maps to black, 0 dB to white.
const GAF_DB_MIN: f64 = -30.0;
/// Fallback half-extent [m] when a resolution is degenerate (matches BSARConf).
const GAF_FALLBACK_HALF_EXTENT_M: f64 = 1000.0;
/// Overlaid iso-level contours [dB] with their (R, G, B) colors, on a warm
/// ramp from red (main lobe) to pale yellow (far sidelobes).
const GAF_CONTOURS: [(f64, (u8, u8, u8)); 5] = [
    (-3.0, (255, 60, 60)),   // half-power resolution cell
    (-6.0, (255, 105, 55)),  // half-amplitude
    (-10.0, (255, 150, 60)),
    (-13.0, (255, 182, 74)),
    (-20.0, (255, 210, 90)),
];
/// Default/minimum side of the GAF window's square plot area, in points.
const GAF_WINDOW_SIDE: f32 = 460.0;
const GAF_PLOT_MIN_SIDE: f32 = 180.0;
/// Contour stroke used when baking the plot into an exported PNG.
const GAF_EXPORT_STROKE_PX: f32 = 1.6;
/// File name used when saving/downloading the GAF image.
const GAF_EXPORT_FILE_NAME: &str = "bsargeom_gaf.png";

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
    /// Result of the last "save image" click, shown under the plot.
    save_status: Option<String>,
    /// Save in flight (native: the "save as" dialog; web: resolves at once).
    save_request: Option<SaveRequest>,
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
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct GafKey {
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
pub(crate) fn gaf_key(bsar_infos: &BsarInfos, bandwidth_hz: f64, center_frequency_hz: f64) -> Option<GafKey> {
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
    // All levels in a single pass over the grid. `march_levels` keeps the
    // caller's ordering, so the descending GAF_CONTOURS order is preserved.
    let levels: Vec<f64> = GAF_CONTOURS.iter().map(|&(level, _)| level).collect();
    march_levels(field, &levels)
        .into_iter()
        .zip(GAF_CONTOURS)
        .map(|(contours, (level, (r, g, b)))| {
            let polylines = contours
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

/// Renders the plot content (greyscale heatmap with the iso-dB contours drawn
/// in) to PNG bytes, so what is saved matches what is displayed.
fn gaf_png_bytes(
    key: &GafKey,
    contours: &[(f64, egui::Color32, Vec<Vec<[f64; 2]>>)],
) -> Option<Vec<u8>> {
    use image::ImageEncoder as _;

    let field = compute_gaf_grid(key);
    let size = field.size;
    // Compose in BGRX so the shared anti-aliased rasterizer can be reused
    let mut bgrx = vec![0u8; size * size * 4];
    for (i, &db) in field.data.iter().enumerate() {
        let intensity = ((db - GAF_DB_MIN) / -GAF_DB_MIN).clamp(0.0, 1.0);
        let grey = (intensity * 255.0).round() as u8;
        bgrx[i * 4] = grey;
        bgrx[i * 4 + 1] = grey;
        bgrx[i * 4 + 2] = grey;
    }
    let step = 2.0 * key.half_extent_m / (size - 1) as f64;
    for (_level, color, polylines) in contours {
        for polyline in polylines {
            let points: Vec<(f32, f32)> = polyline
                .iter()
                .map(|&[x, y]| {
                    (
                        ((x + key.half_extent_m) / step) as f32,
                        ((key.half_extent_m - y) / step) as f32,
                    )
                })
                .collect();
            draw_polyline_bgrx(
                &mut bgrx,
                size,
                size,
                &points,
                GAF_EXPORT_STROKE_PX,
                (color.r(), color.g(), color.b()),
                None,
            );
        }
    }
    let mut rgb = vec![0u8; size * size * 3];
    for (i, pixel) in bgrx.chunks_exact(4).enumerate() {
        rgb[i * 3] = pixel[2]; // R
        rgb[i * 3 + 1] = pixel[1]; // G
        rgb[i * 3 + 2] = pixel[0]; // B
    }
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&rgb, size as u32, size as u32, image::ExtendedColorType::Rgb8)
        .ok()?;
    Some(png)
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
    // Drive an in-flight save first: on native its dialog is a window of its
    // own, so it must keep running even if the GAF window was closed meanwhile.
    if let Some(request) = &mut gaf_state.save_request
        && let Some(status) = request.update(ctx)
    {
        gaf_state.save_status = Some(status);
        gaf_state.save_request = None;
    }

    if !*open {
        return;
    }
    let key = gaf_key(bsar_infos, bandwidth_hz, center_frequency_hz);
    // Refit the view only when the plotted extent actually changes, so the
    // bounds stay fixed from frame to frame (see `refit_bounds` below).
    let mut refit_bounds = false;
    match key {
        Some(key) => {
            if gaf_state.cache_key != Some(key) {
                let field = compute_gaf_grid(&key);
                let image = render_gaf_image(&field);
                gaf_state.texture = Some(ctx.load_texture("gaf", image, egui::TextureOptions::LINEAR));
                gaf_state.contours = gaf_contours(&field, &key);
                refit_bounds = gaf_state
                    .cache_key
                    .is_none_or(|previous| previous.half_extent_m != key.half_extent_m);
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
        .default_size(egui::vec2(GAF_WINDOW_SIDE, GAF_WINDOW_SIDE + 40.0))
        .min_size(egui::vec2(GAF_PLOT_MIN_SIDE, GAF_PLOT_MIN_SIDE))
        // No anchor: the window is freely draggable, opening centered the first
        // time and then remembering wherever the user leaves it.
        .default_pos(
            ctx.content_rect().center()
                - 0.5 * egui::vec2(GAF_WINDOW_SIDE, GAF_WINDOW_SIDE + 40.0),
        )
        .show(ctx, |ui| {
            match (&gaf_state.texture, gaf_state.cache_key) {
                (Some(texture), Some(key)) => {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Point-target response, 0 dB = white")
                                .color(egui::Color32::from_rgb(200, 200, 200)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let hover = egui::RichText::new(
                                "Saves the plotted image as a PNG (downloaded by the browser on the web build)",
                            )
                            .color(egui::Color32::from_rgb(200, 200, 200))
                            .monospace();
                            let saving = gaf_state.save_request.is_some();
                            if ui
                                .add_enabled(!saving, egui::Button::new("Save image"))
                                .on_hover_text(hover)
                                .clicked()
                            {
                                match gaf_png_bytes(&key, &gaf_state.contours) {
                                    Some(png) => {
                                        gaf_state.save_status = None;
                                        gaf_state.save_request =
                                            Some(SaveRequest::new(GAF_EXPORT_FILE_NAME, png));
                                    }
                                    None => {
                                        gaf_state.save_status =
                                            Some("Save failed: could not encode the image".to_string());
                                    }
                                }
                            }
                        });
                    });

                    // egui_plot frames the image with metric axes and gives
                    // zoom/pan, a cursor coordinate readout and a legend that
                    // toggles the individual iso-dB contours.
                    let extent = 2.0 * key.half_extent_m;
                    // Square plot filling whatever the (resizable) window
                    // leaves after the caption, so dragging the window corner
                    // actually grows the plot.
                    let available = ui.available_size();
                    let side = available.x.min(available.y).max(GAF_PLOT_MIN_SIDE);
                    egui_plot::Plot::new("gaf_plot")
                        .width(side)
                        .height(side)
                        .data_aspect(1.0) // Equal Easting/Northing scales
                        .x_axis_label("Easting [m]")
                        .y_axis_label("Northing [m]")
                        // Insertion order keeps the legend listed from the
                        // highest level down (egui_plot sorts alphabetically by
                        // default, which would read -10, -13, -20, -3, -6).
                        .legend(
                            egui_plot::Legend::default().follow_insertion_order(true),
                        )
                        // Fixed bounds rather than an automatic fit: egui_plot
                        // carries layout state between frames (it remembers the
                        // previous axis thickness to fit tick labels), so with
                        // `data_aspect` an automatic fit can depend on the
                        // previous frame. Pinning the bounds keeps the view
                        // reproducible frame to frame; `refit_bounds` re-fits it
                        // whenever the plotted patch changes size.
                        .auto_bounds(egui::Vec2b::FALSE)
                        .show(ui, |plot_ui| {
                            // Fit the patch when it changes; the user keeps
                            // control of zoom/pan the rest of the time.
                            if refit_bounds {
                                plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                                    [-key.half_extent_m, -key.half_extent_m],
                                    [key.half_extent_m, key.half_extent_m],
                                ));
                            }
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
                    if let Some(status) = &gaf_state.save_status {
                        ui.label(
                            egui::RichText::new(status)
                                .small()
                                .color(egui::Color32::from_rgb(160, 160, 160)),
                        );
                    }
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
        // Levels are ordered from the main lobe outwards, so each contour must
        // enclose the previous one (checked on the farthest reach of each).
        let extent_of = |i: usize| {
            contours[i]
                .2
                .iter()
                .flatten()
                .fold(0.0f64, |max, &[x, y]| max.max(x.hypot(y)))
        };
        for i in 1..contours.len() {
            assert!(
                extent_of(i - 1) < extent_of(i),
                "the {} dB contour must lie inside the {} dB one",
                contours[i - 1].0,
                contours[i].0
            );
        }
    }

    /// Frame-to-frame determinism guard for the GAF window.
    ///
    /// egui_plot's layout carries state across frames (it stores the previous
    /// frame's axis thickness to fit tick labels), so with `data_aspect` an
    /// automatic fit can feed back: bounds -> tick digits -> axis width ->
    /// aspect correction -> bounds. Fixed bounds break that loop; this asserts
    /// that a settled window with unchanged inputs renders byte-identically.
    ///
    /// Note: this does *not* reproduce the monostatic flicker that was reported
    /// — that was never reproduced headlessly, so treat this as an invariant
    /// guard rather than proof of a fix.
    #[test]
    fn monostatic_gaf_renders_identically_across_frames() {
        use crate::entities::AntennaBeamFootprintState;

        // Monostatic: the receiver sits exactly on the transmitter
        let position = DVec3::new(0.0, -8000.0, 6000.0);
        let velocity = DVec3::new(120.0, 0.0, 0.0);
        let mut infos = BsarInfos::default();
        infos.update(
            &(-position), &velocity, &(-position), &velocity,
            &AntennaBeamFootprintState::default(),
            &AntennaBeamFootprintState::default(),
            9.65e9, 300.0e6, 1.0, true, true,
        );
        assert!(
            gaf_key(&infos, 300.0e6, 9.65e9).is_some(),
            "the monostatic reference geometry must produce a GAF"
        );

        let ctx = egui::Context::default();
        let mut gaf_state = GafState::default();
        let mut frame = || {
            let mut open = true;
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(1600.0, 1000.0),
                )),
                ..Default::default()
            };
            let output = ctx.run_ui(input, |ui| {
                show_gaf_window(ui.ctx(), &mut open, &mut gaf_state, &infos, 300.0e6, 9.65e9);
            });
            format!("{:?}", output.shapes)
        };
        // Let fonts, window animation and the first layout settle
        for _ in 0..8 {
            frame();
        }
        let settled = frame();
        for round in 0..4 {
            assert_eq!(
                frame(),
                settled,
                "GAF plot output changed on settled frame {round}: the layout oscillates (flicker)"
            );
        }
    }


    /// The exported PNG must be a real, decodable image of the plotted patch,
    /// carrying the contour colors that are only overlaid in the live plot.
    #[test]
    fn gaf_png_export_is_a_decodable_image_with_contours() {
        let key = reference_key();
        let field = compute_gaf_grid(&key);
        let contours = gaf_contours(&field, &key);
        let png = gaf_png_bytes(&key, &contours).expect("the GAF must encode to PNG");
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "not a PNG stream");

        let decoded = image::load_from_memory(&png)
            .expect("the exported PNG must decode")
            .to_rgb8();
        assert_eq!(
            decoded.dimensions(),
            (GAF_RENDER_SIZE as u32, GAF_RENDER_SIZE as u32)
        );
        // The main lobe is white at the centre ...
        let centre = decoded.get_pixel(GAF_RENDER_SIZE as u32 / 2, GAF_RENDER_SIZE as u32 / 2);
        assert!(centre.0.iter().all(|&c| c > 240), "centre should be the 0 dB peak");
        // ... and the baked-in contours add non-grey (colored) pixels, unlike
        // the purely greyscale live heatmap texture.
        assert!(
            decoded.pixels().any(|p| p.0[0] != p.0[1] || p.0[1] != p.0[2]),
            "contours must be baked into the exported image"
        );
    }

    /// A freshly started save must stay pending while its dialog is on screen
    /// instead of resolving (or vanishing) on the first frame.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_save_request_stays_pending_while_the_dialog_is_open() {
        let ctx = egui::Context::default();
        let mut request = SaveRequest::new("bsargeom_gaf.png", b"\x89PNG\r\n\x1a\n".to_vec());
        for frame in 0..3 {
            let mut outcome = None;
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                outcome = request.update(ui.ctx());
            });
            assert!(
                outcome.is_none(),
                "the save resolved on frame {frame} without the user choosing a path"
            );
        }
    }

}
