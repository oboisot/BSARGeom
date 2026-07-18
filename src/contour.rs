use bevy::platform::collections::{HashMap, HashSet};

/// A scalar field.
pub trait Field {
    /// Get the width and height of the scalar field.
    fn dimensions(&self) -> (usize, usize);

    /// Calculate the z value at the given position. The position is always inside the range of
    /// `dimensions`.
    fn z_at(&self, x: usize, y: usize) -> f64;

    /// Helper to force a Field to have all the Z values at the boundaries of the field to be set
    /// to `border_z`. Useful to ensure each path is closed.
    fn framed(&self, border_z: f64) -> Framed<'_, Self>
    where
        Self: Sized,
    {
        Framed {
            field: self,
            border_z,
        }
    }
}

/// Contours of a shape.
pub type Contours = Vec<Vec<(f64, f64)>>;

/// A `SegmentsMap` is used to speedup contour building on the average case. It's simply a map from
/// the start position of the segment rounded with integers coordinates to the list of all the
/// segments that start in that position. Usually, shapes have very few segments that start at the
/// same integer position thus this simple optimization allows to find the next segment in O(1)
/// which is great.
///
/// Note that a valid `SegmentsMap` must not have entries for an empty list of segments.
type SegmentsMap = HashMap<(u64, u64), Vec<((f64, f64), (f64, f64))>>;

/// Find the contours of a given scalar field using `z` as the threshold value.
pub fn march(field: &impl Field, z: f64) -> Contours {
    let (width, height) = field.dimensions();

    let mut segments: SegmentsMap = HashMap::default();
    let mut add_seg = |s: (f64, f64), e| {
        segments
            .entry((s.0 as u64, s.1 as u64))
            .or_default()
            .push((s, e));
    };

    // avoid calling z_at multiple times for the same cell by storing the z values for the current
    // row and by storing the values for the next row as soon as they're calculated.
    let mut current_row_zs = (0..width).map(|x| field.z_at(x, 0)).collect::<Vec<_>>();
    let mut next_row_zs = Vec::with_capacity(width);

    for y in 0..height.saturating_sub(1) {
        next_row_zs.clear();
        next_row_zs.push(field.z_at(0, y + 1));

        for x in 0..width.saturating_sub(1) {
            let ulz = current_row_zs[x];
            let urz = current_row_zs[x + 1];
            let blz = next_row_zs[x];
            let brz = field.z_at(x + 1, y + 1);

            next_row_zs.push(brz);

            let mut case = 0;
            if blz > z {
                case |= 1;
            }
            if brz > z {
                case |= 2;
            }
            if urz > z {
                case |= 4;
            }
            if ulz > z {
                case |= 8;
            }
            let x = x as f64;
            let y = y as f64;

            let top = (x + fraction(z, (ulz, urz)), y);
            let bottom = (x + fraction(z, (blz, brz)), y + 1.0);
            let left = (x, y + fraction(z, (ulz, blz)));
            let right = (x + 1.0, y + fraction(z, (urz, brz)));

            match case {
                0 | 15 => {}
                1 => {
                    add_seg(bottom, left);
                }
                2 => {
                    add_seg(right, bottom);
                }
                3 => {
                    add_seg(right, left);
                }
                4 => {
                    add_seg(top, right);
                }
                5 => {
                    add_seg(top, left);
                    add_seg(bottom, right);
                }
                6 => {
                    add_seg(top, bottom);
                }
                7 => {
                    add_seg(top, left);
                }
                8 => {
                    add_seg(left, top);
                }
                9 => {
                    add_seg(bottom, top);
                }
                10 => {
                    add_seg(left, bottom);
                    add_seg(right, top);
                }
                11 => {
                    add_seg(right, top);
                }
                12 => {
                    add_seg(left, right);
                }
                13 => {
                    add_seg(bottom, right);
                }
                14 => {
                    add_seg(left, bottom);
                }
                _ => unreachable!(),
            }
        }

        std::mem::swap(&mut current_row_zs, &mut next_row_zs);
    }

    build_contours(segments, (width as u64, height as u64))
}

fn build_contours(mut segments: SegmentsMap, (w, h): (u64, u64)) -> Contours {
    use bevy::platform::collections::hash_map::Entry;

    let mut contours = vec![];

    let mut boundaries = segments
        .keys()
        .cloned()
        .filter(|s| s.0 == 0 || s.0 == w - 1 || s.1 == 0 || s.1 == h - 1)
        .collect::<HashSet<_>>();

    while !segments.is_empty() {
        // prefer to start on a boundary, but if no point lie on a bounday just
        // pick a random one. This allows to connect open paths entirely without
        // breaking them in multiple chunks.
        let first_k = boundaries
            .iter()
            .next()
            .map_or_else(|| *segments.keys().next().unwrap(), |k| *k);

        let mut first_e = match segments.entry(first_k) {
            Entry::Occupied(o) => o,
            Entry::Vacant(_) => unreachable!(),
        };

        let first = first_e.get_mut().pop().unwrap();
        if first_e.get().is_empty() {
            first_e.remove_entry();
            boundaries.remove(&first_k);
        }

        let mut contour = vec![first.0, first.1];

        loop {
            let prev = contour[contour.len() - 1];

            let segments_k = (prev.0 as u64, prev.1 as u64);
            let mut segments = match segments.entry(segments_k) {
                Entry::Vacant(_) => break,
                Entry::Occupied(o) => o,
            };

            let next = segments
                .get()
                .iter()
                .enumerate()
                .find(|(_, (s, _))| s == &prev);

            match next {
                None => break,
                Some((i, seg)) => {
                    contour.push(seg.1);

                    segments.get_mut().swap_remove(i);
                    if segments.get().is_empty() {
                        segments.remove_entry();
                        boundaries.remove(&segments_k);
                    }
                }
            }
        }

        contours.push(contour);
    }

    contours
}

fn fraction(z: f64, (z0, z1): (f64, f64)) -> f64 {
    if z0 == z1 {
        return 0.5;
    }

    let t = (z - z0) / (z1 - z0);
    t.clamp(0.0, 1.0)
}

#[derive(Debug, Clone)]
pub struct Framed<'s, F> {
    field: &'s F,
    border_z: f64,
}

impl<T: Field> Field for Framed<'_, T> {
    fn dimensions(&self) -> (usize, usize) {
        self.field.dimensions()
    }

    fn z_at(&self, x: usize, y: usize) -> f64 {
        let (w, h) = self.dimensions();

        if x == 0 || x == w.saturating_sub(1) || y == 0 || y == h.saturating_sub(1) {
            self.border_z + 1e-9
        } else {
            self.field.z_at(x, y)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scalar field defined by a closure over grid indices.
    struct FnField<F: Fn(usize, usize) -> f64> {
        width: usize,
        height: usize,
        f: F,
    }

    impl<F: Fn(usize, usize) -> f64> Field for FnField<F> {
        fn dimensions(&self) -> (usize, usize) {
            (self.width, self.height)
        }
        fn z_at(&self, x: usize, y: usize) -> f64 {
            (self.f)(x, y)
        }
    }

    #[test]
    fn fraction_interpolates_and_clamps() {
        assert_eq!(fraction(5.0, (0.0, 10.0)), 0.5);
        assert_eq!(fraction(2.5, (0.0, 10.0)), 0.25);
        assert_eq!(fraction(20.0, (0.0, 10.0)), 1.0); // Clamped
        assert_eq!(fraction(-5.0, (0.0, 10.0)), 0.0); // Clamped
        assert_eq!(fraction(1.0, (3.0, 3.0)), 0.5);   // Degenerate edge
    }

    #[test]
    fn linear_ramp_gives_a_straight_contour_line() {
        // z = x: the level set z = 1.5 is the vertical line x = 1.5
        let field = FnField { width: 5, height: 5, f: |x, _| x as f64 };
        let contours = march(&field, 1.5);
        assert_eq!(contours.len(), 1);
        let line = &contours[0];
        assert_eq!(line.len(), 5); // One point per cell row + 1
        for (x, _) in line.iter() {
            assert!((x - 1.5).abs() < 1e-12);
        }
        // Spans the full field height
        let ys: Vec<f64> = line.iter().map(|p| p.1).collect();
        assert_eq!(ys.iter().cloned().fold(f64::MAX, f64::min), 0.0);
        assert_eq!(ys.iter().cloned().fold(f64::MIN, f64::max), 4.0);
    }

    #[test]
    fn circular_field_gives_a_closed_loop() {
        // z = squared distance to the grid centre; the level set z = r² is a circle
        let (centre, radius) = (10.0f64, 4.0f64);
        let field = FnField {
            width: 21,
            height: 21,
            f: move |x, y| (x as f64 - centre).powi(2) + (y as f64 - centre).powi(2),
        };
        let contours = march(&field, radius * radius);
        // The implementation may split a loop into several abutting polylines;
        // the geometric invariant is that every vertex lies on the circle
        assert!(!contours.is_empty());
        let mut n_points = 0;
        for contour in contours.iter() {
            assert!(contour.len() >= 2);
            for (x, y) in contour.iter() {
                let r = ((x - centre).powi(2) + (y - centre).powi(2)).sqrt();
                assert!((r - radius).abs() < 0.2, "point ({x}, {y}) at radius {r}");
                n_points += 1;
            }
        }
        assert!(n_points > 20); // Full circle sampled on a 21x21 grid
    }

    #[test]
    fn level_outside_field_range_gives_no_contours() {
        let field = FnField { width: 5, height: 5, f: |x, _| x as f64 };
        assert!(march(&field, -1.0).is_empty());
        assert!(march(&field, 100.0).is_empty());
    }

    #[test]
    fn framed_field_closes_border_contours() {
        // Constant zero field: without the frame there is no level set at all;
        // the frame forces the border slightly above the level so a closed
        // rectangle just inside the border is produced
        let field = FnField { width: 6, height: 6, f: |_, _| 0.0 };
        let contours = march(&field.framed(0.0), 0.0);
        // A rectangle hugging the border is produced (possibly as several
        // abutting polylines); without the frame there would be no contour
        assert!(!contours.is_empty());
        for contour in contours.iter() {
            for (x, y) in contour.iter() {
                // Every point lies exactly on the rectangle x, y ∈ {1, 4}
                // (the frame/interior transition interpolates to the interior ring)
                let on_frame = ((x - 1.0).abs() < 1e-9 || (x - 4.0).abs() < 1e-9 ||
                                (y - 1.0).abs() < 1e-9 || (y - 4.0).abs() < 1e-9) &&
                               (1.0..=4.0).contains(x) && (1.0..=4.0).contains(y);
                assert!(on_frame, "point ({x}, {y}) is not on the frame rectangle");
            }
        }
        assert!(march(&field, 0.0).is_empty()); // No contour without the frame
    }
}
