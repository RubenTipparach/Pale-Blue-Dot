//! The frame graph: an INSTRUMENT, like the near-field line (`hud.rs`), at the
//! owner's request while the frame spikes are chased. Off by default; `F3`
//! shows and hides it, and `--frame-graph` starts with it shown.
//!
//! It draws the last `BARS` frames' wall times as bars, top left over every
//! panel, on a log scale (4 ms to 1 s): green within the 120 Hz budget, amber
//! within 60 Hz, orange within 30 Hz, red to 100 ms and white past it (a
//! stall), against lines at 60 Hz, 30 Hz and 100 ms, with the window's frame rate, median, 99th percentile and
//! worst frame above. The numbers are `FrameStats`, the same samples the
//! capture's `FRAME_WALL_MS` and `--frame-log` report, so the graph and the
//! logs cannot disagree.
//!
//! Pressing `F3` again adds the breakdown under the graph: the GPU time of
//! each top-level render pass over the last second, sorted, and how much of the frame
//! the GPU was busy. It reads Bevy's render-diagnostic spans, the same ones
//! `--frame-log` writes per pass (`lighting-perf` 1.2). A third press hides
//! both.
//!
//! locale: dev

use bevy::prelude::*;

use super::FrameStats;

/// Frames shown, and the graph's size in pixels.
const BARS: usize = 240;
const BAR_W: f32 = 0.5;
const GRAPH_H: f32 = 50.0;
/// The graph's height is a log scale from `FOOT_MS` to `TOP_MS`, so a slow
/// run (33 ms) and a stall (1 s) are far apart (`frame-budget`, "The F3
/// graph shows stalls apart from slow frames").
const FOOT_MS: f32 = 4.0;
const TOP_MS: f32 = 1000.0;
const BUDGET_120: f32 = 1000.0 / 120.0;
const BUDGET_60: f32 = 1000.0 / 60.0;
const BUDGET_30: f32 = 1000.0 / 30.0;
/// Past this a frame is a stall, drawn white.
const STALL_MS: f32 = 100.0;

/// A frame time's height on the graph, px.
fn bar_height(ms: f32) -> f32 {
    let t = (ms.max(FOOT_MS) / FOOT_MS).ln() / (TOP_MS / FOOT_MS).ln();
    GRAPH_H * t.clamp(0.02, 1.0)
}

/// A frame time's colour: green, amber, orange, red, and white for a stall.
fn bar_colour(ms: f32) -> Color {
    if ms <= BUDGET_120 {
        Color::srgb(0.35, 0.85, 0.35)
    } else if ms <= BUDGET_60 {
        Color::srgb(0.95, 0.80, 0.25)
    } else if ms <= BUDGET_30 {
        Color::srgb(0.98, 0.55, 0.15)
    } else if ms <= STALL_MS {
        Color::srgb(0.90, 0.20, 0.18)
    } else {
        Color::srgb(1.0, 1.0, 1.0)
    }
}

#[derive(Component)]
pub struct FrameGraph;

/// The bars, drawn into one image (`memory-leak`, towns at 60 fps: 240 bar
/// nodes resized each frame laid the whole UI out again every frame).
#[derive(Component)]
pub struct FrameGraphBars(Handle<Image>);

/// The bars' image: two texels a logical pixel, so a half-pixel bar is a
/// column of texels.
const TEXELS_PER_PX: f32 = 2.0;

#[derive(Component)]
pub struct FrameGraphText;

#[derive(Component)]
pub struct FrameBreakdownText;

/// One batch of GPU spans: when it was read back, and each pass's ms.
type SpanBatch = (std::time::Instant, Vec<(String, f64)>);

/// Passes listed in the breakdown, the costliest first.
const BREAKDOWN_ROWS: usize = 12;

pub fn setup(
    mut commands: Commands,
    launch: Res<super::Launch>,
    mut images: ResMut<Assets<Image>>,
) {
    let width = BARS as f32 * BAR_W;
    let mut image = Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width: (width * TEXELS_PER_PX) as u32,
            height: (GRAPH_H * TEXELS_PER_PX) as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &[0, 0, 0, 0],
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    let bars = images.add(image);
    commands
        .spawn((
            FrameGraph,
            // Top left and over every panel (owner, 2026-11-22: "its being
            // blocked by stuff", the missions panel over it top right).
            GlobalZIndex(1000),
            Node {
                position_type: PositionType::Absolute,
                left: px(4),
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
                plot.spawn((
                    FrameGraphBars(bars.clone()),
                    ImageNode::new(bars),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        bottom: px(0),
                        width: px(width),
                        height: px(GRAPH_H),
                        ..default()
                    },
                ));
                for budget in [BUDGET_60, BUDGET_30, STALL_MS] {
                    plot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            bottom: px(bar_height(budget)),
                            width: px(width),
                            height: px(1),
                            ..default()
                        },
                        BackgroundColor(bar_colour(budget).with_alpha(0.5)),
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
    bars: Query<&FrameGraphBars>,
    mut images: ResMut<Assets<Image>>,
    mut text: Query<&mut Text, With<FrameGraphText>>,
) {
    if graphs.iter().all(|v| *v == Visibility::Hidden) {
        return;
    }
    let samples = &stats.samples;
    let recent = &samples[samples.len().saturating_sub(BARS)..];
    let offset = BARS - recent.len();
    for bars in &bars {
        let Some(image) = images.get_mut(&bars.0) else {
            continue;
        };
        let (w, h) = (image.width() as usize, image.height() as usize);
        let Some(data) = image.data.as_mut() else {
            continue;
        };
        data.fill(0);
        let per = (w / BARS).max(1);
        for bar in 0..BARS {
            let Some(&ms) = bar.checked_sub(offset).and_then(|i| recent.get(i)) else {
                continue;
            };
            let ms = ms as f32;
            let tall = ((bar_height(ms) * TEXELS_PER_PX).round() as usize).min(h);
            let [r, g, b, a] = bar_colour(ms).to_srgba().to_u8_array();
            for x in bar * per..((bar + 1) * per).min(w) {
                for y in h - tall..h {
                    let i = (y * w + x) * 4;
                    data[i..i + 4].copy_from_slice(&[r, g, b, a]);
                }
            }
        }
    }
    if recent.is_empty() {
        return;
    }
    let mut sorted: Vec<f64> = recent.to_vec();
    sorted.sort_by(f64::total_cmp);
    let at = |p: f64| sorted[((sorted.len() - 1) as f64 * p) as usize];
    let total: f64 = recent.iter().sum();
    let stalls = recent
        .iter()
        .filter(|&&ms| ms > f64::from(STALL_MS))
        .count();
    let line = format!(
        "{:.0} fps | p50 {:.1} | p99 {:.1} | max {:.1} ms | {} over {} ms | F3",
        recent.len() as f64 * 1000.0 / total.max(1e-6),
        at(0.5),
        at(0.99),
        sorted[sorted.len() - 1],
        stalls,
        STALL_MS
    );
    for mut text in &mut text {
        if text.0 != line {
            text.0 = line.clone();
        }
    }
}
