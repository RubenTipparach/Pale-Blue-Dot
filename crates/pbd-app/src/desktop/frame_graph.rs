//! The frame graph: an INSTRUMENT, like the near-field line (`hud.rs`), at the
//! owner's request while the frame spikes are chased. Off by default; `F3`
//! shows and hides it, and `--frame-graph` starts with it shown.
//!
//! It draws the last `BARS` frames' wall times as bars, top right: green within
//! the 120 Hz budget, amber within 60 Hz, red past it, against dashed lines at
//! both budgets, with the window's frame rate, median, 99th percentile and
//! worst frame above. The numbers are `FrameStats`, the same samples the
//! capture's `FRAME_WALL_MS` and `--frame-log` report, so the graph and the
//! logs cannot disagree.

use bevy::prelude::*;

use super::FrameStats;

/// Frames shown, and the graph's size in pixels.
const BARS: usize = 240;
const BAR_W: f32 = 1.5;
const GRAPH_H: f32 = 100.0;
/// The frame time at the top of the graph, ms; taller frames are clipped and
/// drawn solid red.
const TOP_MS: f32 = 50.0;
const BUDGET_120: f32 = 1000.0 / 120.0;
const BUDGET_60: f32 = 1000.0 / 60.0;

#[derive(Component)]
pub struct FrameGraph;

#[derive(Component)]
pub struct FrameGraphBar(usize);

#[derive(Component)]
pub struct FrameGraphText;

pub fn setup(mut commands: Commands, launch: Res<super::Launch>) {
    let width = BARS as f32 * BAR_W;
    commands
        .spawn((
            FrameGraph,
            Node {
                position_type: PositionType::Absolute,
                right: px(8),
                top: px(6),
                width: px(width + 8.0),
                height: px(GRAPH_H + 26.0),
                padding: UiRect::all(px(4)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.62)),
            if launch.frame_graph {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
        ))
        .with_children(|root| {
            root.spawn((
                FrameGraphText,
                Text::new(""),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.97, 0.95)),
                Node {
                    height: px(16),
                    ..default()
                },
            ));
            root.spawn(Node {
                width: px(width),
                height: px(GRAPH_H),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                ..default()
            })
            .with_children(|plot| {
                for i in 0..BARS {
                    plot.spawn((
                        FrameGraphBar(i),
                        Node {
                            width: px(BAR_W),
                            height: px(0),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ));
                }
                for budget in [BUDGET_120, BUDGET_60] {
                    plot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            bottom: px(GRAPH_H * budget / TOP_MS),
                            width: px(width),
                            height: px(1),
                            ..default()
                        },
                        BackgroundColor(if budget < BUDGET_60 {
                            Color::srgba(0.35, 0.85, 0.35, 0.55)
                        } else {
                            Color::srgba(0.95, 0.30, 0.25, 0.55)
                        }),
                    ));
                }
            });
        });
}

/// `F3` shows and hides the graph.
pub fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut graphs: Query<&mut Visibility, With<FrameGraph>>,
) {
    if !keys.just_pressed(KeyCode::F3) {
        return;
    }
    for mut visibility in &mut graphs {
        *visibility = match *visibility {
            Visibility::Hidden => Visibility::Inherited,
            _ => Visibility::Hidden,
        };
    }
}

/// Redraw the bars and the numbers from the newest frames, while shown.
pub fn update(
    stats: Res<FrameStats>,
    graphs: Query<&Visibility, With<FrameGraph>>,
    mut bars: Query<(&FrameGraphBar, &mut Node, &mut BackgroundColor)>,
    mut text: Query<&mut Text, With<FrameGraphText>>,
) {
    if graphs.iter().all(|v| *v == Visibility::Hidden) {
        return;
    }
    let samples = &stats.samples;
    let recent = &samples[samples.len().saturating_sub(BARS)..];
    let offset = BARS - recent.len();
    for (bar, mut node, mut colour) in &mut bars {
        let Some(&ms) = bar.0.checked_sub(offset).and_then(|i| recent.get(i)) else {
            node.height = px(0);
            *colour = BackgroundColor(Color::NONE);
            continue;
        };
        let ms = ms as f32;
        node.height = px(GRAPH_H * (ms / TOP_MS).min(1.0));
        *colour = BackgroundColor(if ms <= BUDGET_120 {
            Color::srgb(0.35, 0.85, 0.35)
        } else if ms <= BUDGET_60 {
            Color::srgb(0.95, 0.75, 0.25)
        } else {
            Color::srgb(0.95, 0.25, 0.20)
        });
    }
    if recent.is_empty() {
        return;
    }
    let mut sorted: Vec<f64> = recent.to_vec();
    sorted.sort_by(f64::total_cmp);
    let at = |p: f64| sorted[((sorted.len() - 1) as f64 * p) as usize];
    let total: f64 = recent.iter().sum();
    let line = format!(
        "{:.0} fps | p50 {:.1} | p99 {:.1} | max {:.1} ms | F3",
        recent.len() as f64 * 1000.0 / total.max(1e-6),
        at(0.5),
        at(0.99),
        sorted[sorted.len() - 1]
    );
    for mut text in &mut text {
        if text.0 != line {
            text.0 = line.clone();
        }
    }
}
