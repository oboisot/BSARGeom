//! Minimal text rasterizer for the plotters bitmap buffers.
//!
//! plotters 0.3 cannot rasterize text on wasm32 (its wasm font backend only
//! measures text through the browser canvas), so labels are drawn directly
//! with `ab_glyph` into the BGRX pixel buffers — one uniform code path for
//! native and web targets, using the embedded DejaVu Sans font.

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

/// Draws `text` into a BGRX8888 buffer of `width`×`height` pixels with its
/// top-left corner at `(x, y)`, blending the glyph coverage over the existing
/// content. `color` is (R, G, B); out-of-bounds pixels are clipped.
pub fn draw_text_bgrx(
    bytes: &mut [u8],
    width: usize,
    height: usize,
    (x, y): (f32, f32),
    size_px: f32,
    color: (u8, u8, u8),
    text: &str,
) {
    let font = font();
    let scaled = font.as_scaled(PxScale::from(size_px));
    let baseline_y = y + scaled.ascent();
    let mut caret_x = x;
    let mut previous: Option<GlyphId> = None;
    for character in text.chars() {
        let id = font.glyph_id(character);
        if let Some(previous_id) = previous {
            caret_x += scaled.kern(previous_id, id);
        }
        let glyph = id.with_scale_and_position(PxScale::from(size_px), point(caret_x, baseline_y));
        caret_x += scaled.h_advance(id);
        previous = Some(id);
        let Some(outlined) = font.outline_glyph(glyph) else {
            continue; // whitespace and glyphs without outline
        };
        let bounds = outlined.px_bounds();
        outlined.draw(|glyph_x, glyph_y, coverage| {
            let pixel_x = bounds.min.x as i32 + glyph_x as i32;
            let pixel_y = bounds.min.y as i32 + glyph_y as i32;
            if pixel_x < 0 || pixel_y < 0 || pixel_x as usize >= width || pixel_y as usize >= height {
                return;
            }
            let index = (pixel_y as usize * width + pixel_x as usize) * 4;
            let blend = |background: u8, foreground: u8| -> u8 {
                (coverage * foreground as f32 + (1.0 - coverage) * background as f32).round() as u8
            };
            bytes[index] = blend(bytes[index], color.2); // B
            bytes[index + 1] = blend(bytes[index + 1], color.1); // G
            bytes[index + 2] = blend(bytes[index + 2], color.0); // R
        });
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
        draw_text_bgrx(&mut bytes, W, H, (2.0, 2.0), 20.0, (255, 0, 0), "42");
        // Red channel (index 2 in BGRX) must have been written somewhere
        assert!(bytes.chunks(4).any(|px| px[2] > 0));
        // Green/blue stay black for a pure red text on black background
        assert!(bytes.chunks(4).all(|px| px[0] == 0 && px[1] == 0));
        // Drawing partially/completely outside the buffer must not panic
        draw_text_bgrx(&mut bytes, W, H, (-10.0, -10.0), 20.0, (0, 255, 0), "clip");
        draw_text_bgrx(&mut bytes, W, H, (1000.0, 1000.0), 20.0, (0, 255, 0), "clip");
    }
}
