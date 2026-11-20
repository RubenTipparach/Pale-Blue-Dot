//! A toast for every item gained (`item-toasts`): its icon, its name and how
//! many, stacked by item, about three seconds each, ten at most on screen and
//! the rest paged in as those fade.

use super::menu::Screen;
use super::slots::ItemIcons;
use bevy::prelude::*;
use pbd_app::hotbar::{Gained, PickedUp};
use pbd_app::ui_kit::Localized;
use pbd_app::ui_kit::{self, KitText};
use pbd_core::inventory::{Item, Stack};

/// How long a toast is shown at full strength, seconds (decision 3).
pub const SHOW_S: f32 = 3.0;
/// How long it then takes to fade out.
pub const FADE_S: f32 = 0.4;
/// How many are on screen at once; the rest wait their turn.
pub const SHOWN: usize = 10;

/// One item's toast: how many came in, and how long it has been shown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Toast {
    pub item: Item,
    pub count: u32,
    pub age_s: f32,
}

impl Toast {
    /// Its strength, 1 while shown, falling to 0 over the fade.
    pub fn alpha(&self) -> f32 {
        (1.0 - (self.age_s - SHOW_S) / FADE_S).clamp(0.0, 1.0)
    }
}

/// Every toast, oldest first: the first [`SHOWN`] are on screen.
#[derive(Resource, Default, Debug)]
pub struct Toasts(pub Vec<Toast>);

impl Toasts {
    /// An item came in. Its toast on screen counts it and starts again; one
    /// waiting counts it; otherwise a new one joins the end (decision 2).
    pub fn add(&mut self, item: Item, count: u32) {
        if count == 0 {
            return;
        }
        if let Some((i, toast)) = self.0.iter_mut().enumerate().find(|(_, t)| t.item == item) {
            toast.count += count;
            if i < SHOWN {
                toast.age_s = 0.0;
            }
            return;
        }
        self.0.push(Toast {
            item,
            count,
            age_s: 0.0,
        });
    }

    /// Time passes for the toasts on screen only; the spent ones go and the
    /// waiting ones move up.
    pub fn tick(&mut self, dt: f32) {
        for toast in self.0.iter_mut().take(SHOWN) {
            toast.age_s += dt;
        }
        self.0.retain(|t| t.age_s < SHOW_S + FADE_S);
    }

    /// The toasts on screen, oldest first.
    pub fn shown(&self) -> &[Toast] {
        &self.0[..self.0.len().min(SHOWN)]
    }
}

/// The column of toast rows.
#[derive(Component)]
pub struct ToastColumn;

/// Row `n` of the column: its icon and its words.
#[derive(Component)]
pub struct ToastRow(pub usize);
#[derive(Component)]
pub struct ToastIcon(pub usize);
#[derive(Component)]
pub struct ToastText(pub usize);

/// Build the column, right of the screen above the hotbar's line, newest at
/// the bottom (decision 4). After the icons are loaded.
pub fn spawn(
    mut commands: Commands,
    icons: Res<ItemIcons>,
    existing: Query<(), With<ToastColumn>>,
) {
    if !existing.is_empty() {
        return;
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(8),
                bottom: px(super::slots::HOTBAR_BOTTOM + 34.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                row_gap: px(2),
                ..default()
            },
            GlobalZIndex(6),
            ToastColumn,
        ))
        .with_children(|column| {
            for n in 0..SHOWN {
                column
                    .spawn((
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: px(4),
                            padding: UiRect::axes(px(4), px(2)),
                            display: Display::None,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        ToastRow(n),
                    ))
                    .with_children(|row| {
                        row.spawn((
                            ImageNode::new(icons.atlas.clone()),
                            Node {
                                width: px(16),
                                height: px(16),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            ToastIcon(n),
                        ));
                        row.spawn((
                            KitText::new(Localized::empty(), ui_kit::TEXT).shadow(ui_kit::INK),
                            ToastText(n),
                        ));
                    });
            }
        });
}

/// The words on a toast: "Dirt x2".
pub fn words(toast: &Toast) -> Localized {
    pbd_core::locale::tr_with(
        "toasts.words",
        &[("item", &toast.item.name()), ("count", &toast.count)],
    )
}

/// Count this frame's gains into the toasts and let their time run.
pub fn gather(
    time: Res<Time>,
    mut toasts: ResMut<Toasts>,
    mut gained: MessageReader<Gained>,
    mut picked: MessageReader<PickedUp>,
) {
    for g in gained.read() {
        toasts.add(g.item, u32::from(g.count));
    }
    for p in picked.read() {
        toasts.add(p.item, u32::from(p.count));
    }
    toasts.tick(time.delta_secs());
}

/// Paint the rows from the toasts. Shown only in play (decision 5).
#[allow(clippy::type_complexity)]
pub fn paint(
    toasts: Res<Toasts>,
    screen: Res<Screen>,
    icons: Res<ItemIcons>,
    images: Res<Assets<Image>>,
    mut rows: Query<(&ToastRow, &mut Node, &mut BackgroundColor)>,
    mut pictures: Query<(&ToastIcon, &mut ImageNode)>,
    mut texts: Query<(&ToastText, &mut KitText)>,
) {
    let shown = if *screen == Screen::Playing {
        toasts.shown()
    } else {
        &[]
    };
    for (row, mut node, mut fill) in &mut rows {
        let toast = shown.get(row.0);
        let display = if toast.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        let want = Color::srgba(0.04, 0.06, 0.09, 0.62 * toast.map_or(0.0, Toast::alpha));
        if fill.0 != want {
            fill.0 = want;
        }
    }
    for (icon, mut image) in &mut pictures {
        let Some(toast) = shown.get(icon.0) else {
            continue;
        };
        let count = toast.count.min(u32::from(u16::MAX)) as u16;
        super::slots::paint_icon(
            &mut image,
            Some(Stack::new(toast.item, count)),
            &icons,
            &images,
        );
        if image.color != Color::NONE {
            image.color = image.color.with_alpha(toast.alpha());
        }
    }
    for (text, mut kit) in &mut texts {
        let Some(toast) = shown.get(text.0) else {
            continue;
        };
        let want = words(toast);
        if kit.text != want {
            kit.text = want;
        }
        let colour = ui_kit::TEXT.with_alpha(toast.alpha());
        if kit.colour != colour {
            kit.colour = colour;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::terrain::Material;

    const DIRT: Item = Item::Block(Material::Dirt);
    const SAND: Item = Item::Block(Material::Sand);

    /// The same item stacks on its toast and holds it up again; another item
    /// gets its own (owner: "dirt x1, dirt x2").
    #[test]
    fn the_same_item_stacks_and_holds_its_toast_up() {
        let mut t = Toasts::default();
        t.add(DIRT, 1);
        t.tick(2.0);
        t.add(DIRT, 1);
        t.add(SAND, 1);
        assert_eq!(t.shown().len(), 2);
        assert_eq!(words(&t.shown()[0]), "dirt x2");
        assert_eq!(
            t.shown()[0].age_s,
            0.0,
            "a second dirt starts its three seconds again"
        );
    }

    /// About three seconds, then a fade, then gone.
    #[test]
    fn a_toast_shows_three_seconds_then_fades() {
        let mut t = Toasts::default();
        t.add(DIRT, 1);
        t.tick(2.9);
        assert_eq!(t.shown()[0].alpha(), 1.0);
        t.tick(0.3);
        assert!(t.shown()[0].alpha() < 1.0 && t.shown()[0].alpha() > 0.0);
        t.tick(0.3);
        assert!(t.shown().is_empty());
    }

    /// Ten on screen; the rest wait without ageing and come in as the first
    /// go (owner: "up to like 10 items, then page them").
    #[test]
    fn more_than_ten_wait_their_turn() {
        let mut t = Toasts::default();
        for good in pbd_core::goods::Good::all().take(12) {
            t.add(Item::Good(good), 1);
        }
        assert_eq!(t.shown().len(), SHOWN);
        t.tick(SHOW_S + FADE_S);
        assert_eq!(t.shown().len(), 2, "the two waiting come in");
        assert_eq!(t.shown()[0].age_s, 0.0, "and start their time now");
    }
}
