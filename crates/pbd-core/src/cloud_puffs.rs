//! The cloud layer as puffs (`cloud-puffs`): one billboard each, laid out by
//! the weather. A fixed set of sites covers the sphere, stable by index; the
//! cover, cloud top and optical depth the weather shows at a site decide
//! whether it carries a puff, how wide and tall, and how dark. The renderer
//! draws what this returns and keeps no layout of its own.
//!
//! The puff and its look are Crab-breeze's (`cb-core/src/world/clouds.rs`,
//! `assets/shaders/cloud.wesl`, origin/main `d663b86c`), on a sphere instead of
//! a flat map: "up" is a site's own direction.

use glam::Vec3;

/// A place a puff may stand: a direction from the body's centre, and three
/// stable numbers in `0..1` from its index.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Site {
    pub direction: Vec3,
    /// The cover past which it carries a puff.
    pub threshold: f32,
    /// How large its puff is against the others, 0..1.
    pub size: f32,
    /// A number for its noise, 0..100.
    pub seed: f32,
}

/// The weather at a site, as the shown maps give it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// Cloud cover, 0..1.
    pub cover: f32,
    /// How far up the layer the cloud stands, 0..1 of its thickness.
    pub top: f32,
    /// The column's optical depth.
    pub optical_depth: f32,
}

/// The layer the puffs stand in, and how they are made.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layer {
    /// Radius of the cloud base from the body's centre, metres.
    pub base_radius_m: f32,
    /// The tallest a cloud stands above the base, metres.
    pub thickness_m: f32,
    /// The sites' spacing, metres.
    pub spacing_m: f32,
    /// A puff's opacity at full presence, 0..1.
    pub opacity: f32,
    /// The share of sites a full, low deck leaves open: the march's
    /// `cloud_deck_floor`, its lanes in a full cover. Convective cloud keeps
    /// none.
    pub deck_floor: f32,
    /// The optical depth at which a puff is about two thirds opaque: thin
    /// cloud is drawn thin, as the march draws it.
    pub optical_depth_scale: f32,
}

/// One billboard.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Puff {
    /// Its centre, body-local metres.
    pub centre: Vec3,
    /// Its radius across and up, metres.
    pub radius_across_m: f32,
    pub radius_up_m: f32,
    /// 0 white to 1 storm grey.
    pub dark: f32,
    /// How opaque it is drawn, 0..1.
    pub opacity: f32,
    /// The base and top of its column, radii from the body's centre.
    pub base_radius_m: f32,
    pub top_radius_m: f32,
    /// Its site's noise number.
    pub seed: f32,
    /// The cover over its site, 0..1: the sky's light on it (`dusk-clouds`).
    pub cover: f32,
}

/// Half the band of cover over which a puff fades in: a puff appears over a
/// change of 0.12 in cover rather than in one frame.
pub const FADE_HALF: f32 = 0.06;
/// The fewest metres a puff stands.
pub const MIN_HEIGHT_M: f32 = 80.0;
/// A puff taller than this many times its radius across is stacked.
pub const STACK_ASPECT: f32 = 1.4;
/// The most billboards up one column.
pub const MAX_STACK: usize = 3;
/// Below this presence a puff is not drawn.
pub const MIN_PRESENCE: f32 = 0.01;
/// The optical depths over which a column goes from white to storm grey
/// (shown maps, 2026-10-28: median 7, 90th percentile 52, 99th 130).
pub const STORM_OPTICAL_DEPTH: [f32; 2] = [25.0, 90.0];
/// The cloud top (0..1 of the layer) over which a column turns from a deck,
/// which keeps lanes, to convective cloud, which keeps none.
pub const CONVECTIVE_TOP: [f32; 2] = [0.35, 0.7];

/// A stable hash of an index and a lane to `0..1`.
fn hash(index: u32, lane: u32) -> f32 {
    let mut h = index.wrapping_mul(0x9E37_79B9) ^ lane.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// The clump field's two octaves: their cells on the cloud layer, metres, and
/// their weights (`space-clouds` design 3).
pub const CLUMP_CELLS_M: [f32; 2] = [1_100.0, 550.0];
pub const CLUMP_WEIGHTS: [f32; 2] = [0.65, 0.35];
/// How much of a site's carry order and of its size the clump field decides.
pub const CLUMP_CARRY: f32 = 0.55;
pub const CLUMP_SIZE: f32 = 0.5;

/// A stable hash of a lattice point to `0..1`.
fn hash3(x: i32, y: i32, z: i32, lane: u32) -> f32 {
    let i = (x as u32)
        .wrapping_mul(0x8DA6_B343)
        .wrapping_add((y as u32).wrapping_mul(0xD816_3841))
        .wrapping_add((z as u32).wrapping_mul(0xCB1A_B31F));
    hash(i, lane)
}

/// Value noise, 0..1, smoothstepped trilinear on the integer lattice.
fn value_noise(p: Vec3, lane: u32) -> f32 {
    let f = p.floor();
    let t = p - f;
    let s = t * t * (Vec3::splat(3.0) - 2.0 * t);
    let (x, y, z) = (f.x as i32, f.y as i32, f.z as i32);
    let at = |dx: i32, dy: i32, dz: i32| hash3(x + dx, y + dy, z + dz, lane);
    let lerp = |a: f32, b: f32, k: f32| a + (b - a) * k;
    let x00 = lerp(at(0, 0, 0), at(1, 0, 0), s.x);
    let x10 = lerp(at(0, 1, 0), at(1, 1, 0), s.x);
    let x01 = lerp(at(0, 0, 1), at(1, 0, 1), s.x);
    let x11 = lerp(at(0, 1, 1), at(1, 1, 1), s.x);
    lerp(lerp(x00, x10, s.y), lerp(x01, x11, s.y), s.z)
}

/// Where cloud gathers, 0..1: a smooth field over the sphere, about a
/// kilometre across, read at `direction` on a layer of `radius_m`.
pub fn clump(direction: Vec3, radius_m: f32) -> f32 {
    CLUMP_CELLS_M
        .iter()
        .zip(CLUMP_WEIGHTS)
        .enumerate()
        .map(|(k, (cell, w))| w * value_noise(direction * (radius_m / cell), 7 + k as u32))
        .sum()
}

/// Each key's rank among them, as `(rank + 0.5) / n`: exactly uniform over
/// 0..1 whatever the keys' own spread. Ties go by index, so it is stable.
fn ranked(keys: &[f32]) -> Vec<f32> {
    let mut order: Vec<u32> = (0..keys.len() as u32).collect();
    order.sort_by(|&a, &b| {
        keys[a as usize]
            .total_cmp(&keys[b as usize])
            .then(a.cmp(&b))
    });
    let n = keys.len().max(1) as f32;
    let mut out = vec![0.0; keys.len()];
    for (rank, &i) in order.iter().enumerate() {
        out[i as usize] = (rank as f32 + 0.5) / n;
    }
    out
}

/// The sites over a body of `radius_m`, about `spacing_m` apart: a Fibonacci
/// lattice of `4 pi R^2 / s^2` directions, each nudged by up to half the
/// spacing so the lattice's spirals do not show.
///
/// Which sites carry cloud first, and how big their puffs are, is ranked
/// from a mix of each site's own hash and the clump field (`space-clouds`):
/// the thresholds and sizes stay uniform over 0..1, so the share carrying
/// cloud is the cover, but the carriers gather and the big ones stand
/// among them.
pub fn sites(radius_m: f32, spacing_m: f32) -> Vec<Site> {
    let count = (4.0 * std::f32::consts::PI * radius_m * radius_m / (spacing_m * spacing_m))
        .round()
        .max(1.0) as u32;
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    let nudge = spacing_m / radius_m / 2.0;
    let directions: Vec<Vec3> = (0..count)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / count as f32;
            let ring = (1.0 - y * y).max(0.0).sqrt();
            let a = golden * i as f32;
            let d = Vec3::new(ring * a.cos(), y, ring * a.sin());
            let (u, v) = d.any_orthonormal_pair();
            let push = (hash(i, 3) * 2.0 - 1.0) * u + (hash(i, 4) * 2.0 - 1.0) * v;
            (d + push * nudge).normalize()
        })
        .collect();
    // Ranked too, so the field weighs what it is meant to against a hash
    // whatever its own spread.
    let clumps = ranked(
        &directions
            .iter()
            .map(|&d| clump(d, radius_m))
            .collect::<Vec<_>>(),
    );
    let carry: Vec<f32> = (0..count)
        .map(|i| (1.0 - CLUMP_CARRY) * hash(i, 0) + CLUMP_CARRY * (1.0 - clumps[i as usize]))
        .collect();
    let size: Vec<f32> = (0..count)
        .map(|i| (1.0 - CLUMP_SIZE) * hash(i, 1) + CLUMP_SIZE * clumps[i as usize])
        .collect();
    let (thresholds, sizes) = (ranked(&carry), ranked(&size));
    (0..count as usize)
        .map(|i| Site {
            direction: directions[i],
            threshold: thresholds[i],
            size: sizes[i],
            seed: hash(i as u32, 2) * 100.0,
        })
        .collect()
}

/// A puff's radius against the spacing's share, from its site's `size`:
/// 0.5 to 1.8, many small and a few large, with the mean square of the old
/// `0.8 + 0.4 size` (`space-clouds` design 3).
pub fn size_factor(size: f32) -> f32 {
    let u = size.clamp(0.0, 1.0);
    0.5 + 1.3 * u * u
}

/// How much of a puff a site carries under `cover`, 0..1: nothing below its
/// threshold, all of it above, a smooth ramp between; and nothing at all
/// under no cover, however low the threshold.
pub fn presence(site: &Site, cover: f32) -> f32 {
    smoothstep(
        site.threshold - FADE_HALF,
        site.threshold + FADE_HALF,
        cover,
    ) * smoothstep(0.0, 2.0 * FADE_HALF, cover)
}

/// The puffs `site` carries under `look`, pushed onto `out`: none, or one to
/// [`MAX_STACK`] billboards up its column.
pub fn puffs_at(site: &Site, look: Look, layer: &Layer, out: &mut Vec<Puff>) {
    let cover = if look.cover.is_finite() {
        look.cover.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let top = if look.top.is_finite() {
        look.top.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let convective = smoothstep(CONVECTIVE_TOP[0], CONVECTIVE_TOP[1], top);
    let carried = cover * (1.0 - layer.deck_floor.clamp(0.0, 1.0) * (1.0 - convective));
    let a = presence(site, carried);
    if a < MIN_PRESENCE {
        return;
    }
    // About two puffs over a point at full cover: wider, and four or five
    // overlapping puffs stacked to an opaque sheet whatever their opacity.
    let r = layer.spacing_m * (0.5 + 0.35 * cover) * size_factor(site.size);
    // Kept a tenth under the layer's top, so a tower's top billboard stays
    // inside the shell the composite lays cloud over.
    let height = (top * layer.thickness_m * 0.9).max(MIN_HEIGHT_M);
    // One spacing up the column, whatever its height (`cloud-pop`): it was
    // `height / stack`, with the stack a whole number, so as the weather moved
    // the top across a step every billboard of the tower was resized and
    // moved in one frame. The spacing only grows past `MAX_STACK` of them,
    // where the two forms meet.
    let each = (STACK_ASPECT * r).max(height / MAX_STACK as f32);
    let count = (height / each).min(MAX_STACK as f32);
    let stack = (count.ceil() as usize).clamp(1, MAX_STACK);
    let od = if look.optical_depth.is_finite() {
        look.optical_depth.max(0.0)
    } else {
        0.0
    };
    let dark = smoothstep(STORM_OPTICAL_DEPTH[0], STORM_OPTICAL_DEPTH[1], od);
    let thick = 1.0 - (-od / layer.optical_depth_scale.max(1e-3)).exp();
    let base = layer.base_radius_m;
    for k in 0..stack {
        // Each billboard is centred in its share and reaches a tenth past it
        // both ways: the flat cut at the base takes the lowest one's underside
        // and neighbours up a tower overlap. The lowest is whole, and as tall
        // as the column while the column is under one spacing; one above it
        // fades in as the column grows into its share, centred inside the
        // column until it has its whole share.
        let (mid, share, present) = if k == 0 {
            let share = each.min(height);
            (base + share * 0.5, share, 1.0)
        } else {
            let mid = (each * (k as f32 + 0.5)).min(height - each * 0.5);
            (base + mid, each, (count - k as f32).clamp(0.0, 1.0))
        };
        out.push(Puff {
            centre: site.direction * mid,
            // The upper billboards of a tower narrow toward its top.
            radius_across_m: r * (1.0 - 0.15 * k as f32),
            radius_up_m: share * 0.6,
            dark,
            opacity: a * thick * layer.opacity * present,
            base_radius_m: base,
            top_radius_m: base + height,
            seed: site.seed + 17.0 * k as f32,
            cover,
        });
    }
}

/// Whether any of `puff` can show over the sea's sphere of `sea_radius_m`
/// from `eye` (body-local): false when it lies wholly behind the planet's
/// horizon (design 3b). Generous: its top and its full width count, and an
/// eye on or under the sea culls nothing.
pub fn over_horizon(eye: Vec3, puff: &Puff, sea_radius_m: f32) -> bool {
    let e = eye.length();
    if e <= sea_radius_m || !e.is_finite() {
        return true;
    }
    let reach = puff.top_radius_m + puff.radius_across_m;
    let c = puff.centre.length().max(1e-3);
    let angle = (eye.dot(puff.centre) / (e * c)).clamp(-1.0, 1.0).acos();
    let horizon = (sea_radius_m / e).acos() + (sea_radius_m / reach.max(sea_radius_m)).acos();
    angle <= horizon + puff.radius_across_m / sea_radius_m
}

/// How far any corner of `puff`'s billboard can lie from its centre, in
/// every orientation the vertex stage turns it to (`cloud_puff_vertex`): its
/// corners sit at `right * across +- up * h` on an orthonormal pair, with
/// `h` between its radius up and its radius across. Grown by 1% and a metre
/// against roundoff (`puff-frustum-cull`).
pub fn bound_radius(puff: &Puff) -> f32 {
    let across = puff.radius_across_m.abs();
    let up = puff.radius_up_m.abs().max(across);
    (across * across + up * up).sqrt() * 1.01 + 1.0
}

fn smoothstep(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAYER: Layer = Layer {
        base_radius_m: 5_100.0,
        thickness_m: 450.0,
        spacing_m: 250.0,
        opacity: 1.0,
        deck_floor: 0.45,
        optical_depth_scale: 6.0,
    };

    fn look(cover: f32) -> Look {
        Look {
            cover,
            top: 0.8,
            optical_depth: 30.0,
        }
    }

    #[test]
    fn the_same_body_lays_the_same_sites() {
        let a = sites(4_800.0, 250.0);
        let b = sites(4_800.0, 250.0);
        assert_eq!(a, b);
        assert_eq!(a.len(), 4_632);
        assert!(a.iter().all(|s| (s.direction.length() - 1.0).abs() < 1e-5));
    }

    #[test]
    fn more_cover_carries_more_puffs() {
        let sites = sites(4_800.0, 250.0);
        let carried = |cover: f32| {
            sites.iter().filter(|s| presence(s, cover) > 0.5).count() as f32 / sites.len() as f32
        };
        assert!((carried(0.2) - 0.2).abs() < 0.03, "{}", carried(0.2));
        assert!((carried(0.8) - 0.8).abs() < 0.03, "{}", carried(0.8));
        assert_eq!(carried(0.0), 0.0);
    }

    #[test]
    fn a_full_low_deck_keeps_lanes_and_a_tall_one_does_not() {
        let sites = sites(4_800.0, 250.0);
        let carried = |top: f32| {
            let mut out = Vec::new();
            sites
                .iter()
                .filter(|s| {
                    out.clear();
                    let look = Look {
                        cover: 1.0,
                        top,
                        optical_depth: 30.0,
                    };
                    puffs_at(s, look, &LAYER, &mut out);
                    out.first().is_some_and(|p| p.opacity > 0.5)
                })
                .count() as f32
                / sites.len() as f32
        };
        assert!((carried(0.3) - 0.55).abs() < 0.04, "{}", carried(0.3));
        assert!(carried(1.0) > 0.97, "{}", carried(1.0));
    }

    #[test]
    fn thin_cloud_is_drawn_thin_and_a_storm_grey() {
        let site = Site {
            direction: Vec3::Z,
            threshold: 0.0,
            size: 0.5,
            seed: 0.0,
        };
        let one = |od: f32| {
            let mut out = Vec::new();
            let look = Look {
                cover: 1.0,
                top: 1.0,
                optical_depth: od,
            };
            puffs_at(&site, look, &LAYER, &mut out);
            out[0]
        };
        assert!(one(1.0).opacity < 0.2);
        assert!(one(20.0).opacity > 0.9);
        assert_eq!(one(7.0).dark, 0.0);
        assert!(one(100.0).dark > 0.99);
    }

    #[test]
    fn a_puff_fades_in_with_the_cover() {
        let site = Site {
            direction: Vec3::Y,
            threshold: 0.5,
            size: 0.5,
            seed: 1.0,
        };
        let steps: Vec<f32> = (0..=100)
            .map(|i| {
                let mut out = Vec::new();
                puffs_at(&site, look(i as f32 / 100.0), &LAYER, &mut out);
                out.first().map_or(0.0, |p| p.opacity)
            })
            .collect();
        let between = steps.iter().filter(|&&o| o > 0.05 && o < 0.95).count();
        assert!(between >= 5, "only {between} intermediate steps");
        assert!(steps.windows(2).all(|w| w[1] - w[0] < 0.3));
    }

    #[test]
    fn a_tall_cloud_is_a_stack_on_a_flat_base() {
        let site = Site {
            direction: Vec3::X,
            threshold: 0.0,
            size: 0.0,
            seed: 3.0,
        };
        let mut low = Vec::new();
        puffs_at(
            &site,
            Look {
                cover: 0.3,
                top: 0.1,
                optical_depth: 0.0,
            },
            &LAYER,
            &mut low,
        );
        let mut tall = Vec::new();
        puffs_at(
            &site,
            Look {
                cover: 0.3,
                top: 1.0,
                optical_depth: 100.0,
            },
            &LAYER,
            &mut tall,
        );
        assert_eq!(low.len(), 1);
        assert!(tall.len() > 1 && tall.len() <= MAX_STACK);
        assert!(tall.iter().all(|p| p.base_radius_m == LAYER.base_radius_m));
        assert!(
            tall.windows(2)
                .all(|w| w[1].centre.length() > w[0].centre.length())
        );
        assert!(tall[0].dark > 0.99 && low[0].dark == 0.0);
    }

    /// `cloud-pop`: as the weather moves a site's top a little each frame,
    /// no billboard of its tower jumps. Matched by their place up the column;
    /// one present on only one side of a step must be all but transparent.
    #[test]
    fn a_tower_grows_and_shrinks_without_re_forming() {
        let site = Site {
            direction: Vec3::X,
            threshold: 0.0,
            size: 0.5,
            seed: 1.0,
        };
        let at = |top: f32| {
            let mut out = Vec::new();
            puffs_at(
                &site,
                Look {
                    cover: 0.6,
                    top,
                    optical_depth: 40.0,
                },
                &LAYER,
                &mut out,
            );
            out
        };
        let full = at(1.0)[0].opacity;
        let steps = 2000;
        let mut before = at(0.0);
        for i in 1..=steps {
            let top = i as f32 / steps as f32;
            let now = at(top);
            for k in 0..before.len().max(now.len()) {
                match (before.get(k), now.get(k)) {
                    (Some(a), Some(b)) => {
                        let moved = a.centre.distance(b.centre);
                        assert!(moved < 5.0, "billboard {k} jumped {moved} m at top {top}");
                        for (x, y, what) in [
                            (a.radius_across_m, b.radius_across_m, "across"),
                            (a.radius_up_m, b.radius_up_m, "up"),
                        ] {
                            assert!(
                                (x - y).abs() < 5.0,
                                "billboard {k}'s {what} jumped {x} -> {y} at top {top}"
                            );
                        }
                        assert!(
                            (a.opacity - b.opacity).abs() < 0.02 * full,
                            "billboard {k}'s opacity jumped {} -> {} at top {top}",
                            a.opacity,
                            b.opacity
                        );
                    }
                    (Some(p), None) | (None, Some(p)) => assert!(
                        p.opacity < 0.02 * full,
                        "billboard {k} came or went at opacity {} at top {top}",
                        p.opacity
                    ),
                    (None, None) => {}
                }
            }
            before = now;
        }
    }

    #[test]
    fn a_puff_past_the_horizon_is_hidden_and_one_near_it_is_not() {
        let sea = 4_800.0;
        let site = |direction: Vec3| Site {
            direction,
            threshold: 0.0,
            size: 0.5,
            seed: 0.0,
        };
        let one = |direction: Vec3| {
            let mut out = Vec::new();
            puffs_at(&site(direction), look(1.0), &LAYER, &mut out);
            out[0]
        };
        let low_eye = Vec3::Y * (sea + 2.0);
        assert!(over_horizon(low_eye, &one(Vec3::Y), sea));
        assert!(!over_horizon(low_eye, &one(-Vec3::Y), sea));
        assert!(!over_horizon(low_eye, &one(Vec3::X), sea));
        // About 2.5 km out over the sea from 2 m up: a cloud there stands
        // over the horizon.
        let near = Vec3::new(0.5, 1.0, 0.0).normalize();
        assert!(over_horizon(low_eye, &one(near), sea));
        // From orbit, the whole near hemisphere and the limb.
        let orbit = Vec3::Y * 20_000.0;
        assert!(over_horizon(orbit, &one(Vec3::X), sea));
        assert!(!over_horizon(orbit, &one(-Vec3::Y), sea));
        // Under the sea nothing is culled.
        assert!(over_horizon(Vec3::Y * 4_700.0, &one(-Vec3::Y), sea));
    }

    #[test]
    fn every_billboard_corner_lies_inside_the_bound() {
        let mut out = Vec::new();
        for (i, site) in sites(4_800.0, 250.0).iter().take(400).enumerate() {
            puffs_at(site, look(0.3 + (i % 7) as f32 * 0.1), &LAYER, &mut out);
        }
        assert!(!out.is_empty());
        for (i, puff) in out.iter().enumerate() {
            let bound = bound_radius(puff);
            // The vertex stage's up runs from the radius up to the radius
            // across as the view turns down onto the puff.
            for k in [0.0f32, 0.3, 0.7, 1.0] {
                let up = puff.radius_up_m + (puff.radius_across_m - puff.radius_up_m) * k * k;
                let turn = i as f32 * 0.37 + k;
                let right = Vec3::new(turn.cos(), 0.0, turn.sin());
                let upward = Vec3::new(0.0, 1.0, 0.0);
                for (x, y) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                    let corner = right * (x * puff.radius_across_m) + upward * (y * up);
                    assert!(corner.length() < bound, "{} >= {bound}", corner.length());
                }
            }
        }
    }

    /// `space-clouds`: under one cover the puffs run from small to large,
    /// not all of a size.
    #[test]
    fn puffs_come_in_many_sizes() {
        let mut radii: Vec<f32> = sites(4_800.0, 250.0)
            .iter()
            .filter_map(|s| {
                let mut out = Vec::new();
                let site = Site {
                    threshold: 0.0,
                    ..*s
                };
                puffs_at(&site, look(0.6), &LAYER, &mut out);
                out.first().map(|p| p.radius_across_m)
            })
            .collect();
        radii.sort_by(f32::total_cmp);
        let at = |q: f32| radii[((radii.len() - 1) as f32 * q) as usize];
        let spread = at(0.9) / at(0.1);
        eprintln!("p90/p10 radius {spread}");
        assert!(spread > 2.5, "p90/p10 {spread}");
    }

    /// `space-clouds`: under a partial cover the carrying sites gather, and
    /// the share carrying still follows the cover.
    #[test]
    fn puffs_gather_in_clumps() {
        let sites = sites(4_800.0, 250.0);
        let cover = 0.3;
        let carried: Vec<bool> = sites.iter().map(|s| presence(s, cover) > 0.5).collect();
        let share = carried.iter().filter(|&&c| c).count() as f32 / sites.len() as f32;
        assert!((share - cover).abs() < 0.03, "{share}");
        // Each carrier's six nearest sites: how many carry too.
        let (mut together, mut seen) = (0usize, 0usize);
        for (i, s) in sites.iter().enumerate().filter(|(i, _)| carried[*i]) {
            let mut near: Vec<(f32, usize)> = sites
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(j, t)| (-s.direction.dot(t.direction), j))
                .collect();
            near.select_nth_unstable_by(6, |a, b| a.0.total_cmp(&b.0));
            for &(_, j) in &near[..6] {
                seen += 1;
                together += carried[j] as usize;
            }
        }
        let gathered = together as f32 / seen as f32;
        eprintln!("neighbours carrying {gathered}");
        assert!(gathered > 0.45, "neighbours carrying {gathered}");
    }

    #[test]
    fn no_cover_and_bad_numbers_carry_nothing() {
        let site = sites(4_800.0, 250.0)[7];
        let mut out = Vec::new();
        puffs_at(&site, look(0.0), &LAYER, &mut out);
        puffs_at(&site, look(f32::NAN), &LAYER, &mut out);
        assert!(out.is_empty());
    }
}
