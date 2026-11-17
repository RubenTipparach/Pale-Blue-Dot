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
//!
//! Pressing `F3` again adds the breakdown under the graph: the GPU time of
//! each top-level render pass over the last second, sorted, and how much of the frame
//! the GPU was busy. It reads Bevy's render-diagnostic spans, the same ones
//! `--frame-log` writes per pass (`lighting-perf` 1.2). A third press hides
//! both.

use bevy::prelude::*;

use super::FrameStats;

/// Frames shown, and the graph's size in pixels.
const BARS: usize = 240;
const BAR_W: f32 = 0.5;
const GRAPH_H: f32 = 50.0;
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

#[derive(Component)]
pub struct FrameBreakdownText;

/// One batch of GPU spans: when it was read back, and each pass's ms.
type SpanBatch = (std::time::Instant, Vec<(String, f64)>);

/// Passes listed in the breakdown, the costliest first.
const BREAKDOWN_ROWS: usize = 12;

pub fn setup(mut commands: Commands, launch: Res<super::Launch>) {
    let width = BARS as f32 * BAR_W;
    commands
        .spawn((
            FrameGraph,
            Node {
                position_type: PositionType::Absolute,
                right: px(4),
                top: px(3),
                width: px(width + 4.0),
                padding: UiRect::all(px(2)),
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
                    font_size: 8.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.97, 0.95)),
                Node {
                    height: px(8),
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
            root.spawn((
                FrameBreakdownText,
                Text::new(""),
                TextFont {
                    font_size: 8.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.97, 0.95)),
                Node {
                    margin: UiRect::top(px(2)),
                    ..default()
                },
                Visibility::Hidden,
            ));
        });
}

/// `F3` steps through: the graph, the graph with the breakdown, nothing.
#[allow(clippy::type_complexity)]
pub fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut graphs: Query<&mut Visibility, (With<FrameGraph>, Without<FrameBreakdownText>)>,
    mut breakdowns: Query<&mut Visibility, (With<FrameBreakdownText>, Without<FrameGraph>)>,
    debug: Option<ResMut<pbd_app::controls::DebugOverlay>>,
) {
    if !keys.just_pressed(KeyCode::F3) {
        return;
    }
    let graph_shown = graphs.iter().any(|v| *v != Visibility::Hidden);
    let breakdown_shown = breakdowns.iter().any(|v| *v != Visibility::Hidden);
    let (graph, breakdown) = match (graph_shown, breakdown_shown) {
        (false, _) => (true, false),
        (true, false) => (true, true),
        (true, true) => (false, false),
    };
    // The boat instruments come and go with the graph.
    if let Some(mut debug) = debug {
        debug.0 = graph;
    }
    let shown = |on: bool| {
        if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    for mut visibility in &mut graphs {
        *visibility = shown(graph);
    }
    for mut visibility in &mut breakdowns {
        *visibility = shown(breakdown);
    }
}

/// The breakdown: each top-level GPU span averaged over the last second, the
/// costliest first, under the GPU's total and the frame's.
pub fn breakdown(
    stats: Res<FrameStats>,
    store: Option<Res<bevy::diagnostic::DiagnosticsStore>>,
    shown: Query<&Visibility, With<FrameBreakdownText>>,
    mut text: Query<&mut Text, With<FrameBreakdownText>>,
    mut batches: Local<std::collections::VecDeque<SpanBatch>>,
) {
    if shown.iter().all(|v| *v == Visibility::Hidden) {
        return;
    }
    let mut spans: Vec<(&str, f64, std::time::Instant)> = Vec::new();
    if let Some(store) = store.as_deref() {
        for d in store.iter() {
            let mut parts = d.path().as_str().split('/');
            let (Some("render"), Some(name), Some("elapsed_gpu"), None) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            if let Some(m) = d.measurement() {
                spans.push((name, m.value, m.time));
            }
        }
    }
    // Each new batch of spans is kept for a second; a pass that did not run
    // in a batch counts as nought in it.
    if let Some(newest) = spans.iter().map(|s| s.2).max()
        && batches.back().is_none_or(|b| b.0 != newest)
    {
        let batch = spans
            .iter()
            .filter(|s| s.2 == newest)
            .map(|s| (s.0.to_owned(), s.1))
            .collect();
        batches.push_back((newest, batch));
    }
    while batches
        .front()
        .is_some_and(|b| b.0.elapsed() > std::time::Duration::from_secs(1))
    {
        batches.pop_front();
    }
    let mut averaged: Vec<(&str, f64)> = Vec::new();
    for (_, batch) in batches.iter() {
        for (name, ms) in batch {
            match averaged.iter_mut().find(|row| row.0 == name) {
                Some(row) => row.1 += ms,
                None => averaged.push((name, *ms)),
            }
        }
    }
    for row in &mut averaged {
        row.1 /= batches.len() as f64;
    }
    averaged.sort_by(|a, b| b.1.total_cmp(&a.1));
    let smoothed = averaged;
    // The frame over the last second of frames.
    let (mut sum, mut count) = (0.0, 0usize);
    for ms in stats.samples.iter().rev() {
        if sum >= 1000.0 {
            break;
        }
        sum += ms;
        count += 1;
    }
    let frame = sum / count.max(1) as f64;
    let mut lines = String::new();
    if smoothed.is_empty() {
        lines.push_str("GPU times: none yet (not measured on this GPU)");
    } else {
        let gpu: f64 = smoothed.iter().map(|row| row.1).sum();
        lines.push_str(&format!(
            "frame {frame:.1} ms | GPU {gpu:.1} ms ({:.0}%) | {}",
            100.0 * gpu / frame.max(1e-6),
            if gpu > 0.85 * frame {
                "GPU bound"
            } else {
                "CPU or pacing bound"
            }
        ));
        for (name, ms) in smoothed.iter().take(BREAKDOWN_ROWS) {
            lines.push_str(&format!("\n{ms:5.2}  {name}"));
        }
        let rest: f64 = smoothed.iter().skip(BREAKDOWN_ROWS).map(|row| row.1).sum();
        if rest > 0.0 {
            lines.push_str(&format!(
                "\n{rest:5.2}  {} more",
                smoothed.len() - BREAKDOWN_ROWS
            ));
        }
    }
    for mut text in &mut text {
        if text.0 != lines {
            text.0 = lines.clone();
        }
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
