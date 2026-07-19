use bevy::{
    asset::RenderAssetUsages,
    ecs::query::QueryFilter,
    math::DVec3,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat}
};
use plotters::{
    backend::{BitMapBackend, BGRXPixel},
    chart::ChartBuilder,
    drawing::IntoDrawingArea,
    series::{LineSeries, DashedLineSeries},
    style::{RGBAColor, ShapeStyle}
};

use crate::{
    bsar::{SPEED_OF_LIGHT_IN_VACUUM, bistatic_range_sg, doppler_frequency_sg},
    contour::{march, Field},
    constants::HALF_PLANE_LENGTH,
    entities::AntennaBeamFootprintState,
    scene::{IsoRangeDopplerPlane, TxCarrierState, RxCarrierState},
    textdraw::{draw_text_bgrx, text_width},
};

const MAX_PLANE_LENGTH: f64 = 2.0 * HALF_PLANE_LENGTH as f64;
const TEXTURE_WIDTH: usize  = 2048;
const TEXTURE_HEIGHT: usize = 2048;
const GRID_SIZE: usize = 251;
const NLEVELS: usize = 50;
// Colors for the IsoRange and IsoDoppler
const GROUND_GREY: RGBAColor = RGBAColor(128, 128, 128, 1.0);
const ISO_RANGE_RED: RGBAColor = RGBAColor(214, 39, 40, 1.0);
const ISO_DOPPLER_BLUE: RGBAColor = RGBAColor(31, 119, 180, 1.0);
// IsoRange style
const ISO_RANGE_STYLE: ShapeStyle = ShapeStyle {
    color: ISO_RANGE_RED,
    filled: false,
    stroke_width: 6,
};
// IsoDoppler style
const ISO_DOPPLER_STYLE: ShapeStyle = ShapeStyle {
    color: ISO_DOPPLER_BLUE,
    filled: false,
    stroke_width: 6,
};
// Contour value labels: ~45 px on the 2048² texture matches the ~12 px labels
// of BSARConf's ~500 px plotly plot; tiny chunks are left unlabeled.
const LABEL_FONT_SIZE: f32 = 45.0;
const LABEL_MIN_CHUNK_POINTS: usize = 8;
// Minimum spacing between two labels of the same family, in texture pixels.
const LABEL_MIN_SPACING_PX: f32 = 220.0;
// Label colors as (R, G, B), matching the ISO_RANGE_RED/ISO_DOPPLER_BLUE lines.
const ISO_RANGE_LABEL_RGB: (u8, u8, u8) = (214, 39, 40);
const ISO_DOPPLER_LABEL_RGB: (u8, u8, u8) = (31, 119, 180);

/// A pending contour label: value text at a grid-coordinate anchor, drawn into
/// the pixel buffer after the plotters drawing area is released.
struct Label {
    text: String,
    anchor: (f64, f64), // grid coordinates (0..GRID_SIZE-1)
    color: (u8, u8, u8),
}

/// Contour label formatter with the unit chosen once per contour family (from
/// its largest absolute level), so all labels of a family share the same unit.
fn label_formatter(levels: &[f64], base_unit: &'static str, kilo_unit: &'static str) -> impl Fn(f64) -> String {
    let max_abs = levels.iter().fold(0.0f64, |max, level| max.max(level.abs()));
    let kilo = max_abs >= 10_000.0;
    move |level: f64| {
        if kilo {
            format!("{:.1} {}", level / 1000.0, kilo_unit)
        } else {
            format!("{:.0} {}", level, base_unit)
        }
    }
}

pub fn spawn_iso_range_doppler_plane(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
) -> (Entity, Handle<Image>) {
    // Create the image texture for the plane
    let image_handle = images.add(Image::new_fill(
        Extent3d {
            width: TEXTURE_WIDTH as u32,
            height: TEXTURE_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0], // Initial color (black)
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD
    ));

    // Create the plane mesh
    let plane = Plane3d::new(Vec3::Y, Vec2::splat(0.5));
    // Create the material for the plane
    let material = StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(image_handle.clone()),
        cull_mode: None,
        unlit: true,
        ..Default::default()
    };

    let id = commands.spawn((
        Mesh3d(meshes.add(plane)),
        MeshMaterial3d(materials.add(material)),
    )).id();
    
    (id, image_handle)
}

/// Updates the IsoRangeDopplerPlaneState texture and returns the transform for the plane.
pub fn iso_range_doppler_plane_transform_from_state(
    tx_carrier_state: &TxCarrierState,
    rx_carrier_state: &RxCarrierState,
    tx_antenna_beam_footprint_state: &AntennaBeamFootprintState,
    rx_antenna_beam_footprint_state: &AntennaBeamFootprintState,
    image: &mut Image,
    iso_range_doppler_plane_state: &mut IsoRangeDopplerPlaneState,
) -> Result<Transform, Box<dyn std::error::Error>> {
    let lem = SPEED_OF_LIGHT_IN_VACUUM /
        (tx_carrier_state.center_frequency_ghz * 1e9); // wavelength λ [m] (= c/f, consistent with bsar.rs)
    let extent = f64::min(
        MAX_PLANE_LENGTH,
        2.1 * tx_antenna_beam_footprint_state.ground_max_extent_m.max(
            rx_antenna_beam_footprint_state.ground_max_extent_m
        )
    );
    // Update the texture of the IsoRangeDopplerPlaneState
    iso_range_doppler_plane_state.update_texture(
        &tx_carrier_state.inner.position_m, // OT in world frame
        &tx_carrier_state.inner.velocity_vector_mps, // VT in world frame
        &rx_carrier_state.inner.position_m, // OR in world frame
        &rx_carrier_state.inner.velocity_vector_mps, // VR in world frame
        lem, extent,
        image
    )?;
    // Update the transform of the IsoRangeDopplerPlaneState
    let transform = Transform {
        translation: Vec3::new(0.0, 0.1, 0.0), // Slightly above the ground
        rotation: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2), // Rotate 90 degrees around Y-axis
        scale: Vec3::new(extent as f32, 1.0, extent as f32),
    };

    Ok(transform)
}

/// Recomputes the iso-range/iso-Doppler plane texture and transform from the
/// current Tx/Rx states. Shared by the Tx and Rx panel update systems; generic
/// over the transform-query filter since each system disambiguates its
/// `&mut Transform` queries with its own `Without<...>` chain.
pub fn refresh_iso_range_doppler_plane<F: QueryFilter>(
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    tx_carrier_state: &TxCarrierState,
    rx_carrier_state: &RxCarrierState,
    tx_antenna_beam_footprint_state: &AntennaBeamFootprintState,
    rx_antenna_beam_footprint_state: &AntennaBeamFootprintState,
    iso_range_doppler_plane_state: &mut IsoRangeDopplerPlaneState,
    iso_range_doppler_q: &mut Query<&mut Transform, F>,
    iso_range_doppler_material_q: &Query<&MeshMaterial3d<StandardMaterial>, With<IsoRangeDopplerPlane>>,
) {
    for mut iso_range_doppler_plane_transform in iso_range_doppler_q.iter_mut() {
        for material_handle in iso_range_doppler_material_q.iter() {
            if let Some(mut material) = materials.get_mut(material_handle)
                && let Some(ref image_handle) = material.base_color_texture {
                    if let Some(mut image) = images.get_mut(image_handle)
                        && let Ok(transform) = iso_range_doppler_plane_transform_from_state(
                            tx_carrier_state,
                            rx_carrier_state,
                            tx_antenna_beam_footprint_state,
                            rx_antenna_beam_footprint_state,
                            &mut image,
                            iso_range_doppler_plane_state
                        ) {
                            // Update iso-range doppler plane transform
                            *iso_range_doppler_plane_transform = transform;
                        };
                    // Update iso-range doppler plane texture with newly calculated image
                    material.base_color_texture = Some(image_handle.clone());
                }
        }
    }
}

#[derive(Resource)]
pub struct IsoRangeDopplerPlaneState {
    iso_range: IsoRange,
    iso_doppler: IsoDoppler,
}

impl Default for IsoRangeDopplerPlaneState {
    fn default() -> Self {
        Self {
            iso_range: IsoRange::new(
                &DVec3::ZERO,
                &DVec3::ZERO,
                1000.0,
                GRID_SIZE,
                GRID_SIZE
            ),
            iso_doppler: IsoDoppler::new(
                &DVec3::ZERO, &DVec3::ONE,
                &DVec3::ZERO, &DVec3::ONE,
                0.3, 1000.0,
                GRID_SIZE,
                GRID_SIZE
            ),
        }
    }
}

impl IsoRangeDopplerPlaneState {
    fn update_texture(
        &mut self,
        ot: &DVec3,
        vt: &DVec3,
        or: &DVec3,
        vr: &DVec3,
        lem: f64,
        extent: f64,
        image: &mut Image
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Update iso-range data
        self.iso_range.update_data(
            ot, or, extent
        );
        // Update iso-doppler data
        self.iso_doppler.update_data(
            ot, vt, or, vr, lem, extent
        );
        // Compute the levels for iso-range and iso-doppler
        let iso_range_levels = self.iso_range.levels(NLEVELS);
        let iso_doppler_levels = self.iso_doppler.levels(NLEVELS);
        // Value labels: adaptive unit per family, one label per level
        let format_range = label_formatter(&iso_range_levels, "m", "km");
        let format_doppler = label_formatter(&iso_doppler_levels, "Hz", "kHz");
        //
        if let Some(ref mut bytes) = image.data {
            let mut labels: Vec<Label> = Vec::new();
            // Draw the contour lines with plotters; the drawing area borrows
            // `bytes`, so it is scoped and dropped before the labels (which
            // re-borrow `bytes`) are rasterized.
            {
                let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
                    bytes,
                    (TEXTURE_WIDTH as u32, TEXTURE_HEIGHT as u32)
                )?.into_drawing_area();
                root.fill(&GROUND_GREY)?;

                let mut chart = ChartBuilder::on(&root)
                    .build_cartesian_2d(
                        0.0..(GRID_SIZE-1) as f64,
                        (GRID_SIZE-1) as f64..0.0 // Invert Y
                    )?;
                // Iso-range
                for &level in &iso_range_levels {
                    let mut longest_chunk: Vec<(f64, f64)> = Vec::new();
                    for line in march(&self.iso_range, level) { // Compute contours
                        if line.len() > longest_chunk.len() {
                            longest_chunk = line.clone();
                        }
                        chart.draw_series(
                            LineSeries::new(line, ISO_RANGE_STYLE) // here Contours are the same type as Coord for plotters
                        )?;
                    }
                    // One value label per level, on its longest contour chunk
                    if longest_chunk.len() >= LABEL_MIN_CHUNK_POINTS {
                        labels.push(Label {
                            text: format_range(level),
                            anchor: longest_chunk[longest_chunk.len() / 2],
                            color: ISO_RANGE_LABEL_RGB,
                        });
                    }
                }
                // Iso-doppler
                for &level in &iso_doppler_levels {
                    let mut longest_chunk: Vec<(f64, f64)> = Vec::new();
                    for line in march(&self.iso_doppler, level) { // Compute contours
                        if line.len() > longest_chunk.len() {
                            longest_chunk = line.clone();
                        }
                        if level >= 0.0 {
                            chart.draw_series(
                                LineSeries::new(line, ISO_DOPPLER_STYLE)
                            )?;
                        } else {
                            chart.draw_series(
                                DashedLineSeries::new(line, 6, 10, ISO_DOPPLER_STYLE)
                            )?;
                        }
                    }
                    // One value label per level, on its longest contour chunk
                    if longest_chunk.len() >= LABEL_MIN_CHUNK_POINTS {
                        labels.push(Label {
                            text: format_doppler(level),
                            anchor: longest_chunk[longest_chunk.len() / 2],
                            color: ISO_DOPPLER_LABEL_RGB,
                        });
                    }
                }
            }
            // Rasterize the labels directly into the pixel buffer, centered on
            // their anchor. Grid coords map linearly to the whole texture: the
            // chart uses the full drawing area and its reversed y-range
            // (`(GRID_SIZE-1)..0.0`) puts grid row 0 at the top of the image,
            // so the row index maps directly to the pixel row (no flip — an
            // inverted mapping here silently moves every label onto the
            // vertically mirrored contour, i.e. the opposite Doppler sign).
            // To keep the map readable (50 levels/family), a label is skipped
            // when it lands too close to one already placed in the same family
            // (decluttering, like plotly's `showlabels`).
            let sx = (TEXTURE_WIDTH - 1) as f64 / (GRID_SIZE - 1) as f64;
            let sy = (TEXTURE_HEIGHT - 1) as f64 / (GRID_SIZE - 1) as f64;
            let mut placed: Vec<(f32, f32, (u8, u8, u8))> = Vec::new();
            for label in &labels {
                let px = (label.anchor.0 * sx) as f32;
                let py = (label.anchor.1 * sy) as f32;
                let too_close = placed.iter().any(|&(ox, oy, color)| {
                    color == label.color
                        && (px - ox).hypot(py - oy) < LABEL_MIN_SPACING_PX
                });
                if too_close {
                    continue;
                }
                placed.push((px, py, label.color));
                let half_width = 0.5 * text_width(&label.text, LABEL_FONT_SIZE);
                draw_text_bgrx(
                    bytes,
                    TEXTURE_WIDTH,
                    TEXTURE_HEIGHT,
                    (px - half_width, py - 0.5 * LABEL_FONT_SIZE),
                    LABEL_FONT_SIZE,
                    label.color,
                    &label.text,
                );
            }
        }

        Ok(())
    }
}

struct IsoRange {
    width: usize,
    height: usize,
    min: f64,
    max: f64,    
    data: Vec<f64>,
}

impl IsoRange {
    pub fn new(
        ot: &DVec3,
        or: &DVec3,
        extent: f64,
        width: usize,
        height: usize
    ) -> Self {
        let mut iso_range = Self {
            width,
            height,
            min: f64::MAX,
            max: 0.0,
            data: vec![0.0f64; width * height],
        };
        iso_range.update_data(ot, or, extent);
        iso_range
    }

    pub fn update_data(
        &mut self,
        ot: &DVec3,
        or: &DVec3,
        extent: f64
    ) {
        // Axes parameters
        let ystart = 0.5 * extent; // Top-left corner
        let xstart = -ystart;
        let dx =  extent / (self.width - 1) as f64;
        let dy = -extent / (self.height - 1) as f64;
        // X and Y axes
        let xaxis = (0..self.width).into_iter()
            .map(|j| xstart + j as f64 * dx)
            .collect::<Vec<f64>>();
        let yaxis = (0..self.height).into_iter()
            .map(|i| ystart + i as f64 * dy)
            .collect::<Vec<f64>>();
        //
        self.min = f64::MAX;
        self.max = 0.0;
        // Temporary variables
        let mut op = DVec3::ZERO;
        let mut tmp: f64;
        for (i, y) in yaxis.iter().enumerate() {
            for (j, x) in xaxis.iter().enumerate() {
                op.x = *x;
                op.y = *y;
                tmp = bistatic_range_sg(&(op - ot), &(op - or));
                if tmp < self.min {
                    self.min = tmp;
                }
                if tmp > self.max {
                    self.max = tmp;
                }
                // Compute bistatic range
                self.data[i * self.width + j] = tmp;
            }
        }
    }

    pub fn levels(&self, nlevels: usize) -> Vec<f64> {
        let min = self.min.ceil(); // Round to meter up
        let max = self.max.floor(); // Round to meter down
        let dv = (max - min) / (nlevels - 1) as f64;
        (0..nlevels).into_iter().map(|i| {
            min + dv * i as f64
        }).collect()
    }
}

impl Field for IsoRange {
    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn z_at(&self, x: usize, y: usize) -> f64 {
        self.data[y * self.width + x] // y -> i, x -> j
    }
}


struct IsoDoppler {
    width: usize,
    height: usize,
    min: f64,
    max: f64,    
    data: Vec<f64>,
}

impl IsoDoppler {
    pub fn new(        
        ot: &DVec3,
        vt: &DVec3,
        or: &DVec3,
        vr: &DVec3,
        lem: f64,
        extent: f64,
        width: usize,
        height: usize
    ) -> Self {
        let mut iso_range = Self {
            width,
            height,
            min: f64::MAX,
            max: f64::MIN,
            data: vec![0.0f64; width * height],
        };
        iso_range.update_data(
            ot, vt, or, vr, lem, extent
        );
        iso_range
    }

    pub fn update_data(
        &mut self,
        ot: &DVec3,
        vt: &DVec3,
        or: &DVec3,
        vr: &DVec3,
        lem: f64,
        extent: f64
    ) {
        // Axes parameters
        let ystart = 0.5 * extent; // Top-left corner
        let xstart = -ystart;
        let dx =  extent / (self.width - 1) as f64;
        let dy = -extent / (self.height - 1) as f64;
        // X and Y axes
        let xaxis = (0..self.width).into_iter()
            .map(|j| xstart + j as f64 * dx)
            .collect::<Vec<f64>>();
        let yaxis = (0..self.height).into_iter()
            .map(|i| ystart + i as f64 * dy)
            .collect::<Vec<f64>>();
        //
        self.min = f64::MAX;
        self.max = -f64::MAX;
        // Temporary variables
        let mut op = DVec3::ZERO;
        let mut tmp: f64;
        for (i, y) in yaxis.iter().enumerate() {
            for (j, x) in xaxis.iter().enumerate() {
                op.x = *x;
                op.y = *y;
                tmp = doppler_frequency_sg(
                    lem, &(op - ot), vt, &(op - or), vr
                );
                if tmp < self.min {
                    self.min = tmp;
                }
                if tmp > self.max {
                    self.max = tmp;
                }
                // Compute bistatic range
                self.data[i * self.width + j] = tmp;
            }
        }
    }

    pub fn levels(&self, nlevels: usize) -> Vec<f64> {
        let dv = (self.max - self.min) / (nlevels - 1) as f64;
        (0..nlevels).into_iter().map(|i| {
            self.min + dv * i as f64
        }).collect()
    }
}

impl Field for IsoDoppler {
    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn z_at(&self, x: usize, y: usize) -> f64 {
        self.data[y * self.width + x] // y -> i, x -> j
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end texture draw including the contour value labels: a font or
    /// plotters-feature regression makes this return Err — which the in-app
    /// caller silently ignores, so this test is the loud failure path.
    #[test]
    fn update_texture_draws_contours_and_labels() {
        let mut state = IsoRangeDopplerPlaneState::default();
        let mut image = Image::new_fill(
            Extent3d {
                width: TEXTURE_WIDTH as u32,
                height: TEXTURE_HEIGHT as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        state
            .update_texture(
                &DVec3::new(0.0, -8000.0, 6000.0),
                &DVec3::new(150.0, 0.0, 0.0),
                &DVec3::new(3000.0, 0.0, 4000.0),
                &DVec3::new(0.0, 100.0, 0.0),
                0.03,
                20_000.0,
                &mut image,
            )
            .expect("texture drawing including labels must succeed");
        // The texture must contain drawn content, not just the grey ground fill
        let bytes = image.data.as_ref().unwrap();
        assert!(bytes
            .chunks(4)
            .any(|px| px[0] != 128 || px[1] != 128 || px[2] != 128));
    }



    /// Regression test for the label placement mapping.
    ///
    /// Draws a horizontal contour at a known grid row through the very same
    /// plotters chart configuration used by `update_texture`, then checks the
    /// row of pixels plotters actually inked against the mapping the label
    /// rasterizer uses. The chart's reversed y-range puts grid row 0 at the top,
    /// so pixel_row = grid_row * sy; the previously used flipped mapping placed
    /// every label on the vertically mirrored contour (opposite Doppler sign).
    #[test]
    fn label_pixel_mapping_matches_plotters_contour_rows() {
        const GRID_ROW: f64 = 25.0; // Well inside the top quarter of the grid
        let mut bytes = vec![128u8; TEXTURE_WIDTH * TEXTURE_HEIGHT * 4]; // grey fill
        {
            let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
                &mut bytes,
                (TEXTURE_WIDTH as u32, TEXTURE_HEIGHT as u32),
            )
            .unwrap()
            .into_drawing_area();
            let mut chart = ChartBuilder::on(&root)
                .build_cartesian_2d(
                    0.0..(GRID_SIZE - 1) as f64,
                    (GRID_SIZE - 1) as f64..0.0, // Invert Y, as in update_texture
                )
                .unwrap();
            chart
                .draw_series(LineSeries::new(
                    (0..GRID_SIZE).map(|col| (col as f64, GRID_ROW)),
                    ISO_DOPPLER_STYLE,
                ))
                .unwrap();
        }
        // Row of the inked (non-grey) pixels plotters produced
        let inked_row = (0..TEXTURE_HEIGHT)
            .find(|&row| {
                (0..TEXTURE_WIDTH).any(|col| {
                    let i = (row * TEXTURE_WIDTH + col) * 4;
                    bytes[i] != 128 || bytes[i + 1] != 128 || bytes[i + 2] != 128
                })
            })
            .expect("the contour must be drawn somewhere");
        // The mapping used to place labels must agree with it
        let sy = (TEXTURE_HEIGHT - 1) as f64 / (GRID_SIZE - 1) as f64;
        let label_row = (GRID_ROW * sy) as usize;
        let tolerance = (2.0 * sy) as usize + ISO_DOPPLER_STYLE.stroke_width as usize;
        assert!(
            label_row.abs_diff(inked_row) <= tolerance,
            "label row {label_row} does not match the drawn contour row {inked_row}"
        );
    }



}
