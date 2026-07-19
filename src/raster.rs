//! Anti-aliased polyline rasterization into the BGRX pixel buffers.
//!
//! plotters' bitmap backend draws lines with an integer-coordinate Bresenham
//! rasterizer and no anti-aliasing, which makes the iso-range/iso-Doppler
//! contours look jagged and snaps every vertex to a whole pixel. These helpers
//! draw the same polylines from floating-point coordinates, blending each pixel
//! by its coverage (distance to the stroke), which removes both the staircase
//! edges and the vertex snapping.

/// Fills the whole BGRX buffer with a single (R, G, B) color.
pub fn fill_bgrx(bytes: &mut [u8], color: (u8, u8, u8)) {
    for pixel in bytes.chunks_exact_mut(4) {
        pixel[0] = color.2; // B
        pixel[1] = color.1; // G
        pixel[2] = color.0; // R
    }
}

/// Blends one anti-aliased segment with round caps into the buffer.
///
/// The segment constants (direction, inverse squared length) are hoisted out of
/// the pixel loop and far pixels are rejected on the *squared* distance, so the
/// per-pixel work stays a handful of multiply-adds — no division, and a square
/// root only for the thin band that actually contributes coverage.
fn draw_segment(
    bytes: &mut [u8],
    width: usize,
    height: usize,
    a: (f32, f32),
    b: (f32, f32),
    half_stroke: f32,
    color: (u8, u8, u8),
) {
    // Pixels within half a stroke (plus one pixel of AA falloff) can be covered
    let reach = half_stroke + 0.5;
    // `reach` is exactly where coverage vanishes; floor/ceil below adds the
    // remaining slack, so no extra ring of pixels needs to be visited.
    let margin = reach;
    let x_min = (a.0.min(b.0) - margin).floor().max(0.0) as usize;
    let x_max = (a.0.max(b.0) + margin).ceil().min(width as f32 - 1.0) as usize;
    let y_min = (a.1.min(b.1) - margin).floor().max(0.0) as usize;
    let y_max = (a.1.max(b.1) + margin).ceil().min(height as f32 - 1.0) as usize;
    if x_min > x_max || y_min > y_max {
        return; // Fully outside the buffer
    }

    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let length_squared = abx * abx + aby * aby;
    let inverse_length_squared = if length_squared > 0.0 { 1.0 / length_squared } else { 0.0 };
    let reach_squared = reach * reach;

    let (red, green, blue) = (color.0 as f32, color.1 as f32, color.2 as f32);
    for py in y_min..=y_max {
        let apy = py as f32 + 0.5 - a.1;
        // Work on this row's slice so the per-pixel writes need no bounds check
        let row = &mut bytes[py * width * 4..(py + 1) * width * 4];
        for px in x_min..=x_max {
            let apx = px as f32 + 0.5 - a.0;
            let t = ((apx * abx + apy * aby) * inverse_length_squared).clamp(0.0, 1.0);
            let (dx, dy) = (apx - t * abx, apy - t * aby);
            let distance_squared = dx * dx + dy * dy;
            if distance_squared >= reach_squared {
                continue; // Outside the stroke: no sqrt, no blend
            }
            // Coverage falls off over the last pixel, giving the smooth edge
            let coverage = (reach - distance_squared.sqrt()).min(1.0);
            let inverse = 1.0 - coverage;
            let pixel = &mut row[px * 4..px * 4 + 3];
            // `+ 0.5` then truncate: rounding without a `round()` call
            pixel[0] = (coverage * blue + inverse * pixel[0] as f32 + 0.5) as u8;
            pixel[1] = (coverage * green + inverse * pixel[1] as f32 + 0.5) as u8;
            pixel[2] = (coverage * red + inverse * pixel[2] as f32 + 0.5) as u8;
        }
    }
}

/// Draws an anti-aliased polyline of `stroke_width` pixels.
///
/// `dash` is an optional `(on, off)` pattern in pixels, applied along the
/// polyline's arc length (used for the negative iso-Doppler contours).
pub fn draw_polyline_bgrx(
    bytes: &mut [u8],
    width: usize,
    height: usize,
    points: &[(f32, f32)],
    stroke_width: f32,
    color: (u8, u8, u8),
    dash: Option<(f32, f32)>,
) {
    if points.len() < 2 {
        return;
    }
    let half_stroke = 0.5 * stroke_width;
    let Some((on, off)) = dash else {
        for pair in points.windows(2) {
            draw_segment(bytes, width, height, pair[0], pair[1], half_stroke, color);
        }
        return;
    };
    // Dashed: walk the arc length, emitting only the "on" spans
    let period = (on + off).max(1e-3);
    let mut travelled = 0.0f32;
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let segment_length = (b.0 - a.0).hypot(b.1 - a.1);
        if segment_length <= 0.0 {
            continue;
        }
        let mut position = 0.0f32;
        while position < segment_length {
            let phase = (travelled + position) % period;
            if phase < on {
                // Inside a dash: draw up to its end or the segment's end
                let span = (on - phase).min(segment_length - position);
                let t0 = position / segment_length;
                let t1 = (position + span) / segment_length;
                let start = (a.0 + (b.0 - a.0) * t0, a.1 + (b.1 - a.1) * t0);
                let end = (a.0 + (b.0 - a.0) * t1, a.1 + (b.1 - a.1) * t1);
                draw_segment(bytes, width, height, start, end, half_stroke, color);
                position += span;
            } else {
                // Inside a gap: skip to the next dash
                position += period - phase;
            }
        }
        travelled += segment_length;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: usize = 64;
    const H: usize = 64;

    fn red_at(bytes: &[u8], x: usize, y: usize) -> u8 {
        bytes[(y * W + x) * 4 + 2]
    }

    #[test]
    fn fill_sets_every_pixel() {
        let mut bytes = vec![0u8; W * H * 4];
        fill_bgrx(&mut bytes, (128, 129, 130));
        assert!(bytes
            .chunks_exact(4)
            .all(|px| px[0] == 130 && px[1] == 129 && px[2] == 128));
    }

    /// A diagonal line must have partially-covered pixels along its edge —
    /// that is exactly what a non-anti-aliased rasterizer cannot produce.
    #[test]
    fn diagonal_line_is_anti_aliased() {
        let mut bytes = vec![0u8; W * H * 4];
        draw_polyline_bgrx(
            &mut bytes,
            W,
            H,
            &[(4.0, 4.0), (60.0, 60.0)],
            3.0,
            (255, 0, 0),
            None,
        );
        let mut partial = 0;
        let mut full = 0;
        for y in 0..H {
            for x in 0..W {
                match red_at(&bytes, x, y) {
                    0 => {}
                    255 => full += 1,
                    _ => partial += 1,
                }
            }
        }
        assert!(full > 0, "the line core must be fully opaque");
        assert!(partial > full / 4, "anti-aliased edges must be present");
    }

    /// Sub-pixel positions must be honoured: shifting a line by half a pixel
    /// changes the coverage instead of snapping to the same pixels.
    #[test]
    fn sub_pixel_shift_changes_coverage() {
        let render = |offset: f32| {
            let mut bytes = vec![0u8; W * H * 4];
            draw_polyline_bgrx(
                &mut bytes,
                W,
                H,
                &[(4.0, 10.0 + offset), (60.0, 10.0 + offset)],
                2.0,
                (255, 0, 0),
                None,
            );
            (0..H).map(|y| red_at(&bytes, 32, y)).collect::<Vec<_>>()
        };
        assert_ne!(render(0.0), render(0.5));
    }

    #[test]
    fn dashes_leave_gaps_along_the_line() {
        let mut bytes = vec![0u8; W * H * 4];
        draw_polyline_bgrx(
            &mut bytes,
            W,
            H,
            &[(2.0, 32.0), (62.0, 32.0)],
            2.0,
            (255, 0, 0),
            Some((6.0, 10.0)),
        );
        let along: Vec<u8> = (2..62).map(|x| red_at(&bytes, x, 32)).collect();
        assert!(along.iter().any(|&v| v > 200), "dashes must be drawn");
        assert!(along.contains(&0), "gaps must stay empty");
    }

    #[test]
    fn drawing_outside_the_buffer_is_clipped() {
        let mut bytes = vec![0u8; W * H * 4];
        draw_polyline_bgrx(&mut bytes, W, H, &[(-100.0, -100.0), (-50.0, -50.0)], 4.0, (255, 0, 0), None);
        draw_polyline_bgrx(&mut bytes, W, H, &[(500.0, 500.0), (900.0, 900.0)], 4.0, (255, 0, 0), None);
        assert!(bytes.iter().all(|&b| b == 0));
    }
}
