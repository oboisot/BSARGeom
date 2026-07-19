//! Minimal text rasterizer for the plotters bitmap buffers.
//!
//! plotters 0.3 cannot rasterize text on wasm32 (its wasm font backend only
//! measures text through the browser canvas), so labels are drawn directly
//! with `ab_glyph` into the BGRX pixel buffers — one uniform code path for
//! native and web targets, using the embedded DejaVu Sans font.
//!
//! Text is rendered once into an alpha mask and then blitted at an arbitrary
//! angle, which lets contour labels follow the slope of the line they annotate
//! (and carry a padded background that interrupts the line underneath, like
//! plotly's inline contour labels).

use std::sync::OnceLock;

use ab_glyph::{point, Font, FontRef, GlyphId, PxScale, ScaleFont};

static FONT: OnceLock<FontRef<'static>> = OnceLock::new();

fn font() -> &'static FontRef<'static> {
    FONT.get_or_init(|| {
        FontRef::try_from_slice(include_bytes!("../assets/fonts/DejaVuSans.ttf"))
            .expect("embedded DejaVuSans.ttf must be a valid font")
    })
}

/// Width in pixels of `text` rendered at `size_px`.
pub fn text_width(text: &str, size_px: f32) -> f32 {
    let font = font();
    let scaled = font.as_scaled(PxScale::from(size_px));
    let mut width = 0.0f32;
    let mut previous: Option<GlyphId> = None;
    for character in text.chars() {
        let id = font.glyph_id(character);
        if let Some(previous_id) = previous {
            width += scaled.kern(previous_id, id);
        }
        width += scaled.h_advance(id);
        previous = Some(id);
    }
    width
}

/// An 8-bit coverage mask of a rendered string.
struct TextMask {
    width: usize,
    height: usize,
    alpha: Vec<u8>,
}

/// Rasterizes `text` into a tightly-sized coverage mask.
fn text_mask(text: &str, size_px: f32) -> TextMask {
    let font = font();
    let scaled = font.as_scaled(PxScale::from(size_px));
    let width = text_width(text, size_px).ceil().max(1.0) as usize;
    let ascent = scaled.ascent();
    let height = (ascent - scaled.descent()).ceil().max(1.0) as usize;
    let mut alpha = vec![0u8; width * height];

    let mut caret_x = 0.0f32;
    let mut previous: Option<GlyphId> = None;
    for character in text.chars() {
        let id = font.glyph_id(character);
        if let Some(previous_id) = previous {
            caret_x += scaled.kern(previous_id, id);
        }
        let glyph = id.with_scale_and_position(PxScale::from(size_px), point(caret_x, ascent));
        caret_x += scaled.h_advance(id);
        previous = Some(id);
        let Some(outlined) = font.outline_glyph(glyph) else {
            continue; // whitespace and glyphs without outline
        };
        let bounds = outlined.px_bounds();
        outlined.draw(|glyph_x, glyph_y, coverage| {
            let x = bounds.min.x as i32 + glyph_x as i32;
            let y = bounds.min.y as i32 + glyph_y as i32;
            if x < 0 || y < 0 || x as usize >= width || y as usize >= height {
                return;
            }
            let index = y as usize * width + x as usize;
            // Glyphs never overlap here, but saturate to stay safe
            alpha[index] = alpha[index].saturating_add((coverage * 255.0).round() as u8);
        });
    }
    TextMask { width, height, alpha }
}

/// Bilinear coverage sample of the mask at (continuous) mask coordinates.
fn sample_mask(mask: &TextMask, x: f32, y: f32) -> f32 {
    if x < -1.0 || y < -1.0 || x > mask.width as f32 || y > mask.height as f32 {
        return 0.0;
    }
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let at = |cx: i32, cy: i32| -> f32 {
        if cx < 0 || cy < 0 || cx as usize >= mask.width || cy as usize >= mask.height {
            0.0
        } else {
            mask.alpha[cy as usize * mask.width + cx as usize] as f32 / 255.0
        }
    };
    let top = at(x0, y0) * (1.0 - fx) + at(x0 + 1, y0) * fx;
    let bottom = at(x0, y0 + 1) * (1.0 - fx) + at(x0 + 1, y0 + 1) * fx;
    top * (1.0 - fy) + bottom * fy
}

/// Draws `text` into a BGRX8888 buffer of `width`×`height` pixels, centered on
/// `center` and rotated by `angle_rad` (positive angles turn clockwise on
/// screen, since y grows downwards). `color` is (R, G, B).
///
/// When `background` is given, a padded rectangle of that color is painted
/// under the text first, interrupting whatever the label sits on (a contour
/// line) so it stays legible — the same effect as plotly's inline contour
/// labels. Out-of-bounds pixels are clipped.
pub fn draw_text_bgrx(
    bytes: &mut [u8],
    width: usize,
    height: usize,
    center: (f32, f32),
    angle_rad: f32,
    size_px: f32,
    color: (u8, u8, u8),
    background: Option<(u8, u8, u8)>,
    padding_px: f32,
    text: &str,
) {
    let mask = text_mask(text, size_px);
    let (mask_w, mask_h) = (mask.width as f32, mask.height as f32);
    let (sin, cos) = angle_rad.sin_cos();

    // Destination area covered by the rotated, padded mask
    let half_w = 0.5 * mask_w + padding_px;
    let half_h = 0.5 * mask_h + padding_px;
    let reach = (half_w * cos.abs() + half_h * sin.abs())
        .max(half_w * sin.abs() + half_h * cos.abs());
    let x_min = (center.0 - reach).floor().max(0.0) as usize;
    let x_max = (center.0 + reach).ceil().min(width as f32 - 1.0) as usize;
    let y_min = (center.1 - reach).floor().max(0.0) as usize;
    let y_max = (center.1 + reach).ceil().min(height as f32 - 1.0) as usize;
    if x_min > x_max || y_min > y_max {
        return; // Fully outside the buffer
    }

    for py in y_min..=y_max {
        for px in x_min..=x_max {
            // Inverse-rotate the destination pixel into mask space
            let dx = px as f32 - center.0;
            let dy = py as f32 - center.1;
            let mx = dx * cos + dy * sin + 0.5 * mask_w;
            let my = -dx * sin + dy * cos + 0.5 * mask_h;

            let index = (py * width + px) * 4;
            if let Some((br, bg, bb)) = background
                && mx >= -padding_px
                && mx <= mask_w + padding_px
                && my >= -padding_px
                && my <= mask_h + padding_px
            {
                bytes[index] = bb; // B
                bytes[index + 1] = bg; // G
                bytes[index + 2] = br; // R
            }

            let coverage = sample_mask(&mask, mx, my);
            if coverage <= 0.0 {
                continue;
            }
            let blend = |background: u8, foreground: u8| -> u8 {
                (coverage * foreground as f32 + (1.0 - coverage) * background as f32).round() as u8
            };
            bytes[index] = blend(bytes[index], color.2); // B
            bytes[index + 1] = blend(bytes[index + 1], color.1); // G
            bytes[index + 2] = blend(bytes[index + 2], color.0); // R
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_width_is_positive_and_additive() {
        let single = text_width("8", 45.0);
        let double = text_width("88", 45.0);
        assert!(single > 0.0);
        assert!(double > single);
    }

    #[test]
    fn draw_text_blends_pixels_and_clips_bounds() {
        const W: usize = 64;
        const H: usize = 32;
        let mut bytes = vec![0u8; W * H * 4];
        draw_text_bgrx(&mut bytes, W, H, (32.0, 16.0), 0.0, 20.0, (255, 0, 0), None, 0.0, "42");
        // Red channel (index 2 in BGRX) must have been written somewhere
        assert!(bytes.chunks(4).any(|px| px[2] > 0));
        // Green/blue stay black for a pure red text on black background
        assert!(bytes.chunks(4).all(|px| px[0] == 0 && px[1] == 0));
        // Drawing partially/completely outside the buffer must not panic
        draw_text_bgrx(&mut bytes, W, H, (-40.0, -40.0), 0.0, 20.0, (0, 255, 0), None, 0.0, "clip");
        draw_text_bgrx(&mut bytes, W, H, (1000.0, 1000.0), 0.0, 20.0, (0, 255, 0), None, 0.0, "clip");
    }

    /// The padded background must erase what is underneath (a contour line)
    /// around the glyphs, and rotation must keep the ink inside the buffer.
    #[test]
    fn background_padding_erases_underlying_pixels() {
        const W: usize = 128;
        const H: usize = 128;
        // Fill with a "contour line" color everywhere
        let mut bytes = vec![0u8; W * H * 4];
        for px in bytes.chunks_mut(4) {
            px[0] = 200; // B
            px[1] = 10;
            px[2] = 10;
        }
        draw_text_bgrx(
            &mut bytes,
            W,
            H,
            (64.0, 64.0),
            0.5, // rotated
            24.0,
            (255, 255, 255),
            Some((128, 128, 128)), // ground grey halo
            6.0,
            "123 m",
        );
        // Some pixels are now the background grey (the halo) ...
        assert!(bytes
            .chunks(4)
            .any(|px| px[0] == 128 && px[1] == 128 && px[2] == 128));
        // ... some are the white text ...
        assert!(bytes.chunks(4).any(|px| px[0] > 200 && px[1] > 200 && px[2] > 200));
        // ... and the original color still exists far from the label
        assert!(bytes.chunks(4).any(|px| px[0] == 200 && px[1] == 10));
    }

    /// Rotating by a right angle must swap the inked extent from wide to tall.
    #[test]
    fn rotation_changes_ink_orientation() {
        const W: usize = 160;
        const H: usize = 160;
        let inked_extent = |angle: f32| -> (usize, usize) {
            let mut bytes = vec![0u8; W * H * 4];
            draw_text_bgrx(
                &mut bytes, W, H, (80.0, 80.0), angle, 20.0, (255, 255, 255), None, 0.0, "1234567",
            );
            let inked: Vec<(usize, usize)> = (0..H)
                .flat_map(|y| (0..W).map(move |x| (x, y)))
                .filter(|&(x, y)| bytes[(y * W + x) * 4 + 2] > 0)
                .collect();
            let x_span = inked.iter().map(|p| p.0).max().unwrap()
                - inked.iter().map(|p| p.0).min().unwrap();
            let y_span = inked.iter().map(|p| p.1).max().unwrap()
                - inked.iter().map(|p| p.1).min().unwrap();
            (x_span, y_span)
        };
        let (flat_x, flat_y) = inked_extent(0.0);
        let (turned_x, turned_y) = inked_extent(std::f32::consts::FRAC_PI_2);
        assert!(flat_x > flat_y, "unrotated text must be wider than tall");
        assert!(turned_y > turned_x, "quarter-turned text must be taller than wide");
    }
}
