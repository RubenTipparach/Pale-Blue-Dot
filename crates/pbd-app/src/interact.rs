//! The interact key (F) and the prompt that names what it will do
//! (`stone-and-wood-ui` design, "Interact on F, and the prompt that names
//! it"; owner, 2026-11-16).
//!
//! One press does one thing. [`ACTS`] is the one table of what F can do, in
//! priority order. Every frame the systems that know a kind of thing (the
//! husbandry hands, the fruit, the farm and its stations, the town doors,
//! the boats) [`Interact::offer`] what the eye is on; [`resolve`] keeps the
//! highest row offered as [`Interact::now`]. The prompt prints `now`, and
//! an F reader stands aside when `now` names another reader's act ([`may`]),
//! so the press does what the prompt names. With nothing named (no offer, or
//! a reader that runs before the offers have been seen), each reader does
//! what it always did, and the first to use the press takes it.

use crate::ui_kit::{Localized, tr};
use bevy::prelude::*;
use pbd_core::locale::{caps, tr_with};

/// Who carries an act out: the system that reads F for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reader {
    Husbandry,
    Fruit,
    /// The glowing mushrooms (`glowing-mushrooms` design 3).
    Glow,
    Farm,
    Doors,
    Boats,
    /// The townsfolk: talk, then trade and jobs (`townsfolk` design 5 to 9).
    Talk,
    /// A log on the water (`log-drives`).
    Logs,
    /// A broken town wall (`townsfolk` design 29).
    Mend,
    /// The carts (`land-transport` design 11).
    Carts,
    /// A town lamp at dusk (`town-missions-tree` design 8).
    Lamps,
    /// A seat, a bed or a container (`furniture` design 6, 11).
    Furniture,
    /// The drafting table (`building-planner` design 3).
    Planner,
    /// A construction site's pieces (`building-planner` design 6, 7).
    Site,
    /// The sea floor's pieces, crawlers and ornaments (`sea-gathering`).
    Sea,
    /// Survival's own: drinking (`survival-mode` design 15).
    Survival,
    /// Fire: a torch held to a plant or to a torch (`wildfire` design 6, 9).
    Fire,
}

/// A thing F can do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Act {
    Collect,
    SetEggs,
    Feed,
    LetStay,
    Lead,
    SetInIncubator,
    TakeIncubator,
    TakeUp,
    PickFruit,
    /// Pick a glowing mushroom (`glowing-mushrooms` design 3).
    PickGlow,
    Open,
    Harvest,
    Check,
    PickWild,
    /// Take a stone or fibre from the ground clutter (`gather-by-hand`).
    Gather,
    OpenDoor,
    ShutDoor,
    Till,
    /// Take a log out of the water (`log-drives`).
    TakeLog,
    Board,
    /// Talk to a townsperson.
    Talk,
    /// Mend a broken town wall with planks.
    Mend,
    /// Let go of a hand cart's handles (`land-transport` design 11).
    LetGo,
    /// Step down from a horse cart's seat.
    StepDown,
    /// Take a hand cart's handles.
    Pull,
    /// Take a horse cart's seat.
    Drive,
    /// Put a horse in a horse cart's shafts.
    Hitch,
    /// Take it out of them.
    Unhitch,
    /// Light a town lamp at dusk, for a lamps job.
    Light,
    /// Mount a kept plodder or horse (`land-transport` design 14).
    Ride,
    /// Get off it.
    GetOff,
    /// Sit on a chair, a stool or a bench (`furniture` design 6).
    Sit,
    /// Lie down on a bed.
    Lie,
    /// Open drawers, a closet or a chest (`furniture` design 11).
    OpenStore,
    /// Draw a plan at the drafting table (`building-planner` design 3).
    Plan,
    /// Strike a site's piece with the mallet (design 6).
    Build,
    /// Take a marked piece down (design 7).
    TakeDown,
    /// Set a piece of furniture in its slot (design 6).
    Fill,
    /// Take a piece or a crawler off the sea floor (`sea-gathering` design 2).
    GatherSea,
    /// Take a set-down ornament back up (`sea-gathering` design 8).
    TakeOrnament,
    /// Feed a growing crop fertiliser (`sea-gathering` design 6).
    Fertilize,
    /// Take a laid bedroll back up (`survival-mode` task 6.3).
    TakeUpBedroll,
    /// Pour into or drink from a still (`survival-mode` design 15, 18).
    UseStill,
    /// A cactus sipped through a straw (`survival-mode` design 15).
    Sip,
    /// A sip of water with an empty hand (`survival-mode` design 15).
    Drink,
    /// A lit torch held to a plant, or a torch lit again (`wildfire`).
    Kindle,
}

/// One row of the table: an act, the verb the prompt names it by (the
/// thing follows it, or stands at its `{}`), and the reader that carries it
/// out.
#[derive(Debug)]
pub struct Row {
    pub act: Act,
    pub verb: &'static str,
    pub reader: Reader,
}

const fn row(act: Act, verb: &'static str, reader: Reader) -> Row {
    Row { act, verb, reader }
}

/// Everything F does, highest priority first: the one table the prompt and
/// the readers share.
pub const ACTS: [Row; 46] = [
    // A cart in hand, a seat or a back taken comes first: F lets it go.
    row(Act::LetGo, "let_go", Reader::Carts),
    row(Act::StepDown, "step_down", Reader::Carts),
    row(Act::GetOff, "get_off", Reader::Carts),
    row(Act::Collect, "collect", Reader::Husbandry),
    row(Act::SetEggs, "set_eggs", Reader::Husbandry),
    row(Act::Feed, "feed", Reader::Husbandry),
    // A kept plodder or horse is ridden before it is led or let stay.
    row(Act::Ride, "ride", Reader::Carts),
    row(Act::LetStay, "let_stay", Reader::Husbandry),
    row(Act::Lead, "lead", Reader::Husbandry),
    row(Act::SetInIncubator, "set_in_incubator", Reader::Husbandry),
    row(Act::TakeIncubator, "take_incubator", Reader::Husbandry),
    row(Act::TakeUp, "take_up", Reader::Husbandry),
    row(Act::PickFruit, "pick_fruit", Reader::Fruit),
    row(Act::PickGlow, "pick_glow", Reader::Glow),
    row(Act::GatherSea, "gather_sea", Reader::Sea),
    row(Act::TakeOrnament, "take_ornament", Reader::Sea),
    row(Act::Open, "open", Reader::Farm),
    row(Act::Plan, "plan", Reader::Planner),
    row(Act::Harvest, "harvest", Reader::Farm),
    row(Act::Fertilize, "fertilize", Reader::Farm),
    row(Act::Check, "check", Reader::Farm),
    row(Act::PickWild, "pick_wild", Reader::Farm),
    row(Act::Gather, "gather", Reader::Farm),
    row(Act::Talk, "talk", Reader::Talk),
    row(Act::OpenDoor, "open_door", Reader::Doors),
    row(Act::ShutDoor, "shut_door", Reader::Doors),
    row(Act::OpenStore, "open_store", Reader::Furniture),
    row(Act::Sit, "sit", Reader::Furniture),
    row(Act::Lie, "lie", Reader::Furniture),
    row(Act::Fill, "fill", Reader::Site),
    row(Act::TakeDown, "take_down", Reader::Site),
    row(Act::Build, "build", Reader::Site),
    row(Act::Till, "till", Reader::Farm),
    row(Act::TakeLog, "take_log", Reader::Logs),
    row(Act::Board, "board", Reader::Boats),
    row(Act::Pull, "pull", Reader::Carts),
    row(Act::Drive, "drive", Reader::Carts),
    row(Act::Hitch, "hitch", Reader::Carts),
    row(Act::Unhitch, "unhitch", Reader::Carts),
    row(Act::Mend, "mend", Reader::Mend),
    row(Act::Light, "light", Reader::Lamps),
    row(Act::Kindle, "light", Reader::Fire),
    row(Act::TakeUpBedroll, "take_up", Reader::Survival),
    row(Act::UseStill, "use_still", Reader::Survival),
    row(Act::Sip, "sip", Reader::Survival),
    // Last: anything else at the water's edge comes first.
    row(Act::Drink, "drink", Reader::Survival),
];

impl Act {
    /// Its row of [`ACTS`].
    pub fn row(self) -> &'static Row {
        ACTS.iter()
            .find(|r| r.act == self)
            .expect("every act has its row")
    }

    /// Its place in [`ACTS`]: lower wins.
    pub fn rank(self) -> usize {
        ACTS.iter()
            .position(|r| r.act == self)
            .expect("every act has its row")
    }
}

/// An act on a named thing: what one source says F would do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub act: Act,
    /// What it is done to: "wheat", "Yard hen", "workbench".
    pub thing: Localized,
}

impl Offer {
    pub fn new(act: Act, thing: Localized) -> Self {
        Self { act, thing }
    }

    /// The prompt's words after the key: the verb and the thing,
    /// `interact.act.<verb>` in the locale file.
    pub fn words(&self) -> Localized {
        let key = format!("interact.act.{}", self.act.row().verb);
        let words = tr_with(&key, &[("thing", &self.thing)]);
        Localized::verbatim(words.trim())
    }
}

/// How long a thing stays named after the aim leaves it, s: long enough to
/// cross the seam between two cells of a field, short enough to read as
/// "gone" when the aim moves off.
pub const HOLD_S: f32 = 0.15;

/// The offers of this frame and what F does now.
#[derive(Resource, Default, Debug)]
pub struct Interact {
    offers: Vec<Offer>,
    /// What the next press of F does; `None` is nothing.
    pub now: Option<Offer>,
    /// Seconds since `now` was last offered.
    unseen_s: f32,
}

impl Interact {
    /// A source's offer for this frame.
    pub fn offer(&mut self, offer: Offer) {
        self.offers.push(offer);
    }

    /// Whether `reader` may carry out the press: `now` is one of its acts,
    /// or nothing is named.
    pub fn lets(&self, reader: Reader) -> bool {
        self.now
            .as_ref()
            .is_none_or(|o| o.act.row().reader == reader)
    }

    /// Take the frame's offers: the highest wins; with none, `now` is held
    /// for [`HOLD_S`] and then dropped. `shut` (a menu or panel open, not on
    /// foot) drops everything at once.
    pub fn settle(&mut self, dt: f32, shut: bool) {
        let best = self.offers.drain(..).min_by_key(|o| o.act.rank());
        if shut {
            self.now = None;
            self.unseen_s = 0.0;
            return;
        }
        match best {
            Some(offer) => {
                self.now = Some(offer);
                self.unseen_s = 0.0;
            }
            None => {
                self.unseen_s += dt;
                if self.unseen_s > HOLD_S {
                    self.now = None;
                }
            }
        }
    }
}

/// Whether `reader` acts on this press of F: not when the prompt names
/// another reader's act. With no [`Interact`] (a test of one reader, a tool)
/// every reader acts as it always did.
pub fn may(interact: Option<&Interact>, reader: Reader) -> bool {
    interact.is_none_or(|i| i.lets(reader))
}

/// When a source that searches the cells round the eye looks again: when
/// the eye moves 2 cm or turns a fifth of a degree, the fine set changes,
/// or a quarter second has passed (a save's change: a picked plant). Between
/// those it offers what it found last.
#[derive(Default)]
pub struct Recheck {
    eye: Vec3,
    look: Vec3,
    version: u64,
    since_s: f32,
    primed: bool,
}

impl Recheck {
    pub fn due(&mut self, eye: Vec3, look: Vec3, version: u64, dt: f32) -> bool {
        self.since_s += dt;
        let moved = eye.distance(self.eye) > 0.02 || look.dot(self.look) < 0.999_994;
        if !self.primed || moved || version != self.version || self.since_s > 0.25 {
            *self = Self {
                eye,
                look,
                version,
                since_s: 0.0,
                primed: true,
            };
            return true;
        }
        false
    }
}

/// The systems that offer, and the one that resolves after them.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum InteractSet {
    Offer,
    Resolve,
}

/// Whether the player is on foot with the pointer: the only time F
/// interacts and the prompt shows.
pub fn on_foot(
    walking: Option<&crate::walking::WalkingReadout>,
    aboard: Option<&crate::vehicles::Aboard>,
) -> bool {
    walking.is_some_and(|w| w.active && w.captured) && !aboard.is_some_and(|a| a.0.is_some())
}

/// Settle the frame's offers into [`Interact::now`].
pub fn resolve(
    time: Res<Time>,
    mut interact: ResMut<Interact>,
    menu: Option<Res<crate::controls::MenuOpen>>,
    walking: Option<Res<crate::walking::WalkingReadout>>,
    aboard: Option<Res<crate::vehicles::Aboard>>,
) {
    let shut = menu.is_some_and(|m| m.0) || !on_foot(walking.as_deref(), aboard.as_deref());
    interact.settle(time.delta_secs(), shut);
}

/// The interact slot's key now: what the prompt's cap prints.
pub fn key(binds: &crate::controls::KeyBinds) -> KeyCode {
    crate::controls::slots()
        .find(|s| s.name() == crate::controls::INTERACT_SLOT)
        .map_or(KeyCode::KeyF, |s| binds.key(&s))
}

/// The prompt's box.
#[derive(Component)]
pub struct Prompt;

/// The prompt's key cap text.
#[derive(Component)]
pub struct PromptKey;

/// The prompt's verb and thing.
#[derive(Component)]
pub struct PromptWords;

/// The fishing buttons, bottom left: both mouse buttons in every phase, each
/// with what it does now, the idle one dimmed (owner, 2026-11-18: "need to
/// show both right and left click for fishing, and they both need to be on
/// the left corner of the screen").
#[derive(Component)]
pub struct FishKeys;

/// What one of the fishing buttons does now: 0 the left, 1 the right.
#[derive(Component)]
pub struct FishKeyWords(pub usize);

/// The label on one fishing button's key glyph: 0 the left, 1 the right.
#[derive(Component)]
pub struct FishKeyCap(pub usize);

/// A fishing button that does nothing now: grey and faded (owner,
/// 2026-11-18: "otherwise it should appear grey and faded").
const FADED: Color = Color::srgba(0.57, 0.6, 0.62, 0.45);

/// The prompt, just under the crosshair: a stone tooltip holding the key in
/// an oak cap and the verb and thing.
pub fn spawn_prompt(mut commands: Commands, kit: Res<crate::ui_kit::UiKit>) {
    use crate::ui_kit::KitText;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: percent(52),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Visibility::Hidden,
            Prompt,
        ))
        .with_children(|row| {
            // No box round it (owner, 2026-11-17: "we dont have a box around
            // stuff, just the letter glyph and teh text with black outline"),
            // as the boat panel.
            row.spawn(Node {
                column_gap: px(5),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|tip| {
                crate::ui_kit::key_cap(tip, &kit, tr("interact.key"), PromptKey);
                tip.spawn((
                    Node {
                        padding: UiRect::top(px(2)),
                        ..default()
                    },
                    KitText::new(Localized::empty(), crate::ui_kit::LABEL)
                        .outline(crate::ui_kit::INK),
                    PromptWords,
                ));
            });
        });
}

/// The fishing buttons, bottom left (`FishKeys`): a key glyph and its words
/// for each mouse button, hidden until the rod is out at fishable water.
pub fn spawn_fish_keys(mut commands: Commands, kit: Res<crate::ui_kit::UiKit>) {
    use crate::ui_kit::KitText;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                bottom: px(12),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            },
            Visibility::Hidden,
            FishKeys,
        ))
        .with_children(|col| {
            for (k, button) in [MouseButton::Left, MouseButton::Right]
                .into_iter()
                .enumerate()
            {
                col.spawn(Node {
                    column_gap: px(5),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    let label = crate::controls::Key::Mouse(button).label();
                    crate::ui_kit::key_cap(row, &kit, label, FishKeyCap(k));
                    row.spawn((
                        Node {
                            padding: UiRect::top(px(2)),
                            ..default()
                        },
                        KitText::new(Localized::empty(), crate::ui_kit::LABEL)
                            .outline(crate::ui_kit::INK),
                        FishKeyWords(k),
                    ));
                });
            }
        });
}

/// Show the fishing buttons with what each does now, or hide them.
pub fn show_fish_keys(
    time: Res<Time>,
    fishing: Option<Res<crate::fish::FishingStatus>>,
    mut root: Query<&mut Visibility, With<FishKeys>>,
    mut words: Query<(&mut crate::ui_kit::KitText, &FishKeyWords, &ChildOf), Without<FishKeyCap>>,
    mut caps: Query<(&mut crate::ui_kit::KitText, &FishKeyCap), Without<FishKeyWords>>,
    mut rows: Query<&mut Node, Without<FishKeyWords>>,
) {
    let keys = fishing.as_deref().and_then(|f| f.keys);
    for mut visibility in &mut root {
        let want = if keys.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
    }
    let Some(keys) = keys else { return };
    let bite = fishing.as_deref().is_some_and(|f| f.bite);
    for (mut text, which, parent) in &mut words {
        let (line, live) = keys[which.0];
        // At the bite the left click's words turn yellow and its row shakes.
        let alert = bite && which.0 == 0;
        let colour = if alert {
            crate::ui_kit::TITLE
        } else if live {
            crate::ui_kit::LABEL
        } else {
            FADED
        };
        let line = if line.is_empty() {
            Localized::empty()
        } else {
            tr(line)
        };
        if text.text != line || text.colour != colour {
            text.text = line.clone();
            text.colour = colour;
        }
        if let Ok(mut row) = rows.get_mut(parent.parent()) {
            let shake = if alert {
                px((time.elapsed_secs() * 2.0 * std::f32::consts::PI * 18.0)
                    .sin()
                    .round()
                    * 2.0)
            } else {
                px(0)
            };
            if row.left != shake {
                row.left = shake;
            }
            // A button with nothing to say is not shown (the right click
            // before the line is cast).
            let display = if line.is_empty() {
                Display::None
            } else {
                Display::Flex
            };
            if row.display != display {
                row.display = display;
            }
        }
    }
    for (mut text, which) in &mut caps {
        let colour = if keys[which.0].1 {
            crate::ui_kit::LABEL
        } else {
            FADED
        };
        if text.colour != colour {
            text.colour = colour;
        }
    }
}

/// Show `now`, or hide the prompt.
pub fn show_prompt(
    interact: Res<Interact>,
    binds: Option<Res<crate::controls::KeyBinds>>,
    mut prompt: Query<&mut Visibility, With<Prompt>>,
    mut key_text: Query<&mut crate::ui_kit::KitText, (With<PromptKey>, Without<PromptWords>)>,
    mut words: Query<&mut crate::ui_kit::KitText, (With<PromptWords>, Without<PromptKey>)>,
) {
    let shown = interact.now.as_ref();
    for mut visibility in &mut prompt {
        let want = if shown.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
    }
    let (label, line) = match shown {
        Some(offer) => {
            let none = crate::controls::KeyBinds::default();
            (
                crate::controls::key_label(key(binds.as_deref().unwrap_or(&none))),
                caps(&offer.words()),
            )
        }
        None => return,
    };
    for mut text in &mut key_text {
        if text.text != label {
            text.text = label.clone();
        }
    }
    for mut text in &mut words {
        if text.text != line {
            text.text = line.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every act's words are in the locale file.
    #[test]
    fn every_act_has_its_words() {
        let english = pbd_core::locale::Locale::english();
        for row in &ACTS {
            let key = format!("interact.act.{}", row.verb);
            assert!(english.get(&key).is_some(), "{key}");
        }
    }

    #[test]
    fn every_act_has_one_row_and_a_verb() {
        for (k, r) in ACTS.iter().enumerate() {
            assert_eq!(r.act.rank(), k, "{:?} has one row", r.act);
            assert!(!r.verb.is_empty());
        }
    }

    /// The highest offer wins whatever order the sources ran in; a crop
    /// beside a boat is harvested, not boarded.
    #[test]
    fn the_highest_offer_wins() {
        let mut i = Interact::default();
        i.offer(Offer::new(Act::Board, Localized::verbatim("loon")));
        i.offer(Offer::new(Act::Harvest, Localized::verbatim("wheat")));
        i.offer(Offer::new(Act::Till, Localized::verbatim("grass")));
        i.settle(0.016, false);
        assert_eq!(
            i.now,
            Some(Offer::new(Act::Harvest, Localized::verbatim("wheat")))
        );
        assert!(i.lets(Reader::Farm));
        assert!(!i.lets(Reader::Boats));
        assert_eq!(i.now.as_ref().unwrap().words(), "Harvest wheat");
        i.offer(Offer::new(Act::Board, Localized::verbatim("loon")));
        i.settle(0.016, false);
        assert!(i.lets(Reader::Boats), "nothing usable aimed: F boards");
    }

    /// Crossing a seam between cells for a frame or two does not blink the
    /// prompt; leaving does, after the hold; a panel drops it at once.
    #[test]
    fn the_prompt_holds_across_a_seam_and_goes_after() {
        let mut i = Interact::default();
        i.offer(Offer::new(Act::Harvest, Localized::verbatim("wheat")));
        i.settle(0.016, false);
        for _ in 0..3 {
            i.settle(0.016, false);
            assert!(i.now.is_some(), "held across the seam");
        }
        for _ in 0..12 {
            i.settle(0.016, false);
        }
        assert!(i.now.is_none(), "gone after the hold");
        i.offer(Offer::new(Act::Open, Localized::verbatim("forge")));
        i.settle(0.016, false);
        i.settle(0.016, true);
        assert!(i.now.is_none(), "a panel hides it at once");
    }

    #[test]
    fn with_nothing_named_every_reader_may_act() {
        assert!(may(None, Reader::Farm));
        let mut i = Interact::default();
        assert!(may(Some(&i), Reader::Boats), "nothing named");
        i.offer(Offer::new(Act::Open, Localized::verbatim("forge")));
        i.settle(0.016, false);
        assert!(may(Some(&i), Reader::Farm));
        assert!(!may(Some(&i), Reader::Boats), "the forge, not the boat");
    }
}
