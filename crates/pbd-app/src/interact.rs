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

use bevy::prelude::*;

/// Who carries an act out: the system that reads F for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reader {
    Husbandry,
    Fruit,
    Farm,
    Doors,
    Boats,
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
    Open,
    Harvest,
    Check,
    PickWild,
    /// Take a stone or fibre from the ground clutter (`gather-by-hand`).
    Gather,
    OpenDoor,
    ShutDoor,
    Till,
    Board,
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
pub const ACTS: [Row; 18] = [
    row(Act::Collect, "Collect", Reader::Husbandry),
    row(Act::SetEggs, "Set eggs under", Reader::Husbandry),
    row(Act::Feed, "Feed", Reader::Husbandry),
    row(Act::LetStay, "Let {} stay", Reader::Husbandry),
    row(Act::Lead, "Lead", Reader::Husbandry),
    row(Act::SetInIncubator, "Set eggs in", Reader::Husbandry),
    row(Act::TakeIncubator, "Take", Reader::Husbandry),
    row(Act::TakeUp, "Take up", Reader::Husbandry),
    row(Act::PickFruit, "Pick", Reader::Fruit),
    row(Act::Open, "Open", Reader::Farm),
    row(Act::Harvest, "Harvest", Reader::Farm),
    row(Act::Check, "Check", Reader::Farm),
    row(Act::PickWild, "Pick wild", Reader::Farm),
    row(Act::Gather, "Gather", Reader::Farm),
    row(Act::OpenDoor, "Open", Reader::Doors),
    row(Act::ShutDoor, "Shut", Reader::Doors),
    row(Act::Till, "Till", Reader::Farm),
    row(Act::Board, "Board", Reader::Boats),
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
    /// What it is done to, lower case: "wheat", "hen", "workbench".
    pub thing: String,
}

impl Offer {
    pub fn new(act: Act, thing: impl Into<String>) -> Self {
        Self {
            act,
            thing: thing.into(),
        }
    }

    /// The prompt's words after the key: the verb and the thing.
    pub fn words(&self) -> String {
        let verb = self.act.row().verb;
        if verb.contains("{}") {
            verb.replace("{}", &self.thing)
        } else if self.thing.is_empty() {
            verb.to_string()
        } else {
            format!("{verb} {}", self.thing)
        }
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
                crate::ui_kit::key_cap(tip, &kit, "F", PromptKey);
                tip.spawn((
                    Node {
                        padding: UiRect::top(px(2)),
                        ..default()
                    },
                    KitText::new("", crate::ui_kit::LABEL).outline(crate::ui_kit::INK),
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
                        KitText::new("", crate::ui_kit::LABEL).outline(crate::ui_kit::INK),
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
        if text.text != line || text.colour != colour {
            text.text = line.to_string();
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
                offer.words().to_uppercase(),
            )
        }
        None => return,
    };
    for mut text in &mut key_text {
        if text.text != label {
            text.text = label.to_string();
        }
    }
    for mut text in &mut words {
        if text.text != line {
            text.text.clone_from(&line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        i.offer(Offer::new(Act::Board, "rowboat"));
        i.offer(Offer::new(Act::Harvest, "wheat"));
        i.offer(Offer::new(Act::Till, "grass"));
        i.settle(0.016, false);
        assert_eq!(i.now, Some(Offer::new(Act::Harvest, "wheat")));
        assert!(i.lets(Reader::Farm));
        assert!(!i.lets(Reader::Boats));
        assert_eq!(i.now.as_ref().unwrap().words(), "Harvest wheat");
        i.offer(Offer::new(Act::Board, "rowboat"));
        i.settle(0.016, false);
        assert!(i.lets(Reader::Boats), "nothing usable aimed: F boards");
    }

    /// Crossing a seam between cells for a frame or two does not blink the
    /// prompt; leaving does, after the hold; a panel drops it at once.
    #[test]
    fn the_prompt_holds_across_a_seam_and_goes_after() {
        let mut i = Interact::default();
        i.offer(Offer::new(Act::Harvest, "wheat"));
        i.settle(0.016, false);
        for _ in 0..3 {
            i.settle(0.016, false);
            assert!(i.now.is_some(), "held across the seam");
        }
        for _ in 0..12 {
            i.settle(0.016, false);
        }
        assert!(i.now.is_none(), "gone after the hold");
        i.offer(Offer::new(Act::Open, "forge"));
        i.settle(0.016, false);
        i.settle(0.016, true);
        assert!(i.now.is_none(), "a panel hides it at once");
    }

    #[test]
    fn with_nothing_named_every_reader_may_act() {
        assert!(may(None, Reader::Farm));
        let mut i = Interact::default();
        assert!(may(Some(&i), Reader::Boats), "nothing named");
        i.offer(Offer::new(Act::Open, "forge"));
        i.settle(0.016, false);
        assert!(may(Some(&i), Reader::Farm));
        assert!(!may(Some(&i), Reader::Boats), "the forge, not the boat");
    }
}
