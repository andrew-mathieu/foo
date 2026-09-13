use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::note::Note;
use crate::oscillator::Waveform;
use crate::synth::SynthEngine;

const BG: Color32 = Color32::from_rgb(7, 8, 13);
const PANEL: Color32 = Color32::from_rgb(13, 15, 22);
const PANEL_LIGHT: Color32 = Color32::from_rgb(18, 20, 29);

const TEXT: Color32 = Color32::from_rgb(226, 228, 235);
const TEXT_DIM: Color32 = Color32::from_rgb(105, 110, 124);

const ACCENT: Color32 = Color32::from_rgb(195, 115, 255);
const ACCENT_SOFT: Color32 = Color32::from_rgb(104, 75, 145);

const KEY_WHITE: Color32 = Color32::from_rgb(222, 223, 228);
const KEY_BLACK: Color32 = Color32::from_rgb(18, 19, 26);

pub struct SynthApp {
    synth: Arc<Mutex<SynthEngine>>,

    // Keyboard physique
    held_keys: HashSet<egui::Key>,

    // Note actuellement jouée avec la souris
    mouse_note: Option<u8>,

    // Waveform affichée dans l'interface
    waveform: Waveform,

    // Animation
    started_at: Instant,

    // Paramètres purement UI pour le moment
    octave: i32,
    velocity: f32,
}

impl SynthApp {
    pub fn new(synth: Arc<Mutex<SynthEngine>>) -> Self {
        Self {
            synth,
            held_keys: HashSet::new(),
            mouse_note: None,
            waveform: Waveform::Saw,
            started_at: Instant::now(),
            octave: 0,
            velocity: 0.8,
        }
    }

    fn note_on(&mut self, midi: u8) {
        let note = Note { midi };

        if let Ok(mut synth) = self.synth.lock() {
            synth.note_on(note);
        }
    }

    fn note_off(&mut self, midi: u8) {
        let note = Note { midi };

        if let Ok(mut synth) = self.synth.lock() {
            synth.note_off(note);
        }
    }

    fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;

        if let Ok(mut synth) = self.synth.lock() {
            synth.set_waveform(waveform);
        }
    }

    fn keyboard_note_is_active(&self, midi: u8) -> bool {
        self.held_keys
            .iter()
            .filter_map(|key| key_to_midi(*key))
            .any(|note| note == midi)
            || self.mouse_note == Some(midi)
    }

    fn active_notes(&self) -> Vec<u8> {
        let mut notes = Vec::new();

        for key in &self.held_keys {
            if let Some(midi) = key_to_midi(*key) {
                if !notes.contains(&midi) {
                    notes.push(midi);
                }
            }
        }

        if let Some(midi) = self.mouse_note {
            if !notes.contains(&midi) {
                notes.push(midi);
            }
        }

        notes
    }

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let events = ctx.input(|input| input.events.clone());

        for event in events {
            let egui::Event::Key {
                key,
                pressed,
                repeat,
                ..
            } = event
            else {
                continue;
            };

            let Some(midi) = key_to_midi(key) else {
                continue;
            };

            if pressed {
                if repeat {
                    continue;
                }

                if self.held_keys.insert(key) {
                    self.note_on(midi);
                }
            } else {
                if !self.held_keys.remove(&key) {
                    continue;
                }

                // Une même note MIDI peut être accessible depuis
                // plusieurs touches physiques.
                let still_held = self
                    .held_keys
                    .iter()
                    .filter_map(|k| key_to_midi(*k))
                    .any(|n| n == midi);

                if !still_held && self.mouse_note != Some(midi) {
                    self.note_off(midi);
                }
            }
        }
    }

    fn draw_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new("F O O")
                    .font(FontId::proportional(17.0))
                    .strong()
                    .color(TEXT),
            );

            ui.label(
                egui::RichText::new(" /  SYNTH")
                    .font(FontId::proportional(11.0))
                    .color(TEXT_DIM),
            );

            ui.add_space(20.0);

            ui.label(
                egui::RichText::new("POLYPHONIC")
                    .font(FontId::proportional(9.0))
                    .color(TEXT_DIM),
            );

            ui.add_space(6.0);

            ui.label(
                egui::RichText::new("● ONLINE")
                    .font(FontId::proportional(9.0))
                    .color(Color32::from_rgb(112, 230, 165)),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("48.0 kHz")
                        .font(FontId::monospace(10.0))
                        .color(TEXT_DIM),
                );

                ui.add_space(15.0);

                ui.label(
                    egui::RichText::new("RUST AUDIO ENGINE")
                        .font(FontId::monospace(9.0))
                        .color(TEXT_DIM),
                );
            });
        });
    }

    fn draw_hero(&self, ui: &mut egui::Ui) {
        let desired_size = Vec2::new(ui.available_width(), 300.0);

        let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());

        let painter = ui.painter_at(rect);

        let center = rect.center();

        let time = self.started_at.elapsed().as_secs_f32();

        let active = self.active_notes();
        let voice_count = active.len();

        // ------------------------------------------------------------
        // Background grid
        // ------------------------------------------------------------

        for i in 0..10 {
            let y = rect.top() + 20.0 + i as f32 * 28.0;

            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 5)),
            );
        }

        for i in 0..16 {
            let x = rect.left() + i as f32 * 90.0;

            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 4)),
            );
        }

        // ------------------------------------------------------------
        // Outer rings
        // ------------------------------------------------------------

        let pulse = if voice_count > 0 {
            1.0 + (time * 5.0).sin() * 0.04
        } else {
            1.0
        };

        let base_radius = 72.0 * pulse;

        for ring in 0..5 {
            let radius = base_radius + ring as f32 * 15.0;

            let rotation = time * (0.12 + ring as f32 * 0.035);

            let alpha = 35u8.saturating_sub(ring as u8 * 5);

            painter.circle_stroke(
                center,
                radius,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(195, 115, 255, alpha)),
            );

            // petits marqueurs orbitaux
            let marker_count = 16 + ring * 4;

            for i in 0..marker_count {
                let angle = rotation + i as f32 / marker_count as f32 * std::f32::consts::TAU;

                let inner = radius - 2.0;
                let outer = radius + if i % 4 == 0 { 7.0 } else { 3.0 };

                let p1 = Pos2::new(
                    center.x + angle.cos() * inner,
                    center.y + angle.sin() * inner,
                );

                let p2 = Pos2::new(
                    center.x + angle.cos() * outer,
                    center.y + angle.sin() * outer,
                );

                painter.line_segment(
                    [p1, p2],
                    Stroke::new(
                        if i % 4 == 0 { 1.5 } else { 0.7 },
                        Color32::from_rgba_unmultiplied(
                            195,
                            115,
                            255,
                            if i % 4 == 0 { 80 } else { 28 },
                        ),
                    ),
                );
            }
        }

        // ------------------------------------------------------------
        // Active note orbitals
        // ------------------------------------------------------------

        for (index, midi) in active.iter().enumerate() {
            let frequency = Note { midi: *midi }.frequency();

            let radius = 25.0 + ((*midi as f32 % 24.0) / 24.0) * 55.0;

            let speed = 0.15 + frequency / 2000.0;

            let angle = time * speed + index as f32 * 1.7;

            let point = Pos2::new(
                center.x + angle.cos() * radius,
                center.y + angle.sin() * radius,
            );

            painter.circle_filled(point, 3.0, Color32::from_rgb(220, 160, 255));

            painter.circle_stroke(
                point,
                7.0,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(195, 115, 255, 70)),
            );
        }

        // ------------------------------------------------------------
        // Central object
        // ------------------------------------------------------------

        let core_radius = if voice_count > 0 {
            25.0 + (time * 8.0).sin().abs() * 7.0 + voice_count as f32 * 1.5
        } else {
            24.0
        };

        painter.circle_filled(
            center,
            core_radius + 12.0,
            Color32::from_rgba_unmultiplied(195, 115, 255, 10),
        );

        painter.circle_filled(center, core_radius, Color32::from_rgb(11, 11, 18));

        painter.circle_stroke(
            center,
            core_radius,
            Stroke::new(1.5, Color32::from_rgba_unmultiplied(220, 165, 255, 180)),
        );

        // ------------------------------------------------------------
        // Center text
        // ------------------------------------------------------------

        let center_text = if voice_count == 0 { "READY" } else { "PLAYING" };

        painter.text(
            center,
            Align2::CENTER_CENTER,
            center_text,
            FontId::monospace(9.0),
            TEXT_DIM,
        );

        // ------------------------------------------------------------
        // Left metadata
        // ------------------------------------------------------------

        painter.text(
            Pos2::new(rect.left() + 20.0, rect.top() + 22.0),
            Align2::LEFT_TOP,
            "OSCILLATOR",
            FontId::monospace(9.0),
            TEXT_DIM,
        );

        painter.text(
            Pos2::new(rect.left() + 20.0, rect.top() + 40.0),
            Align2::LEFT_TOP,
            waveform_name(self.waveform),
            FontId::proportional(15.0),
            TEXT,
        );

        // ------------------------------------------------------------
        // Right metadata
        // ------------------------------------------------------------

        painter.text(
            Pos2::new(rect.right() - 20.0, rect.top() + 22.0),
            Align2::RIGHT_TOP,
            "VOICES",
            FontId::monospace(9.0),
            TEXT_DIM,
        );

        painter.text(
            Pos2::new(rect.right() - 20.0, rect.top() + 40.0),
            Align2::RIGHT_TOP,
            format!("{:02}", voice_count),
            FontId::monospace(18.0),
            TEXT,
        );

        // ------------------------------------------------------------
        // Bottom frequency readout
        // ------------------------------------------------------------

        if let Some(midi) = active.first() {
            let frequency = Note { midi: *midi }.frequency();

            painter.text(
                Pos2::new(center.x, rect.bottom() - 24.0),
                Align2::CENTER_CENTER,
                format!("{}   {} Hz", midi_to_name(*midi), frequency.round()),
                FontId::monospace(10.0),
                TEXT_DIM,
            );
        } else {
            painter.text(
                Pos2::new(center.x, rect.bottom() - 24.0),
                Align2::CENTER_CENTER,
                "NO SIGNAL",
                FontId::monospace(10.0),
                TEXT_DIM,
            );
        }
    }

    fn draw_waveform_panel(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            ui.horizontal(|ui| {
                section_title(ui, "OSCILLATOR");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("01")
                            .font(FontId::monospace(9.0))
                            .color(TEXT_DIM),
                    );
                });
            });

            ui.add_space(14.0);

            ui.horizontal(|ui| {
                for (waveform, label) in [
                    (Waveform::Sine, "SINE"),
                    (Waveform::Saw, "SAW"),
                    (Waveform::Square, "SQUARE"),
                    (Waveform::Triangle, "TRI"),
                ] {
                    let selected = same_waveform(self.waveform, waveform);

                    let button = egui::Button::new(
                        egui::RichText::new(label)
                            .font(FontId::monospace(9.0))
                            .color(if selected { TEXT } else { TEXT_DIM }),
                    )
                    .fill(if selected {
                        Color32::from_rgb(38, 27, 48)
                    } else {
                        Color32::from_rgb(17, 18, 25)
                    })
                    .stroke(Stroke::new(
                        1.0,
                        if selected {
                            ACCENT
                        } else {
                            Color32::from_rgb(35, 37, 46)
                        },
                    ))
                    .corner_radius(6.0);

                    if ui.add(button).clicked() {
                        self.set_waveform(waveform);
                    }
                }
            });

            ui.add_space(18.0);

            draw_waveform_preview(ui, self.waveform);
        });
    }

    fn draw_macro_panel(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            section_title(ui, "CHARACTER");

            ui.add_space(15.0);

            ui.horizontal(|ui| {
                knob(ui, "OCTAVE", self.octave as f32, -2.0, 2.0, |value| {
                    self.octave = value.round() as i32;
                });

                knob(ui, "VELOCITY", self.velocity, 0.0, 1.0, |value| {
                    self.velocity = value;
                });

                knob(ui, "SPACE", 0.32, 0.0, 1.0, |_| {});
            });
        });
    }

    fn draw_keyboard(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            ui.horizontal(|ui| {
                section_title(ui, "KEYBOARD");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("AZERTY / C MAJOR")
                            .font(FontId::monospace(9.0))
                            .color(TEXT_DIM),
                    );
                });
            });

            ui.add_space(10.0);

            draw_piano(ui, self);
        });
    }
}

impl eframe::App for SynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);

        // Animation permanente du visuel central.
        ctx.request_repaint();

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(BG).inner_margin(20.0))
            .show(ctx, |ui| {
                self.draw_header(ui);

                ui.add_space(14.0);

                self.draw_hero(ui);

                ui.add_space(10.0);

                ui.columns(2, |columns| {
                    self.draw_waveform_panel(&mut columns[0]);

                    self.draw_macro_panel(&mut columns[1]);
                });

                ui.add_space(10.0);

                self.draw_keyboard(ui);

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("W X C V B N")
                            .font(FontId::monospace(8.0))
                            .color(TEXT_DIM),
                    );

                    ui.add_space(10.0);

                    ui.label(
                        egui::RichText::new("LOW")
                            .font(FontId::monospace(8.0))
                            .color(TEXT_DIM),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("FOO SYNTH  /  0.1.0")
                                .font(FontId::monospace(8.0))
                                .color(TEXT_DIM),
                        );
                    });
                });
            });
    }
}

// ============================================================================
// PANELS
// ============================================================================

fn panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::NONE
        .fill(PANEL)
        .stroke(Stroke::new(1.0, Color32::from_rgb(28, 30, 39)))
        .corner_radius(10.0)
        .inner_margin(16.0)
        .show(ui, |ui| {
            add_contents(ui);
        });
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .font(FontId::monospace(10.0))
            .strong()
            .color(TEXT),
    );
}

// ============================================================================
// KNOB
// ============================================================================

fn knob(
    ui: &mut egui::Ui,
    label: &str,
    value: f32,
    min: f32,
    max: f32,
    mut on_change: impl FnMut(f32),
) {
    let size = Vec2::new(90.0, 100.0);

    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

    let painter = ui.painter_at(rect);

    let center = Pos2::new(rect.center().x, rect.top() + 38.0);

    let radius = 24.0;

    let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0);

    let start_angle = std::f32::consts::PI * 0.75;

    let end_angle = std::f32::consts::PI * 2.25;

    let value_angle = start_angle + normalized * (end_angle - start_angle);

    if response.dragged() {
        let delta = -response.drag_delta().y * 0.008;

        let new_value = (value + delta * (max - min)).clamp(min, max);

        on_change(new_value);
    }

    painter.circle_filled(center, radius, Color32::from_rgb(10, 11, 16));

    painter.circle_stroke(
        center,
        radius,
        Stroke::new(1.0, Color32::from_rgb(42, 44, 53)),
    );

    let segments = 32;

    for i in 0..segments {
        let t = i as f32 / segments as f32;

        let angle = start_angle + t * (end_angle - start_angle);

        let inner = radius + 4.0;
        let outer = radius + if i % 4 == 0 { 8.0 } else { 5.0 };

        let p1 = Pos2::new(
            center.x + angle.cos() * inner,
            center.y + angle.sin() * inner,
        );

        let p2 = Pos2::new(
            center.x + angle.cos() * outer,
            center.y + angle.sin() * outer,
        );

        painter.line_segment(
            [p1, p2],
            Stroke::new(
                if t <= normalized { 1.5 } else { 1.0 },
                if t <= normalized {
                    ACCENT
                } else {
                    Color32::from_rgb(47, 49, 58)
                },
            ),
        );
    }

    let indicator_start = Pos2::new(
        center.x + value_angle.cos() * 14.0,
        center.y + value_angle.sin() * 14.0,
    );

    let indicator_end = Pos2::new(
        center.x + value_angle.cos() * 20.0,
        center.y + value_angle.sin() * 20.0,
    );

    painter.line_segment([indicator_start, indicator_end], Stroke::new(2.0, TEXT));

    painter.text(
        Pos2::new(rect.center().x, rect.bottom() - 23.0),
        Align2::CENTER_CENTER,
        label,
        FontId::monospace(8.0),
        TEXT_DIM,
    );

    painter.text(
        Pos2::new(rect.center().x, rect.bottom() - 8.0),
        Align2::CENTER_CENTER,
        format!("{:.2}", value),
        FontId::monospace(8.0),
        TEXT,
    );
}

// ============================================================================
// WAVEFORM
// ============================================================================

fn draw_waveform_preview(ui: &mut egui::Ui, waveform: Waveform) {
    let desired = Vec2::new(ui.available_width(), 80.0);

    let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());

    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 6.0, Color32::from_rgb(9, 10, 15));

    let points = 160;

    let mut previous = Pos2::ZERO;

    for i in 0..points {
        let t = i as f32 / (points - 1) as f32;

        let phase = t * std::f32::consts::TAU * 2.0;

        let value = match waveform {
            Waveform::Sine => phase.sin(),

            Waveform::Saw => 2.0 * (t * 2.0).fract() - 1.0,

            Waveform::Square => {
                if phase.sin() >= 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }

            Waveform::Triangle => 2.0 * (2.0 * (t * 2.0).fract() - 1.0).abs() - 1.0,
        };

        let x = rect.left() + t * rect.width();

        let y = rect.center().y - value * rect.height() * 0.32;

        let current = Pos2::new(x, y);

        if i > 0 {
            painter.line_segment([previous, current], Stroke::new(1.5, ACCENT));
        }

        previous = current;
    }

    painter.line_segment(
        [
            Pos2::new(rect.left(), rect.center().y),
            Pos2::new(rect.right(), rect.center().y),
        ],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 12)),
    );
}

// ============================================================================
// PIANO
// ============================================================================

fn draw_piano(ui: &mut egui::Ui, app: &mut SynthApp) {
    let white_midi = [
        48, 50, 52, 53, 55, 57, 59, 60, 62, 64, 65, 67, 69, 71, 72, 74, 76, 77, 79, 81, 83, 84,
    ];

    let shortcuts = [
        "W", "X", "C", "V", "B", "N", "", "Q", "S", "D", "F", "G", "H", "J", "A", "Z", "E", "R",
        "T", "Y", "U", "I",
    ];

    let keyboard_height = 145.0;

    let available_width = ui.available_width();

    let white_width = available_width / white_midi.len() as f32;

    let white_rect = ui
        .allocate_exact_size(Vec2::new(available_width, keyboard_height), Sense::hover())
        .0;

    let painter = ui.painter_at(white_rect);

    let primary_down = ui.input(|i| i.pointer.primary_down());

    let pointer_pos = ui.input(|i| i.pointer.interact_pos());

    let mut hovered_note: Option<u8> = None;

    // ------------------------------------------------------------
    // WHITE KEYS
    // ------------------------------------------------------------

    for (index, midi) in white_midi.iter().enumerate() {
        let left = white_rect.left() + index as f32 * white_width;

        let right = left + white_width - 2.0;

        let key_rect = Rect::from_min_max(
            Pos2::new(left, white_rect.top()),
            Pos2::new(right, white_rect.bottom()),
        );

        let id = ui.make_persistent_id(("white-key", *midi));

        let response = ui.interact(key_rect, id, Sense::click_and_drag());

        let active = app.keyboard_note_is_active(*midi);

        let hovered = response.hovered();

        if hovered {
            hovered_note = Some(*midi);
        }

        let fill = if active {
            Color32::from_rgb(180, 105, 235)
        } else if hovered {
            Color32::from_rgb(240, 240, 246)
        } else {
            KEY_WHITE
        };

        painter.rect_filled(key_rect, 4.0, fill);

        painter.rect_stroke(
            key_rect,
            4.0,
            Stroke::new(
                1.0,
                if active {
                    ACCENT
                } else {
                    Color32::from_rgb(70, 71, 80)
                },
            ),
            StrokeKind::Inside,
        );

        let shortcut = shortcuts[index];

        if !shortcut.is_empty() {
            painter.text(
                Pos2::new(key_rect.center().x, key_rect.bottom() - 20.0),
                Align2::CENTER_CENTER,
                shortcut,
                FontId::monospace(9.0),
                if active {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(70, 71, 78)
                },
            );
        }

        painter.text(
            Pos2::new(key_rect.center().x, key_rect.bottom() - 6.0),
            Align2::CENTER_CENTER,
            midi_to_name(*midi),
            FontId::monospace(7.0),
            if active {
                Color32::from_rgba_unmultiplied(255, 255, 255, 180)
            } else {
                Color32::from_rgb(145, 146, 154)
            },
        );
    }

    // ------------------------------------------------------------
    // BLACK KEYS
    // ------------------------------------------------------------

    let black_offsets = [1, 3, 6, 8, 10, 13, 15, 18, 20];

    let black_width = white_width * 0.58;

    let black_height = keyboard_height * 0.62;

    for &white_index in &black_offsets {
        let midi = white_midi[white_index - 1] + 1;

        let x = white_rect.left() + white_index as f32 * white_width - black_width / 2.0;

        let key_rect = Rect::from_min_max(
            Pos2::new(x, white_rect.top()),
            Pos2::new(x + black_width, white_rect.top() + black_height),
        );

        let id = ui.make_persistent_id(("black-key", midi));

        let response = ui.interact(key_rect, id, Sense::click_and_drag());

        let active = app.keyboard_note_is_active(midi);

        if response.hovered() {
            hovered_note = Some(midi);
        }

        let fill = if active {
            Color32::from_rgb(125, 66, 168)
        } else if response.hovered() {
            Color32::from_rgb(50, 43, 58)
        } else {
            KEY_BLACK
        };

        painter.rect_filled(key_rect, 3.0, fill);

        painter.rect_stroke(
            key_rect,
            3.0,
            Stroke::new(
                1.0,
                if active {
                    ACCENT
                } else {
                    Color32::from_rgb(55, 57, 67)
                },
            ),
            StrokeKind::Inside,
        );
    }

    // ------------------------------------------------------------
    // MOUSE INTERACTION
    // ------------------------------------------------------------

    if primary_down {
        if let Some(note) = hovered_note {
            if app.mouse_note != Some(note) {
                if let Some(previous) = app.mouse_note.take() {
                    let keyboard_still_holds = app.keyboard_note_is_active(previous);

                    if !keyboard_still_holds {
                        app.note_off(previous);
                    }
                }

                app.mouse_note = Some(note);
                app.note_on(note);
            }
        }
    } else if let Some(previous) = app.mouse_note.take() {
        let keyboard_still_holds = app.keyboard_note_is_active(previous);

        if !keyboard_still_holds {
            app.note_off(previous);
        }
    }

    // Petite ligne lumineuse sous le clavier.
    let glow_y = white_rect.bottom() + 4.0;

    painter.line_segment(
        [
            Pos2::new(white_rect.left(), glow_y),
            Pos2::new(white_rect.right(), glow_y),
        ],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(195, 115, 255, 45)),
    );

    // Affichage du pointeur / note.
    if let Some(pos) = pointer_pos {
        if white_rect.contains(pos) {
            if let Some(note) = hovered_note {
                painter.text(
                    Pos2::new(pos.x, white_rect.top() - 8.0),
                    Align2::CENTER_BOTTOM,
                    midi_to_name(note),
                    FontId::monospace(8.0),
                    ACCENT,
                );
            }
        }
    }
}

// ============================================================================
// KEYBOARD MAPPING
// ============================================================================

fn major_scale_degree(degree: usize) -> u8 {
    match degree {
        0 => 0,
        1 => 2,
        2 => 4,
        3 => 5,
        4 => 7,
        5 => 9,
        6 => 11,
        _ => unreachable!(),
    }
}

fn scale_note(root_midi: u8, octave: i8, degree: usize) -> u8 {
    let octave_offset = octave as i16 * 12;

    let degree_octave = degree / 7;

    let degree_in_octave = degree % 7;

    let midi = root_midi as i16
        + octave_offset
        + degree_octave as i16 * 12
        + major_scale_degree(degree_in_octave) as i16;

    midi as u8
}

fn key_to_midi(key: egui::Key) -> Option<u8> {
    match key {
        // ========================================================
        // RANGÉE HAUTE
        // ========================================================
        egui::Key::A => Some(scale_note(12, 5, 0)),

        egui::Key::Z => Some(scale_note(12, 5, 1)),

        egui::Key::E => Some(scale_note(12, 5, 2)),

        egui::Key::R => Some(scale_note(12, 5, 3)),

        egui::Key::T => Some(scale_note(12, 5, 4)),

        egui::Key::Y => Some(scale_note(12, 5, 5)),

        egui::Key::U => Some(scale_note(12, 5, 6)),

        egui::Key::I => Some(scale_note(12, 5, 7)),

        egui::Key::O => Some(scale_note(12, 5, 8)),

        egui::Key::P => Some(scale_note(12, 5, 9)),

        // ========================================================
        // RANGÉE MILIEU
        // ========================================================
        egui::Key::Q => Some(scale_note(12, 4, 0)),

        egui::Key::S => Some(scale_note(12, 4, 1)),

        egui::Key::D => Some(scale_note(12, 4, 2)),

        egui::Key::F => Some(scale_note(12, 4, 3)),

        egui::Key::G => Some(scale_note(12, 4, 4)),

        egui::Key::H => Some(scale_note(12, 4, 5)),

        egui::Key::J => Some(scale_note(12, 4, 6)),

        egui::Key::K => Some(scale_note(12, 4, 7)),

        egui::Key::L => Some(scale_note(12, 4, 8)),

        egui::Key::M => Some(scale_note(12, 4, 9)),

        // ========================================================
        // RANGÉE BASSE
        // ========================================================
        egui::Key::W => Some(scale_note(12, 3, 0)),

        egui::Key::X => Some(scale_note(12, 3, 1)),

        egui::Key::C => Some(scale_note(12, 3, 2)),

        egui::Key::V => Some(scale_note(12, 3, 3)),

        egui::Key::B => Some(scale_note(12, 3, 4)),

        egui::Key::N => Some(scale_note(12, 3, 5)),

        _ => None,
    }
}

// ============================================================================
// UTILS
// ============================================================================

fn midi_to_name(midi: u8) -> String {
    let names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    let octave = midi as i32 / 12 - 1;

    format!("{}{}", names[(midi % 12) as usize], octave)
}

fn waveform_name(waveform: Waveform) -> &'static str {
    match waveform {
        Waveform::Sine => "SINE",
        Waveform::Saw => "SAW",
        Waveform::Square => "SQUARE",
        Waveform::Triangle => "TRIANGLE",
    }
}

fn same_waveform(a: Waveform, b: Waveform) -> bool {
    matches!(
        (a, b),
        (Waveform::Sine, Waveform::Sine)
            | (Waveform::Saw, Waveform::Saw)
            | (Waveform::Square, Waveform::Square)
            | (Waveform::Triangle, Waveform::Triangle)
    )
}
