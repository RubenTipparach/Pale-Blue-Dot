//! The clouds' cellular noise, baked once into a tileable 3D texture.
//!
//! `clouds.wgsl` draws the rounded heads of cumulus with cellular noise: the
//! smooth distance to the nearest of points scattered one per lattice cell,
//! turned over so a cell's middle is high. Evaluated in the shader it was 27
//! cells, each three hashes and an `exp`, twice per density sample, and it was
//! three quarters of the clouds pass (`cloud-budget`). Here it is built once,
//! and the shader reads one trilinear tap. This is derived render state: it
//! decides nothing but how a cloud looks.
//!
//! The lattice wraps every `PERIOD` cells, so the texture tiles and the
//! shader samples it with a repeating sampler at `q / PERIOD`.

/// Texels along each edge of the texture.
pub const SIZE: u32 = 128;
/// Lattice cells along each edge: `SIZE / PERIOD` texels per cell.
pub const PERIOD: u32 = 16;
/// The smooth minimum's sharpness. A hard minimum has a crease wherever two
/// cells meet, and the cover remap sharpened each into a thin dark line.
const SMOOTH: f32 = 8.0;

/// One integer hash to 0..1 (a PCG step), keyed by a lattice cell and a lane.
fn hash(x: u32, y: u32, z: u32, lane: u32) -> f32 {
    let mut h = x
        .wrapping_mul(1973)
        .wrapping_add(y.wrapping_mul(9277))
        .wrapping_add(z.wrapping_mul(26699))
        .wrapping_add(lane.wrapping_mul(0x9e37_79b9))
        .wrapping_add(1);
    h = h.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    h = ((h >> ((h >> 28) + 4)) ^ h).wrapping_mul(277_803_737);
    h = (h >> 22) ^ h;
    (h & 0x00ff_ffff) as f32 / 16_777_216.0
}

/// The feature point of every lattice cell, in lattice units, x fastest.
fn feature_points(period: u32) -> Vec<[f32; 3]> {
    let mut points = Vec::with_capacity((period * period * period) as usize);
    for z in 0..period {
        for y in 0..period {
            for x in 0..period {
                points.push([
                    x as f32 + hash(x, y, z, 0),
                    y as f32 + hash(x, y, z, 1),
                    z as f32 + hash(x, y, z, 2),
                ]);
            }
        }
    }
    points
}

/// The noise at `p`, lattice units, off a lattice of `period` cells that
/// wraps: `clamp(1 - smooth F1, 0, 1)`.
fn value(points: &[[f32; 3]], period: u32, p: [f32; 3]) -> f32 {
    let n = period as i32;
    let cell = p.map(|v| v.floor() as i32);
    let mut sum = 0.0f32;
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let c = [cell[0] + dx, cell[1] + dy, cell[2] + dz];
                let w = c.map(|v| v.rem_euclid(n) as usize);
                let point = points[(w[2] * period as usize + w[1]) * period as usize + w[0]];
                // The wrapped cell's point, moved back to where this cell is.
                let q = [
                    point[0] + (c[0] - w[0] as i32) as f32,
                    point[1] + (c[1] - w[1] as i32) as f32,
                    point[2] + (c[2] - w[2] as i32) as f32,
                ];
                let d =
                    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
                sum += (-SMOOTH * d).exp();
            }
        }
    }
    let nearest = -sum.max(1e-12).ln() / SMOOTH;
    (1.0 - nearest).clamp(0.0, 1.0)
}

/// The texture's bytes, `R8Unorm`, x fastest then y then z: `size` texels a
/// side over `period` cells. Built on every core, one band of slices each.
pub fn bake(size: u32, period: u32) -> Vec<u8> {
    let points = feature_points(period);
    let slice = (size * size) as usize;
    let mut bytes = vec![0u8; slice * size as usize];
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(size as usize);
    let per = (size as usize).div_ceil(threads);
    let scale = period as f32 / size as f32;
    std::thread::scope(|scope| {
        for (band, chunk) in bytes.chunks_mut(slice * per).enumerate() {
            let points = &points;
            scope.spawn(move || {
                for (i, texel) in chunk.iter_mut().enumerate() {
                    let z = band * per + i / slice;
                    let y = (i % slice) / size as usize;
                    let x = i % size as usize;
                    // Texel centres, so the repeating sampler's wrap lands
                    // between the last texel and the first.
                    let p = [
                        (x as f32 + 0.5) * scale,
                        (y as f32 + 0.5) * scale,
                        (z as f32 + 0.5) * scale,
                    ];
                    *texel = (value(points, period, p) * 255.0).round() as u8;
                }
            });
        }
    });
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The texture tiles: the step from the last texel on an edge to the first
    /// is no larger than the steps between neighbours anywhere inside it.
    #[test]
    fn the_cells_texture_tiles() {
        let (size, period) = (32u32, 4u32);
        let bytes = bake(size, period);
        let at = |x: u32, y: u32, z: u32| bytes[((z * size + y) * size + x) as usize] as i32;
        let mut inside = 0;
        let mut wrap = 0;
        for z in 0..size {
            for y in 0..size {
                for x in 0..size - 1 {
                    inside = inside.max((at(x + 1, y, z) - at(x, y, z)).abs());
                    inside = inside.max((at(y, x + 1, z) - at(y, x, z)).abs());
                    inside = inside.max((at(y, z, x + 1) - at(y, z, x)).abs());
                }
                wrap = wrap.max((at(0, y, z) - at(size - 1, y, z)).abs());
                wrap = wrap.max((at(y, 0, z) - at(y, size - 1, z)).abs());
                wrap = wrap.max((at(y, z, 0) - at(y, z, size - 1)).abs());
            }
        }
        assert!(inside > 0, "the noise is flat");
        assert!(
            wrap <= inside,
            "a step of {wrap} across the wrap against {inside} inside"
        );
    }

    /// The noise spans its range: cell middles near one, cell edges low.
    #[test]
    fn the_cells_texture_spans_its_range() {
        let bytes = bake(32, 4);
        let low = *bytes.iter().min().unwrap();
        let high = *bytes.iter().max().unwrap();
        assert!(high > 200 && low < 100, "range {low}..{high}");
    }
}
