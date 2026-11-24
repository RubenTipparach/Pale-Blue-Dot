//! Talking to the townsfolk, their trade and their job board (`townsfolk`
//! design 5 to 9; M5's Talk and standing tab, in the stone-and-wood kit),
//! and the coin count beside the hotbar.
//!
//! The interact key on a townsperson ([`pbd_app::economy::Talker`]) opens
//! the talk panel: a short line and the buttons the person offers. The
//! town's trader offers TRADE, its job giver WORK, anyone who is owed a job
//! back HAND IN; everyone says BYE. A parcel for the town is handed over on
//! talking to anyone there. TRADE and WORK open their own panels; Escape or
//! BACK returns to the talk, and the interact key or Escape again leaves.
//!
//! Every rule is `pbd_core::economy`'s through `pbd_app::economy::Shop`;
//! this module only shows it and passes the presses on. Labels and plain
//! lines only (CLAUDE.md, "Game text is plain").

use super::menu::{self, Panel, Screen};
use super::scroll_list::{self, ListView};
use super::slots::{self, Hotbar, ItemIcons};
use bevy::prelude::*;
use pbd_app::economy::{
    Economy, EconomyConfig, Shop, Talker, TownInfo, Why, fish_ids, job_line, keepers, roster,
    town_info,
};
use pbd_app::interact::{Act, Interact, Offer, Reader};
use pbd_app::saves::WorldSave;
use pbd_app::towns::Towns;
use pbd_app::ui_kit::{self, KitText, Localized, Set, Tooltip, UiKit, tr};
use pbd_core::economy::jobs::Job;
use pbd_core::economy::trade::Basket;
use pbd_core::economy::values::{Shelf, Values, item_of, trade_code};
use pbd_core::inventory::Stack;
use pbd_core::locale::tr_with;

/// How near a townsperson must be to talk to, metres from the eye.
pub const TALK_REACH_M: f32 = 3.0;
/// The rows a trade column and the board show at once; a longer list
/// scrolls (design 38b).
const VISIBLE_ROWS: usize = 7;
/// A row's height and the gap under it, native px: a slot well and 2.
const ROW_H: f32 = 24.0;
const ROW_GAP: f32 = 2.0;
/// The window's height: the panel's, whatever the list holds.
const LIST_H: f32 = VISIBLE_ROWS as f32 * (ROW_H + ROW_GAP) - ROW_GAP;
/// A trade row's width, and a column's with its scroll bar.
const TRADE_ROW_W: f32 = 214.0;
const BAR_GAP: f32 = 3.0;
/// A board row's width.
const BOARD_ROW_W: f32 = 390.0;

/// How a dialog with a townsperson is framed (`townsfolk` design 37a,
/// framing; owner, survey P1, 2026-11-24: "Move the camera so the merchant
/// is on left, and the trade UI is on the right"; and the second pass the
/// same day: "center the character between the left most edge of panel
/// and left edge of the screen", "add some lights that fade in light the
/// character a bit", "same presentation style for dialog boxes of any kind
/// of interaction with npcs"), from `assets/config/talk.ron`.
#[derive(Resource, Clone, Copy, Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TalkView {
    /// Seconds the look takes to turn to the townsperson; 0 turns at once.
    pub turn_secs: f32,
    /// The height the townsperson's aim point lands at, normalised screen
    /// space (-1 bottom to 1 top); across, it lands in the middle of the
    /// open space left of the panel ([`open_space_x`]).
    pub frame_y: f32,
    /// The aim point's height over the townsperson's feet, metres.
    pub aim_height_m: f32,
    /// The gap between a talk panel and the screen's right edge, native px.
    pub panel_margin_px: f32,
    /// The talk light's colour, linear RGB.
    pub light_tint: [f32; 3],
    /// Its level on the field's 0..15 scale (a torch is 14).
    pub light_level: f32,
    /// How far it reaches, metres.
    pub light_reach_m: f32,
    /// Seconds it takes to fade in, and out; 0 is at once.
    pub light_fade_secs: f32,
    /// Its height over the townsperson's feet, metres.
    pub light_rise_m: f32,
    /// How far it stands out from them toward the eye, level, metres.
    pub light_toward_m: f32,
}

impl TalkView {
    /// Parse and validate `talk.ron`'s text.
    pub fn parse(text: &str) -> Result<Self, String> {
        let view: Self = ron::from_str(text).map_err(|e| e.to_string())?;
        let within = |name: &str, v: f32, lo: f32, hi: f32| {
            (v.is_finite() && (lo..=hi).contains(&v))
                .then_some(())
                .ok_or(format!("{name} must be from {lo} to {hi}"))
        };
        within("turn_secs", view.turn_secs, 0.0, 2.0)?;
        within("frame_y", view.frame_y, -0.95, 0.95)?;
        within("aim_height_m", view.aim_height_m, 0.0, 3.0)?;
        within("panel_margin_px", view.panel_margin_px, 0.0, 200.0)?;
        for c in view.light_tint {
            within("light_tint", c, 0.0, 1.0)?;
        }
        within("light_level", view.light_level, 0.0, 15.0)?;
        within("light_reach_m", view.light_reach_m, 0.5, 16.0)?;
        within("light_fade_secs", view.light_fade_secs, 0.0, 2.0)?;
        within("light_rise_m", view.light_rise_m, 0.0, 3.0)?;
        within("light_toward_m", view.light_toward_m, 0.0, 2.0)?;
        Ok(view)
    }
}

impl Default for TalkView {
    fn default() -> Self {
        let path = pbd_app::paths::assets_dir().join("config/talk.ron");
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        Self::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }
}

/// The dialogs opened by interacting with a townsperson, and each panel's
/// width, native px: the one table every NPC dialog's presentation reads
/// (`townsfolk` design 37a, framing, second pass; owner: "same presentation
/// style for dialog boxes of any kind of interaction with npcs"). A screen
/// in it is an NPC dialog: its panel is spawned on the right with
/// [`npc_dialog`], the look turns the townsperson into the open space left
/// of it, and the talk light burns on them while it is open. A new NPC
/// dialog joins this table and gets all three.
pub fn panel_width(screen: Screen) -> Option<f32> {
    match screen {
        Screen::Talk => Some(300.0),
        Screen::Trade => Some(490.0),
        Screen::Board => Some(450.0),
        _ => None,
    }
}

/// Whether `screen` is a dialog with a townsperson.
pub fn is_npc_dialog(screen: Screen) -> bool {
    panel_width(screen).is_some()
}

/// An NPC dialog's layer and framed panel, on the right of the screen
/// `panel_margin_px` in from its edge and centred top to bottom: the one
/// way every NPC dialog's panel is laid out.
pub fn npc_dialog(
    commands: &mut Commands,
    kit: &UiKit,
    view: &TalkView,
    screen: Screen,
    body: impl Bundle,
) {
    let width = panel_width(screen).unwrap_or_default();
    let layer = Node {
        justify_content: JustifyContent::FlexEnd,
        padding: UiRect::right(px(view.panel_margin_px)),
        ..menu::layer()
    };
    commands
        .spawn((layer, GlobalZIndex(10), Panel(screen)))
        .with_children(|layer| {
            layer.spawn((menu::framed(kit, Set::Stone, width), body));
        });
}

/// Where, across, the middle of the open space between the screen's left
/// edge and a panel's left edge falls, in normalised screen space: for a
/// viewport `width` logical px wide, the UI scale `ui_scale`, and a panel
/// `panel_px` native px wide `margin_px` from the right edge
/// (`townsfolk` design 37a, framing, second pass). The panel's left edge is
/// at `width - (margin + panel) * scale`, half that is the middle, and in
/// -1..1 that is `-(margin + panel) * scale / width`; never further left
/// than -0.95, on a screen the panel all but fills.
pub fn open_space_x(width: f32, ui_scale: f32, margin_px: f32, panel_px: f32) -> f32 {
    if width <= 0.0 {
        return -0.95;
    }
    (-(margin_px + panel_px) * ui_scale / width).clamp(-0.95, 0.0)
}

/// The look's turn to frame the townsperson a talk opened with: the eye to
/// their aim point as it was at the press (a townsperson who walks on is
/// not followed), the panel it was last framed for, and the turn itself
/// once it is solved.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct TalkTurn {
    /// Eye to aim point, render space, kept for the talk.
    aim: Option<Vec3>,
    /// The panel the turn was solved for: a different one re-aims.
    framed: Option<Screen>,
    /// From and to, each a tangent heading and a pitch, and how far along.
    turn: Option<((Vec3, f32), (Vec3, f32), f32)>,
}

/// The look (tangent heading, pitch) that puts the direction `to` (eye to
/// the aim point) at `at` on screen, in normalised device coordinates, for
/// a camera with no roll about `up` whose vertical field of view is `fov`
/// radians and width over height `aspect`; as the walker builds its camera
/// (`Transform::looking_to(heading * cos p + up * sin p, up)`). `None` when
/// `to` is straight up or down, or the point cannot be put there.
pub fn frame_look(to: Vec3, up: Vec3, fov: f32, aspect: f32, at: Vec2) -> Option<(Vec3, f32)> {
    let d = to.try_normalize()?;
    let up = up.try_normalize()?;
    let half = (fov * 0.5).tan();
    // The view-space direction that projects to `at` (forward is -z).
    let c = Vec3::new(at.x * half * aspect, at.y * half, -1.0).normalize();
    // Pitch: the camera pitched by p carries c to an elevation of
    // c.y cos p - c.z sin p, which must be the target's.
    let sin_e = d.dot(up).clamp(-1.0, 1.0);
    let r = (c.y * c.y + c.z * c.z).sqrt();
    if sin_e.abs() > r {
        return None;
    }
    let phi = (-c.z).atan2(c.y);
    let pitch = phi - (sin_e / r).acos();
    // Heading: c, pitched, lies `off` to the left of the camera's forward
    // in the tangent plane; the target's own heading is that far left.
    let z = c.y * pitch.sin() + c.z * pitch.cos();
    let off = (-c.x).atan2(-z);
    let flat = (d - up * d.dot(up)).try_normalize()?;
    Some((Quat::from_axis_angle(up, -off) * flat, pitch))
}

/// Turn the walker's look to frame the townsperson the talk opened with,
/// eased over [`TalkView::turn_secs`]; left where it ends when the talk
/// closes. While a talk panel is open the menu holds the pointer and no
/// mouse is read, so this smooths no mouse look.
pub fn turn_to_talker(
    time: Res<Time>,
    (view, ui_scale): (Res<TalkView>, Option<Res<UiScale>>),
    screen: Res<Screen>,
    mut turn: ResMut<TalkTurn>,
    cameras: Query<(&Camera, &Projection), With<Camera3d>>,
    walker: Option<ResMut<pbd_app::walking::WalkingState>>,
) {
    if !is_npc_dialog(*screen) {
        if *turn != TalkTurn::default() {
            *turn = TalkTurn::default();
        }
        return;
    }
    let Some(mut walker) = walker else {
        return;
    };
    // Solved when the dialog opens, and again when the panel changes
    // width (TRADE, WORK, BACK): from wherever the look is, to the same aim.
    if let Some(to) = turn.aim
        && turn.framed != Some(*screen)
        && let Some((camera, projection)) = cameras.iter().find(|(c, _)| c.is_active)
    {
        turn.framed = Some(*screen);
        let lens = match projection {
            Projection::Perspective(p) => Some((p.fov, p.aspect_ratio)),
            _ => None,
        };
        let width = camera.logical_viewport_size().map_or(0.0, |s| s.x);
        let scale = ui_scale.map_or(1.0, |s| s.0);
        let x = open_space_x(
            width,
            scale,
            view.panel_margin_px,
            panel_width(*screen).unwrap_or(0.0),
        );
        let at = Vec2::new(x, view.frame_y);
        if let Some(target) =
            lens.and_then(|(fov, aspect)| frame_look(to, walker.up(), fov, aspect, at))
        {
            turn.turn = Some((walker.view(), target, 0.0));
        }
    }
    let Some((from, to, t)) = turn.turn.as_mut() else {
        return;
    };
    *t = if view.turn_secs > 0.0 {
        (*t + time.delta_secs() / view.turn_secs).min(1.0)
    } else {
        1.0
    };
    let s = *t * *t * (3.0 - 2.0 * *t);
    let up = walker.up();
    let angle = up.dot(from.0.cross(to.0)).atan2(from.0.dot(to.0));
    let heading = Quat::from_axis_angle(up, angle * s) * from.0;
    walker.set_look(heading, from.1 + (to.1 - from.1) * s);
    if *t >= 1.0 {
        turn.turn = None;
    }
}

/// The light on the townsperson an NPC dialog is with (`townsfolk` design
/// 37a, framing, second pass): how far it has faded in (0..1) and who it is
/// on, kept through the fade out after the dialog closes.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct TalkGlow {
    pub level: f32,
    pub on: Option<(u32, u32)>,
}

/// One step of the talk light's fade: toward 1 while a dialog is open and
/// toward 0 after, over `fade_secs` (0 at once).
pub fn glow_step(level: f32, open: bool, dt: f32, fade_secs: f32) -> f32 {
    let target = if open { 1.0 } else { 0.0 };
    if fade_secs <= 0.0 {
        return target;
    }
    let step = dt / fade_secs;
    if open {
        (level + step).min(1.0)
    } else {
        (level - step).max(0.0)
    }
}

/// Where the talk light burns, render space: from the townsperson's `feet`
/// `light_rise_m` up and `light_toward_m` along the level line to the
/// `eye`, `up` the walker's camera's (a difference of render-space points,
/// never a planet-centred position).
pub fn glow_at(view: &TalkView, feet: Vec3, eye: Vec3, up: Vec3) -> Vec3 {
    let to = eye - feet;
    let level = (to - up * to.dot(up)).try_normalize().unwrap_or(Vec3::ZERO);
    feet + up * view.light_rise_m + level * view.light_toward_m
}

/// The talk light: a soft warm carried light in front of and a little above
/// the townsperson's face while an NPC dialog is open, faded in and out
/// over `light_fade_secs`. It goes in the carried list
/// (`carried_light::NearLights`) that the terrain, the water and every
/// field-lit thing evaluate as they do a torch's, with its own reach, so it
/// lights the townsperson as any lamp would, at night and in shade alike.
/// After the transforms are propagated and before the list is gathered.
#[allow(clippy::too_many_arguments)]
pub fn talk_light(
    time: Res<Time>,
    view: Res<TalkView>,
    screen: Res<Screen>,
    talking: Res<Talking>,
    mut glow: ResMut<TalkGlow>,
    talkers: Query<(&Talker, &GlobalTransform)>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    mut near: ResMut<pbd_app::carried_light::NearLights>,
) {
    let with = talking
        .with
        .as_ref()
        .filter(|_| is_npc_dialog(*screen))
        .map(|w| (w.talker.site, w.talker.index));
    if with.is_some() && glow.on != with {
        glow.on = with;
    }
    let level = glow_step(
        glow.level,
        with.is_some(),
        time.delta_secs(),
        view.light_fade_secs,
    );
    if glow.level != level {
        glow.level = level;
    }
    let light = glow.on.filter(|_| level > 0.0).and_then(|(site, index)| {
        let (_, feet) = talkers
            .iter()
            .find(|(t, _)| t.site == site && t.index == index)?;
        let (eye, _) = cameras.iter().find(|(_, c)| c.is_active)?;
        let at = glow_at(
            &view,
            feet.translation(),
            eye.translation(),
            eye.up().as_vec3(),
        );
        let k = view.light_level / 14.0 * level;
        Some((
            pbd_core::light::Carried {
                // In the planet's frame, subtracted in f64 before the cast.
                at: pbd_app::render_origin::local_of(frame.as_deref(), at),
                colour: view.light_tint.map(|c| c * k),
            },
            view.light_reach_m,
        ))
    });
    if level <= 0.0 && glow.on.is_some() {
        glow.on = None;
    }
    let next: Vec<_> = light.into_iter().collect();
    if near.0 != next {
        near.0 = next;
    }
}

/// A list of the trade and board panels that scrolls on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum List {
    Buy,
    Sell,
    Board,
}

/// Where one list is scrolled to, and its filter (none is ALL).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Column {
    pub first: usize,
    pub shelf: Option<Shelf>,
    /// Its rows when last painted, for the wheel's and the drag's clamp.
    pub rows: usize,
}

/// The trade and board lists' scroll and filters, kept while the talk is
/// open and forgotten when it closes; and a thumb being dragged, with where
/// it was taken hold of (native px from its top).
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct TalkScroll {
    pub buy: Column,
    pub sell: Column,
    pub board: Column,
    pub drag: Option<(List, f32)>,
}

impl TalkScroll {
    pub fn column(&mut self, list: List) -> &mut Column {
        match list {
            List::Buy => &mut self.buy,
            List::Sell => &mut self.sell,
            List::Board => &mut self.board,
        }
    }

    /// A list's view as last painted.
    fn view(&mut self, list: List) -> ListView {
        let c = *self.column(list);
        ListView::new(c.rows, VISIBLE_ROWS, c.first)
    }

    /// What the panel shows of the scroll, for its "rebuild when it moves".
    fn key(&self) -> String {
        format!(
            "{:?}{:?}|{:?}{:?}|{}|{}",
            self.buy.first,
            self.buy.shelf,
            self.sell.first,
            self.sell.shelf,
            self.board.first,
            self.drag.map_or(0, |d| d.0 as u8 + 1)
        )
    }
}

/// A list's window, which the wheel scrolls while the pointer is over it.
#[derive(Component)]
pub struct ScrollArea(pub List);
/// A list's scroll bar track; a press on it pages.
#[derive(Component)]
pub struct ScrollTrack(pub List);
/// A list's scroll bar thumb, which drags.
#[derive(Component)]
pub struct ScrollThumb(pub List);
/// A filter tab, and whether it is the chosen one.
#[derive(Component)]
pub struct ShelfTab(pub bool);

/// Who the open talk is with.
#[derive(Clone, Debug, PartialEq)]
pub struct TalkWith {
    pub talker: Talker,
    pub town: TownInfo,
    pub trader: bool,
    pub giver: bool,
}

/// The open talk, and the last thing a press said.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct Talking {
    pub with: Option<TalkWith>,
    pub said: Option<Localized>,
    /// The deal being put together (`townsfolk` design 37a): what the
    /// player takes and gives, emptied on a deal, CLEAR or leaving.
    pub basket: Basket,
}

/// What a talk, trade or board button does.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum TalkAction {
    Trade,
    Work,
    Bye,
    Back,
    /// One of the trader's goods into YOU TAKE.
    Buy(String),
    /// One of the pack's goods into YOU GIVE.
    Sell(String),
    /// One back out of a basket: YOU TAKE (`true`) or YOU GIVE.
    Unbasket(bool, String),
    /// Settle the baskets.
    Deal,
    /// Empty both baskets.
    Clear,
    Take(u64),
    HandIn(u64),
    /// Give a taken job up (`town-missions-tree` design 9).
    Abandon(u64),
    /// A trade column's filter tab: ALL (none) or a shelf.
    Shelf(List, Option<Shelf>),
    /// Buy a horse (`land-transport` design 10): a kept animal, not a good.
    BuyHorse,
}

#[derive(Component)]
pub struct TalkBody;
#[derive(Component)]
pub struct TradeBody;
#[derive(Component)]
pub struct BoardBody;
/// The coin icon, for every price shown.
#[derive(Resource, Clone)]
pub struct CoinIcon(pub Handle<Image>);
/// The lost villager job's own icon (`town-missions-tree` design 10).
#[derive(Resource, Clone)]
pub struct LostIcon(pub Handle<Image>);

/// A job's row on the board, by its kind: what a capture's `take-lost`
/// press finds.
#[derive(Component)]
pub struct JobRow(pub pbd_core::economy::jobs::JobKind);

/// The coin count beside the hotbar.
#[derive(Component)]
pub struct CoinCount;
/// The "job done" line over it.
#[derive(Component)]
pub struct CoinNotice;

/// Build the three panels, hidden, and the coin count. `PostStartup`.
pub fn spawn(
    mut commands: Commands,
    kit: Res<UiKit>,
    view: Res<TalkView>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<AssetServer>,
) {
    commands.insert_resource(LostIcon(assets.load("items/missions/lost-villager.png")));
    let kit = &*kit;
    // On the right, the townsperson framed on the left (design 37a,
    // framing): every NPC dialog laid out one way.
    npc_dialog(&mut commands, kit, &view, Screen::Talk, TalkBody);
    npc_dialog(&mut commands, kit, &view, Screen::Trade, TradeBody);
    npc_dialog(&mut commands, kit, &view, Screen::Board, BoardBody);
    let coin = coin_image(&mut images);
    commands.insert_resource(CoinIcon(coin.clone()));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(slots::HOTBAR_BOTTOM + 4.0),
                left: percent(50),
                margin: UiRect::left(px(
                    pbd_core::inventory::SLOTS as f32 * slots::SLOT / 2.0 + 6.0
                )),
                align_items: AlignItems::Center,
                column_gap: px(2),
                ..default()
            },
            menu::InPlayHud,
        ))
        .with_children(|row| {
            row.spawn((
                ImageNode::new(coin),
                Node {
                    width: px(16),
                    height: px(16),
                    ..default()
                },
            ));
            row.spawn((
                KitText::new(Localized::verbatim("0"), ui_kit::LABEL).outline(ui_kit::INK),
                CoinCount,
            ));
        });
    commands.spawn((
        KitText::new(Localized::empty(), ui_kit::TITLE).outline(ui_kit::INK),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(slots::HOTBAR_BOTTOM + slots::SLOT + 8.0),
            left: percent(50),
            margin: UiRect::left(px(-60.0)),
            ..default()
        },
        Visibility::Hidden,
        CoinNotice,
        menu::InPlayHud,
    ));
}

/// The coin icon (`assets/items/ui/coin.png`), sampled nearest.
fn coin_image(images: &mut Assets<Image>) -> Handle<Image> {
    let path = pbd_app::paths::assets_dir().join("items/ui/coin.png");
    let pixels = image::open(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut img = Image::from_dynamic(pixels, true, bevy::asset::RenderAssetUsages::default());
    img.sampler = bevy::image::ImageSampler::nearest();
    images.add(img)
}

/// Keep the coin count and the "job done" line current.
pub fn coin_hud(
    eco: Res<Economy>,
    mut count: Query<&mut KitText, (With<CoinCount>, Without<CoinNotice>)>,
    mut notice: Query<(&mut KitText, &mut Visibility), With<CoinNotice>>,
) {
    if !eco.is_changed() {
        return;
    }
    for mut text in &mut count {
        let words = Localized::verbatim(eco.purse.coins.to_string());
        if text.text != words {
            text.text = words;
        }
    }
    for (mut text, mut shown) in &mut notice {
        match &eco.notice {
            Some((words, _)) => {
                if &text.text != words {
                    text.text = words.clone();
                }
                *shown = Visibility::Inherited;
            }
            None => *shown = Visibility::Hidden,
        }
    }
}

/// The talker in reach: the nearest within [`TALK_REACH_M`] of the eye and
/// in front of it.
fn talker_in_reach<'a>(
    eye: &GlobalTransform,
    talkers: impl Iterator<Item = (&'a Talker, &'a GlobalTransform)>,
) -> Option<(Talker, Vec3)> {
    let at = eye.translation();
    let look = eye.forward().as_vec3();
    let up = eye.up().as_vec3();
    talkers
        .filter_map(|(t, g)| {
            let chest = g.translation() + up * 1.0;
            let to = chest - at;
            let d = to.length();
            (d <= TALK_REACH_M && to.dot(look) >= 0.5 * d).then_some((d, t, g.translation()))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, t, feet)| (t.clone(), feet))
}

/// What a deal reads and writes: the rules, the purse, the save, the pack,
/// the clock and what tells a fish from a fish.
type ShopParams<'w> = (
    Res<'w, EconomyConfig>,
    ResMut<'w, Economy>,
    Option<ResMut<'w, WorldSave>>,
    ResMut<'w, Hotbar>,
    Res<'w, pbd_app::sky::Sun>,
    Res<'w, pbd_app::config::FaunaConfig>,
    Res<'w, pbd_app::fish::Body>,
);

/// A talker, where it stands, and whether it is shown.
type TalkerQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Talker,
        &'static GlobalTransform,
        Option<&'static Visibility>,
    ),
>;

/// The talkers on screen: a figure faded out and hidden is no one to talk to.
fn shown<'a>(talkers: &'a TalkerQuery) -> impl Iterator<Item = (&'a Talker, &'a GlobalTransform)> {
    talkers
        .iter()
        .filter(|(_, _, v)| v.is_none_or(|v| *v != Visibility::Hidden))
        .map(|(t, g, _)| (t, g))
}

/// A role's name, as the prompt and the talk panel call the person.
pub fn role_name(role: &str) -> Localized {
    tr(&format!("talk.role.{role}"))
}

/// Offer the townsperson in reach to the interact key.
pub fn offer_talk(
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    talkers: TalkerQuery,
    mut interact: ResMut<Interact>,
) {
    let Some((eye, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    if let Some((t, _)) = talker_in_reach(eye, shown(&talkers)) {
        interact.offer(Offer::new(Act::Talk, role_name(&t.role)));
    }
}

/// Open the talk with `talker`: who they are to their town, and any parcel
/// owed there handed over.
#[allow(clippy::too_many_arguments)]
pub fn open_talk(
    talker: Talker,
    all: &[&Talker],
    towns: &Towns,
    cfg: &EconomyConfig,
    shop: Option<&mut Shop>,
    talking: &mut Talking,
    screen: &mut Screen,
) -> bool {
    let Some(town) = town_info(towns, talker.site) else {
        return false;
    };
    let k = keepers(&cfg.jobs, all, talker.site);
    let trader = k.is_some_and(|k| k.trader == talker.index as usize);
    let giver = k.is_some_and(|k| k.giver == talker.index as usize);
    let mut said = None;
    if let Some(shop) = shop {
        shop.talked(&talker);
        match shop.deliver(&town) {
            Ok(n) if n > 0 => said = Some(tr("talk.parcel_thanks")),
            Err(why) => said = Some(why.words()),
            _ => {}
        }
    }
    *talking = Talking {
        with: Some(TalkWith {
            talker,
            town,
            trader,
            giver,
        }),
        said,
        basket: Basket::default(),
    };
    *screen = Screen::Talk;
    true
}

/// The interact key on a townsperson opens the talk; on an open talk panel
/// it closes it.
#[allow(clippy::too_many_arguments)]
pub fn use_talk(
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    interact: Option<Res<Interact>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    talkers: TalkerQuery,
    towns: Res<Towns>,
    (cfg, mut eco, mut save, mut hotbar, sun, fauna, body): ShopParams,
    mut talking: ResMut<Talking>,
    folk: Option<Res<pbd_app::townsfolk::Townsfolk>>,
    (view, mut turn): (Res<TalkView>, ResMut<TalkTurn>),
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    if is_npc_dialog(*screen) {
        *screen = Screen::Playing;
        return;
    }
    if *screen != Screen::Playing
        || !walking.is_some_and(|w| w.active && w.captured)
        || !pbd_app::interact::may(interact.as_deref(), Reader::Talk)
    {
        return;
    }
    let Some((eye, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let Some((talker, feet)) = talker_in_reach(eye, shown(&talkers)) else {
        return;
    };
    // The aim point, at the press: the eye's up (the origin's at the
    // walker) stands for the townsperson's within the talk's three metres.
    let up = eye.up().as_vec3();
    let aim = feet + up * view.aim_height_m - eye.translation();
    let drawn: Vec<&Talker> = talkers.iter().map(|(t, _, _)| t).collect();
    let people = roster(folk.as_deref(), &drawn, talker.site);
    let all: Vec<&Talker> = people.iter().collect();
    // A taken job's lost villager, found: they follow from now on
    // (`townsfolk` design 28).
    let found = save.as_deref_mut().is_some_and(|save| {
        pbd_app::economy::special::find_lost(save, &eco, talker.site, talker.index)
    });
    let ids = fish_ids(&fauna, &body.0);
    let mut shop = save.as_deref_mut().map(|save| Shop {
        save,
        slots: &mut hotbar.0,
        eco: &mut eco,
        cfg: &cfg,
        fish_ids: &ids,
        now_s: sun.clock.seconds,
    });
    if open_talk(
        talker,
        &all,
        &towns,
        &cfg,
        shop.as_mut(),
        &mut talking,
        &mut screen,
    ) {
        *turn = TalkTurn {
            aim: Some(aim),
            framed: None,
            turn: None,
        };
    }
    if found {
        talking.said = Some(tr("talk.lost_found"));
    }
}

/// Leaving every talk panel forgets who it was with.
pub fn forget(screen: Res<Screen>, mut talking: ResMut<Talking>, mut scroll: ResMut<TalkScroll>) {
    if screen.is_changed() && !is_npc_dialog(*screen) && talking.with.is_some() {
        *talking = Talking::default();
        *scroll = TalkScroll::default();
    }
}

/// A button pressed.
#[allow(clippy::too_many_arguments)]
pub fn press(
    buttons: Query<(&Interaction, &TalkAction), Changed<Interaction>>,
    mut screen: ResMut<Screen>,
    mut talking: ResMut<Talking>,
    mut scroll: ResMut<TalkScroll>,
    mut horses: ResMut<pbd_app::vehicles::carts::HorseOrders>,
    (cfg, mut eco, mut save, mut hotbar, sun, fauna, body): ShopParams,
) {
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // A filter tab: that column from its top (design 38c); nothing said.
        if let TalkAction::Shelf(list, shelf) = action {
            let column = scroll.column(*list);
            column.shelf = *shelf;
            column.first = 0;
            continue;
        }
        let ids = fish_ids(&fauna, &body.0);
        let Some(with) = talking.with.clone() else {
            continue;
        };
        // The baskets fill and empty with nothing said (design 37a); the
        // panel shows only what can still go in.
        match action {
            TalkAction::Buy(code) => {
                talking.basket.add(code, true, 1);
                continue;
            }
            TalkAction::Sell(code) => {
                talking.basket.add(code, false, 1);
                continue;
            }
            TalkAction::Unbasket(taking, code) => {
                talking.basket.remove(code, *taking);
                continue;
            }
            TalkAction::Clear => {
                talking.basket = Basket::default();
                continue;
            }
            _ => {}
        }
        let said = match action {
            TalkAction::Trade => {
                *screen = Screen::Trade;
                None
            }
            TalkAction::Work => {
                *screen = Screen::Board;
                None
            }
            TalkAction::Bye => {
                *screen = Screen::Playing;
                None
            }
            TalkAction::Back => {
                *screen = Screen::Talk;
                None
            }
            TalkAction::BuyHorse => {
                horses.0.push(with.town.site);
                None
            }
            TalkAction::Deal => {
                let Some(save) = save.as_deref_mut() else {
                    continue;
                };
                let mut shop = Shop {
                    save,
                    slots: &mut hotbar.0,
                    eco: &mut eco,
                    cfg: &cfg,
                    fish_ids: &ids,
                    now_s: sun.clock.seconds,
                };
                let basket = talking.basket.clone();
                let said = settle(&mut shop, &with.town, &basket);
                if said.1 {
                    talking.basket = Basket::default();
                }
                Some(said.0)
            }
            other => {
                let Some(save) = save.as_deref_mut() else {
                    continue;
                };
                let mut shop = Shop {
                    save,
                    slots: &mut hotbar.0,
                    eco: &mut eco,
                    cfg: &cfg,
                    fish_ids: &ids,
                    now_s: sun.clock.seconds,
                };
                act(&mut shop, &with.town, other)
            }
        };
        talking.said = said;
    }
}

/// Settle `basket` at `town`: what the panel says, and whether it went.
pub fn settle(shop: &mut Shop, town: &TownInfo, basket: &Basket) -> (Localized, bool) {
    match shop.barter(town, basket) {
        Ok(deal) => {
            let owed = deal.tally.balance();
            let coins = owed.unsigned_abs();
            let words = match owed.cmp(&0) {
                std::cmp::Ordering::Greater => tr_with("trade.bought", &[("coins", &coins)]),
                std::cmp::Ordering::Less => tr_with("trade.sold", &[("coins", &coins)]),
                std::cmp::Ordering::Equal => tr("trade.traded"),
            };
            (words, true)
        }
        Err(why) => (why.words(), false),
    }
}

/// Do a deal's button: what the panel says after it.
pub fn act(shop: &mut Shop, town: &TownInfo, action: &TalkAction) -> Option<Localized> {
    let result: Result<Localized, Why> = match action {
        TalkAction::Buy(code) => shop
            .buy(town, code)
            .map(|d| tr_with("trade.bought", &[("coins", &d.coins)])),
        TalkAction::Sell(code) => shop
            .sell(town, code)
            .map(|d| tr_with("trade.sold", &[("coins", &d.coins)])),
        TalkAction::Take(id) => {
            let job = shop.board(town).into_iter().find(|j| j.id == *id);
            match job {
                Some(job) => shop.take(job).map(|()| tr("jobs.taken_line")),
                None => Err(Why::Job(pbd_core::economy::jobs::JobRefused::Taken)),
            }
        }
        TalkAction::HandIn(id) => shop
            .hand_in(town, *id)
            .map(|j| tr_with("jobs.paid", &[("coins", &j.reward)])),
        TalkAction::Abandon(id) => shop.abandon(*id).map(|_| tr("jobs.abandoned_line")),
        _ => return None,
    };
    Some(result.unwrap_or_else(Why::words))
}

/// A plaque button with its label and action.
fn button(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    label: Localized,
    action: TalkAction,
    width: f32,
) {
    parent
        .spawn((ui_kit::plaque(kit, Set::Stone, width), action))
        .with_children(|p| {
            p.spawn(ui_kit::plaque_label(Set::Stone, label));
        });
}

fn line(parent: &mut ChildSpawnerCommands, text: Localized, colour: Color) {
    parent.spawn(KitText::new(text, colour).shadow(ui_kit::INK));
}

/// A coin amount: the coin and the number.
fn coins(parent: &mut ChildSpawnerCommands, n: u64, colour: Color, coin: &Handle<Image>) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(1),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                ImageNode::new(coin.clone()),
                Node {
                    width: px(12),
                    height: px(12),
                    ..default()
                },
            ));
            row.spawn(
                KitText::new(tr_with("trade.coins", &[("coins", &n)]), colour).shadow(ui_kit::INK),
            );
        });
}

/// An item's picture in the pack's own slot well (decision 38a): the
/// dressed-stone `slot-idle` piece, [`slots::SLOT`] square, its icon at the
/// item PNG's own size on the well's floor. No item, an empty well.
fn icon(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    item: Option<pbd_core::inventory::Item>,
    icons: &ItemIcons,
    images: &Assets<Image>,
) {
    icon_or(parent, kit, item, None, icons, images);
}

/// [`icon`], or an icon of its own where a job has one and no good.
fn icon_or(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    item: Option<pbd_core::inventory::Item>,
    own: Option<&Handle<Image>>,
    icons: &ItemIcons,
    images: &Assets<Image>,
) {
    let mut node = ImageNode {
        color: Color::NONE,
        ..default()
    };
    slots::paint_icon(&mut node, item.map(|i| Stack::new(i, 1)), icons, images);
    if let (None, Some(own)) = (item, own) {
        node.image = own.clone();
        node.rect = None;
        node.color = Color::WHITE;
    }
    parent
        .spawn((
            Node {
                width: px(slots::SLOT),
                height: px(slots::SLOT),
                flex_shrink: 0.0,
                ..default()
            },
            kit.whole(Set::Stone, slots::slot_piece(false)),
        ))
        .with_children(|well| {
            well.spawn((
                node,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(slots::ICON_AT),
                    top: px(slots::ICON_AT),
                    width: px(slots::ICON),
                    height: px(slots::ICON),
                    ..default()
                },
            ));
        });
}

/// The good a job asks for, for its well: its thing (the first of a
/// choice), a delivery's parcel, a repair's planks (design 29 mends with
/// them); none for a job with no good (a lost villager, lamps).
fn job_item(job: &Job, ids: &[String]) -> Option<pbd_core::inventory::Item> {
    use pbd_core::economy::jobs::JobKind;
    match job.kind {
        JobKind::Deliver => return item_of("parcel", ids),
        JobKind::Repair => return item_of("planks", ids),
        _ => {}
    }
    let first = job.thing.split('|').next().unwrap_or_default();
    item_of(first, ids)
}

/// What every panel reads to paint.
#[derive(bevy::ecs::system::SystemParam)]
pub struct PaintParams<'w> {
    screen: Res<'w, Screen>,
    talking: Res<'w, Talking>,
    eco: Res<'w, Economy>,
    cfg: Res<'w, EconomyConfig>,
    hotbar: Res<'w, Hotbar>,
    save: Option<Res<'w, WorldSave>>,
    sun: Res<'w, pbd_app::sky::Sun>,
    towns: Res<'w, Towns>,
    kit: Res<'w, UiKit>,
    icons: Option<Res<'w, ItemIcons>>,
    images: Res<'w, Assets<Image>>,
    fauna: Res<'w, pbd_app::config::FaunaConfig>,
    body: Res<'w, pbd_app::fish::Body>,
    coin: Option<Res<'w, CoinIcon>>,
    lost: Option<Res<'w, LostIcon>>,
}

/// Rebuild the open panel when what it shows changes.
pub fn paint(
    mut commands: Commands,
    mut shown: Local<String>,
    p: PaintParams,
    mut scroll: ResMut<TalkScroll>,
    talk: Query<Entity, With<TalkBody>>,
    trade: Query<Entity, With<TradeBody>>,
    board: Query<Entity, With<BoardBody>>,
) {
    let open = *p.screen;
    if !matches!(open, Screen::Talk | Screen::Trade | Screen::Board) {
        return;
    }
    // What the panel shows, as one line: it is rebuilt only when this moves.
    let now = format!(
        "{open:?}|{:?}|{}|{:?}|{:?}|{}|{}",
        p.talking,
        p.eco.purse.coins,
        p.eco.jobs,
        p.hotbar.0,
        p.sun.clock.day(),
        scroll.key()
    );
    if *shown == now && !p.screen.is_changed() {
        return;
    }
    let (Some(with), Some(save), Some(icons), Some(coin)) = (
        p.talking.with.as_ref(),
        p.save.as_deref(),
        p.icons.as_deref(),
        p.coin.as_deref(),
    ) else {
        return;
    };
    let coin = &coin.0;
    let kit = &*p.kit;
    let body = match open {
        Screen::Talk => talk.single(),
        Screen::Trade => trade.single(),
        _ => board.single(),
    };
    let Ok(body) = body else {
        return;
    };
    *shown = now;
    commands.entity(body).despawn_children();
    let ids = fish_ids(&p.fauna, &p.body.0);
    let now_s = p.sun.clock.seconds;
    commands.entity(body).with_children(|panel| match open {
        Screen::Talk => paint_talk(panel, kit, with, &p.talking.said, &p.eco),
        Screen::Trade => paint_trade(
            panel,
            kit,
            with,
            &p,
            &mut scroll,
            save,
            icons,
            &ids,
            now_s,
            coin,
        ),
        _ => paint_board(panel, kit, with, &p, &mut scroll, icons, &ids, coin),
    });
}

fn said_line(panel: &mut ChildSpawnerCommands, said: &Option<Localized>) {
    if let Some(said) = said {
        line(panel, said.clone(), ui_kit::TITLE);
    }
}

fn paint_talk(
    panel: &mut ChildSpawnerCommands,
    kit: &UiKit,
    with: &TalkWith,
    said: &Option<Localized>,
    eco: &Economy,
) {
    menu::lintel(panel, kit, Localized::verbatim(with.town.name.clone()));
    line(panel, role_name(&with.talker.role), ui_kit::TEXT_DIM);
    let ready = !eco.jobs.ready_at(with.town.site).is_empty();
    let greeting = tr(if ready {
        "talk.line.ready"
    } else if with.trader {
        "talk.line.trader"
    } else if with.giver {
        "talk.line.giver"
    } else {
        "talk.line.hello"
    });
    line(panel, greeting, ui_kit::TEXT);
    said_line(panel, said);
    panel
        .spawn(Node {
            column_gap: px(4),
            margin: UiRect::top(px(4)),
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(4),
            ..default()
        })
        .with_children(|row| {
            if with.trader {
                button(row, kit, tr("talk.trade"), TalkAction::Trade, 60.0);
            }
            if with.giver || ready {
                button(row, kit, tr("talk.work"), TalkAction::Work, 60.0);
            }
            button(row, kit, tr("talk.bye"), TalkAction::Bye, 50.0);
        });
}

#[allow(clippy::too_many_arguments)]
fn paint_trade(
    panel: &mut ChildSpawnerCommands,
    kit: &UiKit,
    with: &TalkWith,
    p: &PaintParams,
    scroll: &mut TalkScroll,
    save: &WorldSave,
    icons: &ItemIcons,
    ids: &[String],
    now_s: f64,
    coin: &Handle<Image>,
) {
    menu::lintel(panel, kit, tr("trade.title"));
    let goodwill = pbd_app::economy::goodwill_of(save, with.town.site).points;
    let trader = pbd_core::economy::trade::Trader {
        cfg: &p.cfg.trade,
        values: p.eco.prices.as_ref().unwrap_or(&p.cfg.values),
        kind: with.town.kind,
        goodwill,
        fish_ids: ids,
    };
    let basket = &p.talking.basket;
    let market = pbd_app::economy::market_of(save, with.town.site).settled(now_s, &p.cfg.trade);
    // Every good the trader sells, at its price and stock.
    let buy: Vec<Good> = trader
        .stock_list()
        .into_iter()
        .filter_map(|code| {
            let item = item_of(&code, ids)?;
            let q = trader.quote(&market, &code);
            Some(Good {
                price: q.sell?,
                count: q.stock.saturating_sub(basket.count(&code, true)),
                item,
                code,
            })
        })
        .collect();
    // What the pack holds that the trader buys, in slot order, once each.
    let mut sell: Vec<Good> = Vec::new();
    for stack in p.hotbar.0.iter().flatten() {
        let Some(code) = trade_code(stack.item, ids) else {
            continue;
        };
        if !trader.buys(&code) || sell.iter().any(|g| g.code == code) {
            continue;
        }
        let q = trader.quote(&market, &code);
        if let Some(price) = q.buy {
            let held = p.hotbar.0.count(stack.item).min(q.wants);
            sell.push(Good {
                price,
                count: held.saturating_sub(basket.count(&code, false)),
                item: stack.item,
                code,
            });
        }
    }
    let art = (icons, &*p.images, coin, ids);
    let values = &p.cfg.values;
    panel
        .spawn(Node {
            column_gap: px(12),
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .with_children(|cols| {
            let buy_side = Side {
                list: List::Buy,
                heading: tr("trade.buy"),
                label: tr("trade.add"),
                act: TalkAction::Buy,
                empty: None,
            };
            trade_column(cols, kit, art, values, scroll, buy_side, buy);
            let sell_side = Side {
                list: List::Sell,
                heading: tr("trade.sell"),
                label: tr("trade.add"),
                act: TalkAction::Sell,
                empty: Some(tr("trade.nothing_to_sell")),
            };
            trade_column(cols, kit, art, values, scroll, sell_side, sell);
        });
    baskets(panel, kit, art, &trader.tally(&market, basket), basket);
    // A horse, sold by the trader (`land-transport` design 10).
    panel
        .spawn(Node {
            column_gap: px(8),
            align_items: AlignItems::Center,
            margin: UiRect::top(px(4)),
            ..default()
        })
        .with_children(|row| {
            line(row, tr("species.horse.name"), ui_kit::TEXT);
            coins(row, p.cfg.trade.horse_coins, ui_kit::TITLE, coin);
            button(row, kit, tr("trade.buy_one"), TalkAction::BuyHorse, 50.0);
        });
    footer(panel, kit, p, coin);
}

/// The two baskets under the columns, the coins between them, DEAL and
/// CLEAR (design 37a, M5's Basket view).
fn baskets(
    panel: &mut ChildSpawnerCommands,
    kit: &UiKit,
    art: Art,
    tally: &pbd_core::economy::trade::Tally,
    basket: &Basket,
) {
    let (icons, images, coin, ids) = art;
    panel
        .spawn(Node {
            column_gap: px(12),
            align_items: AlignItems::FlexStart,
            margin: UiRect::top(px(2)),
            ..default()
        })
        .with_children(|cols| {
            for (taking, heading, list, total) in [
                (true, tr("trade.you_take"), &basket.take, tally.take),
                (false, tr("trade.you_give"), &basket.give, tally.give),
            ] {
                cols.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    width: px(TRADE_ROW_W + BAR_GAP + scroll_list::BAR_W),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|col| {
                    menu::section(col, heading);
                    col.spawn(Node {
                        column_gap: px(6),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn(Node {
                            column_gap: px(2),
                            flex_wrap: FlexWrap::Wrap,
                            row_gap: px(2),
                            max_width: px(6.0 * (slots::SLOT + 2.0)),
                            ..default()
                        })
                        .with_children(|wells| {
                            let mut shown = 0;
                            for (code, n) in list.iter().filter(|(_, n)| *n > 0) {
                                let item = item_of(code, ids);
                                let action = TalkAction::Unbasket(taking, code.clone());
                                basket_well(wells, kit, item, *n, action, icons, images);
                                shown += 1;
                            }
                            for _ in shown..BASKET_WELLS {
                                icon(wells, kit, None, icons, images);
                            }
                        });
                        coins(row, total, ui_kit::TITLE, coin);
                    });
                });
            }
        });
    let owed = tally.balance();
    panel
        .spawn(Node {
            column_gap: px(8),
            align_items: AlignItems::Center,
            margin: UiRect::top(px(2)),
            ..default()
        })
        .with_children(|row| {
            let words = match owed.cmp(&0) {
                std::cmp::Ordering::Greater => tr("trade.you_pay"),
                std::cmp::Ordering::Less => tr("trade.they_pay"),
                std::cmp::Ordering::Equal => tr("trade.even"),
            };
            line(row, words, ui_kit::TEXT);
            coins(row, owed.unsigned_abs(), ui_kit::TITLE, coin);
            button(row, kit, tr("trade.deal"), TalkAction::Deal, 50.0);
            button(row, kit, tr("trade.clear"), TalkAction::Clear, 50.0);
        });
}

/// The empty wells a basket always shows, so it keeps its width.
const BASKET_WELLS: usize = 4;

/// A basket's well: the good and how many, a press taking one back out.
fn basket_well(
    wells: &mut ChildSpawnerCommands,
    kit: &UiKit,
    item: Option<pbd_core::inventory::Item>,
    n: u32,
    action: TalkAction,
    icons: &ItemIcons,
    images: &Assets<Image>,
) {
    let mut node = ImageNode {
        color: Color::NONE,
        ..default()
    };
    slots::paint_icon(&mut node, item.map(|i| Stack::new(i, 1)), icons, images);
    wells
        .spawn((
            Button,
            Node {
                width: px(slots::SLOT),
                height: px(slots::SLOT),
                flex_shrink: 0.0,
                ..default()
            },
            kit.whole(Set::Stone, slots::slot_piece(false)),
            action,
        ))
        .with_children(|well| {
            well.spawn((
                node,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(slots::ICON_AT),
                    top: px(slots::ICON_AT),
                    width: px(slots::ICON),
                    height: px(slots::ICON),
                    ..default()
                },
            ));
            well.spawn((
                KitText::new(tr_with("trade.count", &[("count", &n)]), ui_kit::TEXT)
                    .shadow(ui_kit::INK),
                Node {
                    position_type: PositionType::Absolute,
                    right: px(1),
                    bottom: px(0),
                    ..default()
                },
            ));
        });
}

/// A good in a trade column: its code, its item, its price and how many.
struct Good {
    code: String,
    item: pbd_core::inventory::Item,
    price: u64,
    count: u32,
}

/// What tells the two trade columns apart.
struct Side {
    list: List,
    heading: Localized,
    label: Localized,
    act: fn(String) -> TalkAction,
    empty: Option<Localized>,
}

/// What a column's wells and prices paint from.
type Art<'a> = (
    &'a ItemIcons,
    &'a Assets<Image>,
    &'a Handle<Image>,
    &'a [String],
);

/// A shelf's name, for its tab's tooltip.
fn shelf_name(shelf: Option<Shelf>) -> Localized {
    tr(match shelf {
        None => "trade.shelf.all",
        Some(Shelf::Food) => "trade.shelf.food",
        Some(Shelf::Organics) => "trade.shelf.organics",
        Some(Shelf::Raw) => "trade.shelf.raw",
        Some(Shelf::Tools) => "trade.shelf.tools",
        Some(Shelf::Goods) => "trade.shelf.goods",
    })
}

/// One trade column (design 38b and 38c): its heading, its filter tabs,
/// and the rows of its filter in a window of [`VISIBLE_ROWS`] with a scroll
/// bar when they overflow.
fn trade_column(
    cols: &mut ChildSpawnerCommands,
    kit: &UiKit,
    art: Art,
    values: &Values,
    scroll: &mut TalkScroll,
    side: Side,
    goods: Vec<Good>,
) {
    let (icons, images, coin, ids) = art;
    let held = scroll.drag.is_some_and(|d| d.0 == side.list);
    let column = scroll.column(side.list);
    let shelves = values.shelves_of(goods.iter().map(|g| g.code.as_str()));
    // A filter whose last good went (sold, or bought out of the pack) is ALL.
    if column.shelf.is_some_and(|s| !shelves.contains(&s)) {
        column.shelf = None;
        column.first = 0;
    }
    let chosen = column.shelf;
    let shown: Vec<&Good> = goods
        .iter()
        .filter(|g| values.on_shelf(&g.code, chosen))
        .collect();
    let view = ListView::new(shown.len(), VISIBLE_ROWS, column.first);
    column.first = view.first;
    column.rows = shown.len();
    cols.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(2),
        width: px(TRADE_ROW_W + BAR_GAP + scroll_list::BAR_W),
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|col| {
        menu::section(col, side.heading);
        // The tabs: ALL and a shelf a tab, only shelves with goods here. The
        // row keeps its height with none, so the two windows line up.
        col.spawn(Node {
            column_gap: px(2),
            height: px(slots::SLOT),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|tabs| {
            if !shelves.is_empty() {
                shelf_tab(tabs, kit, (icons, images, ids), side.list, None, chosen);
                for shelf in &shelves {
                    let art = (icons, images, ids);
                    shelf_tab(tabs, kit, art, side.list, Some(*shelf), chosen);
                }
            }
        });
        col.spawn(Node {
            column_gap: px(BAR_GAP),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|window| {
            window
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(ROW_GAP),
                        width: px(TRADE_ROW_W),
                        height: px(LIST_H),
                        flex_shrink: 0.0,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    bevy::ui::RelativeCursorPosition::default(),
                    ScrollArea(side.list),
                ))
                .with_children(|rows| {
                    if goods.is_empty()
                        && let Some(empty) = side.empty
                    {
                        line(rows, empty, ui_kit::TEXT_DIM);
                    }
                    for good in &shown[view.window()] {
                        trade_row(
                            rows,
                            kit,
                            (icons, images, coin),
                            good.item,
                            (good.price, good.count),
                            (side.act)(good.code.clone()),
                            side.label.clone(),
                        );
                    }
                });
            scroll_list::bar(
                window,
                kit,
                view,
                LIST_H,
                held,
                ScrollTrack(side.list),
                ScrollThumb(side.list),
            );
        });
    });
}

/// A filter tab: a pack slot well (chosen, the selected well) holding the
/// shelf's good, or ALL its word; the shelf's name its tooltip.
fn shelf_tab(
    tabs: &mut ChildSpawnerCommands,
    kit: &UiKit,
    (icons, images, ids): (&ItemIcons, &Assets<Image>, &[String]),
    list: List,
    shelf: Option<Shelf>,
    chosen: Option<Shelf>,
) {
    let on = shelf == chosen;
    tabs.spawn((
        Button,
        Node {
            width: px(slots::SLOT),
            height: px(slots::SLOT),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        kit.whole(Set::Stone, slots::slot_piece(on)),
        ShelfTab(on),
        Tooltip(shelf_name(shelf)),
        TalkAction::Shelf(list, shelf),
    ))
    .with_children(|well| match shelf {
        None => {
            well.spawn(KitText::new(shelf_name(None), ui_kit::TEXT).shadow(ui_kit::INK));
        }
        Some(shelf) => {
            let mut node = ImageNode {
                color: Color::NONE,
                ..default()
            };
            let item = item_of(shelf.picture(), ids);
            slots::paint_icon(&mut node, item.map(|i| Stack::new(i, 1)), icons, images);
            well.spawn((
                node,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(slots::ICON_AT),
                    top: px(slots::ICON_AT),
                    width: px(slots::ICON),
                    height: px(slots::ICON),
                    ..default()
                },
            ));
        }
    });
}

/// A filter tab under the pointer shows the pack's hover well.
pub fn paint_tabs(
    kit: Res<UiKit>,
    mut tabs: Query<(&Interaction, &ShelfTab, &mut ImageNode), Changed<Interaction>>,
) {
    for (interaction, tab, mut image) in &mut tabs {
        let piece = if tab.0 {
            "slot-selected"
        } else if *interaction == Interaction::None {
            "slot-idle"
        } else {
            "slot-hover"
        };
        image.image = kit.image(&Set::Stone.key(piece));
    }
}

fn trade_row(
    col: &mut ChildSpawnerCommands,
    kit: &UiKit,
    (icons, images, coin): (&ItemIcons, &Assets<Image>, &Handle<Image>),
    item: pbd_core::inventory::Item,
    (price, count): (u64, u32),
    action: TalkAction,
    label: Localized,
) {
    col.spawn(Node {
        align_items: AlignItems::Center,
        column_gap: px(4),
        height: px(ROW_H),
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|row| {
        icon(row, kit, Some(item), icons, images);
        row.spawn((
            KitText::new(item.name(), ui_kit::TEXT).shadow(ui_kit::INK),
            Node {
                width: px(86),
                overflow: Overflow::clip(),
                ..default()
            },
        ));
        row.spawn((
            KitText::new(
                tr_with("trade.count", &[("count", &count)]),
                ui_kit::TEXT_DIM,
            )
            .shadow(ui_kit::INK),
            Node {
                width: px(22),
                ..default()
            },
        ));
        row.spawn(Node {
            width: px(30),
            ..default()
        })
        .with_children(|c| coins(c, price, ui_kit::TITLE, coin));
        // None left to put in a basket: no button (design 37a).
        if count > 0 {
            button(row, kit, label, action, 40.0);
        }
    });
}

/// The purse, the last thing said, and BACK.
fn footer(panel: &mut ChildSpawnerCommands, kit: &UiKit, p: &PaintParams, coin: &Handle<Image>) {
    // The line keeps its height when nothing is said, so the panel does not
    // grow as a deal is made (design 38b).
    panel
        .spawn(Node {
            min_height: px(10),
            margin: UiRect::top(px(3)),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|said| said_line(said, &p.talking.said));
    panel
        .spawn(Node {
            column_gap: px(8),
            align_items: AlignItems::Center,
            margin: UiRect::top(px(4)),
            ..default()
        })
        .with_children(|row| {
            line(row, tr("trade.purse"), ui_kit::TEXT_DIM);
            coins(row, p.eco.purse.coins, ui_kit::TITLE, coin);
            button(row, kit, tr("talk.back"), TalkAction::Back, 50.0);
        });
}

#[allow(clippy::too_many_arguments)]
fn paint_board(
    panel: &mut ChildSpawnerCommands,
    kit: &UiKit,
    with: &TalkWith,
    p: &PaintParams,
    scroll: &mut TalkScroll,
    icons: &ItemIcons,
    ids: &[String],
    coin: &Handle<Image>,
) {
    menu::lintel(panel, kit, tr("jobs.title"));
    let well = (icons, &*p.images, ids);
    let town = &with.town;
    let eco = &p.eco;
    let towns = &*p.towns;
    // One list (design 38b): the headings are rows of it.
    let mut entries: Vec<Entry> = Vec::new();
    // The jobs taken here: done ones to hand in, the rest in hand.
    let mine: Vec<Job> = eco
        .jobs
        .taken
        .iter()
        .filter(|j| j.site == town.site)
        .cloned()
        .collect();
    if !mine.is_empty() {
        entries.push(Entry::Heading(tr("jobs.yours")));
        for job in mine {
            let done = eco.jobs.is_done(job.id);
            let counts = eco.jobs.counts(job.id);
            let (action, label) = if done {
                (TalkAction::HandIn(job.id), tr("jobs.hand_in"))
            } else {
                (TalkAction::Abandon(job.id), tr("jobs.abandon"))
            };
            entries.push(Entry::Job(job, counts, Some(action), label));
        }
    }
    if with.giver {
        entries.push(Entry::Heading(tr("jobs.posted")));
        // The board as the rules post it today.
        let posted = {
            let board = pbd_core::economy::jobs::Board {
                jobs: &p.cfg.jobs,
                trade: &p.cfg.trade,
                values: &p.cfg.values,
                seed: p.save.as_ref().map_or(0, |s| s.seed()),
            };
            let day = p.sun.clock.day();
            let goodwill = p
                .save
                .as_deref()
                .map_or(0, |s| pbd_app::economy::goodwill_of(s, town.site).points);
            let hour = p.sun.clock.hour();
            board
                .posted(town.as_town(), day, goodwill, &eco.jobs.gone())
                .into_iter()
                .filter(|j| pbd_core::economy::jobs::shown_at(j, hour))
                .collect::<Vec<_>>()
        };
        if posted.is_empty() {
            entries.push(Entry::Line(tr("jobs.none")));
        }
        for job in posted {
            let action = Some(TalkAction::Take(job.id));
            entries.push(Entry::Job(job, Vec::new(), action, tr("jobs.take")));
        }
    }
    let held = scroll.drag.is_some_and(|d| d.0 == List::Board);
    let column = scroll.column(List::Board);
    let view = ListView::new(entries.len(), VISIBLE_ROWS, column.first);
    column.first = view.first;
    column.rows = entries.len();
    panel
        .spawn(Node {
            column_gap: px(BAR_GAP),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|window| {
            window
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(ROW_GAP),
                        width: px(BOARD_ROW_W),
                        height: px(LIST_H),
                        flex_shrink: 0.0,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    bevy::ui::RelativeCursorPosition::default(),
                    ScrollArea(List::Board),
                ))
                .with_children(|rows| {
                    for entry in entries.drain(view.window()) {
                        match entry {
                            Entry::Heading(text) => {
                                rows.spawn(Node {
                                    height: px(ROW_H),
                                    align_items: AlignItems::FlexEnd,
                                    flex_shrink: 0.0,
                                    ..default()
                                })
                                .with_children(|r| menu::section(r, text));
                            }
                            Entry::Line(text) => line(rows, text, ui_kit::TEXT_DIM),
                            Entry::Job(job, counts, action, label) => job_row(
                                rows,
                                (kit, coin, well, p.lost.as_deref().map(|l| &l.0)),
                                &job,
                                &counts,
                                towns,
                                action,
                                label,
                            ),
                        }
                    }
                });
            scroll_list::bar(
                window,
                kit,
                view,
                LIST_H,
                held,
                ScrollTrack(List::Board),
                ScrollThumb(List::Board),
            );
        });
    footer(panel, kit, p, coin);
}

/// A row of the board's list: a heading, a plain line, or a job with what
/// its button does.
enum Entry {
    Heading(Localized),
    Line(Localized),
    Job(Job, Vec<u32>, Option<TalkAction>, Localized),
}

/// What a job row's well paints from: the icons, the images, the fish ids.
type Well<'a> = (&'a ItemIcons, &'a Assets<Image>, &'a [String]);

fn job_row(
    panel: &mut ChildSpawnerCommands,
    (kit, coin, (icons, images, ids), lost): (&UiKit, &Handle<Image>, Well, Option<&Handle<Image>>),
    job: &Job,
    counts: &[u32],
    towns: &Towns,
    action: Option<TalkAction>,
    label: Localized,
) {
    panel
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: px(6),
                ..default()
            },
            JobRow(job.kind),
        ))
        .with_children(|row| {
            let own = lost.filter(|_| job.kind == pbd_core::economy::jobs::JobKind::Lost);
            icon_or(row, kit, job_item(job, ids), own, icons, images);
            row.spawn((
                KitText::new(job_line(job, counts, towns), ui_kit::TEXT).shadow(ui_kit::INK),
                Node {
                    width: px(250),
                    ..default()
                },
            ));
            row.spawn(Node {
                width: px(36),
                ..default()
            })
            .with_children(|c| coins(c, job.reward, ui_kit::TITLE, coin));
            if let Some(action) = action {
                button(row, kit, label, action, 56.0);
            }
        });
}

/// The mouse wheel over a list scrolls it by whole rows (design 38b): a
/// notch a row; pixel motion gathered into rows.
pub fn scroll_wheel(
    screen: Res<Screen>,
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    areas: Query<(&ScrollArea, &bevy::ui::RelativeCursorPosition)>,
    mut scroll: ResMut<TalkScroll>,
    mut carry: Local<f32>,
) {
    let mut turned = 0;
    for w in wheel.read() {
        let pixels = w.unit == bevy::input::mouse::MouseScrollUnit::Pixel;
        turned += scroll_list::notches(w.y, pixels, ROW_H + ROW_GAP, &mut carry);
    }
    if turned == 0 || !matches!(*screen, Screen::Trade | Screen::Board) {
        return;
    }
    let Some(list) = areas
        .iter()
        .find(|(_, at)| at.cursor_over())
        .map(|(area, _)| area.0)
    else {
        return;
    };
    let first = scroll.view(list).wheel(turned).first;
    scroll.column(list).first = first;
}

/// The scroll bar's thumb drags, snapping to whole rows; a press on the
/// track beside it pages (design 38b). The drag is held by the mouse
/// button, not the thumb: the panel is rebuilt as it scrolls.
pub fn scroll_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    thumbs: Query<(
        &Interaction,
        &ScrollThumb,
        &bevy::ui::RelativeCursorPosition,
    )>,
    tracks: Query<(
        &Interaction,
        &ScrollTrack,
        &bevy::ui::RelativeCursorPosition,
    )>,
    mut scroll: ResMut<TalkScroll>,
) {
    let inner = LIST_H - 2.0 * scroll_list::BAR_INSET;
    // Where the pointer is down the track's inside, native px.
    let along = |list: List| {
        tracks
            .iter()
            .find(|(_, t, _)| t.0 == list)
            .and_then(|(_, _, at)| {
                at.normalized
                    .map(|n| (n.y + 0.5) * LIST_H - scroll_list::BAR_INSET)
            })
    };
    if !mouse.pressed(MouseButton::Left) {
        if scroll.drag.is_some() {
            scroll.drag = None;
        }
        return;
    }
    if let Some((list, grab)) = scroll.drag {
        if let Some(y) = along(list) {
            let first = scroll.view(list).drag(inner, y - grab).first;
            if scroll.column(list).first != first {
                scroll.column(list).first = first;
            }
        }
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    // Taken hold of: where on the thumb, so it does not jump to the pointer.
    if let Some((_, thumb, _)) = thumbs.iter().find(|(i, _, _)| **i == Interaction::Pressed) {
        let list = thumb.0;
        if let Some(y) = along(list) {
            let (top, _) = scroll.view(list).thumb(inner);
            scroll.drag = Some((list, y - top));
        }
        return;
    }
    // The track above or below the thumb: a window's worth.
    if let Some((_, track, _)) = tracks.iter().find(|(i, _, _)| **i == Interaction::Pressed) {
        let list = track.0;
        if let Some(y) = along(list) {
            let view = scroll.view(list);
            let (top, _) = view.thumb(inner);
            scroll.column(list).first = view.page(y > top).first;
        }
    }
}

/// The townsperson the talk is with talks: their mouth moves while the
/// talk, trade or board panel is open with them, and stops when it closes
/// or turns to someone else (`townsfolk` design 30).
pub fn talking_mouth(
    talking: Res<Talking>,
    folk: Option<ResMut<pbd_app::townsfolk::Townsfolk>>,
    mut was: Local<Option<pbd_core::townsfolk::PersonId>>,
) {
    let Some(mut folk) = folk else {
        return;
    };
    let now = talking
        .with
        .as_ref()
        .map(|w| pbd_core::townsfolk::PersonId {
            site: w.talker.site,
            index: w.talker.index,
        });
    if let Some(old) = *was
        && Some(old) != now
    {
        folk.set_talking(old, false);
    }
    // Held every frame it is open, so a town stood again keeps it talking.
    if let Some(id) = now {
        folk.set_talking(id, true);
    }
    *was = now;
}

/// A capture instrument (`townsfolk` design 35): a `PBD_KEY_SCRIPT` step
/// `<frame>:press:<button>` presses a talk panel's button on that capture
/// frame, as a click on it would: `trade`, `work`, `back`, `bye`, `deal`,
/// `clear`, the first `buy`, `sell`, `take` or `handin` row, `take-lost` a
/// lost villager's TAKE, or a filter tab, `<buy|sell>-<all|
/// food|organics|raw|tools|goods>` (design 38c); `<frame>:wheel:<buy|sell|
/// board>:<notches>` turns the wheel over that list as the wheel would
/// (design 38b; negative is toward you, down the list). Nothing reads it in
/// play.
pub fn scripted_press(
    state: Res<super::CaptureState>,
    mut buttons: Query<(&mut Interaction, &TalkAction, Option<&ChildOf>)>,
    rows: Query<&JobRow>,
    mut scroll: ResMut<TalkScroll>,
    script: Local<std::sync::OnceLock<Vec<(u32, String)>>>,
    mut done: Local<std::collections::BTreeSet<usize>>,
) {
    let script = script.get_or_init(|| {
        std::env::var("PBD_KEY_SCRIPT")
            .unwrap_or_default()
            .split(',')
            .filter_map(|step| {
                let (frame, rest) = step.trim().split_once(':')?;
                let rest = rest.trim();
                let what = rest
                    .strip_prefix("press:")
                    .map(str::to_string)
                    .or_else(|| rest.starts_with("wheel:").then(|| rest.to_string()))?;
                Some((frame.trim().parse().ok()?, what))
            })
            .collect()
    });
    // From its frame on, until the button is there to press (a panel is
    // drawn a frame after it opens), for at most a second of frames.
    for (k, (frame, what)) in script.iter().enumerate() {
        if state.frame < *frame || done.contains(&k) {
            continue;
        }
        if let Some(turn) = what.strip_prefix("wheel:") {
            let mut parts = turn.split(':');
            let list = match parts.next() {
                Some("buy") => Some(List::Buy),
                Some("sell") => Some(List::Sell),
                Some("board") => Some(List::Board),
                _ => None,
            };
            let notches = parts.next().and_then(|n| n.parse::<i32>().ok());
            if let (Some(list), Some(notches)) = (list, notches) {
                let first = scroll.view(list).wheel(notches).first;
                scroll.column(list).first = first;
                info!(
                    "PBD_KEY_SCRIPT: wheel {notches} over {list:?} at frame {}",
                    state.frame
                );
            }
            done.insert(k);
            continue;
        }
        let tab = |a: &TalkAction| match a {
            TalkAction::Shelf(list, shelf) => {
                let side = if *list == List::Buy { "buy" } else { "sell" };
                let name = shelf.map_or("all", Shelf::id);
                *what == format!("{side}-{name}")
            }
            _ => false,
        };
        let wanted = |a: &TalkAction| {
            tab(a)
                || matches!(
                    (what.as_str(), a),
                    ("trade", TalkAction::Trade)
                        | ("work", TalkAction::Work)
                        | ("back", TalkAction::Back)
                        | ("bye", TalkAction::Bye)
                        | ("buy", TalkAction::Buy(_))
                        | ("sell", TalkAction::Sell(_))
                        | ("deal", TalkAction::Deal)
                        | ("clear", TalkAction::Clear)
                        | ("take", TalkAction::Take(_))
                        | ("handin", TalkAction::HandIn(_))
                        | ("abandon", TalkAction::Abandon(_))
                )
        };
        // A lost villager's TAKE: the button whose row is a lost job.
        let lost = |a: &TalkAction, parent: Option<&ChildOf>| {
            matches!(a, TalkAction::Take(_))
                && parent
                    .and_then(|p| rows.get(p.parent()).ok())
                    .is_some_and(|r| r.0 == pbd_core::economy::jobs::JobKind::Lost)
        };
        let found = buttons.iter_mut().find(|(_, a, parent)| {
            if what == "take-lost" {
                lost(a, *parent)
            } else {
                wanted(a)
            }
        });
        match found.map(|(i, a, _)| (i, a)) {
            Some((mut interaction, action)) => {
                info!(
                    "PBD_KEY_SCRIPT: pressed {action:?} at frame {}",
                    state.frame
                );
                *interaction = Interaction::Pressed;
                done.insert(k);
            }
            None if state.frame >= frame + 60 => {
                warn!("PBD_KEY_SCRIPT: no {what} button by frame {}", state.frame);
                done.insert(k);
            }
            None => {}
        }
    }
}

/// A capture instrument (`townsfolk` task 11.3): `PBD_PACK=<code>:<n>,...`
/// puts those goods in the pack on the capture's frame 100, so a capture of
/// the trade panel has something the trader buys. Nothing reads it in play.
pub fn scripted_pack(
    state: Res<super::CaptureState>,
    mut hotbar: ResMut<Hotbar>,
    mut done: Local<bool>,
) {
    if *done || state.frame < 100 {
        return;
    }
    *done = true;
    let Ok(spec) = std::env::var("PBD_PACK") else {
        return;
    };
    for step in spec.split(',') {
        let Some((code, n)) = step.trim().split_once(':') else {
            continue;
        };
        let (Some(item), Ok(n)) = (item_of(code, &[]), n.parse::<u16>()) else {
            warn!("PBD_PACK: no {code}");
            continue;
        };
        hotbar.0.give(item, n);
        info!("PBD_PACK: {n} {code} in the pack");
    }
}

pub struct TalkPlugin;

impl Plugin for TalkPlugin {
    fn build(&self, app: &mut App) {
        use pbd_app::interact::InteractSet;
        app.init_resource::<Talking>()
            .init_resource::<TalkScroll>()
            .init_resource::<TalkView>()
            .init_resource::<TalkTurn>()
            .init_resource::<TalkGlow>()
            .init_resource::<pbd_app::carried_light::NearLights>()
            .add_systems(
                PostUpdate,
                talk_light
                    .after(bevy::transform::TransformSystems::Propagate)
                    .before(pbd_app::carried_light::gather),
            )
            .add_systems(PostStartup, spawn)
            .add_systems(PostUpdate, offer_talk.in_set(InteractSet::Offer))
            .add_systems(
                Update,
                (
                    use_talk,
                    turn_to_talker,
                    forget,
                    scripted_pack.run_if(resource_exists::<super::CaptureState>),
                    scripted_press,
                    press,
                    scroll_wheel,
                    scroll_drag,
                    paint,
                    paint_tabs,
                    coin_hud,
                    talking_mouth,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::CameraProjection;
    const TALK_RON: &str = include_str!("../../../../assets/config/talk.ron");

    /// Where `point` lands on screen, normalised, for the walker's camera
    /// at `eye` with this look: the walker's own rotation and Bevy's own
    /// perspective.
    fn projected(eye: Vec3, up: Vec3, look: (Vec3, f32), lens: (f32, f32), point: Vec3) -> Vec3 {
        let (heading, pitch) = look;
        let camera = Transform::from_translation(eye)
            .looking_to(heading * pitch.cos() + up * pitch.sin(), up);
        let lens = PerspectiveProjection {
            fov: lens.0,
            aspect_ratio: lens.1,
            ..default()
        };
        let clip = lens.get_clip_from_view() * camera.to_matrix().inverse() * point.extend(1.0);
        clip.truncate() / clip.w
    }

    /// `townsfolk` design 37a, framing, second pass: for each NPC
    /// dialog's panel, at three windows and UI scales, the townsperson's aim
    /// point projects half way between the screen's left edge and the
    /// panel's left edge, at the middle height.
    #[test]
    fn the_talker_lands_in_the_open_space() {
        let view = TalkView::parse(TALK_RON).unwrap();
        assert_eq!(view.frame_y, 0.0);
        let tilted = Vec3::new(0.3, 0.9, -0.2).normalize();
        for screen in [Screen::Talk, Screen::Trade, Screen::Board] {
            let panel = panel_width(screen).unwrap();
            for (w, h, scale) in [
                (1920.0_f32, 1080.0_f32, 2.0_f32),
                (2560.0, 1440.0, 3.0),
                (1280.0, 800.0, 1.0),
            ] {
                // Where the open space's middle is, in logical px, and so
                // in normalised screen space.
                let edge = w - (view.panel_margin_px + panel) * scale;
                let middle = edge / 2.0;
                let x = open_space_x(w, scale, view.panel_margin_px, panel);
                assert!((x - (2.0 * middle / w - 1.0)).abs() < 1e-5, "{x}");
                assert!((-1.0..0.0).contains(&x));
                let at = Vec2::new(x, view.frame_y);
                let lens = (1.0_f32, w / h);
                for up in [Vec3::Y, tilted] {
                    let side = up.any_orthonormal_vector();
                    let ahead = up.cross(side);
                    // A townsperson at a talking distance, level, on a step
                    // below, on a step above, off to one side and behind.
                    for (along, across, rise) in [
                        (2.7, 0.0, 0.0),
                        (2.5, 0.8, -1.0),
                        (2.0, -1.2, 1.0),
                        (-2.0, 0.5, 0.0),
                    ] {
                        let eye = Vec3::new(120.0, -40.0, 7.0);
                        let feet = eye + ahead * along + side * across + up * (rise - 1.6);
                        let aim = feet + up * view.aim_height_m;
                        let look = frame_look(aim - eye, up, lens.0, lens.1, at).unwrap();
                        let ndc = projected(eye, up, look, lens, aim);
                        assert!(ndc.z > 0.0 && ndc.z < 1.0, "in front: {ndc}");
                        let px = (ndc.x + 1.0) / 2.0 * w;
                        assert!(
                            (px - middle).abs() < 0.5,
                            "{screen:?} {w}x{h} at {scale}: {px} px, the middle {middle}"
                        );
                        assert!(px > 0.0 && px < edge, "left of the panel: {px}");
                        assert!(ndc.y.abs() < 1e-3, "at the middle height: {ndc}");
                    }
                }
            }
        }
        // Each panel its own middle: the wider Trade puts it further left.
        let x = |s| open_space_x(1920.0, 2.0, 24.0, panel_width(s).unwrap());
        assert!(x(Screen::Trade) < x(Screen::Board) && x(Screen::Board) < x(Screen::Talk));
        // A panel wider than the screen: no further left than -0.95.
        assert_eq!(open_space_x(400.0, 2.0, 24.0, 490.0), -0.95);
        // Only the townsfolk's dialogs are NPC dialogs.
        assert!(!is_npc_dialog(Screen::Pack) && !is_npc_dialog(Screen::Missions));
    }

    /// The talk light burns on the townsperson while a dialog is open,
    /// rising over its fade; once the dialog closes it fades out and is
    /// gone from the carried list.
    #[test]
    fn the_talk_light_burns_only_while_a_talk_is_open() {
        use pbd_app::carried_light::NearLights;
        use pbd_app::economy::TownInfo;
        let view = TalkView::parse(TALK_RON).unwrap();
        let mut app = App::new();
        app.insert_resource(view)
            .insert_resource(Time::<()>::default())
            .insert_resource(Screen::Playing)
            .init_resource::<Talking>()
            .init_resource::<TalkGlow>()
            .init_resource::<NearLights>()
            .add_systems(Update, talk_light);
        let talker = Talker {
            site: 3,
            index: 1,
            role: "trader".into(),
        };
        app.world_mut().spawn((
            talker.clone(),
            GlobalTransform::from_translation(Vec3::new(0.0, 0.0, -2.7)),
        ));
        app.world_mut().spawn((
            Camera3d::default(),
            GlobalTransform::from_translation(Vec3::new(0.0, 1.6, 0.0)),
        ));
        let step = |app: &mut App, secs: f32| {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(secs));
            app.update();
            app.world().resource::<NearLights>().0.clone()
        };
        assert!(step(&mut app, 0.1).is_empty(), "no light with no talk");
        // The talk opens.
        app.world_mut().resource_mut::<Talking>().with = Some(TalkWith {
            talker,
            town: TownInfo {
                site: 3,
                kind: pbd_core::sites::SiteKind::Village,
                name: "Ash".into(),
                neighbours: vec![],
                roads_to: vec![],
            },
            trader: true,
            giver: false,
        });
        *app.world_mut().resource_mut::<Screen>() = Screen::Talk;
        let first = step(&mut app, 0.1);
        assert_eq!(first.len(), 1, "lit while the talk is open");
        let (light, reach) = first[0];
        assert_eq!(reach, view.light_reach_m);
        // Before the face, between the townsperson and the eye.
        assert!((light.at.y - view.light_rise_m).abs() < 1e-4, "{light:?}");
        assert!((light.at.z - (-2.7 + view.light_toward_m)).abs() < 1e-4);
        let full = view.light_level / 14.0;
        assert!(light.colour[0] > 0.0 && light.colour[0] < full * view.light_tint[0]);
        // Risen to full over the fade, and held there through TRADE.
        *app.world_mut().resource_mut::<Screen>() = Screen::Trade;
        let lit = step(&mut app, 1.0);
        assert!((lit[0].0.colour[0] - full * view.light_tint[0]).abs() < 1e-5);
        // The talk closes: still fading out, then gone.
        *app.world_mut().resource_mut::<Screen>() = Screen::Playing;
        app.world_mut().resource_mut::<Talking>().with = None;
        let fading = step(&mut app, 0.1);
        assert_eq!(fading.len(), 1, "fading out");
        assert!(fading[0].0.colour[0] < lit[0].0.colour[0]);
        assert!(step(&mut app, 1.0).is_empty(), "gone after the fade");
        assert_eq!(*app.world().resource::<TalkGlow>(), TalkGlow::default());
    }

    #[test]
    fn the_fade_runs_to_its_end_and_stops() {
        assert_eq!(glow_step(0.0, true, 0.2, 0.4), 0.5);
        assert_eq!(glow_step(0.9, true, 0.2, 0.4), 1.0);
        assert_eq!(glow_step(0.1, false, 0.2, 0.4), 0.0);
        assert_eq!(glow_step(0.0, true, 0.0, 0.0), 1.0);
    }

    #[test]
    fn a_framing_with_nowhere_to_look_is_refused() {
        let at = Vec2::new(-2.0 / 3.0, 0.0);
        assert!(frame_look(Vec3::Y, Vec3::Y, 1.0, 1.6, at).is_none());
        assert!(frame_look(Vec3::ZERO, Vec3::Y, 1.0, 1.6, at).is_none());
    }

    #[test]
    fn talk_view_refuses_a_frame_off_screen() {
        assert!(TalkView::parse(TALK_RON).is_ok());
        assert!(TalkView::parse(&TALK_RON.replace("frame_y: 0.0", "frame_y: -1.5")).is_err());
        assert!(
            TalkView::parse(&TALK_RON.replace("light_level: 7.0", "light_level: 20.0")).is_err()
        );
        assert!(
            TalkView::parse(&TALK_RON.replace("light_reach_m: 4.0", "light_reach_m: 0.0")).is_err()
        );
        assert!(TalkView::parse(&TALK_RON.replace("0.25", "-1.0")).is_err());
    }
}
