use rustc_hash::{FxHashMap, FxHashSet};

/// A scalar field.
pub trait Field {
    /// Get the width and height of the scalar field.
    fn dimensions(&self) -> (usize, usize);

    /// Calculate the z value at the given position. The position is always inside the range of
    /// `dimensions`.
    fn z_at(&self, x: usize, y: usize) -> f64;

    /// Helper to force a Field to have all the Z values at the boundaries of the field to be set
    /// to `border_z`. Useful to ensure each path is closed.
    fn framed(&self, border_z: f64) -> Framed<Self>
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
type SegmentsMap = FxHashMap<(u64, u64), Vec<((f64, f64), (f64, f64))>>;

/// Find the contours of a given scalar field using `z` as the threshold value.
pub fn march(field: &impl Field, z: f64) -> Contours {
    let (width, height) = field.dimensions();

    // let mut segments: SegmentsMap = HashMap::new();
    let mut segments: SegmentsMap = FxHashMap::default();
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
    use std::collections::hash_map::Entry;

    let mut contours = vec![];

    let mut boundaries = segments
        .keys()
        .cloned()
        .filter(|s| s.0 == 0 || s.0 == w - 1 || s.1 == 0 || s.1 == h - 1)
        .collect::<FxHashSet<_>>();

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
