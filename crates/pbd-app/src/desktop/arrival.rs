//! The world's name on arrival (`new-world-arrival` decision 6): when the
//! loading screen lifts on a saved world and the player is in it, its name
//! and whether it is new, in the middle of the screen for a few seconds.
//!
//! The owner, 2026-11-14: "I dont really know if my game even really
//! loaded". A switch that changes nothing in view (every world is the one
//! planet, so a new world may open on ground that looks like the last one)
//! still says which world this is.

use bevy::prelude::*;

/// Seconds the banner holds before it fades.
const HOLD_S: f32 = 3.0;
/// Seconds it takes to fade.
const FADE_S: f32 = 1.0;

const INK: Color = Color::srgb(0.88, 0.94, 0.91);
const MINT: Color = Color::srgb(0.48, 0.8, 0.77);

/// A world to name when the player is next in it.
#[derive(Resource, Clone, Debug)]
pub struct Arrival {
    name: String,
    fresh: bool,
    /// Seconds shown so far; `None` until it is on screen.
    shown_s: Option<f32>,
}

impl Arrival {
    pub fn new(name: &str, fresh: bool) -> Self {
        Self {
            name: name.to_string(),
            fresh,
            shown_s: None,
        }
    }

    /// The two lines: the name, and what kind of arrival it is.
    pub fn lines(&self) -> (String, &'static str) {
        (
            self.name.to_uppercase(),
            if self.fresh {
                "A NEW WORLD"
            } else {
                "WELCOME BACK"
            },
        )
    }
}

/// How opaque the banner is `shown_s` seconds after it appeared; `None` once
/// it is gone.
pub fn opacity(shown_s: f32) -> Option<f32> {
    if shown_s < HOLD_S {
        Some(1.0)
    } else if shown_s < HOLD_S + FADE_S {
        Some(1.0 - (shown_s - HOLD_S) / FADE_S)
    } else {
        None
    }
}

#[derive(Component)]
pub struct Banner;

/// `--new-world` in a capture (an instrument): the banner photographed a
/// second after it appears, to this path.
#[derive(Resource, Clone)]
pub struct ArrivalShot(pub std::path::PathBuf);

/// Show, fade and take down the banner. Only while the world is playable
/// and no menu is up, so a world opened behind the saves page is named when
/// the player steps into it.
#[allow(clippy::too_many_arguments)]
pub fn banner(
    mut commands: Commands,
    time: Res<Time>,
    arrival: Option<ResMut<Arrival>>,
    screen: Res<super::menu::Screen>,
    banners: Query<Entity, With<Banner>>,
    mut colours: Query<(&mut TextColor, &BannerLine)>,
    shot: Option<Res<ArrivalShot>>,
    mut taken: Local<bool>,
) {
    let Some(mut arrival) = arrival else {
        return;
    };
    let Some(shown) = arrival.shown_s else {
        if *screen != super::menu::Screen::Playing {
            return;
        }
        let (name, kind) = arrival.lines();
        info!("ARRIVE banner '{name}' {kind}");
        spawn(&mut commands, &name, kind);
        arrival.shown_s = Some(0.0);
        return;
    };
    let shown = shown + time.delta_secs();
    arrival.shown_s = Some(shown);
    if let Some(path) = shot.as_ref().filter(|_| shown >= 1.0 && !*taken) {
        *taken = true;
        info!("ARRIVE banner photographed to {}", path.0.display());
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(path.0.clone()));
    }
    match opacity(shown) {
        Some(alpha) => {
            for (mut colour, line) in &mut colours {
                colour.0 = line.0.with_alpha(alpha);
            }
        }
        None => {
            for entity in &banners {
                commands.entity(entity).despawn();
            }
            commands.remove_resource::<Arrival>();
        }
    }
}

/// A line of the banner and its full colour.
#[derive(Component)]
pub struct BannerLine(Color);

fn spawn(commands: &mut Commands, name: &str, kind: &str) {
    let text = |size: f32| TextFont {
        font_size: size,
        ..default()
    };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                top: percent(28),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(6),
                ..default()
            },
            Banner,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(name),
                text(30.0),
                TextColor(INK),
                TextShadow::default(),
                BannerLine(INK),
            ));
            root.spawn((
                Text::new(kind),
                text(13.0),
                TextColor(MINT),
                TextShadow::default(),
                BannerLine(MINT),
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_banner_holds_then_fades_then_goes() {
        assert_eq!(opacity(0.0), Some(1.0));
        assert_eq!(opacity(HOLD_S - 0.01), Some(1.0));
        let half = opacity(HOLD_S + FADE_S / 2.0).unwrap();
        assert!((half - 0.5).abs() < 1e-4, "{half}");
        assert_eq!(opacity(HOLD_S + FADE_S + 0.01), None);
    }

    #[test]
    fn a_new_world_says_so_and_a_saved_one_welcomes_back() {
        assert_eq!(
            Arrival::new("Cave", true).lines(),
            ("CAVE".to_string(), "A NEW WORLD")
        );
        assert_eq!(Arrival::new("Cave", false).lines().1, "WELCOME BACK");
    }
}
