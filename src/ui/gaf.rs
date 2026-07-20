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
use crate::raster::{draw_polyline_bgrx, fill_bgrx};
use crate::textdraw::{draw_text_bgrx, text_width};

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
/// Reserved width of the plot's y-axis, in points (see `y_axis_min_width`).
const GAF_Y_AXIS_MIN_WIDTH: f32 = 40.0;
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
    /// Last logged frame signature, used by the `BSARGEOM_DEBUG_GAF`
    /// diagnostic to report only what actually changes between frames.
    debug_signature: Option<String>,
    debug_frame: u64,
    /// Plot bounds observed on the last frame, used by the diagnostic and by
    /// the layout-stability test.
    last_bounds: Option<([f64; 2], [f64; 2])>,
}

/// Whether the per-frame GAF diagnostic is enabled (`BSARGEOM_DEBUG_GAF=1`).
///
/// It logs a line only when something it watches changes, so a value that
/// alternates between frames — i.e. a flicker — stands out immediately while a
/// steady plot stays silent.
fn gaf_debug_enabled() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var("BSARGEOM_DEBUG_GAF").is_ok())
    }
    #[cfg(target_arch = "wasm32")]
    {
        false
    }
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
    compute_gaf_grid_sized(key, GAF_RENDER_SIZE)
}

/// Samples the GAF over the ground patch at an arbitrary resolution, so the
/// exported figure can be rendered far finer than the on-screen preview.
fn compute_gaf_grid_sized(key: &GafKey, size: usize) -> GafField {
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

// Exported figure layout. All lengths are given for the on-screen preview
// scale and multiplied by `EXPORT_SCALE`, so the saved figure is rendered (not
// upscaled) at print resolution: the GAF is analytic, so the patch and its
// contours are recomputed at the export resolution.
const EXPORT_SCALE: usize = 3;
const EXPORT_PATCH_PX: usize = GAF_RENDER_SIZE * EXPORT_SCALE;
const EXPORT_MARGIN_LEFT: usize = 108 * EXPORT_SCALE;
const EXPORT_MARGIN_RIGHT: usize = 34 * EXPORT_SCALE;
const EXPORT_MARGIN_TOP: usize = 46 * EXPORT_SCALE;
const EXPORT_MARGIN_BOTTOM: usize = 84 * EXPORT_SCALE;
const EXPORT_PAPER_RGB: (u8, u8, u8) = (255, 255, 255);
const EXPORT_INK_RGB: (u8, u8, u8) = (20, 20, 20);
const EXPORT_TITLE_PX: f32 = 21.0 * EXPORT_SCALE as f32;
const EXPORT_LABEL_PX: f32 = 17.0 * EXPORT_SCALE as f32;
const EXPORT_TICK_PX: f32 = 15.0 * EXPORT_SCALE as f32;
const EXPORT_TICK_LEN: f32 = 7.0 * EXPORT_SCALE as f32;
const EXPORT_HAIRLINE_PX: f32 = 1.5 * EXPORT_SCALE as f32;
/// Print resolution of the saved PNG, recorded in its `pHYs` chunk so print
/// and layout software reports the intended size rather than assuming 96 dpi.
const EXPORT_DPI: f64 = 300.0;

/// A "nice" tick step (1, 2 or 5 times a power of ten) close to `rough`.
fn nice_step(rough: f64) -> f64 {
    // Also rejects NaN, which would poison the log10 below
    if rough <= 0.0 || !rough.is_finite() {
        return 1.0;
    }
    let magnitude = 10f64.powf(rough.log10().floor());
    let normalized = rough / magnitude;
    let nice = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

/// Tick values covering `[-half_extent, +half_extent]` on a nice step.
fn axis_ticks(half_extent: f64) -> (Vec<f64>, usize) {
    let step = nice_step(2.0 * half_extent / 6.0);
    let decimals = if step >= 1.0 {
        0
    } else if step >= 0.1 {
        1
    } else {
        2
    };
    let first = (-half_extent / step).ceil() as i64;
    let last = (half_extent / step).floor() as i64;
    ((first..=last).map(|i| i as f64 * step).collect(), decimals)
}

/// Encodes an RGB buffer to PNG, tagging it with [`EXPORT_DPI`].
fn encode_png_with_dpi(rgb: &[u8], width: usize, height: usize) -> Option<Vec<u8>> {
    let mut png = Vec::new();
    let mut encoder = png::Encoder::new(&mut png, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    // pHYs is expressed in pixels per metre
    let pixels_per_metre = (EXPORT_DPI / 0.0254).round() as u32;
    encoder.set_pixel_dims(Some(png::PixelDimensions {
        xppu: pixels_per_metre,
        yppu: pixels_per_metre,
        unit: png::Unit::Meter,
    }));
    let mut writer = encoder.write_header().ok()?;
    writer.write_image_data(rgb).ok()?;
    drop(writer);
    Some(png)
}

/// Renders the plot as a standalone figure — the heatmap with its iso-dB
/// contours, framed by metric axes with ticks and a legend — at print
/// resolution, and encodes it to PNG.
fn gaf_png_bytes(key: &GafKey) -> Option<Vec<u8>> {
    let field = compute_gaf_grid_sized(key, EXPORT_PATCH_PX);
    let contours = gaf_contours(&field, key);
    let patch = field.size;
    let width = EXPORT_MARGIN_LEFT + patch + EXPORT_MARGIN_RIGHT;
    let height = EXPORT_MARGIN_TOP + patch + EXPORT_MARGIN_BOTTOM;
    let mut bgrx = vec![0u8; width * height * 4];
    fill_bgrx(&mut bgrx, EXPORT_PAPER_RGB);

    // Plot area: the greyscale intensity patch
    for row in 0..patch {
        for col in 0..patch {
            let db = field.data[row * patch + col];
            let intensity = ((db - GAF_DB_MIN) / -GAF_DB_MIN).clamp(0.0, 1.0);
            let grey = (intensity * 255.0).round() as u8;
            let index = ((row + EXPORT_MARGIN_TOP) * width + col + EXPORT_MARGIN_LEFT) * 4;
            bgrx[index] = grey;
            bgrx[index + 1] = grey;
            bgrx[index + 2] = grey;
        }
    }

    // Iso-dB contours, mapped from ground metres back into patch pixels
    let step = 2.0 * key.half_extent_m / (patch - 1) as f64;
    for (_level, color, polylines) in &contours {
        for polyline in polylines {
            let points: Vec<(f32, f32)> = polyline
                .iter()
                .map(|&[x, y]| {
                    (
                        ((x + key.half_extent_m) / step) as f32 + EXPORT_MARGIN_LEFT as f32,
                        ((key.half_extent_m - y) / step) as f32 + EXPORT_MARGIN_TOP as f32,
                    )
                })
                .collect();
            draw_polyline_bgrx(
                &mut bgrx,
                width,
                height,
                &points,
                GAF_EXPORT_STROKE_PX,
                (color.r(), color.g(), color.b()),
                None,
            );
        }
    }

    // Axis frame
    let (left, top) = (EXPORT_MARGIN_LEFT as f32, EXPORT_MARGIN_TOP as f32);
    let (right, bottom) = (left + patch as f32 - 1.0, top + patch as f32 - 1.0);
    let frame = [
        (left, top),
        (right, top),
        (right, bottom),
        (left, bottom),
        (left, top),
    ];
    draw_polyline_bgrx(&mut bgrx, width, height, &frame, EXPORT_HAIRLINE_PX, EXPORT_INK_RGB, None);

    // Ticks and their values
    let (ticks, decimals) = axis_ticks(key.half_extent_m);
    let to_pixels = |value: f64| ((value + key.half_extent_m) / step) as f32;
    for tick in ticks {
        let text = format!("{tick:.decimals$}");
        // X axis (bottom)
        let x = left + to_pixels(tick);
        draw_polyline_bgrx(
            &mut bgrx,
            width,
            height,
            &[(x, bottom), (x, bottom + EXPORT_TICK_LEN)],
            EXPORT_HAIRLINE_PX,
            EXPORT_INK_RGB,
            None,
        );
        draw_text_bgrx(
            &mut bgrx,
            width,
            height,
            (x, bottom + EXPORT_TICK_LEN + 12.0 * EXPORT_SCALE as f32),
            0.0,
            EXPORT_TICK_PX,
            EXPORT_INK_RGB,
            None,
            0.0,
            &text,
        );
        // Y axis (left); patch rows grow southwards, so mirror the value
        let y = top + to_pixels(-tick);
        draw_polyline_bgrx(
            &mut bgrx,
            width,
            height,
            &[(left - EXPORT_TICK_LEN, y), (left, y)],
            EXPORT_HAIRLINE_PX,
            EXPORT_INK_RGB,
            None,
        );
        draw_text_bgrx(
            &mut bgrx,
            width,
            height,
            (
                left - EXPORT_TICK_LEN - 6.0 * EXPORT_SCALE as f32
                    - 0.5 * text_width(&text, EXPORT_TICK_PX),
                y,
            ),
            0.0,
            EXPORT_TICK_PX,
            EXPORT_INK_RGB,
            None,
            0.0,
            &text,
        );
    }

    // Axis titles and figure title
    draw_text_bgrx(
        &mut bgrx,
        width,
        height,
        (0.5 * (left + right), height as f32 - 22.0 * EXPORT_SCALE as f32),
        0.0,
        EXPORT_LABEL_PX,
        EXPORT_INK_RGB,
        None,
        0.0,
        "Easting [m]",
    );
    draw_text_bgrx(
        &mut bgrx,
        width,
        height,
        (24.0 * EXPORT_SCALE as f32, 0.5 * (top + bottom)),
        -std::f32::consts::FRAC_PI_2,
        EXPORT_LABEL_PX,
        EXPORT_INK_RGB,
        None,
        0.0,
        "Northing [m]",
    );
    draw_text_bgrx(
        &mut bgrx,
        width,
        height,
        (0.5 * (left + right), 0.5 * top),
        0.0,
        EXPORT_TITLE_PX,
        EXPORT_INK_RGB,
        None,
        0.0,
        "Generalized Ambiguity Function [dB]",
    );

    // Legend, inside the plot area so it needs its own opaque background
    let entries: Vec<String> = contours
        .iter()
        .map(|(level, _, _)| format!("{level:.0} dB"))
        .collect();
    let text_widest = entries
        .iter()
        .fold(0.0f32, |widest, text| widest.max(text_width(text, EXPORT_TICK_PX)));
    let swatch = 26.0 * EXPORT_SCALE as f32;
    let padding = 8.0 * EXPORT_SCALE as f32;
    let row_height = EXPORT_TICK_PX + 7.0 * EXPORT_SCALE as f32;
    let box_width = padding * 3.0 + swatch + text_widest;
    let box_height = padding * 2.0 + row_height * entries.len() as f32;
    let box_left = right - 10.0 * EXPORT_SCALE as f32 - box_width;
    let box_top = top + 10.0 * EXPORT_SCALE as f32;
    for row in 0..box_height as usize {
        for col in 0..box_width as usize {
            let x = box_left as usize + col;
            let y = box_top as usize + row;
            if x < width && y < height {
                let index = (y * width + x) * 4;
                bgrx[index] = EXPORT_PAPER_RGB.2;
                bgrx[index + 1] = EXPORT_PAPER_RGB.1;
                bgrx[index + 2] = EXPORT_PAPER_RGB.0;
            }
        }
    }
    let legend_frame = [
        (box_left, box_top),
        (box_left + box_width, box_top),
        (box_left + box_width, box_top + box_height),
        (box_left, box_top + box_height),
        (box_left, box_top),
    ];
    draw_polyline_bgrx(&mut bgrx, width, height, &legend_frame, EXPORT_SCALE as f32, EXPORT_INK_RGB, None);
    for (row, ((_, color, _), text)) in contours.iter().zip(&entries).enumerate() {
        let y = box_top + padding + row_height * (row as f32 + 0.5);
        draw_polyline_bgrx(
            &mut bgrx,
            width,
            height,
            &[
                (box_left + padding, y),
                (box_left + padding + swatch, y),
            ],
            2.5 * EXPORT_SCALE as f32,
            (color.r(), color.g(), color.b()),
            None,
        );
        draw_text_bgrx(
            &mut bgrx,
            width,
            height,
            (
                box_left + padding * 2.0 + swatch + 0.5 * text_width(text, EXPORT_TICK_PX),
                y,
            ),
            0.0,
            EXPORT_TICK_PX,
            EXPORT_INK_RGB,
            None,
            0.0,
            text,
        );
    }

    let mut rgb = vec![0u8; width * height * 3];
    for (i, pixel) in bgrx.chunks_exact(4).enumerate() {
        rgb[i * 3] = pixel[2]; // R
        rgb[i * 3 + 1] = pixel[1]; // G
        rgb[i * 3 + 2] = pixel[0]; // B
    }
    encode_png_with_dpi(&rgb, width, height)
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
                                match gaf_png_bytes(&key) {
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
                    let mut observed_bounds: Option<([f64; 2], [f64; 2])> = None;
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
                        // Pin the y-axis width. egui_plot derives the axis
                        // thickness from the previous frame's tick labels, and
                        // with `data_aspect` that thickness feeds back into the
                        // bounds (bounds -> tick labels -> axis width -> plot
                        // width -> bounds). For some geometries the loop has no
                        // fixed point and the y-bounds oscillate between two
                        // values every frame, which is visible as flicker.
                        // A minimum wider than any tick label we produce keeps
                        // the thickness constant and breaks the loop.
                        .y_axis_min_width(GAF_Y_AXIS_MIN_WIDTH)
                        .show(ui, |plot_ui| {
                            let bounds = plot_ui.plot_bounds();
                            observed_bounds = Some((bounds.min(), bounds.max()));
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
                    gaf_state.last_bounds = observed_bounds;
                    if gaf_debug_enabled() {
                        gaf_state.debug_frame += 1;
                        let signature = format!(
                            "avail=({:.2},{:.2}) side={side:.2} bounds={observed_bounds:?} \
                             contours={} key={key:?}",
                            available.x,
                            available.y,
                            gaf_state.contours.iter().map(|(_, _, p)| p.len()).sum::<usize>(),
                        );
                        if gaf_state.debug_signature.as_deref() != Some(signature.as_str()) {
                            eprintln!("[gaf] frame {}: {signature}", gaf_state.debug_frame);
                            gaf_state.debug_signature = Some(signature);
                        }
                    }
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
    /// frame's axis thickness to fit tick labels), and `data_aspect` rescales
    /// the bounds each frame, so the two can feed back into each other. This
    /// asserts that a settled window with unchanged inputs renders
    /// byte-identically.
    ///
    /// Note: this headless harness never reproduced the reported flicker, so
    /// treat it as an invariant guard rather than proof of any fix. The
    /// `BSARGEOM_DEBUG_GAF=1` diagnostic is the tool that does catch it in the
    /// running app — it logs only when a watched value changes.
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
        let png = gaf_png_bytes(&key).expect("the GAF must encode to PNG");
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "not a PNG stream");

        let decoded = image::load_from_memory(&png)
            .expect("the exported PNG must decode")
            .to_rgb8();
        assert_eq!(
            decoded.dimensions(),
            (
                (EXPORT_MARGIN_LEFT + EXPORT_PATCH_PX + EXPORT_MARGIN_RIGHT) as u32,
                (EXPORT_MARGIN_TOP + EXPORT_PATCH_PX + EXPORT_MARGIN_BOTTOM) as u32
            ),
            "the export must be a framed figure, not a bare patch"
        );
        // The main lobe is white at the centre of the plot area ...
        let centre = decoded.get_pixel(
            (EXPORT_MARGIN_LEFT + EXPORT_PATCH_PX / 2) as u32,
            (EXPORT_MARGIN_TOP + EXPORT_PATCH_PX / 2) as u32,
        );
        assert!(centre.0.iter().all(|&c| c > 240), "centre should be the 0 dB peak");
        // ... the margins carry the axes/labels on a white ground ...
        let corner = decoded.get_pixel(4, 4);
        assert_eq!(corner.0, [255, 255, 255], "the figure margin must be paper-white");
        assert!(
            (0..EXPORT_MARGIN_LEFT as u32)
                .any(|x| (0..decoded.height()).any(|y| decoded.get_pixel(x, y).0[0] < 128)),
            "the left margin must carry the Northing title and the y tick labels"
        );
        // ... and the bottom margin the Easting title / x tick labels
        assert!(
            ((EXPORT_MARGIN_TOP + EXPORT_PATCH_PX) as u32..decoded.height())
                .any(|y| (0..decoded.width()).any(|x| decoded.get_pixel(x, y).0[0] < 128)),
            "the bottom margin must carry the Easting title and the x tick labels"
        );
        // The figure must be tagged with its print resolution, otherwise
        // layout software assumes ~96 dpi and prints it far too large.
        let decoder = png::Decoder::new(std::io::Cursor::new(&png));
        let reader = decoder.read_info().expect("the PNG header must parse");
        let dimensions = reader
            .info()
            .pixel_dims
            .expect("the PNG must carry a pHYs chunk");
        let expected = (EXPORT_DPI / 0.0254).round() as u32;
        assert_eq!(dimensions.xppu, expected);
        assert_eq!(dimensions.yppu, expected);
        assert!(matches!(dimensions.unit, png::Unit::Meter));
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




    /// Builds the exact GAF state captured in a user's flicker report: the
    /// default monostatic configuration (10 GHz, 800 MHz, Tx elevation -30 deg
    /// so |betag| = 2cos30, integration time 3.4641 s).
    fn reported_flicker_infos() -> BsarInfos {
        BsarInfos {
            betag: DVec3::new(1.7320508075688776, 1.85743932176809e-16, 0.0),
            dbetag: DVec3::new(4.4014171702121074e-18, -0.04, 0.0),
            integration_time_s: 3.4641016151377553,
            ground_range_resolution_m: 0.1916687585464135,
            ground_lateral_resolution_m: 0.1916687585464135,
            ..Default::default()
        }
    }

    /// The plot layout must settle: with unchanged inputs the bounds must stop
    /// changing. egui_plot re-derives the axis thickness from the previous
    /// frame, so with `data_aspect` the y-bounds can enter a period-2
    /// oscillation (bounds -> y tick labels -> axis width -> plot width ->
    /// bounds), which is visible as flicker.
    #[test]
    fn gaf_plot_layout_settles_for_the_reported_flicker_geometry() {
        let infos = reported_flicker_infos();
        assert!(gaf_key(&infos, 800.0e6, 10.0e9).is_some());
        let ctx = egui::Context::default();
        let mut gaf_state = GafState::default();
        let mut open = true;
        let mut seen = Vec::new();
        for frame in 0..24 {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(1600.0, 1000.0),
                )),
                ..Default::default()
            };
            let _ = ctx.run_ui(input, |ui| {
                show_gaf_window(ui.ctx(), &mut open, &mut gaf_state, &infos, 800.0e6, 10.0e9);
            });
            if frame >= 8 {
                seen.push(format!("{:?}", gaf_state.last_bounds));
            }
        }
        seen.dedup();
        assert_eq!(
            seen.len(),
            1,
            "the plot bounds never settle, they cycle through {} states: {:#?}",
            seen.len(),
            seen
        );
    }



}
