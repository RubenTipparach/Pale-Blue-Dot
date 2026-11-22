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
use super::slots::{self, Hotbar, ItemIcons};
use bevy::prelude::*;
use pbd_app::economy::{
    Economy, EconomyConfig, Shop, Talker, TownInfo, Why, fish_ids, job_line, keepers, roster,
    town_info,
};
use pbd_app::interact::{Act, Interact, Offer, Reader};
use pbd_app::saves::WorldSave;
use pbd_app::towns::Towns;
use pbd_app::ui_kit::{self, KitText, Localized, Set, UiKit, tr};
use pbd_core::economy::jobs::Job;
use pbd_core::economy::values::{item_of, trade_code};
use pbd_core::inventory::Stack;
use pbd_core::locale::tr_with;

/// How near a townsperson must be to talk to, metres from the eye.
pub const TALK_REACH_M: f32 = 3.0;
/// The most rows a trade column lists.
const ROWS: usize = 9;

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
}

/// What a talk, trade or board button does.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum TalkAction {
    Trade,
    Work,
    Bye,
    Back,
    Buy(String),
    Sell(String),
    Take(u64),
    HandIn(u64),
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

/// The coin count beside the hotbar.
#[derive(Component)]
pub struct CoinCount;
/// The "job done" line over it.
#[derive(Component)]
pub struct CoinNotice;

/// Build the three panels, hidden, and the coin count. `PostStartup`.
pub fn spawn(mut commands: Commands, kit: Res<UiKit>, mut images: ResMut<Assets<Image>>) {
    let kit = &*kit;
    for (screen, width) in [
        (Screen::Talk, 300.0),
        (Screen::Trade, 470.0),
        (Screen::Board, 420.0),
    ] {
        commands
            .spawn((menu::layer(), GlobalZIndex(10), Panel(screen)))
            .with_children(|layer| {
                let mut panel = layer.spawn(menu::framed(kit, Set::Stone, width));
                match screen {
                    Screen::Talk => panel.insert(TalkBody),
                    Screen::Trade => panel.insert(TradeBody),
                    _ => panel.insert(BoardBody),
                };
            });
    }
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
) -> Option<Talker> {
    let at = eye.translation();
    let look = eye.forward().as_vec3();
    let up = eye.up().as_vec3();
    talkers
        .filter_map(|(t, g)| {
            let chest = g.translation() + up * 1.0;
            let to = chest - at;
            let d = to.length();
            (d <= TALK_REACH_M && to.dot(look) >= 0.5 * d).then_some((d, t))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, t)| t.clone())
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
    if let Some(t) = talker_in_reach(eye, shown(&talkers)) {
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
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    if matches!(*screen, Screen::Talk | Screen::Trade | Screen::Board) {
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
    let Some(talker) = talker_in_reach(eye, shown(&talkers)) else {
        return;
    };
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
    open_talk(
        talker,
        &all,
        &towns,
        &cfg,
        shop.as_mut(),
        &mut talking,
        &mut screen,
    );
    if found {
        talking.said = Some(tr("talk.lost_found"));
    }
}

/// Leaving every talk panel forgets who it was with.
pub fn forget(screen: Res<Screen>, mut talking: ResMut<Talking>) {
    if screen.is_changed()
        && !matches!(*screen, Screen::Talk | Screen::Trade | Screen::Board)
        && talking.with.is_some()
    {
        *talking = Talking::default();
    }
}

/// A button pressed.
#[allow(clippy::too_many_arguments)]
pub fn press(
    buttons: Query<(&Interaction, &TalkAction), Changed<Interaction>>,
    mut screen: ResMut<Screen>,
    mut talking: ResMut<Talking>,
    (cfg, mut eco, mut save, mut hotbar, sun, fauna, body): ShopParams,
) {
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let ids = fish_ids(&fauna, &body.0);
        let Some(with) = talking.with.clone() else {
            continue;
        };
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

/// An item's picture.
fn icon(
    parent: &mut ChildSpawnerCommands,
    item: pbd_core::inventory::Item,
    icons: &ItemIcons,
    images: &Assets<Image>,
) {
    let mut node = ImageNode::default();
    slots::paint_icon(&mut node, Some(Stack::new(item, 1)), icons, images);
    parent.spawn((
        node,
        Node {
            width: px(16),
            height: px(16),
            flex_shrink: 0.0,
            ..default()
        },
    ));
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
}

/// Rebuild the open panel when what it shows changes.
pub fn paint(
    mut commands: Commands,
    mut shown: Local<String>,
    p: PaintParams,
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
        "{open:?}|{:?}|{}|{:?}|{:?}|{}",
        p.talking,
        p.eco.purse.coins,
        p.eco.jobs,
        p.hotbar.0,
        p.sun.clock.day()
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
        Screen::Trade => paint_trade(panel, kit, with, &p, save, icons, &ids, now_s, coin),
        _ => paint_board(panel, kit, with, &p, coin),
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
        values: &p.cfg.values,
        kind: with.town.kind,
        goodwill,
        fish_ids: ids,
    };
    let market = pbd_app::economy::market_of(save, with.town.site).settled(now_s, &p.cfg.trade);
    // What the pack holds that the trader buys, in slot order, once each.
    let mut held: Vec<(String, pbd_core::inventory::Item, u32)> = Vec::new();
    for stack in p.hotbar.0.iter().flatten() {
        let Some(code) = trade_code(stack.item, ids) else {
            continue;
        };
        if trader.buys(&code) && !held.iter().any(|(c, _, _)| *c == code) {
            held.push((code, stack.item, p.hotbar.0.count(stack.item)));
        }
    }
    panel
        .spawn(Node {
            column_gap: px(12),
            ..default()
        })
        .with_children(|cols| {
            cols.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                width: px(205),
                ..default()
            })
            .with_children(|col| {
                menu::section(col, tr("trade.buy"));
                for code in trader.stock_list().into_iter().take(ROWS) {
                    let Some(item) = item_of(&code, ids) else {
                        continue;
                    };
                    let q = trader.quote(&market, &code);
                    let Some(price) = q.sell else { continue };
                    trade_row(
                        col,
                        kit,
                        (icons, &p.images, coin),
                        item,
                        (price, q.stock),
                        TalkAction::Buy(code),
                        tr("trade.buy_one"),
                    );
                }
            });
            cols.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                width: px(205),
                ..default()
            })
            .with_children(|col| {
                menu::section(col, tr("trade.sell"));
                if held.is_empty() {
                    line(col, tr("trade.nothing_to_sell"), ui_kit::TEXT_DIM);
                }
                for (code, item, count) in held.into_iter().take(ROWS) {
                    let q = trader.quote(&market, &code);
                    let Some(price) = q.buy else { continue };
                    trade_row(
                        col,
                        kit,
                        (icons, &p.images, coin),
                        item,
                        (price, count),
                        TalkAction::Sell(code),
                        tr("trade.sell_one"),
                    );
                }
            });
        });
    footer(panel, kit, p, coin);
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
        ..default()
    })
    .with_children(|row| {
        icon(row, item, icons, images);
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
        button(row, kit, label, action, 40.0);
    });
}

/// The purse, the last thing said, and BACK.
fn footer(panel: &mut ChildSpawnerCommands, kit: &UiKit, p: &PaintParams, coin: &Handle<Image>) {
    said_line(panel, &p.talking.said);
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

fn paint_board(
    panel: &mut ChildSpawnerCommands,
    kit: &UiKit,
    with: &TalkWith,
    p: &PaintParams,
    coin: &Handle<Image>,
) {
    menu::lintel(panel, kit, tr("jobs.title"));
    let town = &with.town;
    let eco = &p.eco;
    let towns = &*p.towns;
    // The jobs taken here: done ones to hand in, the rest in hand.
    let mine: Vec<&Job> = eco
        .jobs
        .taken
        .iter()
        .filter(|j| j.site == town.site)
        .collect();
    if !mine.is_empty() {
        menu::section(panel, tr("jobs.yours"));
        for job in mine {
            let done = eco.jobs.is_done(job.id);
            job_row(
                panel,
                (kit, coin),
                job,
                &eco.jobs.counts(job.id),
                towns,
                done.then_some(TalkAction::HandIn(job.id)),
                tr("jobs.hand_in"),
            );
        }
    }
    if with.giver {
        menu::section(panel, tr("jobs.posted"));
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
            board.posted(town.as_town(), day, goodwill, &eco.jobs.gone())
        };
        if posted.is_empty() {
            line(panel, tr("jobs.none"), ui_kit::TEXT_DIM);
        }
        for job in &posted {
            job_row(
                panel,
                (kit, coin),
                job,
                &[],
                towns,
                Some(TalkAction::Take(job.id)),
                tr("jobs.take"),
            );
        }
    }
    footer(panel, kit, p, coin);
}

fn job_row(
    panel: &mut ChildSpawnerCommands,
    (kit, coin): (&UiKit, &Handle<Image>),
    job: &Job,
    counts: &[u32],
    towns: &Towns,
    action: Option<TalkAction>,
    label: Localized,
) {
    panel
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(6),
            ..default()
        })
        .with_children(|row| {
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
/// frame, as a click on it would: `trade`, `work`, `back`, `bye`, or the
/// first `buy`, `sell`, `take` or `handin` row. Nothing reads it in play.
pub fn scripted_press(
    state: Res<super::CaptureState>,
    mut buttons: Query<(&mut Interaction, &TalkAction)>,
    script: Local<std::sync::OnceLock<Vec<(u32, String)>>>,
) {
    let script = script.get_or_init(|| {
        std::env::var("PBD_KEY_SCRIPT")
            .unwrap_or_default()
            .split(',')
            .filter_map(|step| {
                let (frame, rest) = step.trim().split_once(':')?;
                let what = rest.trim().strip_prefix("press:")?;
                Some((frame.trim().parse().ok()?, what.to_string()))
            })
            .collect()
    });
    for (frame, what) in script {
        if state.frame != *frame {
            continue;
        }
        let wanted = |a: &TalkAction| {
            matches!(
                (what.as_str(), a),
                ("trade", TalkAction::Trade)
                    | ("work", TalkAction::Work)
                    | ("back", TalkAction::Back)
                    | ("bye", TalkAction::Bye)
                    | ("buy", TalkAction::Buy(_))
                    | ("sell", TalkAction::Sell(_))
                    | ("take", TalkAction::Take(_))
                    | ("handin", TalkAction::HandIn(_))
            )
        };
        match buttons.iter_mut().find(|(_, a)| wanted(a)) {
            Some((mut interaction, action)) => {
                info!("PBD_KEY_SCRIPT: pressed {action:?} at frame {frame}");
                *interaction = Interaction::Pressed;
            }
            None => warn!("PBD_KEY_SCRIPT: no {what} button at frame {frame}"),
        }
    }
}

pub struct TalkPlugin;

impl Plugin for TalkPlugin {
    fn build(&self, app: &mut App) {
        use pbd_app::interact::InteractSet;
        app.init_resource::<Talking>()
            .add_systems(PostStartup, spawn)
            .add_systems(PostUpdate, offer_talk.in_set(InteractSet::Offer))
            .add_systems(
                Update,
                (
                    use_talk,
                    forget,
                    scripted_press,
                    press,
                    paint,
                    coin_hud,
                    talking_mouth,
                )
                    .chain(),
            );
    }
}
