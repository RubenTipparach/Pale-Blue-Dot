//! The ground clutter a hand takes (`gather-by-hand`): the pieces on the
//! resident fine set's cells, the one the eye ray meets, and the list of
//! taken pieces near the camera the draw leaves out.
//!
//! Where a piece stands is `pbd_core::scatter`, the draw's own rule; what a
//! player took is the save's `clutter` records. This module only joins the
//! two to the cells the fine set holds.

use crate::config::ScatterSettings;
use crate::planet::{GpuCell, PLANET_RADIUS, PlanetFine, PlanetRenderFrame};
use crate::saves::WorldSave;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use pbd_core::daylight::DAY_S;
use pbd_core::scatter::{self, CLUTTER_KIND, Kind, Odds, Piece, Taken};

/// The odds the draw reads, from `scatter.ron` as the planet packs them.
pub fn odds(s: &ScatterSettings) -> Odds {
    Odds {
        rock_chance: s.rock_chance,
        rock_size_m: s.rock_size_m,
        bush_chance: s.bush_chance,
        bush_size_m: s.bush_size_m,
        shrub_chance: s.shrub_chance,
        shrub_size_m: s.shrub_size_m,
        fern_chance: s.fern_chance,
    }
}

/// The pieces on one of the fine set's records, as the draw rolls them.
pub fn pieces_of(record: &GpuCell, odds: &Odds) -> Vec<Piece> {
    let degree = record.degree().clamp(1, 6);
    let corners: Vec<Vec3> = record.corners[..degree]
        .iter()
        .map(|c| Vec3::new(c[0], c[1], c[2]))
        .collect();
    let cell = scatter::Cell {
        id: record.metadata[3],
        material: record.metadata[1] & 0xff,
        biome: ((record.metadata[1] >> 8) & 0xff).min(7) as usize,
        axis: Vec3::from_slice(&record.direction_height[..3]),
        corners: &corners,
        radius: PLANET_RADIUS + record.direction_height[3],
    };
    scatter::pieces(&cell, odds)
}

/// What a hand took from a cell, by its key.
pub fn taken_at(save: &WorldSave, key: u32) -> Taken {
    save.records
        .get(CLUTTER_KIND, u64::from(key))
        .and_then(|r| r.read::<Taken>())
        .unwrap_or_default()
}

/// The world's day, as the regrowth counts it.
pub fn day_of(seconds: f64) -> f64 {
    seconds / f64::from(DAY_S)
}

/// A piece the eye ray meets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClutterHit {
    /// The cell's key, which the record is stored by.
    pub key: u32,
    pub kind: Kind,
    /// How far along the ray, m.
    pub distance: f32,
}

/// The nearest piece still standing that the ray `eye + look * t` passes
/// within its reach of, inside the hands' reach.
pub fn clutter_on_ray(
    fine: &PlanetFine,
    save: &WorldSave,
    odds: &Odds,
    day: f64,
    eye: Vec3,
    look: Vec3,
) -> Option<ClutterHit> {
    let reach = pbd_core::aim::REACH_M;
    let look = look.normalize_or_zero();
    let toward = eye.normalize_or(Vec3::Y);
    let near = ((reach + 3.0) / PLANET_RADIUS).cos();
    let mut best: Option<ClutterHit> = None;
    for record in fine.set.finest_records() {
        let up = Vec3::from_slice(&record.direction_height[..3]);
        if up.dot(toward) < near * up.length() {
            continue;
        }
        let pieces = pieces_of(record, odds);
        if pieces.is_empty() {
            continue;
        }
        let taken = taken_at(save, record.key());
        for piece in pieces {
            if taken.gone(piece.kind, day) {
                continue;
            }
            let rel = piece.centre - eye;
            let t = rel.dot(look);
            if !(0.0..=reach).contains(&t) {
                continue;
            }
            if (rel - look * t).length() > piece.reach {
                continue;
            }
            if best.is_none_or(|b| t < b.distance) {
                best = Some(ClutterHit {
                    key: record.key(),
                    kind: piece.kind,
                    distance: t,
                });
            }
        }
    }
    best
}

/// How many cells with a taken piece the uniform carries.
pub const TAKEN_CELLS: usize = 32;

/// The cells near the camera with a piece taken or growing back, nearest
/// first: each its clutter id and four bytes, one a piece in [`Kind`]
/// order, 0 drawn whole to 255 sunk away.
#[derive(Resource, Clone, Default, ExtractResource, PartialEq, Debug)]
pub struct ClutterTaken(pub Vec<(u32, u32)>);

impl ClutterTaken {
    /// The list as the uniform packs it: two cells a lane, an id then its
    /// bytes; an id of nought is an empty slot.
    pub fn packed(&self) -> [UVec4; TAKEN_CELLS / 2] {
        let mut out = [UVec4::ZERO; TAKEN_CELLS / 2];
        for (i, (id, sunk)) in self.0.iter().take(TAKEN_CELLS).enumerate() {
            out[i / 2][(i % 2) * 2] = *id;
            out[i / 2][(i % 2) * 2 + 1] = *sunk;
        }
        out
    }
}

/// A cell's four sink bytes on day `day`.
pub fn sunk_bytes(taken: &Taken, day: f64, fade_days: f64) -> u32 {
    Kind::ALL.iter().fold(0, |bytes, kind| {
        let s = (taken.sunk(*kind, day, fade_days) * 255.0).round() as u32;
        bytes | s.min(255) << (8 * *kind as u32)
    })
}

/// Rebuild the list every frame a piece is sinking or rising, and twice a
/// second otherwise: the `clutter` records matched to the resident fine
/// set's cells, nearest the camera first.
#[allow(clippy::too_many_arguments)]
pub(crate) fn refresh(
    time: Res<Time>,
    mut wait: Local<f32>,
    save: Option<Res<WorldSave>>,
    fine: Option<Res<PlanetFine>>,
    frame: Res<PlanetRenderFrame>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    sun: Option<Res<crate::sky::Sun>>,
    scatter: Option<Res<ScatterSettings>>,
    mut taken: ResMut<ClutterTaken>,
) {
    let moving = taken
        .0
        .iter()
        .any(|(_, s)| (0..4).any(|k| !matches!((s >> (8 * k)) & 0xff, 0 | 255)));
    *wait -= time.delta_secs();
    if *wait > 0.0 && !moving {
        return;
    }
    *wait = 0.5;
    let (Some(save), Some(fine)) = (save, fine) else {
        return;
    };
    let day = sun.as_ref().map_or(0.0, |s| day_of(s.clock.seconds));
    // The fade's seconds, as game days at the clock's own pace (a sped-up
    // clock fades faster, which only a captioned wait does).
    let fade_s = scatter.as_ref().map_or(1.0, |s| s.lod_fade_s.max(0.05));
    let fade_days = f64::from(fade_s) / f64::from(DAY_S);
    let mut by_key = std::collections::HashMap::new();
    for r in save.records.of_kind(CLUTTER_KIND) {
        if let Some(t) = r.read::<Taken>() {
            let bytes = sunk_bytes(&t, day, fade_days);
            if bytes != 0 {
                by_key.insert(r.id as u32, bytes);
            }
        }
    }
    if by_key.is_empty() {
        if !taken.0.is_empty() {
            taken.0.clear();
        }
        return;
    }
    let eye = cameras
        .iter()
        .next()
        .map(|t| (t.translation().as_dvec3() - frame.center).as_vec3())
        .unwrap_or(Vec3::Y * PLANET_RADIUS);
    let up = eye.normalize_or(Vec3::Y);
    let mut near: Vec<(f32, u32, u32)> = fine
        .set
        .finest_records()
        .iter()
        .filter_map(|r| {
            let bytes = *by_key.get(&r.key())?;
            let d = Vec3::from_slice(&r.direction_height[..3]).normalize_or(Vec3::Y);
            Some((1.0 - d.dot(up), r.metadata[3], bytes))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let next = ClutterTaken(
        near.into_iter()
            .take(TAKEN_CELLS)
            .map(|(_, id, bytes)| (id, bytes))
            .collect(),
    );
    if *taken != next {
        *taken = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The list packs two cells a lane, nearest first; a cell's bytes say
    /// each piece's sink, and a cell with nothing taken has none.
    #[test]
    fn the_list_packs_two_cells_a_lane_with_a_byte_a_piece() {
        let list = ClutterTaken(vec![(11, 0xff), (22, 0xff00), (33, 1)]);
        let p = list.packed();
        assert_eq!(p[0], UVec4::new(11, 0xff, 22, 0xff00));
        assert_eq!(p[1], UVec4::new(33, 1, 0, 0));
        let mut t = Taken::default();
        assert_eq!(sunk_bytes(&t, 1.0, 0.01), 0);
        t.take(Kind::Bush, 1.0);
        assert_eq!(sunk_bytes(&t, 2.0, 0.01), 0xff << 8, "the bush sunk away");
        assert_eq!(sunk_bytes(&t, 5.0, 0.01), 0, "grown back");
    }
}
