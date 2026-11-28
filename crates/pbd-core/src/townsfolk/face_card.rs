//! A face far off is one baked card (`townsfolk` design "Faces far off are
//! one baked card", tasks 15.1 to 15.3; owner 2026-11-28: "face approved").
//!
//! Near, a face is a stack of pieces on the head's front, each a few
//! millimetres proud of the one under it; far off the depth buffer cannot
//! tell them apart and they fight. So past [`FACE_CARD_M`] the pieces give
//! way to one card: the person's expression drawn flat into a tile of
//! [`TILE_PX`] texels, in the same layout and colours the pieces have
//! ([`bake_tile`]), one piece at one depth. The two cross-fade by dither
//! over [`FACE_FADE_S`] when a person crosses the line ([`FaceLod`]).
//!
//! Everything here is drawn only: nothing reads it.

use super::face::{self, Expression, FSC, FacePose, Mouth};
use super::figure::{self, Figure};
use super::look::{self, Look};

/// Past this from the eye a face is its card, m (the mockup's default).
pub const FACE_CARD_M: f32 = 20.0;
/// How long the cross-fade between the pieces and the card runs, s.
pub const FACE_FADE_S: f32 = 0.3;
/// The band round [`FACE_CARD_M`] that a person must cross before the face
/// changes, m: half of it each side, so a person standing on the line does
/// not flip back and forth.
pub const FACE_BAND_M: f32 = 1.0;
/// A tile's texels across and down.
pub const TILE_PX: usize = 32;
/// How far the card stands proud of the head's front, m.
pub const CARD_PROUD_M: f32 = 0.01;
/// The card's area on the head's front, in the head's own space (the
/// mockup's `CW`, `CY0`, `CY1`): the front's width inside its bevels, and
/// from just over the chin to just under the hair line, m.
pub const CARD_W: f32 = figure::HW - 2.0 * figure::HC;
pub const CARD_Y: [f32; 2] = [0.02, 0.48];

/// An expression's face at rest, as the bake draws it: its own numbers,
/// eyes open and looking ahead, the head straight, no blink and no talk
/// (a talking face shows its expression's own mouth).
pub fn resting_pose(expression: Expression) -> FacePose {
    let p = expression.params();
    let brow = |s: f32| FSC * (p.by + if s > 0.0 { p.asym } else { 0.0 });
    let tilt = |s: f32| -s * p.bt + s * 0.06;
    FacePose {
        eye: p.eye,
        pup: p.pup,
        pupil: [0.0, 0.0],
        lid: p.lid,
        brow_y: [brow(-1.0), brow(1.0)],
        brow_tilt: [tilt(-1.0), tilt(1.0)],
        mouth: p.mouth,
        mouth_x: p.mx * FSC,
        blush: p.blush,
        arcs: p.arcs,
        head_yaw: 0.0,
        head_pitch: 0.0,
        tilt: 0.0,
    }
}

/// What of a person the bake reads: where their eyes sit and how big, and
/// the colours of their lids, brows and blush.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardLook {
    pub gap: f32,
    pub eye: f32,
    pub skin: u32,
    pub brow: u32,
    pub blush: [u32; 3],
}

impl CardLook {
    pub fn of(look: &Look, figure: &Figure) -> Self {
        Self {
            gap: look.gap,
            eye: look.eye,
            skin: figure.skin,
            brow: figure.brow,
            blush: figure.blush,
        }
    }
}

/// Whether `p` (in a regular hexagon's own space, radius one, a corner on
/// +x as [`figure::disc`] lays it) is inside it.
fn in_hexagon(p: [f32; 2]) -> bool {
    let c30 = 3f32.sqrt() / 2.0;
    [(c30, 0.5), (0.0, 1.0), (-c30, 0.5)]
        .iter()
        .all(|(nx, ny)| (p[0] * nx + p[1] * ny).abs() <= c30)
}

/// A mouth atlas cell's texel under `p`, on a `w` x `h` quad centred at
/// `c`, as the kit's `cell_mesh` lays it (the cell's top row at the top).
fn cell_at(
    cell: Mouth,
    atlas: &[[u8; 4]],
    c: [f32; 2],
    w: f32,
    h: f32,
    p: [f32; 2],
) -> Option<u32> {
    let u = (p[0] - c[0]) / w + 0.5;
    let v = 0.5 - (p[1] - c[1]) / h;
    if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
        return None;
    }
    let (x, y) = (
        (u * face::CELL_W as f32) as usize,
        (v * face::CELL_H as f32) as usize,
    );
    let k = cell.cell();
    let (ox, oy) = ((k % 4) * face::CELL_W, (k / 4) * face::CELL_H);
    let [r, g, b, a] = atlas[(oy + y) * face::ATLAS_W + ox + x];
    (a > 127).then_some(u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b))
}

/// A face drawn flat into a tile of `px` by `px` texels over [`CARD_W`] by
/// [`CARD_Y`], rows from the top: each texel the colour (`0xRRGGBB` sRGB)
/// of the topmost piece over its middle, or none where no piece is. The
/// pieces, their places, sizes and colours are the kit's (`pose_parts` in
/// the game's `townsfolk`), drawn in the order they stand off the head:
/// blush, mouth, whites, brows, happy arcs, pupils, catch-lights, lids,
/// lash lines.
pub fn bake_tile(pose: &FacePose, who: &CardLook, px: usize) -> Vec<Option<u32>> {
    let atlas = face::mouth_atlas();
    let (wr, pr) = (figure::WR, figure::PR);
    let es = pose.eye * who.eye;
    let lh = 1.83 * wr * pose.lid;
    let ly = 0.913 * wr - lh + 0.0008;
    let lash_x = ((2.0 * (wr - ly.abs() / 1.732) + 0.01) / (2.04 * wr)).max(0.05);
    let blush_level = (pose.blush > 0.04).then(|| {
        look::BLUSH_LEVELS
            .iter()
            .enumerate()
            .min_by(|a, b| {
                (a.1 - pose.blush)
                    .abs()
                    .total_cmp(&(b.1 - pose.blush).abs())
            })
            .map_or(0, |(k, _)| k)
    });
    let sides = [(0usize, -1.0f32), (1, 1.0)];
    let eye_at = |s: f32| [s * who.gap, figure::EYE_Y];
    // In an eye's own space: its centre at the origin, unscaled.
    let in_eye = |s: f32, p: [f32; 2]| {
        let e = eye_at(s);
        [(p[0] - e[0]) / es, (p[1] - e[1]) / es]
    };
    let open = !pose.arcs;
    let texel = |p: [f32; 2]| -> Option<u32> {
        let mut out = None;
        // The blush, at the level nearest the face's.
        if let Some(k) = blush_level {
            for (_, s) in sides {
                let q = [(p[0] - s * 0.155) / 0.052, (p[1] - 0.178) / 0.029];
                if in_hexagon(q) {
                    out = Some(who.blush[k]);
                }
            }
        }
        // The mouth cell.
        let [mw, mh] = figure::MOUTH_WH;
        if let Some(c) = cell_at(
            pose.mouth,
            &atlas,
            [pose.mouth_x, figure::MOUTH_Y],
            mw,
            mh,
            p,
        ) {
            out = Some(c);
        }
        for (i, s) in sides {
            let q = in_eye(s, p);
            // The white.
            if open && in_hexagon([q[0] / wr, q[1] / wr]) {
                out = Some(figure::WHITE);
            }
            // The brow: a bar turned by its tilt.
            let by = figure::BROW_Y + pose.brow_y[i] + (es - 1.0) * 0.065;
            let (dx, dy) = (p[0] - s * (who.gap + 0.01), p[1] - by);
            let (sn, cs) = (-pose.brow_tilt[i]).sin_cos();
            let (bx, byy) = (dx * cs - dy * sn, dx * sn + dy * cs);
            if bx.abs() <= 0.05 && byy.abs() <= 0.013 {
                out = Some(who.brow);
            }
            // The happy arcs in place of the eyes.
            if pose.arcs
                && let Some(c) = cell_at(Mouth::Happy, &atlas, [0.0, 0.077 * wr], 2.0 * wr, wr, q)
            {
                out = Some(c);
            }
            if open {
                // The pupil and its catch-light, the pupil slid by the gaze.
                let pg = [
                    pose.pupil[0] * (0.98 * wr - pr * pose.pup),
                    pose.pupil[1] * (0.846 * wr - 0.87 * pr * pose.pup),
                ];
                let r = pr * pose.pup;
                if in_hexagon([(q[0] - pg[0]) / r, (q[1] - pg[1]) / r]) {
                    out = Some(figure::PUPIL);
                }
                let g = [pg[0] - 0.017, pg[1] + 0.018];
                if in_hexagon([(q[0] - g[0]) / figure::GR, (q[1] - g[1]) / figure::GR]) {
                    out = Some(figure::GLINT);
                }
                // The lid, down from the white's top by its share.
                if lh > 0.003
                    && q[0].abs() <= 1.095 * wr
                    && q[1] <= 0.913 * wr
                    && q[1] >= 0.913 * wr - lh
                {
                    out = Some(who.skin);
                }
                // The lash line along the lid's edge.
                if q[0].abs() <= 1.02 * wr * lash_x && (q[1] - ly).abs() <= 0.005 {
                    out = Some(figure::LASH);
                }
            }
        }
        out
    };
    let (sx, sy) = (CARD_W / px as f32, (CARD_Y[1] - CARD_Y[0]) / px as f32);
    let mut tile = Vec::with_capacity(px * px);
    for j in 0..px {
        for i in 0..px {
            tile.push(texel([
                -CARD_W / 2.0 + (i as f32 + 0.5) * sx,
                CARD_Y[1] - (j as f32 + 0.5) * sy,
            ]));
        }
    }
    tile
}

/// Which face a drawn person shows, near pieces or far card, and the fade
/// between them: a fade starts when the person crosses [`FACE_CARD_M`] by
/// half of [`FACE_BAND_M`] and runs its [`FACE_FADE_S`] to the end, whatever
/// the person does meanwhile (CLAUDE.md, "A fade is a moment in time").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceLod {
    far: bool,
    /// A fade under way: whether toward the card, and its seconds so far.
    fade: Option<(bool, f32)>,
}

impl FaceLod {
    /// A face first drawn at `distance_m`: whichever side of the line it
    /// is on, with no fade (the figure fades in whole).
    pub fn new(distance_m: f32) -> Self {
        Self {
            far: distance_m > FACE_CARD_M,
            fade: None,
        }
    }

    /// One frame of `dt` s with the eye `distance_m` off: the card's share,
    /// 0 the pieces only to 1 the card only.
    pub fn step(&mut self, dt: f32, distance_m: f32) -> f32 {
        if let Some((to_far, t)) = &mut self.fade {
            *t += dt;
            if *t >= FACE_FADE_S {
                self.far = *to_far;
                self.fade = None;
            }
        }
        let h = FACE_BAND_M / 2.0;
        let want = if distance_m > FACE_CARD_M + h {
            true
        } else if distance_m < FACE_CARD_M - h {
            false
        } else {
            self.fade.map_or(self.far, |(to_far, _)| to_far)
        };
        if self.fade.is_none() && want != self.far {
            self.fade = Some((want, 0.0));
        }
        self.card()
    }

    /// The card's share now.
    pub fn card(&self) -> f32 {
        match self.fade {
            Some((to_far, t)) => {
                let k = (t / FACE_FADE_S).clamp(0.0, 1.0);
                if to_far { k } else { 1.0 - k }
            }
            None if self.far => 1.0,
            None => 0.0,
        }
    }

    /// Whether a fade is under way.
    pub fn fading(&self) -> bool {
        self.fade.is_some()
    }

    /// Whether the face is (or is fading to) its card.
    pub fn far(&self) -> bool {
        self.fade.map_or(self.far, |(to_far, _)| to_far)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn who() -> CardLook {
        let l = look::generate(look::FIELDS, 1, 0, look::FIELDS.cloth[0]);
        CardLook::of(&l, &Figure::of(&l))
    }

    /// 15.1: every expression's tile is drawn, in the face's own colours,
    /// and differs from Neutral's.
    #[test]
    fn every_expression_bakes_its_own_tile() {
        let who = who();
        let neutral = bake_tile(&resting_pose(Expression::Neutral), &who, TILE_PX);
        for e in Expression::ALL {
            let tile = bake_tile(&resting_pose(e), &who, TILE_PX);
            assert_eq!(tile.len(), TILE_PX * TILE_PX);
            let drawn = tile.iter().flatten().count();
            assert!(drawn > 40, "{e:?} drew {drawn} texels");
            assert!(drawn < TILE_PX * TILE_PX / 2, "{e:?} is mostly face");
            if e != Expression::Neutral {
                assert_ne!(tile, neutral, "{e:?} is Neutral's");
            }
            let open = !e.params().arcs;
            assert_eq!(
                tile.contains(&Some(figure::WHITE)),
                open,
                "{e:?}: whites only on open eyes"
            );
            assert!(tile.contains(&Some(figure::PUPIL)) || !open, "{e:?}");
            assert!(tile.contains(&Some(who.brow)), "{e:?} has brows");
            // Every colour it draws is one the atlas has a swatch for.
            let swatches = look::swatches([]);
            for c in tile.iter().flatten() {
                assert!(swatches.contains(c), "{e:?}: {c:06x} has no swatch");
            }
        }
        // Happy blushes; Neutral does not.
        let happy = bake_tile(&resting_pose(Expression::Happy), &who, TILE_PX);
        assert!(happy.iter().flatten().any(|c| who.blush.contains(c)));
        assert!(!neutral.iter().flatten().any(|c| who.blush.contains(c)));
        // The two eyes mirror each other on a straight face.
        for j in 0..TILE_PX {
            for i in 0..TILE_PX / 2 {
                let (a, b) = (
                    neutral[j * TILE_PX + i],
                    neutral[j * TILE_PX + TILE_PX - 1 - i],
                );
                // The catch-light sits up and left on both, so only the
                // eye's outline mirrors.
                let eye = |c: Option<u32>| {
                    c.is_some_and(|c| [figure::WHITE, figure::PUPIL, figure::GLINT].contains(&c))
                };
                assert_eq!(eye(a), eye(b), "row {j}, column {i}");
            }
        }
    }

    /// A look at the tiles, for whoever is changing the bake:
    /// `cargo test -p pbd-core face_card::tests::show -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn show() {
        let who = who();
        for e in Expression::ALL {
            println!("{e:?}");
            let tile = bake_tile(&resting_pose(e), &who, TILE_PX);
            for row in tile.chunks(TILE_PX) {
                let line: String = row
                    .iter()
                    .map(|c| match *c {
                        None => '.',
                        Some(figure::WHITE) => 'o',
                        Some(figure::PUPIL) => '#',
                        Some(figure::GLINT) => '*',
                        Some(figure::LASH) => '-',
                        Some(c) if c == who.skin => 's',
                        Some(c) if c == who.brow => '=',
                        Some(c) if who.blush.contains(&c) => 'b',
                        Some(_) => 'm',
                    })
                    .collect();
                println!("{line}");
            }
        }
    }

    /// The bake's pose is the face's own at rest: what `Face` settles to
    /// with the eyes ahead and no blink.
    #[test]
    fn the_resting_pose_is_the_faces_own() {
        for e in Expression::ALL {
            let p = resting_pose(e);
            let q = e.params();
            assert_eq!(
                (p.eye, p.pup, p.lid, p.blush),
                (q.eye, q.pup, q.lid, q.blush)
            );
            assert_eq!(p.pupil, [0.0, 0.0]);
            assert_eq!(p.mouth, q.mouth);
        }
    }

    /// 15.3: a crossing starts one fade that ends after the fade time,
    /// whatever the person does after; inside the band nothing flips; and
    /// crossing back starts the reverse fade.
    #[test]
    fn a_crossing_fades_once_and_the_band_holds() {
        let dt = 1.0 / 60.0;
        let mut lod = FaceLod::new(10.0);
        assert_eq!(lod.step(dt, 10.0), 0.0, "near: the pieces");
        // Inside the band on the far side: no change.
        for _ in 0..120 {
            assert_eq!(lod.step(dt, FACE_CARD_M + 0.4), 0.0);
        }
        assert!(!lod.fading());
        // Out past the band: a fade toward the card starts.
        let first = lod.step(dt, FACE_CARD_M + 0.6);
        assert!(lod.fading() && lod.far());
        assert!(first < 0.1, "{first}");
        // Back in at once: the fade still runs to its end, rising every
        // frame, and only then does the reverse start.
        let (mut frames, mut last) = (1, first);
        loop {
            let s = lod.step(dt, 5.0);
            frames += 1;
            assert!(s >= last, "{s} after {last}: the fade is not turned back");
            last = s;
            if s >= 1.0 {
                break;
            }
            assert!(frames < 40, "the fade ends");
        }
        let ran = (frames - 1) as f32 * dt;
        assert!(
            (ran - FACE_FADE_S).abs() <= 1.5 * dt,
            "it ran its time: {ran}"
        );
        assert!(lod.fading() && !lod.far(), "the reverse fade has begun");
        // The reverse falls to the pieces over its own fade time, and ends.
        let mut back = 0;
        while lod.fading() {
            let s = lod.step(dt, 5.0);
            assert!(s <= last, "{s} after {last}");
            last = s;
            back += 1;
            assert!(back < 40, "the reverse ends");
        }
        assert_eq!(lod.card(), 0.0);
        assert!(((back - 1) as f32 * dt - FACE_FADE_S).abs() <= 1.5 * dt);
        // Standing in the band on the near side does not flip it either.
        for _ in 0..120 {
            assert_eq!(lod.step(dt, FACE_CARD_M - 0.4), 0.0);
        }
        // A face first drawn far off is the card, with no fade.
        let far = FaceLod::new(40.0);
        assert_eq!(far.card(), 1.0);
        assert!(!far.fading());
    }
}
