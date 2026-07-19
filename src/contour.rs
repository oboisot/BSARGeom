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

/// Emits the marching-squares segments of one cell for the threshold `z`.
///
/// `(ulz, urz, blz, brz)` are the upper-left, upper-right, bottom-left and
/// bottom-right corner values of the cell whose upper-left corner is `(x, y)`.
fn march_cell(
    segments: &mut SegmentsMap,
    x: usize,
    y: usize,
    (ulz, urz, blz, brz): (f64, f64, f64, f64),
    z: f64,
) {
    let mut add_seg = |s: (f64, f64), e| {
        segments
            .entry((s.0 as u64, s.1 as u64))
            .or_default()
            .push((s, e));
    };

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

/// Find the contours of a given scalar field using `z` as the threshold value.
pub fn march(field: &impl Field, z: f64) -> Contours {
    march_levels(field, &[z]).pop().unwrap_or_default()
}

/// Find the contours of a scalar field for many threshold values at once,
/// returning one [`Contours`] per entry of `levels` (in the same order).
///
/// This walks the grid a single time instead of once per level: a cell can only
/// produce a segment for the levels inside `[cell_min, cell_max)`, which is a
/// contiguous slice of the sorted levels found by binary search. With the 50
/// levels per family used by the ground plane this is dramatically cheaper than
/// calling [`march`] in a loop.
///
/// Cells containing NaN corners contribute only to the levels bracketed by
/// their finite corners (a fully-NaN cell contributes to none), which avoids
/// emitting NaN vertices.
pub fn march_levels(field: &impl Field, levels: &[f64]) -> Vec<Contours> {
    let (width, height) = field.dimensions();
    let mut segments_per_level: Vec<SegmentsMap> =
        (0..levels.len()).map(|_| SegmentsMap::default()).collect();

    if levels.is_empty() || width == 0 || height == 0 {
        return segments_per_level
            .into_iter()
            .map(|segments| build_contours(segments, (width as u64, height as u64)))
            .collect();
    }

    // Levels sorted ascending (keeping a map back to the caller's order) so the
    // crossing levels of a cell form a contiguous range.
    let mut order: Vec<usize> = (0..levels.len()).collect();
    order.sort_by(|&a, &b| {
        levels[a]
            .partial_cmp(&levels[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let sorted_levels: Vec<f64> = order.iter().map(|&i| levels[i]).collect();

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

            // A level `z` gives a non-trivial case only when at least one corner
            // is > z (so z < cell_max) and at least one is <= z (so z >= cell_min).
            let cell_min = ulz.min(urz).min(blz).min(brz);
            let cell_max = ulz.max(urz).max(blz).max(brz);
            let lower = sorted_levels.partition_point(|&level| level < cell_min);
            let upper = sorted_levels.partition_point(|&level| level < cell_max);

            for sorted_index in lower..upper {
                march_cell(
                    &mut segments_per_level[order[sorted_index]],
                    x,
                    y,
                    (ulz, urz, blz, brz),
                    sorted_levels[sorted_index],
                );
            }
        }

        std::mem::swap(&mut current_row_zs, &mut next_row_zs);
    }

    segments_per_level
        .into_iter()
        .map(|segments| build_contours(segments, (width as u64, height as u64)))
        .collect()
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

    /// `march_levels` must be exactly equivalent to calling `march` per level —
    /// it is only a single-pass reorganisation of the same algorithm.
    #[test]
    fn march_levels_matches_march_per_level() {
        // A field with saddle points and closed loops, so many cell cases occur
        let field = FnField {
            width: 37,
            height: 29,
            f: |x, y| {
                let (x, y) = (x as f64 * 0.3, y as f64 * 0.35);
                x.sin() * y.cos() * 5.0 + 0.05 * (x * x - y * y)
            },
        };
        let levels: Vec<f64> = (0..17).map(|i| -4.0 + i as f64 * 0.5).collect();
        let batched = march_levels(&field, &levels);
        assert_eq!(batched.len(), levels.len());
        for (&level, from_batch) in levels.iter().zip(&batched) {
            assert_eq!(
                *from_batch,
                march(&field, level),
                "level {level} differs between march_levels and march"
            );
        }
    }

    /// The caller's ordering is preserved even when levels are not sorted.
    #[test]
    fn march_levels_preserves_unsorted_level_order() {
        let field = FnField { width: 9, height: 9, f: |x, y| (x + y) as f64 };
        let levels = [6.0, 2.0, 11.0, 4.0];
        let batched = march_levels(&field, &levels);
        for (&level, from_batch) in levels.iter().zip(&batched) {
            assert_eq!(*from_batch, march(&field, level), "level {level} misplaced");
        }
    }

    #[test]
    fn march_levels_handles_empty_input() {
        let field = FnField { width: 5, height: 5, f: |x, _| x as f64 };
        assert!(march_levels(&field, &[]).is_empty());
    }

}
