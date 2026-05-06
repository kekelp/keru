//! A DAW-like timeline view with horizontal scrolling implemented manually (no built-in scrollable areas).
//!
//! The timeline container clips its children and uses a scroll offset to shift all track content
//! horizontally. Scrolling is detected via [`Ui::is_scrolled()`] on the timeline area.
//!
//! The playhead uses [`Node::sense_time()`] to keep the loop ticking while playing.

use std::time::Instant;

use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

const TIMELINE_DURATION_SECS: f32 = 10000.0;

#[derive(Clone)]
struct Clip {
    start_secs: f32,
    duration_secs: f32,
    label: &'static str,
    color: Color,
}

struct Track {
    clips: Vec<Clip>,
}

pub struct State {
    tracks: Vec<Track>,
    scroll_offset_x: f32,
    pixels_per_second: f32,
    playhead_secs: f32,
    is_playing: bool,
    last_frame: Instant,
    // Ring buffer of recent frame timestamps for a 1-second sliding window average.
    frame_times: std::collections::VecDeque<Instant>,
    fps: f32,
}

impl Default for State {
    fn default() -> Self {
        let colors = [
            Color::KERU_BLUE, Color::KERU_GREEN, Color::KERU_PINK, Color::KERU_RED,
            Color::from_hex(0x5577aa), Color::from_hex(0x33aa66), Color::from_hex(0xaa5533),
            Color::from_hex(0x7755cc), Color::from_hex(0x228899), Color::from_hex(0xcc7722),
            Color::from_hex(0x559944), Color::from_hex(0xaa3366), Color::from_hex(0x336688),
            Color::from_hex(0x884422), Color::from_hex(0x667733), Color::from_hex(0x993355),
        ];

        // Generate tracks with densely packed short clips covering the full timeline.
        let num_tracks = 16;
        let tracks = (0..num_tracks).map(|t| {
            let clip_dur = 2.0 + (t % 4) as f32;   // 2–5 s clips
            let gap = 0.5 + (t % 3) as f32 * 0.25; // small gaps
            let step = clip_dur + gap;
            let color = colors[t % colors.len()];
            let mut clips = vec![];
            let mut start = (t % 3) as f32 * 0.7; // stagger start per track
            while start < TIMELINE_DURATION_SECS {
                clips.push(Clip { start_secs: start, duration_secs: clip_dur, label: "", color });
                start += step;
            }
            Track { clips }
        }).collect();

        Self {
            tracks,
            scroll_offset_x: 0.0,
            pixels_per_second: 80.0,
            playhead_secs: 0.0,
            is_playing: false,
            last_frame: Instant::now(),
            frame_times: std::collections::VecDeque::new(),
            fps: 0.0,
        }
    }
}

const TRACK_HEIGHT: f32 = 60.0;
const HEADER_HEIGHT: f32 = 40.0;
const CLIP_PADDING: f32 = 4.0;
const SCROLL_SPEED: f32 = 300.0;

const COLOR_RULER: Color = Color::new(0.12, 0.12, 0.15, 1.0);
const COLOR_TRACK_EVEN: Color = Color::new(0.10, 0.10, 0.13, 1.0);
const COLOR_TRACK_ODD: Color = Color::new(0.13, 0.13, 0.16, 1.0);

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const TIMELINE_AREA: NodeKey;
    #[node_key] const PLAYHEAD: NodeKey;
    #[node_key] const PLAY_BUTTON: NodeKey;
    #[node_key] const CLIP_CANVAS: NodeKey;

    // Compute delta time for playhead movement and fps.
    let now = Instant::now();
    let dt = now.duration_since(state.last_frame).as_secs_f32();
    state.last_frame = now;
    state.frame_times.push_back(now);
    while state.frame_times.front().map_or(false, |t| now.duration_since(*t).as_secs_f32() > 1.0) {
        state.frame_times.pop_front();
    }
    state.fps = state.frame_times.len() as f32;

    let pps = state.pixels_per_second;
    let scroll = state.scroll_offset_x;
    let timeline_content_width = TIMELINE_DURATION_SECS * pps;
    let max_scroll = (timeline_content_width - 400.0).max(0.0);
    // viewport_w is used as the right-edge culling threshold for elements positioned in
    // container-local coords. Using screen_w is conservative but always correct: the
    // container clips anything that actually falls outside it, so a few extra elements
    // rendered near the right edge cost nothing.
    let (screen_w, _) = ui.screen_size();
    let viewport_w = screen_w;

    let timeline_area = CONTAINER
        .key(TIMELINE_AREA)
        .size_x(Size::Fill)
        .size_y(Size::Fill)
        .clip_children(true)
        .sense_scroll(true);

    let marker_step_secs = 4.0f32;
    let marker_w = marker_step_secs * pps;

    let ruler_marker = BUTTON
        .size_x(Size::Pixels(marker_w - 2.0))
        .size_y(Size::Pixels(HEADER_HEIGHT))
        .position_y(Pos::Pixels(0.0))
        .color(COLOR_RULER);

    let track_bg_even = BUTTON
        .size_x(Size::Pixels(timeline_content_width))
        .size_y(Size::Pixels(TRACK_HEIGHT))
        .position_x(Pos::Pixels(-scroll))
        .color(COLOR_TRACK_EVEN);

    let track_bg_odd = track_bg_even.color(COLOR_TRACK_ODD);

    let clip_h = TRACK_HEIGHT - CLIP_PADDING * 2.0;
    let clip_node = BUTTON
        .size_y(Size::Pixels(clip_h));

    let fps_text = format!("{:.0} fps", state.fps);
    let fps_label = LABEL
        .position_x(Pos::End)
        .position_y(Pos::End)
        .z_index(2.0)
        .text(&fps_text);

    let playhead_x = -scroll + state.playhead_secs * pps;
    let total_height = HEADER_HEIGHT + state.tracks.len() as f32 * TRACK_HEIGHT;
    let playhead = PANEL
        .key(PLAYHEAD)
        .size_x(Size::Pixels(2.0))
        .size_y(Size::Pixels(total_height))
        .position_x(Pos::Pixels(playhead_x))
        .position_y(Pos::Pixels(0.0))
        .color(Color::WHITE)
        .z_index(1.0)
        .sense_time(state.is_playing);

    // UI tree.
    ui.h_stack().nest(|| {

        // Right column: ruler + track rows, clipped and shifted by scroll offset.
        ui.add(timeline_area).nest(|| {

            // Ruler.
            let mut t = 0.0f32;
            while t <= TIMELINE_DURATION_SECS {
                let x = -scroll + t * pps;
                if x + marker_w >= 0.0 && x <= viewport_w {
                    let label = format!("{:.0}s", t);
                    ui.add(ruler_marker.position_x(Pos::Pixels(x)).text(&label));
                }
                t += marker_step_secs;
            }

            // Track backgrounds and clips.
            for (track_i, track) in state.tracks.iter().enumerate() {
                let track_y = HEADER_HEIGHT + track_i as f32 * TRACK_HEIGHT;
                let track_bg = if track_i % 2 == 0 { track_bg_even } else { track_bg_odd };

                ui.add(track_bg.position_y(Pos::Pixels(track_y)));

                for (clip_i, clip) in track.clips.iter().enumerate() {
                    let clip_x = -scroll + clip.start_secs * pps + CLIP_PADDING;
                    let clip_w = clip.duration_secs * pps - CLIP_PADDING * 2.0;
                    if clip_x + clip_w < 0.0 || clip_x > viewport_w {
                        continue;
                    }
                    let canvas_key = CLIP_CANVAS.sibling(track_i).sibling(clip_i);

                    ui.add(
                        clip_node
                            .key(canvas_key)
                            .size_x(Size::Pixels(clip_w))
                            .position_x(Pos::Pixels(clip_x))
                            .position_y(Pos::Pixels(track_y + CLIP_PADDING))
                            .color(clip.color)
                            .text(clip.label)
                    );

                    let seed = track_i as f32 * 7.3 + clip_i as f32 * 3.1;
                    ui.canvas_drawing(canvas_key, |ctx| {
                        use keru_draw::{Segment, ColorFill};

                        let num_points = (clip_w * 2.0) as usize + 2;
                        let mid_y = clip_h / 2.0;
                        let envelope = |x: f32| {
                            let t = x / clip_w;
                            (t * std::f32::consts::PI).sin().powi(2)
                        };
                        let wave = |x: f32| {
                            let t = x / clip_w;
                            // High-frequency dense waveform with different character per clip.
                            0.45 * (t * (40.0 + seed % 20.0) * std::f32::consts::TAU).sin()
                            + 0.30 * (t * (73.0 + seed % 31.0) * std::f32::consts::TAU + seed).sin()
                            + 0.20 * (t * (120.0 + seed % 17.0) * std::f32::consts::TAU + seed * 1.7).sin()
                            + 0.05 * (t * (5.0 + seed % 3.0) * std::f32::consts::TAU + seed * 0.5).sin()
                        };

                        let points: Vec<[f32; 2]> = (0..num_points)
                            .map(|i| {
                                let x = i as f32 * clip_w / (num_points - 1) as f32;
                                let y = mid_y + mid_y * wave(x) * envelope(x);
                                [x, y]
                            })
                            .collect();

                        for i in 0..points.len() - 1 {
                            ctx.draw_segment(Segment {
                                start: points[i],
                                end: points[i + 1],
                                thickness: 1.5,
                                fill: ColorFill::Color(Color::WHITE),
                                dash_length: None,
                                dash_offset: 0.0,
                                blur: 0.0,
                                texture: None,
                                texture_options: None,
                            });
                        }
                    });
                }
            }

            ui.add(playhead);
            ui.add(fps_label);
        });
    });

    // Handle scroll on the timeline area: Ctrl+scroll zooms, plain scroll pans.
    if let Some(scroll_event) = ui.scrolled_at(TIMELINE_AREA) {
        if ui.key_mods().control_key() {
            // Zoom around the cursor position so the time under the cursor stays fixed.
            let cursor_x_in_area = scroll_event.relative_position.x * viewport_w;
            let time_at_cursor = (state.scroll_offset_x + cursor_x_in_area) / state.pixels_per_second;

            let zoom_factor = (1.0 + scroll_event.delta.y * 0.15).clamp(0.5, 2.0);
            state.pixels_per_second = (state.pixels_per_second * zoom_factor).clamp(10.0, 800.0);

            // Reanchor scroll so the time under the cursor stays under the cursor.
            state.scroll_offset_x = time_at_cursor * state.pixels_per_second - cursor_x_in_area;
        } else {
            // delta.x for trackpad horizontal scroll; delta.y for vertical scroll wheel (treated as horizontal).
            state.scroll_offset_x -= scroll_event.delta.x * SCROLL_SPEED;
            state.scroll_offset_x -= scroll_event.delta.y * SCROLL_SPEED;
        }
        state.scroll_offset_x = state.scroll_offset_x.clamp(0.0, max_scroll);
    }

    // Play/stop button.
    if ui.is_clicked(PLAY_BUTTON) {
        state.is_playing = !state.is_playing;
        state.last_frame = Instant::now();
    }

    // Advance playhead while playing.
    if state.is_playing {
        state.playhead_secs += dt;
        if state.playhead_secs > TIMELINE_DURATION_SECS {
            state.playhead_secs = 0.0;
        }
    }
}

fn main() {
    let state = State::default();
    run_example_loop(state, update_ui);
}
