use bevy::{
    asset::RenderAssetUsages,
    math::DVec3,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat}
};
// use plotters::prelude::*;
use plotters::{
    backend::{BitMapBackend, BGRXPixel},
    chart::ChartBuilder,
    drawing::IntoDrawingArea,
    element::PathElement,
    style::{RGBAColor, ShapeStyle}
};

use crate::{
    bsar::{SPEED_OF_LIGHT_IN_VACUUM, bistatic_range_sg, doppler_frequency_sg},
    contour::{march, Field},
    entities::AntennaBeamFootprintState,
    scene::{TxCarrierState, RxCarrierState},
};

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
    let lem = tx_carrier_state.center_frequency_ghz * 1e9 /
        SPEED_OF_LIGHT_IN_VACUUM;
    let extent = 2.1 *
        tx_antenna_beam_footprint_state.ground_max_coord_m.max(
            rx_antenna_beam_footprint_state.ground_max_coord_m
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
    let tranform = Transform {
        translation: Vec3::new(0.0, 0.1, 0.0), // Slightly above the ground
        rotation: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2), // Rotate 90 degrees around Y-axis
        scale: Vec3::new(extent as f32, 1.0, extent as f32),
        ..Default::default()
    };

    Ok(tranform)
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
        //
        if let Some(ref mut bytes) = image.data {
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
            for level in iso_range_levels {
                for line in march(&self.iso_range, level) { // Compute contours
                    chart.draw_series(
                        std::iter::once(
                            PathElement::new(line, ISO_RANGE_STYLE) // here Contours are the same type as Coord for plotters
                        )
                    )?;
                }
            }
            // Iso-doppler
            for level in iso_doppler_levels {
                for line in march(&self.iso_doppler, level) { // Compute contours
                    chart.draw_series(
                        std::iter::once(
                            PathElement::new(line, ISO_DOPPLER_STYLE) // here Contours are the same type as Coord for plotters
                        )
                    )?;
                }
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
