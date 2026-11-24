//! The building planner (`building-planner` design 3, M6's Planner tab and
//! version 3): a plan drawn a storey at a time on a hex grid, top down or
//! isometric with its walls cut away, checked by build mode's own rules as it
//! is drawn, with its bill of materials, the presets and the player's own
//! blueprints beside it. Opened with F at a drafting table or with O in build
//! mode; aimed at a site, O opens that site's plan to change it.
//!
//! The plan view is painted on the CPU into one image ([`paint_plan`]) when
//! the plan, the storey, the view or the hover changes, and a click is mapped
//! back through the same projection ([`View::to_plan`]).

use super::menu::{self, Panel, Screen};
use super::slots::{Hotbar, ItemIcons, paint_icon};
use bevy::asset::RenderAssetUsages;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::RelativeCursorPosition;
use pbd_app::construction::{PlannerData, Sites, good_name, store_blueprint};
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::{self, KitText, Localized, Plaque, Set, UiKit, tr, tr_with};
use pbd_core::building::plan::{Aim, Hex, Plan, PlanPiece, Tool, corner, distance, middle, step};
use pbd_core::building::planner::{Check, Issue, check_plan, on_flat};
use pbd_core::building::site::{BlueprintBody, Size, blueprints};
use pbd_core::building::{Opening, RoofShape};
use pbd_core::furniture::Kind;
use pbd_core::inventory::Stack;

/// The plan view's size on the panel, native UI pixels, and the image's:
/// painted at twice the panel's pixels so it stays sharp at a UI scale of 2.
pub const VIEW_W: f32 = 340.0;
pub const VIEW_H: f32 = 250.0;
const IMG_SCALE: u32 = 2;

/// The tools, in the row's order and the number keys'.
pub const TOOLS: [&str; 8] = [
    "room", "wall", "door", "window", "stair", "lantern", "slot", "erase",
];
/// The furniture a slot can be marked for, as M6 lists them.
pub const SLOT_KINDS: [Kind; 5] = [
    Kind::Bed,
    Kind::Table,
    Kind::Chest,
    Kind::Bench,
    Kind::Chair,
];
/// The kits the planner offers, M6's: a locked one shows its reason.
pub const KITS: [&str; 5] = ["straw", "timber", "stone", "halftimber", "brick"];
/// How many undo steps are kept.
const UNDO: usize = 60;
/// Most characters in a blueprint's name.
const NAME_MAX: usize = 24;

/// What the planner holds.
#[derive(Resource, Debug)]
pub struct Planner {
    pub plan: Plan,
    /// The name it was opened under: a preset's locale key, or the player's.
    pub name: Option<PlanName>,
    pub preset: Option<(String, Size)>,
    /// The player's blueprint it came from, to save over.
    pub blueprint: Option<u64>,
    pub storey: u8,
    /// Into [`TOOLS`].
    pub tool: usize,
    pub slot_kind: usize,
    pub size: Size,
    pub iso: bool,
    pub hover: Option<Aim>,
    pub undo: Vec<Plan>,
    /// The room tool's stroke while the button is held.
    painting: Option<bool>,
    /// A line under the plan: why a tool did nothing, or what it does.
    pub hint: Option<String>,
    /// The site being changed (design 7), and its plan when it was opened.
    pub editing: Option<u64>,
    /// The name being typed for "save as mine", and whether keys go to it.
    pub typed: String,
    pub typing: bool,
    /// Bumped by every change the panel shows.
    pub revision: u64,
    /// The image the plan is painted into.
    pub image: Handle<Image>,
    painted: Option<u64>,
}

/// A plan's name: a preset's or a town's locale key, or the player's words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanName {
    Key(String),
    Mine(String),
}

impl PlanName {
    pub fn text(&self) -> Localized {
        match self {
            PlanName::Key(k) => tr(k),
            PlanName::Mine(m) => Localized::verbatim(m),
        }
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self {
            plan: Plan::new("timber"),
            name: None,
            preset: None,
            blueprint: None,
            storey: 0,
            tool: 0,
            slot_kind: 0,
            size: Size::Standard,
            iso: false,
            hover: None,
            undo: Vec::new(),
            painting: None,
            hint: None,
            editing: None,
            typed: String::new(),
            typing: false,
            revision: 1,
            image: Handle::default(),
            painted: None,
        }
    }
}

impl Planner {
    /// The tool chosen.
    pub fn tool(&self) -> Tool {
        match TOOLS[self.tool.min(TOOLS.len() - 1)] {
            "room" => Tool::Room,
            "wall" => Tool::Wall,
            "door" => Tool::Door,
            "window" => Tool::Window,
            "stair" => Tool::Stair,
            "lantern" => Tool::Lantern,
            "slot" => Tool::Slot(SLOT_KINDS[self.slot_kind % SLOT_KINDS.len()]),
            _ => Tool::Erase,
        }
    }

    /// Load a plan into the planner, kept for undo.
    pub fn load(&mut self, plan: Plan, name: Option<PlanName>, preset: Option<(String, Size)>) {
        self.push_undo();
        // Changing a site keeps the site's name.
        if self.editing.is_none() {
            self.name = name;
            self.preset = preset;
        }
        self.plan = plan;
        self.storey = self.storey.min(self.plan.storeys.len() as u8 - 1);
        self.revision += 1;
    }

    fn push_undo(&mut self) {
        self.undo.push(self.plan.clone());
        if self.undo.len() > UNDO {
            self.undo.remove(0);
        }
    }

    /// A drawn change: a preset changed is the player's plan now.
    fn edited(&mut self) {
        if let Some(PlanName::Key(_)) = self.name
            && self.editing.is_none()
        {
            self.preset = None;
        }
        self.revision += 1;
    }
}

/// The projection of the plan view: top down, or isometric (the plan turned
/// 45 degrees and tipped), in image pixels.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub scale: f32,
    pub cx: f32,
    pub cy: f32,
    pub iso: bool,
}

const IC: f32 = std::f32::consts::FRAC_1_SQRT_2;
const IS: f32 = std::f32::consts::FRAC_1_SQRT_2 * 0.5;
/// A wall stood up in the isometric view, and a cut one, in cell radii.
const WALL_H: f32 = 1.45;
const CUT_H: f32 = 0.3;

impl View {
    /// The view that fits `plan` in a `w` by `h` image.
    pub fn fit(plan: &Plan, w: f32, h: f32, max_radius: i32, iso: bool) -> Self {
        let mut ext = 2.0f32;
        for c in plan.footprint() {
            ext = ext.max(distance(c) as f32 + 1.6);
        }
        ext = ext.min(max_radius as f32 + 1.2);
        let wide = 3f32.sqrt();
        let scale = (w / (2.0 * ext * wide + wide)).min(h / (2.0 * ext * 1.5 + 2.0))
            * if iso { 1.45 } else { 1.0 };
        Self {
            scale,
            cx: w / 2.0,
            cy: h / 2.0 + if iso { scale * 0.4 } else { 0.0 },
            iso,
        }
    }

    /// A plan point (`x`, `z`, lifted `h`) on the image.
    pub fn to_image(self, x: f32, z: f32, h: f32) -> Vec2 {
        if self.iso {
            Vec2::new(
                self.cx + (x - z) * IC * self.scale,
                self.cy + ((x + z) * IS - h) * self.scale,
            )
        } else {
            Vec2::new(self.cx + x * self.scale, self.cy + z * self.scale)
        }
    }

    /// An image point back on the plan's floor.
    pub fn to_plan(self, p: Vec2) -> (f32, f32) {
        let u = (p.x - self.cx) / self.scale;
        let v = (p.y - self.cy) / self.scale;
        if !self.iso {
            return (u, v);
        }
        let (a, b) = (u / IC, v / IS);
        ((a + b) / 2.0, (b - a) / 2.0)
    }

    /// What an image point aims at: its cell, the side it leans to and
    /// whether it is near that side's edge.
    pub fn aim(&self, p: Vec2) -> Aim {
        let (x, z) = self.to_plan(p);
        let cell = pbd_core::building::presets::nearest(x, z);
        let (mx, mz) = middle(cell);
        let (dx, dz) = (x - mx, z - mz);
        let side =
            ((dz.atan2(dx) / (std::f32::consts::PI / 3.0)).round() as i32).rem_euclid(6) as u8;
        Aim {
            cell,
            side,
            at_edge: dx.hypot(dz) > 0.62,
        }
    }
}

// ---------------------------------------------------------------- painting

/// An RGBA canvas with the few shapes the plan needs.
struct Canvas {
    w: usize,
    h: usize,
    px: Vec<[f32; 4]>,
}

type Rgba = [f32; 4];

fn rgb(hex: u32, a: f32) -> Rgba {
    [
        ((hex >> 16) & 255) as f32 / 255.0,
        ((hex >> 8) & 255) as f32 / 255.0,
        (hex & 255) as f32 / 255.0,
        a,
    ]
}

impl Canvas {
    fn new(w: usize, h: usize, bg: Rgba) -> Self {
        Self {
            w,
            h,
            px: vec![bg; w * h],
        }
    }

    fn blend(&mut self, x: usize, y: usize, c: Rgba) {
        let p = &mut self.px[y * self.w + x];
        for k in 0..3 {
            p[k] = p[k] * (1.0 - c[3]) + c[k] * c[3];
        }
    }

    /// Fill a polygon, even-odd, sampling pixel middles.
    fn fill(&mut self, pts: &[Vec2], c: Rgba) {
        if pts.len() < 3 {
            return;
        }
        let y0 = pts
            .iter()
            .map(|p| p.y)
            .fold(f32::MAX, f32::min)
            .floor()
            .max(0.0) as usize;
        let y1 = (pts.iter().map(|p| p.y).fold(f32::MIN, f32::max).ceil() as usize).min(self.h);
        let mut xs = Vec::new();
        for y in y0..y1 {
            let sy = y as f32 + 0.5;
            xs.clear();
            for i in 0..pts.len() {
                let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                if (a.y <= sy) != (b.y <= sy) {
                    xs.push(a.x + (sy - a.y) / (b.y - a.y) * (b.x - a.x));
                }
            }
            xs.sort_by(f32::total_cmp);
            for pair in xs.chunks(2) {
                if let [l, r] = pair {
                    let from = (l - 0.5).ceil().max(0.0) as usize;
                    let to = ((r - 0.5).floor() as isize).min(self.w as isize - 1);
                    for x in from as isize..=to {
                        self.blend(x as usize, y, c);
                    }
                }
            }
        }
    }

    fn line(&mut self, a: Vec2, b: Vec2, width: f32, c: Rgba) {
        let d = (b - a).normalize_or(Vec2::X);
        let n = Vec2::new(-d.y, d.x) * (width / 2.0);
        let e = d * (width / 2.0);
        self.fill(&[a - e + n, b + e + n, b + e - n, a - e - n], c);
    }

    fn disc(&mut self, at: Vec2, r: f32, c: Rgba) {
        let pts: Vec<Vec2> = (0..16)
            .map(|k| {
                let t = k as f32 / 16.0 * std::f32::consts::TAU;
                at + Vec2::new(t.cos(), t.sin()) * r
            })
            .collect();
        self.fill(&pts, c);
    }

    fn bytes(&self) -> Vec<u8> {
        self.px
            .iter()
            .flat_map(|p| {
                [
                    (p[0].clamp(0.0, 1.0) * 255.0) as u8,
                    (p[1].clamp(0.0, 1.0) * 255.0) as u8,
                    (p[2].clamp(0.0, 1.0) * 255.0) as u8,
                    255,
                ]
            })
            .collect()
    }
}

fn hex_points(view: &View, c: Hex, shrink: f32, h: f32) -> Vec<Vec2> {
    let (mx, mz) = middle(c);
    (0..6u8)
        .map(|k| {
            let (x, z) = corner(c, k);
            view.to_image(mx + (x - mx) * shrink, mz + (z - mz) * shrink, h)
        })
        .collect()
}

fn outline(canvas: &mut Canvas, pts: &[Vec2], width: f32, c: Rgba) {
    for i in 0..pts.len() {
        canvas.line(pts[i], pts[(i + 1) % pts.len()], width, c);
    }
}

/// Paint the plan on `storey` with `issues` marked on it.
pub fn paint_plan(
    plan: &Plan,
    storey: u8,
    issues: &[Issue],
    hover: Option<(Aim, bool)>,
    view: &View,
    max_radius: i32,
    (w, h): (usize, usize),
) -> Vec<u8> {
    let mut c = Canvas::new(w, h, rgb(0x11293a, 1.0));
    let s = usize::from(storey);
    let px = view.scale / 30.0;
    // The drafting grid, the plan's reach and a ring past it in red.
    let lim = max_radius;
    for q in -lim - 1..=lim + 1 {
        for r in -lim - 1..=lim + 1 {
            let d = distance((q, r));
            if d > lim + 1 {
                continue;
            }
            let colour = if d > lim {
                rgb(0xff6b57, 0.18)
            } else {
                rgb(0x8fc3d8, 0.16)
            };
            outline(
                &mut c,
                &hex_points(view, (q, r), 0.97, 0.0),
                px.max(1.0),
                colour,
            );
        }
    }
    // The storey below, faint.
    if s > 0 {
        for &cell in &plan.storeys[s - 1] {
            c.fill(&hex_points(view, cell, 0.94, 0.0), rgb(0x8fc3d8, 0.10));
        }
    }
    let wells = plan.wells(storey);
    if let Some(cells) = plan.storeys.get(s) {
        for &cell in cells {
            let colour = if wells.contains(&cell) {
                rgb(0x142837, 0.9)
            } else {
                rgb(0xc4e4f0, 0.26)
            };
            c.fill(&hex_points(view, cell, 0.98, 0.0), colour);
        }
    }
    // Broken checks on their cells.
    for issue in issues {
        if issue.storey.is_some_and(|x| x != storey) && issue.storey.is_some() {
            continue;
        }
        let colour = if issue.check.refuses() {
            rgb(0xff6b57, 0.38)
        } else {
            rgb(0xf0b43c, 0.3)
        };
        for &cell in &issue.cells {
            c.fill(&hex_points(view, cell, 0.9, 0.0), colour);
        }
    }
    // The walls, then what stands in the rooms.
    let pieces = plan.pieces();
    let walls: Vec<&PlanPiece> = pieces
        .iter()
        .filter(|p| matches!(p, PlanPiece::Wall { storey: x, .. } if *x == storey))
        .collect();
    paint_walls(&mut c, view, &walls, px);
    for t in plan.stairs.iter().filter(|t| t.storey == storey) {
        let head = step(t.foot, t.side);
        let (a, b) = (middle(t.foot), middle(head));
        let (a, b) = (view.to_image(a.0, a.1, 0.0), view.to_image(b.0, b.1, 0.0));
        let bad = issues
            .iter()
            .any(|i| i.check.refuses() && i.storey == Some(storey) && i.cells.contains(&t.foot));
        let ink = if bad {
            rgb(0xff6b57, 1.0)
        } else {
            rgb(0xece4d2, 1.0)
        };
        let along = (b - a).normalize_or(Vec2::X);
        let across = Vec2::new(-along.y, along.x) * view.scale * 0.5;
        for k in 0..=8 {
            let at = a + (b - a) * (-0.3 + k as f32 * 0.16);
            c.line(at + across, at - across, 1.5 * px.max(1.0), ink);
        }
        let tip = b + along * view.scale * 0.8;
        c.line(a, tip, 2.5 * px.max(1.0), ink);
        let n = Vec2::new(-along.y, along.x) * 6.0 * px.max(1.0);
        c.fill(
            &[
                tip,
                tip - along * 9.0 * px.max(1.0) + n,
                tip - along * 9.0 * px.max(1.0) - n,
            ],
            ink,
        );
    }
    for l in plan.lanterns.iter().filter(|l| l.storey == storey) {
        let (x, z) = middle(l.cell);
        let at = view.to_image(x, z, 0.0);
        for k in (1..=6).rev() {
            c.disc(at, view.scale * 1.1 * k as f32 / 6.0, rgb(0xffc878, 0.12));
        }
        c.disc(at, (view.scale * 0.12).max(3.0), rgb(0xffd690, 1.0));
    }
    for o in plan.slots.iter().filter(|o| o.storey == storey) {
        let (x, z) = middle(o.cell);
        let at = view.to_image(x, z, 0.0);
        let bad = issues
            .iter()
            .any(|i| i.storey == Some(storey) && i.cells.contains(&o.cell) && i.check.refuses());
        let ink = if bad {
            rgb(0xff6b57, 0.9)
        } else {
            rgb(0xf0b43c, 0.9)
        };
        let r = view.scale * 0.32;
        c.fill(
            &[
                at + Vec2::new(-r, -r * 0.6),
                at + Vec2::new(r, -r * 0.6),
                at + Vec2::new(r, r * 0.6),
                at + Vec2::new(-r, r * 0.6),
            ],
            ink,
        );
        if o.side != pbd_core::furniture::MIDDLE {
            // It faces away from its back.
            let face = (o.side + 3) % 6;
            let (ex, ez) = middle(step(o.cell, face));
            let tip = view.to_image(x + (ex - x) * 0.4, z + (ez - z) * 0.4, 0.0);
            c.line(at, tip, 2.0 * px.max(1.0), ink);
        }
    }
    if let Some((aim, on_edge)) = hover {
        let ink = rgb(0xf0b43c, 1.0);
        if on_edge {
            let (a, b) = (corner(aim.cell, aim.side), corner(aim.cell, aim.side + 1));
            c.line(
                view.to_image(a.0, a.1, 0.0),
                view.to_image(b.0, b.1, 0.0),
                5.0 * px.max(1.0),
                ink,
            );
        } else {
            outline(
                &mut c,
                &hex_points(view, aim.cell, 0.92, 0.0),
                2.0 * px.max(1.0),
                ink,
            );
        }
    }
    c.bytes()
}

/// The walls of a storey: top down as thick lines with their openings;
/// isometric stood up, far ones first, the outer walls at the back full
/// height and the near and inner ones cut low (M6 version 3's cutaway).
fn paint_walls(c: &mut Canvas, view: &View, walls: &[&PlanPiece], px: f32) {
    let mut list: Vec<(f32, &PlanPiece)> = walls
        .iter()
        .map(|p| {
            let PlanPiece::Wall { cell, side, .. } = p else {
                unreachable!("walls only")
            };
            let (a, b) = (corner(*cell, *side), corner(*cell, side + 1));
            (a.0 + a.1 + b.0 + b.1, *p)
        })
        .collect();
    list.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, p) in list {
        let PlanPiece::Wall {
            cell,
            side,
            opening,
            outer,
            ..
        } = *p
        else {
            continue;
        };
        let (a, b) = (corner(cell, side), corner(cell, side + 1));
        let gap = match opening {
            Opening::Door => 1.0 / 1.636,
            Opening::Window => 0.8 / 1.636,
            Opening::None => 0.0,
        };
        let lerp = |t: f32| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        let (g0, g1) = (lerp((1.0 - gap) / 2.0), lerp((1.0 + gap) / 2.0));
        let base = if outer { 0xe2d6b6 } else { 0xbfb08c };
        if !view.iso {
            let wt = (if outer { 0.28 } else { 0.2 }) * view.scale;
            let ink = rgb(base, 1.0);
            let at = |p: (f32, f32)| view.to_image(p.0, p.1, 0.0);
            if gap == 0.0 {
                c.line(at(a), at(b), wt, ink);
                continue;
            }
            c.line(at(a), at(g0), wt, ink);
            c.line(at(g1), at(b), wt, ink);
            let mark = if opening == Opening::Window {
                rgb(0x8fc3d8, 1.0)
            } else {
                rgb(0xf0b43c, 1.0)
            };
            c.line(at(g0), at(g1), 2.0 * px.max(1.0), mark);
            continue;
        }
        // Lit by the face's turn to the upper left, as the icons are.
        let (mx, mz) = middle(cell);
        let (nx, nz) = (((a.0 + b.0) / 2.0 - mx), ((a.1 + b.1) / 2.0 - mz));
        let lit = 0.5 + 0.5 * (-(nx + nz) * IC).clamp(-1.0, 1.0);
        let k = 0.62 + 0.38 * lit;
        let shade = |hex: u32, a: f32| {
            let mut c = rgb(hex, a);
            for v in &mut c[..3] {
                *v *= k;
            }
            c
        };
        let full = outer && nx + nz < 0.0;
        let tall = if full { WALL_H } else { CUT_H };
        let face = |c: &mut Canvas, p: (f32, f32), q: (f32, f32), h0: f32, h1: f32, fill: Rgba| {
            let pts = [
                view.to_image(p.0, p.1, h0),
                view.to_image(q.0, q.1, h0),
                view.to_image(q.0, q.1, h1),
                view.to_image(p.0, p.1, h1),
            ];
            c.fill(&pts, fill);
            outline(c, &pts, 1.0, rgb(0x17150f, 0.55));
        };
        let fill = shade(base, if outer { 0.92 } else { 0.85 });
        if gap == 0.0 {
            face(c, a, b, 0.0, tall, fill);
            continue;
        }
        face(c, a, g0, 0.0, tall, fill);
        face(c, g1, b, 0.0, tall, fill);
        let glass = rgb(0x8fc3d8, 0.5);
        match (opening, full) {
            (Opening::Window, true) => {
                face(c, g0, g1, 0.0, 0.55, fill);
                face(c, g0, g1, 1.15, WALL_H, fill);
                face(c, g0, g1, 0.55, 1.15, glass);
            }
            (Opening::Window, false) => face(c, g0, g1, 0.0, tall, glass),
            (_, true) => {
                face(c, g0, g1, 1.25, WALL_H, fill);
                c.line(
                    view.to_image(g0.0, g0.1, 0.0),
                    view.to_image(g1.0, g1.1, 0.0),
                    2.0 * px.max(1.0),
                    rgb(0xf0b43c, 1.0),
                );
            }
            _ => c.line(
                view.to_image(g0.0, g0.1, 0.0),
                view.to_image(g1.0, g1.1, 0.0),
                2.0 * px.max(1.0),
                rgb(0xf0b43c, 1.0),
            ),
        }
    }
}

// ---------------------------------------------------------------- the panel

/// What a planner button does.
#[derive(Component, Clone, Debug, PartialEq)]
pub enum PlanButton {
    Preset(usize),
    Mine(u64),
    Town(usize),
    Size(Size),
    Storey(u8),
    AddStorey,
    DropStorey,
    View(bool),
    Kit(&'static str),
    Roof(RoofShape),
    Tool(usize),
    SlotKind,
    Name,
    Save,
    SetOut,
    SaveChange,
    Cancel,
    Close,
}

/// The containers the panel rebuilds when the plan changes.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Library,
    Top,
    Tools,
    Checks,
    Bill,
    Actions,
}

/// The plan view's node.
#[derive(Component)]
pub struct PlanView;

/// The line under the plan view.
#[derive(Component)]
pub struct HintLine;

/// The town templates' plain buildings as plans, each with its town.
#[derive(Resource, Default)]
pub struct TownPlans(pub Vec<(String, Plan)>);

/// The town templates' plans, made once (design 4: the towns' houses are
/// presets without being copied by hand).
pub fn load_town_plans(
    assets: Option<Res<pbd_app::towns::TownAssets>>,
    data: Res<PlannerData>,
    mut plans: ResMut<TownPlans>,
    mut done: Local<bool>,
) {
    let Some(assets) = assets else {
        return;
    };
    if *done {
        return;
    }
    *done = true;
    let mut seen = std::collections::BTreeSet::new();
    for (town, t) in [
        ("village", &assets.village),
        ("walled", &assets.walled),
        ("harbour", &assets.harbour),
        ("desert", &assets.desert),
        ("tundra", &assets.tundra),
        ("jungle", &assets.jungle),
    ] {
        for def in &t.buildings {
            let Some(kit) = assets.kits.get(&def.kit) else {
                continue;
            };
            if !pbd_core::building::blueprint::plain(def, kit)
                || !seen.insert((town, def.kit.clone(), def.cells.len(), def.storeys))
            {
                continue;
            }
            if let Some(p) = pbd_core::building::presets::from_template(def, kit)
                && pbd_core::building::planner::plan_ok(&p, &data.tun)
            {
                plans.0.push((town.to_string(), p));
            }
        }
    }
    info!("PLANNER {} town plans", plans.0.len());
}

/// The panel, built once and hidden.
pub fn spawn(
    mut commands: Commands,
    kit: Res<UiKit>,
    mut images: ResMut<Assets<Image>>,
    mut planner: ResMut<Planner>,
) {
    let kit = &*kit;
    let (w, h) = (VIEW_W as u32 * IMG_SCALE, VIEW_H as u32 * IMG_SCALE);
    let mut image = Image::new_fill(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[17, 41, 58, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    planner.image = images.add(image);
    let column = |width: f32| Node {
        width: px(width),
        flex_direction: FlexDirection::Column,
        row_gap: px(3),
        flex_shrink: 0.0,
        ..default()
    };
    commands
        .spawn((menu::layer(), GlobalZIndex(10), Panel(Screen::Planner)))
        .with_children(|layer| {
            layer
                .spawn((
                    kit.nine(Set::Stone, "frame"),
                    Node {
                        padding: UiRect::all(px(12)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ))
                .with_children(|panel| {
                    menu::lintel(panel, kit, tr("planner.title"));
                    panel
                        .spawn(Node {
                            column_gap: px(8),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((column(172.0), Part::Library));
                            row.spawn(column(VIEW_W)).with_children(|mid| {
                                mid.spawn((
                                    Node {
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(2),
                                        ..default()
                                    },
                                    Part::Top,
                                ));
                                mid.spawn((
                                    ImageNode::new(planner.image.clone()),
                                    Node {
                                        width: px(VIEW_W),
                                        height: px(VIEW_H),
                                        ..default()
                                    },
                                    Interaction::default(),
                                    RelativeCursorPosition::default(),
                                    PlanView,
                                ));
                                mid.spawn((
                                    Node {
                                        column_gap: px(2),
                                        ..default()
                                    },
                                    Part::Tools,
                                ));
                                mid.spawn((
                                    KitText::new(Localized::empty(), ui_kit::TEXT_DIM)
                                        .shadow(ui_kit::INK),
                                    HintLine,
                                ));
                            });
                            row.spawn(column(176.0)).with_children(|right| {
                                right.spawn((column(176.0), Part::Checks));
                                right.spawn((column(176.0), Part::Bill));
                                right.spawn((column(176.0), Part::Actions));
                            });
                        });
                });
        });
}

fn label(parent: &mut ChildSpawnerCommands, text: Localized, colour: Color) {
    parent.spawn(KitText::new(text, colour).shadow(ui_kit::INK));
}

fn button(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    text: Localized,
    act: PlanButton,
    width: f32,
    on: bool,
    disabled: bool,
) {
    let mut plaque = Plaque::new(Set::Stone);
    plaque.latched = on;
    plaque.disabled = disabled;
    parent
        .spawn((ui_kit::plaque(kit, Set::Stone, width), act))
        .insert(plaque)
        .with_children(|b| {
            b.spawn(ui_kit::plaque_label(Set::Stone, text));
        });
}

/// Rebuild the panel's parts when the plan or the library changes.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn refresh(
    mut commands: Commands,
    kit: Res<UiKit>,
    planner: Res<Planner>,
    data: Res<PlannerData>,
    towns: Res<TownPlans>,
    save: Option<Res<WorldSave>>,
    (hotbar, icons, images, sites): (
        Res<Hotbar>,
        Res<ItemIcons>,
        Res<Assets<Image>>,
        Option<Res<Sites>>,
    ),
    parts: Query<(Entity, &Part, Option<&Children>)>,
    mut hint: Query<&mut KitText, With<HintLine>>,
    screen: Res<Screen>,
    mut shown: Local<(u64, usize, Option<pbd_core::inventory::Slots>)>,
) {
    if *screen != Screen::Planner {
        return;
    }
    let mine = save
        .as_deref()
        .map(|s| blueprints(&s.records))
        .unwrap_or_default();
    let stamp = (planner.revision, mine.len(), Some(hotbar.0.clone()));
    if stamp == *shown {
        return;
    }
    *shown = stamp;
    let kit = &*kit;
    let p = &planner.plan;
    let issues = check_plan(p, &data.tun);
    let pieces = on_flat(p, data.tun.max_radius)
        .map(|(_, charted)| charted)
        .unwrap_or_default();
    for (entity, part, children) in &parts {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        let mut e = commands.entity(entity);
        match part {
            Part::Library => {
                e.with_children(|col| {
                    label(col, tr("planner.presets"), ui_kit::TITLE);
                    col.spawn(Node {
                        column_gap: px(2),
                        ..default()
                    })
                    .with_children(|row| {
                        for size in Size::ALL {
                            button(
                                row,
                                kit,
                                tr(&format!("blueprint.size.{}", size.id())),
                                PlanButton::Size(size),
                                40.0,
                                planner.size == size,
                                false,
                            );
                        }
                    });
                    for (i, preset) in data.presets.presets.iter().enumerate() {
                        let on = planner.preset.as_ref()
                            == Some(&(preset.id.clone(), planner.size))
                            && planner.blueprint.is_none();
                        button(
                            col,
                            kit,
                            tr(&format!("blueprint.preset.{}", preset.id)),
                            PlanButton::Preset(i),
                            172.0,
                            on,
                            false,
                        );
                    }
                    label(col, tr("planner.mine"), ui_kit::TITLE);
                    if mine.is_empty() {
                        label(col, tr("planner.mine_none"), ui_kit::TEXT_DIM);
                    }
                    for (id, b) in &mine {
                        button(
                            col,
                            kit,
                            Localized::verbatim(&b.name),
                            PlanButton::Mine(*id),
                            172.0,
                            planner.blueprint == Some(*id),
                            false,
                        );
                    }
                    label(col, tr("planner.towns"), ui_kit::TITLE);
                    let mut seen_towns = std::collections::BTreeSet::new();
                    for (i, (town, _)) in towns.0.iter().enumerate() {
                        if !seen_towns.insert(town.clone()) {
                            continue;
                        }
                        button(
                            col,
                            kit,
                            tr(&format!("planner.town.{town}")),
                            PlanButton::Town(i),
                            172.0,
                            false,
                            false,
                        );
                    }
                });
            }
            Part::Top => {
                e.with_children(|top| {
                    top.spawn(Node {
                        column_gap: px(2),
                        ..default()
                    })
                    .with_children(|row| {
                        for s in 0..p.storeys.len() as u8 {
                            let text = if s == 0 {
                                tr("planner.ground")
                            } else {
                                tr_with("planner.floor", &[("n", &s)])
                            };
                            button(
                                row,
                                kit,
                                text,
                                PlanButton::Storey(s),
                                40.0,
                                s == planner.storey,
                                false,
                            );
                        }
                        button(
                            row,
                            kit,
                            Localized::verbatim("+"),
                            PlanButton::AddStorey,
                            22.0,
                            false,
                            p.storeys.len() >= data.tun.max_storeys,
                        );
                        button(
                            row,
                            kit,
                            Localized::verbatim("-"),
                            PlanButton::DropStorey,
                            22.0,
                            false,
                            p.storeys.len() < 2,
                        );
                        for (iso, key) in [(false, "planner.view.top"), (true, "planner.view.iso")]
                        {
                            button(
                                row,
                                kit,
                                tr(key),
                                PlanButton::View(iso),
                                56.0,
                                planner.iso == iso,
                                false,
                            );
                        }
                    });
                    top.spawn(Node {
                        column_gap: px(2),
                        ..default()
                    })
                    .with_children(|row| {
                        for k in KITS {
                            let locked = data.costs.kit(k).is_err();
                            button(
                                row,
                                kit,
                                tr(&format!("planner.kit.{k}")),
                                PlanButton::Kit(k),
                                30.0,
                                p.kit == k,
                                locked,
                            );
                        }
                    });
                    top.spawn(Node {
                        column_gap: px(2),
                        ..default()
                    })
                    .with_children(|row| {
                        for (shape, key) in [
                            (RoofShape::Gable, "planner.roof.gable"),
                            (RoofShape::Cone, "planner.roof.cone"),
                            (RoofShape::Flat, "planner.roof.flat"),
                        ] {
                            button(
                                row,
                                kit,
                                tr(key),
                                PlanButton::Roof(shape),
                                30.0,
                                p.roof == shape,
                                false,
                            );
                        }
                    });
                });
            }
            Part::Tools => {
                let server = TOOL_ICONS;
                e.with_children(|row| {
                    for (i, name) in TOOLS.iter().enumerate() {
                        let mut plaque = Plaque::new(Set::Stone);
                        plaque.latched = i == planner.tool;
                        row.spawn((
                            ui_kit::plaque(kit, Set::Stone, 34.0),
                            PlanButton::Tool(i),
                            ui_kit::Tooltip(tr(&format!("planner.tool.{name}"))),
                        ))
                        .insert(plaque)
                        .with_children(|b| {
                            b.spawn((
                                ToolIcon(server[i]),
                                Node {
                                    width: px(16),
                                    height: px(16),
                                    margin: UiRect::top(px(-5)),
                                    ..default()
                                },
                            ));
                        });
                    }
                    if TOOLS[planner.tool] == "slot" {
                        let kind = SLOT_KINDS[planner.slot_kind % SLOT_KINDS.len()];
                        button(
                            row,
                            kit,
                            kind.name(),
                            PlanButton::SlotKind,
                            40.0,
                            false,
                            false,
                        );
                    }
                });
            }
            Part::Checks => {
                e.with_children(|col| {
                    label(col, tr("planner.checks"), ui_kit::TITLE);
                    let mut any_bad = false;
                    let mut seen = std::collections::BTreeSet::new();
                    for issue in &issues {
                        let words = issue_words(issue);
                        if !seen.insert(words.to_string()) {
                            continue;
                        }
                        any_bad |= issue.check.refuses();
                        let colour = if issue.check.refuses() {
                            Color::srgb(1.0, 0.45, 0.36)
                        } else {
                            Color::srgb(0.94, 0.71, 0.24)
                        };
                        label(col, words, colour);
                    }
                    if !any_bad {
                        label(col, tr("planner.check.pass"), ui_kit::RUNE);
                    }
                });
            }
            Part::Bill => {
                let bill = data
                    .costs
                    .bill(pieces.iter().map(|(_, piece)| piece), data.tun.cost_scale);
                let strokes: u32 = pieces
                    .iter()
                    .map(|(_, piece)| data.tun.strokes_of(piece))
                    .sum();
                e.with_children(|col| {
                    label(col, tr("planner.bill"), ui_kit::TITLE);
                    match &bill {
                        Ok(bill) => {
                            for (good, n) in bill {
                                let item = pbd_core::station::item_named(good);
                                let have = item.map_or(0, |i| hotbar.0.count(i));
                                col.spawn(Node {
                                    column_gap: px(3),
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .with_children(|row| {
                                    let mut node = ImageNode::default();
                                    if let Some(item) = item {
                                        paint_icon(
                                            &mut node,
                                            Some(Stack::new(item, 1)),
                                            &icons,
                                            &images,
                                        );
                                    }
                                    row.spawn((
                                        node,
                                        Node {
                                            width: px(16),
                                            height: px(16),
                                            ..default()
                                        },
                                    ));
                                    let colour = if have < *n {
                                        Color::srgb(1.0, 0.45, 0.36)
                                    } else {
                                        ui_kit::TEXT
                                    };
                                    label(
                                        row,
                                        tr_with(
                                            "planner.bill_row",
                                            &[
                                                ("good", &good_name(good)),
                                                ("count", n),
                                                ("have", &have),
                                            ],
                                        ),
                                        colour,
                                    );
                                });
                            }
                            label(
                                col,
                                tr_with(
                                    "planner.bill_sum",
                                    &[
                                        (
                                            "pieces",
                                            &pieces
                                                .iter()
                                                .filter(|(pp, _)| !matches!(pp, PlanPiece::Slot(_)))
                                                .count(),
                                        ),
                                        ("strokes", &strokes),
                                    ],
                                ),
                                ui_kit::TEXT_DIM,
                            );
                        }
                        Err(locked) => label(col, tr(&locked.0), Color::srgb(1.0, 0.45, 0.36)),
                    }
                });
            }
            Part::Actions => {
                let ok =
                    !issues.iter().any(|i| i.check.refuses()) && data.costs.kit(&p.kit).is_ok();
                let editing = planner.editing;
                let site_name =
                    editing.and_then(|id| sites.as_deref().and_then(|s| s.by_id.get(&id)));
                e.with_children(|col| {
                    if let Some(site) = site_name {
                        label(
                            col,
                            tr_with(
                                "planner.changing",
                                &[("name", &site_display_name(&site.body.name))],
                            ),
                            ui_kit::TEXT,
                        );
                    }
                    let shown_name = if planner.typing || !planner.typed.is_empty() {
                        Localized::verbatim(format!(
                            "{}{}",
                            planner.typed,
                            if planner.typing { "_" } else { "" }
                        ))
                    } else {
                        planner
                            .name
                            .as_ref()
                            .map_or_else(|| tr("planner.name"), PlanName::text)
                    };
                    button(
                        col,
                        kit,
                        shown_name,
                        PlanButton::Name,
                        176.0,
                        planner.typing,
                        false,
                    );
                    button(
                        col,
                        kit,
                        tr("planner.save"),
                        PlanButton::Save,
                        176.0,
                        false,
                        false,
                    );
                    if editing.is_some() {
                        button(
                            col,
                            kit,
                            tr("planner.save_change"),
                            PlanButton::SaveChange,
                            176.0,
                            false,
                            !ok,
                        );
                        button(
                            col,
                            kit,
                            tr("planner.cancel"),
                            PlanButton::Cancel,
                            176.0,
                            false,
                            false,
                        );
                    } else {
                        button(
                            col,
                            kit,
                            tr("planner.set_out"),
                            PlanButton::SetOut,
                            176.0,
                            false,
                            !ok,
                        );
                        button(
                            col,
                            kit,
                            tr("planner.close"),
                            PlanButton::Close,
                            176.0,
                            false,
                            false,
                        );
                    }
                });
            }
        }
    }
    if let Ok(mut text) = hint.single_mut() {
        let want = planner.hint.as_ref().map_or_else(
            || tr(&format!("planner.tool.{}", TOOLS[planner.tool])),
            |k| tr(k),
        );
        if text.text != want {
            text.text = want;
        }
    }
}

/// A tool's icon: build mode's own, by name.
#[derive(Component, Clone, Copy)]
pub struct ToolIcon(pub &'static str);

/// Each tool's icon, build mode's own (`textures/ui/build/<name>.png`).
const TOOL_ICONS: [&str; 8] = [
    "floor",
    "wall",
    "door",
    "window",
    "stairs",
    "lantern",
    "furniture",
    "erase",
];

/// Give each tool icon its image.
pub fn dress_icons(
    server: Res<AssetServer>,
    mut icons: Query<(Entity, &ToolIcon), Without<ImageNode>>,
    mut commands: Commands,
) {
    for (e, icon) in &mut icons {
        commands.entity(e).insert(ImageNode::new(
            server.load(format!("textures/ui/build/{}.png", icon.0)),
        ));
    }
}

/// A site's or a plan's name as shown: a locale key for a preset's.
pub fn site_display_name(name: &str) -> Localized {
    if name.starts_with("blueprint.preset.") || name.starts_with("planner.town.") {
        tr(name)
    } else {
        Localized::verbatim(name)
    }
}

/// A check's words, with the storey it is on.
fn issue_words(issue: &Issue) -> Localized {
    let reason = tr(&issue.check.key());
    match issue.storey {
        Some(0) if issue.check != Check::WayIn => {
            tr_with("planner.on_ground", &[("reason", &reason)])
        }
        Some(s) if s > 0 => tr_with("planner.on_floor", &[("n", &s), ("reason", &reason)]),
        _ => reason,
    }
}

/// Paint the plan view again when anything it shows has changed.
pub fn paint(
    mut planner: ResMut<Planner>,
    data: Res<PlannerData>,
    screen: Res<Screen>,
    mut images: ResMut<Assets<Image>>,
) {
    if *screen != Screen::Planner {
        return;
    }
    let hover_key = planner.hover.map_or(0, |a| {
        (a.cell.0 as u64 & 0xff) << 8
            | (a.cell.1 as u64 & 0xff) << 16
            | u64::from(a.side) << 24
            | u64::from(a.at_edge) << 28
    });
    let key = planner.revision
        ^ hover_key.rotate_left(32)
        ^ u64::from(planner.storey) << 60
        ^ u64::from(planner.iso) << 59
        ^ (planner.tool as u64) << 56;
    if planner.painted == Some(key) {
        return;
    }
    planner.painted = Some(key);
    let (w, h) = (
        (VIEW_W as u32 * IMG_SCALE) as usize,
        (VIEW_H as u32 * IMG_SCALE) as usize,
    );
    let view = View::fit(
        &planner.plan,
        w as f32,
        h as f32,
        data.tun.max_radius,
        planner.iso,
    );
    let issues = check_plan(&planner.plan, &data.tun);
    let tool = planner.tool();
    let hover = planner.hover.map(|a| {
        let a = planner.plan.resolve(tool, planner.storey, a);
        (a, tool.on_edge(&a))
    });
    let bytes = paint_plan(
        &planner.plan,
        planner.storey,
        &issues,
        hover,
        &view,
        data.tun.max_radius,
        (w, h),
    );
    if let Some(image) = images.get_mut(&planner.image) {
        image.data = Some(bytes);
    }
}

/// The plan view's pointer and the keys: hover, paint and erase with the
/// tools, the number keys pick a tool, ctrl-Z undoes, and typed keys go to
/// the name while it has the keyboard.
#[allow(clippy::too_many_arguments)]
pub fn input(
    mut planner: ResMut<Planner>,
    data: Res<PlannerData>,
    screen: Res<Screen>,
    view_node: Query<&RelativeCursorPosition, With<PlanView>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
) {
    if *screen != Screen::Planner {
        typed.clear();
        return;
    }
    if planner.typing {
        for key in typed.read() {
            if !key.state.is_pressed() {
                continue;
            }
            match &key.logical_key {
                Key::Character(text) => {
                    for c in text.chars().filter(|c| !c.is_control()) {
                        if planner.typed.chars().count() < NAME_MAX {
                            planner.typed.push(c);
                        }
                    }
                }
                Key::Space if planner.typed.chars().count() < NAME_MAX => planner.typed.push(' '),
                Key::Backspace => {
                    planner.typed.pop();
                }
                Key::Enter | Key::Escape => planner.typing = false,
                _ => {}
            }
            planner.revision += 1;
        }
        return;
    }
    typed.clear();
    let digits = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
    ];
    for (i, k) in digits.iter().enumerate() {
        if keys.just_pressed(*k) && planner.tool != i {
            planner.tool = i;
            planner.hint = None;
            planner.revision += 1;
        }
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl
        && keys.just_pressed(KeyCode::KeyZ)
        && let Some(last) = planner.undo.pop()
    {
        planner.plan = last;
        planner.storey = planner.storey.min(planner.plan.storeys.len() as u8 - 1);
        planner.revision += 1;
    }
    let Ok(cursor) = view_node.single() else {
        return;
    };
    let Some(n) = cursor.normalized.filter(|_| cursor.cursor_over) else {
        if buttons.just_released(MouseButton::Left) {
            planner.painting = None;
        }
        if planner.hover.take().is_some() {
            planner.revision += 0;
        }
        return;
    };
    let (w, h) = (VIEW_W * IMG_SCALE as f32, VIEW_H * IMG_SCALE as f32);
    let view = View::fit(&planner.plan, w, h, data.tun.max_radius, planner.iso);
    let aim = view.aim(Vec2::new((n.x + 0.5) * w, (n.y + 0.5) * h));
    if planner.hover != Some(aim) {
        planner.hover = Some(aim);
    }
    if buttons.just_released(MouseButton::Left) {
        planner.painting = None;
    }
    let tool = planner.tool();
    let press = buttons.just_pressed(MouseButton::Left);
    let drag =
        buttons.pressed(MouseButton::Left) && tool == Tool::Room && planner.painting.is_some();
    if press || drag {
        if press {
            planner.push_undo();
        }
        let storey = planner.storey;
        let painting = if press { None } else { planner.painting };
        let before = planner.plan.clone();
        match planner
            .plan
            .apply(tool, storey, aim, painting, data.tun.max_radius)
        {
            Ok(next) => {
                if tool == Tool::Room {
                    planner.painting = next;
                }
                planner.hint = None;
                if planner.plan != before {
                    planner.edited();
                }
            }
            Err(no) => {
                planner.hint = Some(no.key().to_string());
                planner.revision += 1;
            }
        }
    }
}

/// A button pressed.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn press(
    mut planner: ResMut<Planner>,
    data: Res<PlannerData>,
    towns: Res<TownPlans>,
    mut save: Option<ResMut<WorldSave>>,
    mut screen: ResMut<Screen>,
    pressed: Query<(&Interaction, &PlanButton, &Plaque), Changed<Interaction>>,
    mut setting_out: ResMut<super::construction::PlanInHand>,
    mut change: ResMut<super::construction::ChangeAsked>,
) {
    for (interaction, act, plaque) in &pressed {
        if *interaction != Interaction::Pressed || plaque.disabled {
            continue;
        }
        let size = planner.size;
        match act.clone() {
            PlanButton::Preset(i) => {
                let Some(preset) = data.presets.presets.get(i) else {
                    continue;
                };
                planner.blueprint = None;
                planner.load(
                    preset.plan(size),
                    Some(PlanName::Key(format!("blueprint.preset.{}", preset.id))),
                    Some((preset.id.clone(), size)),
                );
            }
            PlanButton::Mine(id) => {
                let Some(body) = save
                    .as_deref()
                    .and_then(|s| blueprints(&s.records).remove(&id))
                else {
                    continue;
                };
                let Some(plan) = body.plan.clone() else {
                    continue;
                };
                planner.blueprint = Some(id);
                planner.load(
                    plan,
                    Some(PlanName::Mine(body.name.clone())),
                    body.preset.clone(),
                );
            }
            PlanButton::Town(i) => {
                let Some((town, plan)) = towns.0.get(i).cloned() else {
                    continue;
                };
                planner.blueprint = None;
                planner.load(
                    plan,
                    Some(PlanName::Key(format!("planner.town.{town}"))),
                    None,
                );
            }
            PlanButton::Size(s) => {
                planner.size = s;
                let current = planner.preset.clone();
                match current.and_then(|(id, _)| data.presets.get(&id).cloned()) {
                    Some(preset) if planner.blueprint.is_none() && planner.editing.is_none() => {
                        planner.load(
                            preset.plan(s),
                            Some(PlanName::Key(format!("blueprint.preset.{}", preset.id))),
                            Some((preset.id.clone(), s)),
                        );
                    }
                    _ => planner.revision += 1,
                }
            }
            PlanButton::Storey(s) => {
                planner.storey = s;
                planner.revision += 1;
            }
            PlanButton::AddStorey => {
                planner.push_undo();
                planner.plan.add_storey();
                planner.storey = planner.plan.storeys.len() as u8 - 1;
                planner.edited();
            }
            PlanButton::DropStorey => {
                planner.push_undo();
                planner.plan.drop_storey();
                planner.storey = planner.storey.min(planner.plan.storeys.len() as u8 - 1);
                planner.edited();
            }
            PlanButton::View(iso) => {
                planner.iso = iso;
                planner.revision += 1;
            }
            PlanButton::Kit(k) => {
                planner.push_undo();
                planner.plan.kit = k.to_string();
                planner.edited();
            }
            PlanButton::Roof(r) => {
                planner.push_undo();
                planner.plan.roof = r;
                planner.edited();
            }
            PlanButton::Tool(i) => {
                planner.tool = i;
                planner.hint = None;
                planner.revision += 1;
            }
            PlanButton::SlotKind => {
                planner.slot_kind = (planner.slot_kind + 1) % SLOT_KINDS.len();
                planner.revision += 1;
            }
            PlanButton::Name => {
                planner.typing = !planner.typing;
                planner.revision += 1;
            }
            PlanButton::Save => {
                let Some(save) = save.as_deref_mut() else {
                    continue;
                };
                let typed = planner.typed.trim().to_string();
                let name = if !typed.is_empty() {
                    typed
                } else if let Some(PlanName::Mine(m)) = &planner.name {
                    m.clone()
                } else {
                    tr_with(
                        "planner.default_name",
                        &[("n", &(blueprints(&save.records).len() + 1))],
                    )
                    .to_string()
                };
                // A name the player already has is saved over.
                let over = blueprints(&save.records)
                    .into_iter()
                    .find(|(_, b)| b.name == name)
                    .map(|(id, _)| id);
                let body = BlueprintBody {
                    name: name.clone(),
                    preset: planner.preset.clone(),
                    plan: Some(planner.plan.clone()),
                };
                match store_blueprint(save, over, &body) {
                    Some(id) => {
                        info!("PLANNER saved blueprint {id} \"{name}\"");
                        planner.blueprint = Some(id);
                        planner.name = Some(PlanName::Mine(name));
                        planner.typed.clear();
                        planner.typing = false;
                        planner.hint = Some("planner.saved".into());
                    }
                    None => error!("planner: the blueprint was not saved"),
                }
                planner.revision += 1;
            }
            PlanButton::SetOut => {
                setting_out.0 = Some(super::construction::InHand {
                    plan: planner.plan.clone(),
                    name: match &planner.name {
                        Some(PlanName::Key(k)) => k.clone(),
                        Some(PlanName::Mine(m)) => m.clone(),
                        None => tr("planner.untitled").to_string(),
                    },
                    preset: planner.preset.clone(),
                });
                *screen = Screen::Playing;
                info!("PLANNER set out");
            }
            PlanButton::SaveChange => {
                if let Some(id) = planner.editing {
                    change.0 = Some((id, planner.plan.clone()));
                }
            }
            PlanButton::Cancel | PlanButton::Close => {
                planner.editing = None;
                *screen = Screen::Playing;
            }
        }
    }
}

/// Leaving the planner forgets a site it was changing and the typing.
pub fn forget(screen: Res<Screen>, mut planner: ResMut<Planner>) {
    if screen.is_changed() && *screen != Screen::Planner {
        planner.typing = false;
        planner.hover = None;
        if planner.editing.take().is_some() {
            planner.plan = Plan::new("timber");
            planner.name = None;
        }
    }
}

/// Open the planner on a plan.
#[allow(clippy::type_complexity)]
pub fn open(
    planner: &mut Planner,
    screen: &mut Screen,
    plan: Option<(Plan, PlanName, Option<(String, Size)>)>,
) {
    if let Some((plan, name, preset)) = plan {
        planner.undo.clear();
        planner.plan = plan;
        planner.name = Some(name);
        planner.preset = preset;
        planner.blueprint = None;
        planner.storey = 0;
    }
    planner.hint = None;
    planner.revision += 1;
    *screen = Screen::Planner;
}

/// F at a drafting table offers the planner, and opens it.
#[allow(clippy::too_many_arguments)]
pub fn use_table(
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    aim: Res<super::digging::Aim>,
    fine: Option<Res<pbd_app::planet::PlanetFine>>,
    save: Option<Res<WorldSave>>,
    mut interact: ResMut<pbd_app::interact::Interact>,
    mut planner: ResMut<Planner>,
    data: Res<PlannerData>,
) {
    let (Some(fine), Some(save)) = (fine, save) else {
        return;
    };
    let Some(target) = aim.target else {
        return;
    };
    let at = (target.dig.cell, target.dig.layer);
    if super::digging::material_of(&fine, &save, at)
        != Some(pbd_core::terrain::Material::DraftingTable)
    {
        return;
    }
    if *screen != Screen::Playing || !walking.is_some_and(|w| w.active && w.captured) {
        return;
    }
    interact.offer(pbd_app::interact::Offer::new(
        pbd_app::interact::Act::Plan,
        pbd_core::inventory::material_name(pbd_core::terrain::Material::DraftingTable),
    ));
    if keys.just_pressed(KeyCode::KeyF)
        && pbd_app::interact::may(Some(&interact), pbd_app::interact::Reader::Planner)
    {
        let first = planner.plan.is_empty().then(|| {
            let preset = &data.presets.presets[1.min(data.presets.presets.len() - 1)];
            (
                preset.plan(Size::Standard),
                PlanName::Key(format!("blueprint.preset.{}", preset.id)),
                Some((preset.id.clone(), Size::Standard)),
            )
        });
        open(&mut planner, &mut screen, first);
        info!("PLANNER opened at the drafting table");
    }
}
