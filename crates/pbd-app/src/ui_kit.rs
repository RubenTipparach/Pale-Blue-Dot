//! The interface's kit (`stone-and-wood-ui`, tasks section 1): the smooth
//! Blender renders of the rock, the dressed stone and the carved oak, the
//! kit's pixel font, the body font, the colour tokens and the UI scale.
//!
//! **Smooth, sampled linear: the owner's own exception for the UI.** The
//! owner, 2026-11-14: "Smooth is the best ... start working the current UI as
//! the wood + rendered smooth rock". The kit's pieces are the Cycles renders
//! (`docs/mockups/ui/rendered/hires/`, 8 texels a native pixel; the rock
//! panels 4), not reduced to pixel art, so they are sampled LINEAR. Nothing
//! else changes: the item icons inside a slot and the kit's bitmap font stay
//! pixel art sampled nearest, and every world texture stays nearest
//! (CLAUDE.md, "Terrain textures use nearest point sampling").
//!
//! **Baked for the scale it is shown at.** A render at 8x drawn at 2x would be
//! minified four to one by a sampler with no mip chain, which shimmers. So
//! each piece is resampled once, on the CPU, to exactly the screen's pixels
//! per native pixel (premultiplied, Catmull-Rom), and drawn about one texel a
//! pixel; when the window crosses a step of the UI scale it is baked again
//! into the same handles. Every slice margin is the drawn margin times that
//! scale, read from `assets/ui/slices.json` (the art's own file, so no margin
//! is restated in Rust).
//!
//! **Loaded at once, not by the asset server.** The loading screen is the
//! first frame the window shows (CLAUDE.md, "A loading screen on boot"); an
//! asynchronous load would draw it a few frames without its slab. The PNGs
//! are decoded here, in `Plugin::finish`, before the first frame.
//!
//! **One integer UI scale.** max(1, floor(physical height / 450)): 2 at 900
//! and 1080, 3 at 1440, 4 at 2160 (design section 6). `UiScale` is set so
//! that the window's own scale factor times it is that whole number, so a
//! native pixel is always a whole number of screen pixels. Every `px()` in
//! the interface is in native pixels.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageSampler, TextureAtlas, TextureAtlasLayout};
use bevy::prelude::*;
use bevy::sprite::{BorderRect, SliceScaleMode, TextureSlicer};
use bevy::text::Font;
use bevy::ui::UiSystems;
use bevy::window::PrimaryWindow;
use image::{ImageBuffer, Rgba, RgbaImage};

/// The window height that buys one more step of UI scale.
pub const SCALE_STEP_PX: u32 = 450;

/// The UI scale for a window this many physical pixels tall: a whole number,
/// never below one (design section 6).
pub fn ui_scale_for(physical_height: u32) -> u32 {
    (physical_height / SCALE_STEP_PX).max(1)
}

/// The assets folder (the same one `AssetPlugin` is pointed at).
pub fn assets_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"))
}

// ---------------------------------------------------------------- tokens
//
// One module of colours (spec delta, "One palette module for the
// interface"): every UI file takes its colours from here. The values are
// `ui_art.py`'s, every one Sweaft-64; the pairs a player reads are measured
// in `tokens_clear_four_and_a_half_to_one` below.

const fn hex(rgb: u32) -> Color {
    Color::srgb(
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
    )
}

/// Body text on a stone face.
pub const TEXT: Color = hex(0xeab798);
/// Secondary text.
pub const TEXT_DIM: Color = hex(0xcfaa81);
/// Carved titles on a face, key names.
pub const TITLE: Color = hex(0xfaaf5c);
/// A stone plaque's label, inlaid chalk; the open stone tab's label.
pub const LABEL: Color = hex(0xfffefe);
/// A disabled plaque's label.
pub const LABEL_DIM: Color = hex(0x92a3b2);
/// A shut tab's label.
pub const TAB_DIM: Color = hex(0xbfcad7);
/// Letters and ticks cut into the compass strip.
pub const CUT: Color = hex(0x011c34);
/// The N on the compass, red inlay.
pub const NORTH: Color = hex(0x4a051e);
/// A title or label cut into oak.
pub const OAK_LABEL: Color = hex(0x010000);
/// Focus and selection: the one glow.
pub const RUNE: Color = hex(0x84e3b5);
/// The rune's dim step: a slider's filled groove.
pub const RUNE_DIM: Color = hex(0x53998b);
/// A panel's stone face.
pub const FACE: Color = hex(0x1f3b52);
/// The floor of a slot, groove or window.
pub const WELL: Color = hex(0x011c34);
/// Outlines and the shadow under carved letters.
pub const INK: Color = hex(0x010000);
/// The mid stone a hovered plaque turns to.
pub const STONE_MID: Color = hex(0x5d778d);
/// Behind the loading slab.
pub const NIGHT: Color = hex(0x010e1a);
/// Over the world behind an out-of-play (timber) menu.
pub const DIMMER: Color = Color::srgba(1.0 / 255.0, 14.0 / 255.0, 26.0 / 255.0, 0.45);

/// The shadow under a line of body text: one native pixel down and right,
/// in ink. (Bevy's default is 4 px, which at a UI scale of 2 reads as a
/// second copy of the line, 8 px off.)
pub fn text_shadow() -> TextShadow {
    TextShadow {
        offset: Vec2::splat(1.0),
        color: INK,
    }
}

/// WCAG 2 contrast between two colours.
pub fn contrast(a: Color, b: Color) -> f32 {
    let lum = |c: Color| {
        let c = c.to_linear();
        0.2126 * c.red + 0.7152 * c.green + 0.0722 * c.blue
    };
    let (x, y) = (lum(a), lum(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

// ---------------------------------------------------------------- slices.json

/// One piece of the kit as `slices.json` describes it: its native size and,
/// for a 9-slice, its margins in native pixels (top, right, bottom, left).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub w: u32,
    pub h: u32,
    pub slice: Option<[u32; 4]>,
}

/// What the game reads from `assets/ui/slices.json`.
#[derive(Clone, Debug, Default)]
pub struct Slices {
    pub pieces: HashMap<String, Piece>,
    /// Each in-play rock panel's native size, borders included.
    pub rock_panels: HashMap<String, (u32, u32)>,
    /// Render texels a native pixel: the pieces (8) and the rock panels (4).
    pub render_px: u32,
    pub rock_px: u32,
}

impl Slices {
    pub fn parse(text: &str) -> Result<Self, String> {
        let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let int = |v: &serde_json::Value| v.as_u64().map(|n| n as u32);
        let render_px = v["render_px_per_native_px"]
            .as_u64()
            .ok_or("no render_px")? as u32;
        let rock_px = v["rock_px"].as_u64().ok_or("no rock_px")? as u32;
        let mut pieces = HashMap::new();
        for (name, p) in v["pieces"].as_object().ok_or("no pieces")? {
            let w = int(&p["w"]).ok_or_else(|| format!("{name}: no w"))?;
            let h = int(&p["h"]).ok_or_else(|| format!("{name}: no h"))?;
            let slice = match &p["slice"] {
                serde_json::Value::Null => None,
                serde_json::Value::Number(n) => Some([n.as_u64().unwrap_or(0) as u32; 4]),
                serde_json::Value::Array(a) if a.len() == 4 => {
                    let m: Vec<u32> = a.iter().filter_map(int).collect();
                    (m.len() == 4).then(|| [m[0], m[1], m[2], m[3]])
                }
                other => return Err(format!("{name}: slice {other}")),
            };
            pieces.insert(name.clone(), Piece { w, h, slice });
        }
        let mut rock_panels = HashMap::new();
        for (name, size) in v["rock_panels"].as_object().ok_or("no rock_panels")? {
            let (Some(w), Some(h)) = (int(&size[0]), int(&size[1])) else {
                return Err(format!("rock panel {name}"));
            };
            rock_panels.insert(name.clone(), (w, h));
        }
        Ok(Self {
            pieces,
            rock_panels,
            render_px,
            rock_px,
        })
    }

    pub fn load() -> Self {
        let path = assets_dir().join("ui/slices.json");
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        Self::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    pub fn piece(&self, name: &str) -> Piece {
        *self
            .pieces
            .get(name)
            .unwrap_or_else(|| panic!("slices.json has no piece {name}"))
    }
}

// ---------------------------------------------------------------- the pieces

/// The stone pieces the game draws, by their name in `slices.json` (the file
/// is `assets/ui/stone/<name>.png`).
pub const STONE: [&str; 22] = [
    "frame",
    "lintel",
    "slot-idle",
    "slot-hover",
    "slot-selected",
    "slot-disabled",
    "button-idle",
    "button-hover",
    "button-pressed",
    "button-disabled",
    "button-focus",
    "slider-track",
    "check-off",
    "check-on",
    "tooltip",
    "bar",
    "fill-progress",
    "compass",
    "compass-pointer",
    "compass-town",
    "map-frame",
    "emblem",
];

/// The oak pieces the timber menus draw (`assets/ui/oak/<name>.png`; named
/// `oak/<name>` in `slices.json`).
pub const OAK: [&str; 10] = [
    "frame",
    "button-idle",
    "button-hover",
    "button-pressed",
    "button-disabled",
    "button-focus",
    "tab-open",
    "tab-shut",
    "slider-knob",
    "tooltip",
];

/// The rock panels the game draws, by their name in `rock_panels`
/// (`assets/ui/rock/rock-<w>x<h>.png`).
pub const ROCKS: [&str; 2] = ["pack", "loading"];

/// Which set a control is drawn in: dressed stone in play, carved oak out of
/// play (owner, 2026-11-14: "main menu I think should use the timber stuff,
/// but the game I like the rock stuff", "pause menu = timber too").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Set {
    Stone,
    Oak,
}

impl Set {
    /// The key a piece of this set is filed under.
    pub fn key(self, piece: &str) -> String {
        match self {
            Set::Stone => piece.to_string(),
            Set::Oak => format!("oak/{piece}"),
        }
    }
}

/// A decoded render, kept so the kit can be baked again at another scale.
struct Source {
    handle: Handle<Image>,
    pixels: Arc<RgbaImage>,
    /// Render texels a native pixel in `pixels`.
    per_native: u32,
    /// A frame with no centre (the map's): the face inside its margins is
    /// cleared, since a slicer always draws one.
    hollow: Option<[u32; 4]>,
}

/// The kit's bitmap font (`assets/fonts/kit-font.png` + `.json`): 5 x 7
/// capitals, digits and punctuation, drawn as whole texels, nearest.
#[derive(Clone, Debug)]
pub struct KitFont {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    /// Atlas index and width of each glyph.
    pub glyphs: HashMap<char, (usize, u32)>,
    pub height: u32,
    pub spacing: u32,
    pub space: u32,
}

impl KitFont {
    /// A glyph for `c`: lower case is drawn as upper, and anything the font
    /// lacks as its `?`.
    pub fn glyph(&self, c: char) -> Option<(usize, u32)> {
        let up = c.to_ascii_uppercase();
        self.glyphs
            .get(&up)
            .or_else(|| self.glyphs.get(&'?'))
            .copied()
    }

    /// The width of `text` in native pixels.
    pub fn width(&self, text: &str) -> u32 {
        let mut w = 0;
        let mut n = 0;
        for c in text.chars() {
            w += if c == ' ' {
                self.space
            } else {
                self.glyph(c).map_or(0, |g| g.1)
            };
            n += 1;
        }
        w + self.spacing * n.max(1u32).saturating_sub(1)
    }
}

/// The kit: every piece's handle, its margins, the fonts and the scale the
/// pieces are baked for.
#[derive(Resource)]
pub struct UiKit {
    pub slices: Slices,
    /// Screen pixels a native pixel; what the pieces are baked for.
    pub scale: u32,
    images: HashMap<String, Handle<Image>>,
    sources: Vec<Source>,
    pub font: KitFont,
    /// The body face, VT323 (OFL), installed as the default font so every
    /// `Text` reads in it.
    pub body: Handle<Font>,
}

impl UiKit {
    /// A piece's image by its `slices.json` key (`oak/` for oak, `rock/` +
    /// panel name for a rock panel).
    pub fn image(&self, key: &str) -> Handle<Image> {
        self.images
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("the UI kit has no {key}"))
    }

    /// A piece of `set` drawn whole, stretched to its node (an unsliced
    /// piece: a slot, a knob, a checkbox).
    pub fn whole(&self, set: Set, piece: &str) -> ImageNode {
        ImageNode::new(self.image(&set.key(piece))).with_mode(NodeImageMode::Stretch)
    }

    /// A 9-slice of `set`'s `piece`, its corners fixed at their native size
    /// at every UI scale and its sides and centre stretched (spec delta,
    /// "Panels are 9-slice pieces with finished corners").
    pub fn nine(&self, set: Set, piece: &str) -> ImageNode {
        let key = set.key(piece);
        ImageNode::new(self.image(&key)).with_mode(self.slicer(&key))
    }

    /// The slicer of a piece at the kit's current scale.
    pub fn slicer(&self, key: &str) -> NodeImageMode {
        let piece = self.slices.piece(key);
        let [top, right, bottom, left] = piece.slice.unwrap_or([0; 4]);
        NodeImageMode::Sliced(slicer_at([top, right, bottom, left], self.scale))
    }

    /// The baked rock of an in-play panel, at exactly that panel's size: no
    /// tiled rim, no repeated lip shadow (spec delta, "A rock frame never
    /// shows a repeat").
    pub fn rock(&self, panel: &str) -> (ImageNode, (u32, u32)) {
        let size = self.slices.rock_panels[panel];
        (
            ImageNode::new(self.image(&format!("rock/{panel}"))).with_mode(NodeImageMode::Stretch),
            size,
        )
    }

    /// Bake every piece for `scale` screen pixels a native pixel, into the
    /// handles the nodes already hold.
    fn bake(&mut self, images: &mut Assets<Image>, scale: u32) {
        for source in &self.sources {
            let image = bake(&source.pixels, source.per_native, scale, source.hollow);
            let _ = images.insert(&source.handle, image);
        }
        self.scale = scale;
    }
}

/// A slicer whose margins are `margins` native pixels on a piece baked at
/// `scale` texels a native pixel. The corner's own scale is capped at one
/// over that, so a corner is always its native size in UI units (and so
/// `margin * UiScale` screen pixels) however large the node: Bevy otherwise
/// grows the corners with the node until the texture's own size.
pub fn slicer_at([top, right, bottom, left]: [u32; 4], scale: u32) -> TextureSlicer {
    let s = scale as f32;
    TextureSlicer {
        border: BorderRect {
            min_inset: Vec2::new(left as f32 * s, top as f32 * s),
            max_inset: Vec2::new(right as f32 * s, bottom as f32 * s),
        },
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0 / s,
    }
}

/// A smooth render resampled to `scale` texels a native pixel, sampled
/// linear (the owner's UI exception). Premultiplied while it is filtered, so
/// the transparent margin does not darken the edge.
pub fn bake(pixels: &RgbaImage, per_native: u32, scale: u32, hollow: Option<[u32; 4]>) -> Image {
    let (w, h) = (
        pixels.width() / per_native * scale,
        pixels.height() / per_native * scale,
    );
    let mut out = if scale == per_native {
        pixels.clone()
    } else {
        let premul: ImageBuffer<Rgba<f32>, Vec<f32>> =
            ImageBuffer::from_fn(pixels.width(), pixels.height(), |x, y| {
                let [r, g, b, a] = pixels.get_pixel(x, y).0.map(|c| c as f32 / 255.0);
                Rgba([r * a, g * a, b * a, a])
            });
        let small = image::imageops::resize(
            &premul,
            w.max(1),
            h.max(1),
            image::imageops::FilterType::CatmullRom,
        );
        ImageBuffer::from_fn(small.width(), small.height(), |x, y| {
            let [r, g, b, a] = small.get_pixel(x, y).0;
            let a = a.clamp(0.0, 1.0);
            let un = |c: f32| {
                if a > 1e-4 {
                    (c / a).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            };
            Rgba([
                (un(r) * 255.0).round() as u8,
                (un(g) * 255.0).round() as u8,
                (un(b) * 255.0).round() as u8,
                (a * 255.0).round() as u8,
            ])
        })
    };
    if let Some([top, right, bottom, left]) = hollow {
        let (x0, y0) = (left * scale, top * scale);
        let (x1, y1) = (
            out.width().saturating_sub(right * scale),
            out.height().saturating_sub(bottom * scale),
        );
        for y in y0..y1 {
            for x in x0..x1 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }
    let mut image = Image::from_dynamic(
        image::DynamicImage::ImageRgba8(out),
        true,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
    image
}

/// A PNG decoded now, as RGBA.
fn decode(path: &Path) -> RgbaImage {
    image::open(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .to_rgba8()
}

/// The kit font's sheet as an image sampled nearest (pixel art, whole texels).
pub fn font_image(pixels: RgbaImage) -> Image {
    let mut image = Image::from_dynamic(
        image::DynamicImage::ImageRgba8(pixels),
        true,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// The font's atlas layout, each glyph's character and width in atlas order,
/// and its height, spacing and space advance.
pub type FontTable = (TextureAtlasLayout, Vec<(char, u32)>, [u32; 3]);

/// The kit font's glyph table from `kit-font.json`.
pub fn font_layout(json: &str) -> Result<FontTable, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let height = v["height"].as_u64().ok_or("height")? as u32;
    let spacing = v["spacing"].as_u64().ok_or("spacing")? as u32;
    let space = v["space"].as_u64().ok_or("space")? as u32;
    let glyphs = v["glyphs"].as_object().ok_or("glyphs")?;
    let width = glyphs
        .values()
        .filter_map(|g| Some(g["x"].as_u64()? + g["w"].as_u64()?))
        .max()
        .unwrap_or(1) as u32;
    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(width, height));
    let mut order = Vec::new();
    let mut names: Vec<_> = glyphs.iter().collect();
    names.sort_by(|a, b| a.0.cmp(b.0));
    for (name, g) in names {
        let Some(c) = name.chars().next().filter(|_| name.chars().count() == 1) else {
            continue;
        };
        let x = g["x"].as_u64().ok_or("x")? as u32;
        let w = g["w"].as_u64().ok_or("w")? as u32;
        layout.add_texture(URect::new(x, 0, x + w, height));
        order.push((c, w));
    }
    Ok((layout, order, [height, spacing, space]))
}

/// Load the kit into the world: decode every piece and bake it for the
/// window's scale, build the font atlas, install VT323 as the default font,
/// and set `UiScale`. Idempotent.
pub fn install(world: &mut World) {
    if world.contains_resource::<UiKit>() {
        return;
    }
    let dir = assets_dir();
    let slices = Slices::load();
    let (physical_h, factor) = primary_height(world);
    let scale = ui_scale_for(physical_h);
    let mut sources = Vec::new();
    let mut images_by_key = HashMap::new();
    let mut add = |key: String, path: PathBuf, per_native: u32, hollow: Option<[u32; 4]>| {
        let pixels = Arc::new(decode(&path));
        sources.push((key, pixels, per_native, hollow));
    };
    for name in STONE {
        let hollow = (name == "map-frame")
            .then(|| slices.piece(name).slice)
            .flatten();
        add(
            name.to_string(),
            dir.join(format!("ui/stone/{name}.png")),
            slices.render_px,
            hollow,
        );
    }
    for name in OAK {
        add(
            format!("oak/{name}"),
            dir.join(format!("ui/oak/{name}.png")),
            slices.render_px,
            None,
        );
    }
    for name in ROCKS {
        let (w, h) = slices.rock_panels[name];
        add(
            format!("rock/{name}"),
            dir.join(format!("ui/rock/rock-{w}x{h}.png")),
            slices.rock_px,
            None,
        );
    }
    let mut kept = Vec::new();
    {
        let mut images = world.resource_mut::<Assets<Image>>();
        for (key, pixels, per_native, hollow) in sources {
            let handle = images.add(bake(&pixels, per_native, scale, hollow));
            images_by_key.insert(key, handle.clone());
            kept.push(Source {
                handle,
                pixels,
                per_native,
                hollow,
            });
        }
    }
    let font_json = std::fs::read_to_string(dir.join("fonts/kit-font.json"))
        .expect("assets/fonts/kit-font.json");
    let (layout, order, [height, spacing, space]) = font_layout(&font_json).expect("kit-font.json");
    let sheet = world
        .resource_mut::<Assets<Image>>()
        .add(font_image(decode(&dir.join("fonts/kit-font.png"))));
    let layout = world
        .resource_mut::<Assets<TextureAtlasLayout>>()
        .add(layout);
    let glyphs = order
        .into_iter()
        .enumerate()
        .map(|(i, (c, w))| (c, (i, w)))
        .collect();
    // VT323 in place of Bevy's default face: every `Text` the game has, and
    // every one it will have, reads in the body font without naming it.
    let body = Handle::<Font>::default();
    if let Some(mut fonts) = world.get_resource_mut::<Assets<Font>>() {
        let bytes = std::fs::read(dir.join("fonts/VT323-Regular.ttf"))
            .expect("assets/fonts/VT323-Regular.ttf");
        let font = Font::try_from_bytes(bytes).expect("VT323 parses");
        let _ = fonts.insert(AssetId::default(), font);
    }
    world.insert_resource(UiScale(scale as f32 / factor));
    world.insert_resource(UiKit {
        slices,
        scale,
        images: images_by_key,
        sources: kept,
        font: KitFont {
            image: sheet,
            layout,
            glyphs,
            height,
            spacing,
            space,
        },
        body,
    });
    info!("UI_KIT scale={scale} window_height={physical_h}");
}

/// The kit in a bare app with no window and no renderer, for tests of the
/// screens that are built from it: the asset stores it fills, then
/// [`install`] (scale 2, as a 900 px window).
pub fn install_bare(app: &mut App) {
    app.init_resource::<Assets<Image>>()
        .init_resource::<Assets<TextureAtlasLayout>>()
        .init_resource::<Assets<Font>>();
    install(app.world_mut());
}

/// The primary window's physical height and scale factor (900 and 1 before
/// there is one).
fn primary_height(world: &mut World) -> (u32, f32) {
    let mut q = world.query_filtered::<&Window, With<PrimaryWindow>>();
    q.iter(world).next().map_or((900, 1.0), |w| {
        (w.resolution.physical_height(), w.resolution.scale_factor())
    })
}

/// The UI scale follows the window: a whole number of screen pixels a native
/// pixel (spec delta, "The UI scale follows the window height on the
/// desktop"), the kit baked again when it moves a step.
#[allow(clippy::type_complexity)]
pub fn follow_window(
    windows: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut ui_scale: ResMut<UiScale>,
    mut kit: ResMut<UiKit>,
    mut images: ResMut<Assets<Image>>,
    mut pieces: Query<(&KitPiece, &mut ImageNode)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = ui_scale_for(window.resolution.physical_height());
    let want = scale as f32 / window.resolution.scale_factor();
    if (ui_scale.0 - want).abs() > 1e-4 {
        ui_scale.0 = want;
    }
    if scale != kit.scale {
        kit.bake(&mut images, scale);
        for (piece, mut node) in &mut pieces {
            if matches!(node.image_mode, NodeImageMode::Sliced(_)) {
                node.image_mode = kit.slicer(&piece.0);
            }
        }
        info!("UI_KIT rebaked scale={scale}");
    }
}

/// Marks a node drawing a kit piece by key, so a change of scale can give its
/// slicer the new margins.
#[derive(Component, Clone, Debug)]
pub struct KitPiece(pub String);

// ---------------------------------------------------------------- kit text

/// A line in the kit's pixel font: titles, plaque and tab labels, section
/// labels, compass letters and key caps (spec delta, "Labels are drawn in the
/// kit's pixel font"). Each glyph is an image node of whole texels at the UI
/// scale, sampled nearest; `size` multiplies it (2 for a big title).
#[derive(Component, Clone, Debug, PartialEq)]
#[require(Node)]
pub struct KitText {
    pub text: String,
    pub colour: Color,
    /// A one-pixel shadow under the letters, as carved.
    pub shadow: Option<Color>,
    /// The shadow drawn all round the letters, a one-pixel outline, for a
    /// count laid over an icon.
    pub outline: bool,
    pub size: u32,
}

impl KitText {
    pub fn new(text: impl Into<String>, colour: Color) -> Self {
        Self {
            text: text.into(),
            colour,
            shadow: None,
            outline: false,
            size: 1,
        }
    }

    /// A one-pixel outline all round in `colour` (the slot counts).
    pub fn outline(mut self, colour: Color) -> Self {
        self.shadow = Some(colour);
        self.outline = true;
        self
    }

    pub fn shadow(mut self, shadow: Color) -> Self {
        self.shadow = Some(shadow);
        self
    }

    pub fn size(mut self, size: u32) -> Self {
        self.size = size.max(1);
        self
    }
}

/// What a `KitText` was last built as.
#[derive(Component)]
pub struct KitTextBuilt {
    text: String,
    shadow: bool,
    outline: bool,
    size: u32,
}

/// One glyph's foreground (or shadow) node.
#[derive(Component)]
pub struct KitGlyph {
    shadow: bool,
}

/// Build a `KitText`'s glyphs when its words change, and recolour them when
/// only its colour does (the compass fades its letters every frame).
#[allow(clippy::type_complexity)]
pub fn draw_kit_text(
    mut commands: Commands,
    kit: Option<Res<UiKit>>,
    mut texts: Query<
        (
            Entity,
            &KitText,
            &mut Node,
            Option<&KitTextBuilt>,
            Option<&Children>,
        ),
        Changed<KitText>,
    >,
    mut glyphs: Query<(&KitGlyph, &mut ImageNode)>,
    holders: Query<&Children, Without<KitText>>,
) {
    let Some(kit) = kit else {
        return;
    };
    let font = &kit.font;
    for (entity, text, mut node, built, children) in &mut texts {
        let same = built.is_some_and(|b| {
            b.text == text.text
                && b.shadow == text.shadow.is_some()
                && b.outline == text.outline
                && b.size == text.size
        });
        if same {
            // Only the colour moved.
            for child in children.into_iter().flatten() {
                for leaf in holders.get(*child).into_iter().flatten() {
                    if let Ok((glyph, mut image)) = glyphs.get_mut(*leaf) {
                        let want = if glyph.shadow {
                            text.shadow.unwrap_or(Color::NONE)
                        } else {
                            text.colour
                        };
                        if image.color != want {
                            image.color = want;
                        }
                    }
                }
            }
            continue;
        }
        let s = text.size as f32;
        node.flex_direction = FlexDirection::Row;
        node.column_gap = px(font.spacing as f32 * s);
        node.flex_shrink = 0.0;
        let tall = (font.height + u32::from(text.shadow.is_some() && !text.outline)) as f32 * s;
        node.height = px(tall);
        commands.entity(entity).despawn_related::<Children>();
        commands.entity(entity).insert(KitTextBuilt {
            text: text.text.clone(),
            shadow: text.shadow.is_some(),
            outline: text.outline,
            size: text.size,
        });
        commands.entity(entity).with_children(|row| {
            for c in text.text.chars() {
                if c == ' ' {
                    // A space is its own advance, less the gap either side.
                    let w = (font.space as f32 - font.spacing as f32).max(0.0) * s;
                    row.spawn(Node {
                        width: px(w),
                        height: px(tall),
                        ..default()
                    });
                    continue;
                }
                let Some((index, w)) = font.glyph(c) else {
                    continue;
                };
                let atlas = || TextureAtlas {
                    layout: font.layout.clone(),
                    index,
                };
                let leaf = |left: f32, top: f32| Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(w as f32 * s),
                    height: px(font.height as f32 * s),
                    ..default()
                };
                row.spawn(Node {
                    width: px(w as f32 * s),
                    height: px(tall),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|cell| {
                    if let Some(shadow) = text.shadow {
                        let offsets: &[(f32, f32)] = if text.outline {
                            &[(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)]
                        } else {
                            &[(0.0, 1.0)]
                        };
                        for (dx, dy) in offsets {
                            cell.spawn((
                                ImageNode::from_atlas_image(font.image.clone(), atlas())
                                    .with_color(shadow),
                                leaf(dx * s, dy * s),
                                KitGlyph { shadow: true },
                            ));
                        }
                    }
                    cell.spawn((
                        ImageNode::from_atlas_image(font.image.clone(), atlas())
                            .with_color(text.colour),
                        leaf(0.0, 0.0),
                        KitGlyph { shadow: false },
                    ));
                });
            }
        });
    }
}

// ---------------------------------------------------------------- plaques

/// A plaque (a button) of `set`: the idle, hover, pressed, disabled and
/// focus pieces swap as its `Interaction` and this state change.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plaque {
    pub set: Set,
    /// Drawn with the rune cut round it: the default choice of a menu.
    pub focus: bool,
    /// Drawn dead: it does nothing.
    pub disabled: bool,
    /// Held down: an on toggle, the open overlay, the chosen value.
    pub latched: bool,
}

impl Plaque {
    pub fn new(set: Set) -> Self {
        Self {
            set,
            focus: false,
            disabled: false,
            latched: false,
        }
    }

    /// The piece for this plaque in `interaction`.
    pub fn piece(&self, interaction: Interaction) -> &'static str {
        if self.disabled {
            "button-disabled"
        } else if self.latched || interaction == Interaction::Pressed {
            "button-pressed"
        } else if interaction == Interaction::Hovered {
            "button-hover"
        } else if self.focus {
            "button-focus"
        } else {
            "button-idle"
        }
    }
}

/// The label inside a plaque, which sinks a pixel when it is pressed.
#[derive(Component)]
pub struct PlaqueLabel;

/// The bundle of a plaque node: a button, its 9-slice and its state. The
/// caller adds the label (`plaque_label`) and whatever it acts on.
pub fn plaque(kit: &UiKit, set: Set, min_width: f32) -> impl Bundle {
    (
        Button,
        kit.nine(set, "button-idle"),
        KitPiece(set.key("button-idle")),
        Plaque::new(set),
        Node {
            min_width: px(min_width),
            height: px(22),
            padding: UiRect::new(px(10), px(10), px(8), px(0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_shrink: 0.0,
            ..default()
        },
    )
}

/// A plaque's label: chalk inlaid in stone, cut dark in oak (design 4).
pub fn plaque_label(set: Set, text: &str) -> impl Bundle {
    let words = match set {
        Set::Stone => KitText::new(text, LABEL).shadow(WELL),
        Set::Oak => KitText::new(text, OAK_LABEL),
    };
    (words, PlaqueLabel)
}

/// Repaint a plaque whose interaction or state changed.
#[allow(clippy::type_complexity)]
pub fn paint_plaques(
    kit: Option<Res<UiKit>>,
    mut plaques: Query<
        (
            &Interaction,
            &Plaque,
            &mut ImageNode,
            &mut KitPiece,
            &Children,
        ),
        Or<(Changed<Interaction>, Changed<Plaque>)>,
    >,
    mut labels: Query<(&mut Node, &mut KitText), With<PlaqueLabel>>,
) {
    let Some(kit) = kit else {
        return;
    };
    for (interaction, plaque, mut image, mut piece, children) in &mut plaques {
        let key = plaque.set.key(plaque.piece(*interaction));
        if piece.0 != key {
            image.image = kit.image(&key);
            piece.0 = key;
        }
        let sunk = plaque.latched || *interaction == Interaction::Pressed;
        for child in children.iter() {
            if let Ok((mut node, mut text)) = labels.get_mut(child) {
                let top = px(if sunk && !plaque.disabled { 1.0 } else { 0.0 });
                if node.top != top {
                    node.top = top;
                }
                let colour = match (plaque.set, plaque.disabled) {
                    (_, true) => LABEL_DIM,
                    (Set::Stone, false) => LABEL,
                    (Set::Oak, false) => OAK_LABEL,
                };
                if text.colour != colour {
                    text.colour = colour;
                }
            }
        }
    }
}

/// Install the kit before the first frame and keep it following the window.
pub struct UiKitPlugin;

impl Plugin for UiKitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (follow_window, paint_plaques, draw_kit_text)
                .chain()
                .before(UiSystems::Prepare)
                .run_if(resource_exists::<UiKit>),
        );
    }

    fn finish(&self, app: &mut App) {
        install(app.world_mut());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slices() -> Slices {
        Slices::load()
    }

    /// The window height picks a whole number: 2 at 900 and 1080, 3 at
    /// 1440, 4 at 2160, never below 1 (design 6).
    #[test]
    fn the_ui_scale_is_a_whole_number_from_the_window_height() {
        for (h, s) in [
            (240, 1),
            (449, 1),
            (720, 1),
            (899, 1),
            (900, 2),
            (1080, 2),
            (1349, 2),
            (1350, 3),
            (1440, 3),
            (2160, 4),
        ] {
            assert_eq!(ui_scale_for(h), s, "{h} px");
        }
    }

    /// No two opposite margins in `slices.json` add up to more than the
    /// piece, and every piece the game draws is in it, at the size its render
    /// has (the art's own file, not a copy).
    #[test]
    fn slice_margins_never_exceed_a_piece() {
        let slices = slices();
        for (name, p) in &slices.pieces {
            if let Some([top, right, bottom, left]) = p.slice {
                assert!(left + right <= p.w, "{name}: {left}+{right} > {}", p.w);
                assert!(top + bottom <= p.h, "{name}: {top}+{bottom} > {}", p.h);
            }
        }
        let dir = assets_dir();
        for (set, names) in [(Set::Stone, &STONE[..]), (Set::Oak, &OAK[..])] {
            for name in names {
                let key = set.key(name);
                let p = slices.piece(&key);
                let file = match set {
                    Set::Stone => format!("ui/stone/{name}.png"),
                    Set::Oak => format!("ui/oak/{name}.png"),
                };
                let (w, h) = image::image_dimensions(dir.join(&file)).unwrap();
                assert_eq!(
                    (w, h),
                    (p.w * slices.render_px, p.h * slices.render_px),
                    "{file} is the 8x render of a {}x{} piece",
                    p.w,
                    p.h
                );
            }
        }
        for name in ROCKS {
            let (pw, ph) = slices.rock_panels[name];
            let (w, h) =
                image::image_dimensions(dir.join(format!("ui/rock/rock-{pw}x{ph}.png"))).unwrap();
            assert_eq!((w, h), (pw * slices.rock_px, ph * slices.rock_px), "{name}");
        }
    }

    /// The kit's textures are sampled linear (the owner's UI exception), and
    /// the pixel font nearest; baked for a scale they are exactly that many
    /// texels a native pixel, so a corner is its native size times the scale.
    #[test]
    fn the_kit_bakes_linear_and_the_font_nearest() {
        let slices = slices();
        let dir = assets_dir();
        let px = decode(&dir.join("ui/stone/button-idle.png"));
        for scale in [1, 2, 3, 4] {
            let image = bake(&px, slices.render_px, scale, None);
            assert_eq!(image.width(), 48 * scale);
            assert_eq!(image.height(), 22 * scale);
            assert_eq!(image.sampler, ImageSampler::linear(), "kit at {scale}");
            let NodeImageMode::Sliced(s) =
                NodeImageMode::Sliced(slicer_at(slices.piece("button-idle").slice.unwrap(), scale))
            else {
                unreachable!()
            };
            // A corner of 8 native px is 8 * scale texels, drawn at one over
            // the scale: 8 UI units, 8 * UiScale screen pixels.
            assert_eq!(s.border.min_inset.x * s.max_corner_scale, 8.0);
        }
        let font = font_image(decode(&dir.join("fonts/kit-font.png")));
        assert_eq!(font.sampler, ImageSampler::nearest(), "the pixel font");
    }

    /// The map frame is drawn with no centre: the slicer always fills one,
    /// so the bake clears it.
    #[test]
    fn the_map_frame_is_hollow() {
        let slices = slices();
        let p = slices.piece("map-frame");
        let px = decode(&assets_dir().join("ui/stone/map-frame.png"));
        let image = bake(&px, slices.render_px, 2, p.slice);
        let at = |x: u32, y: u32| image.get_color_at(x, y).unwrap().alpha();
        assert_eq!(at(64, 64), 0.0, "the centre is cleared");
        assert!(at(64, 4) > 0.5, "the moulding stays");
    }

    /// The kit font covers every letter, digit and the punctuation the
    /// labels use, and C, G and S are the kit's open ones.
    #[test]
    fn the_kit_font_has_every_label_glyph() {
        let json = std::fs::read_to_string(assets_dir().join("fonts/kit-font.json")).unwrap();
        let (layout, order, [height, ..]) = font_layout(&json).unwrap();
        assert_eq!(height, 7);
        let have: std::collections::HashSet<char> = order.iter().map(|g| g.0).collect();
        for c in ('A'..='Z').chain('0'..='9') {
            assert!(have.contains(&c), "{c}");
        }
        assert_eq!(layout.len(), order.len());
    }

    /// The interface's colours come from this module: no UI file keeps its
    /// own `const` colour (spec, "One palette module for the interface").
    /// The map's own art (its sites, its ramps) is the map's, not the
    /// interface's, and is not scanned.
    #[test]
    fn no_ui_file_keeps_its_own_palette() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for file in [
            "loading.rs",
            "desktop/menu.rs",
            "desktop/pack.rs",
            "desktop/slots.rs",
            "desktop/equipment.rs",
            "desktop/guide.rs",
            "desktop/compass_bar.rs",
            "desktop/overlay_ui.rs",
            "desktop/time_ui.rs",
            "desktop/weather_ui.rs",
            "desktop/hud.rs",
        ] {
            let text = std::fs::read_to_string(src.join(file)).unwrap();
            for (n, line) in text.lines().enumerate() {
                let l = line.trim_start();
                let own = l.starts_with("const ")
                    || l.starts_with("pub const ")
                    || l.starts_with("pub(super) const ");
                assert!(
                    !(own && (l.contains(": Color = Color::") || l.contains(": (f32, f32, f32)"))),
                    "{file}:{}: a colour of its own: {l}",
                    n + 1
                );
            }
        }
    }

    /// Every text pair a player reads clears 4.5:1 (spec delta, "Text on the
    /// interface is legible"), measured on the tokens themselves.
    #[test]
    fn tokens_clear_four_and_a_half_to_one() {
        let pairs = [
            ("TEXT on FACE", TEXT, FACE),
            ("TEXT on WELL", TEXT, WELL),
            ("TEXT_DIM on FACE", TEXT_DIM, FACE),
            ("TITLE on FACE", TITLE, FACE),
            ("LABEL on idle stone", LABEL, hex(0x3e586e)),
            ("LABEL on hovered stone", LABEL, STONE_MID),
            ("LABEL on pressed stone", LABEL, FACE),
            ("LABEL_DIM on disabled stone", LABEL_DIM, WELL),
            ("TAB_DIM on a shut tab", TAB_DIM, FACE),
            ("CUT on the compass", CUT, hex(0x92a3b2)),
            ("NORTH on the compass", NORTH, hex(0x92a3b2)),
            ("OAK_LABEL on oak", OAK_LABEL, hex(0xab8465)),
            ("RUNE on WELL", RUNE, WELL),
        ];
        for (what, a, b) in pairs {
            let c = contrast(a, b);
            assert!(c >= 4.5, "{what}: {c:.2}:1");
        }
    }
}
