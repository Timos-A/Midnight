use eframe::egui::{self, Color32, Key, Sense, Stroke, Vec2};

const NOTE_COUNT: usize = 24;

const KEYBOARD_NOTES: [Key; NOTE_COUNT] = [
    Key::Z,
    Key::S,
    Key::X,
    Key::D,
    Key::C,
    Key::V,
    Key::G,
    Key::B,
    Key::H,
    Key::N,
    Key::J,
    Key::M,
    Key::Q,
    Key::Num2,
    Key::W,
    Key::Num3,
    Key::E,
    Key::R,
    Key::Num5,
    Key::T,
    Key::Num6,
    Key::Y,
    Key::Num7,
    Key::U,
];

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

#[derive(Default)]
pub struct Piano {
    keyboard_notes: [bool; NOTE_COUNT],
}

impl Piano {
    /// Draws the piano and returns the notes currently held by mouse or keyboard.
    pub fn ui(&mut self, ui: &mut egui::Ui, octave_shift: i8, note_selector_focused: bool) -> [bool; NOTE_COUNT] {
        if note_selector_focused { self.keyboard_notes = [false; NOTE_COUNT]; }
        else { self.update_keyboard_state(ui); }

        if self.keyboard_notes.iter().any(|&pressed| pressed) {
            ui.ctx().request_repaint();
        }

        let keyboard_notes = self.keyboard_notes;
        let mouse_notes = self.draw_keys(ui, &keyboard_notes, octave_shift);
        std::array::from_fn(|index| mouse_notes[index] || keyboard_notes[index])
    }

    fn update_keyboard_state(&mut self, ui: &egui::Ui) {
        let events = ui.input(|input| input.events.clone());
        for event in events {
            match event {
                egui::Event::Key {
                    key,
                    physical_key,
                    pressed,
                    ..
                } => {
                    let note_index = physical_key
                        .and_then(note_index_for_key)
                        .or_else(|| note_index_for_key(key));
                    if let Some(note_index) = note_index {
                        self.keyboard_notes[note_index] = pressed;
                    }
                }
                egui::Event::WindowFocused(false) => self.keyboard_notes = [false; NOTE_COUNT],
                _ => {}
            }
        }
    }

    fn draw_keys(
        &self,
        ui: &mut egui::Ui,
        keyboard_notes: &[bool; NOTE_COUNT],
        octave_shift: i8,
    ) -> [bool; NOTE_COUNT] {
        let mut mouse_notes = [false; NOTE_COUNT];
        let width = ui.available_width().max(350.0);
        let white_key_width = width / 14.0;
        let height = 200.0;
        let (area, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let painter = ui.painter_at(area);

        let mut white_index = 0;
        for note in 0..NOTE_COUNT {
            if is_black_key(note) {
                continue;
            }

            let rect = egui::Rect::from_min_size(
                area.min + Vec2::new(white_index as f32 * white_key_width, 0.0),
                Vec2::new(white_key_width, height),
            );
            let response = ui.interact(
                rect,
                ui.make_persistent_id(("piano-key", note)),
                Sense::click_and_drag(),
            );
            mouse_notes[note] = response.is_pointer_button_down_on();
            let is_pressed = mouse_notes[note] || keyboard_notes[note];

            painter.rect_filled(
                rect,
                2.0,
                if is_pressed {
                    Color32::from_rgb(40, 105, 210)
                } else {
                    Color32::WHITE
                },
            );
            painter.rect_stroke(
                rect,
                2.0,
                Stroke::new(1.0, Color32::DARK_GRAY),
                egui::StrokeKind::Inside,
            );
            painter.text(
                rect.center_bottom() - Vec2::new(0.0, 18.0),
                egui::Align2::CENTER_CENTER,
                format!(
                    "{}{}\n{}",
                    NOTE_NAMES[note % 12],
                    (4_i8 + octave_shift + (note / 12) as i8),
                    keyboard_label(note)
                ),
                egui::FontId::proportional(12.0),
                if is_pressed {
                    Color32::WHITE
                } else {
                    Color32::DARK_GRAY
                },
            );
            white_index += 1;
        }

        // Black keys sit on top of the white keys, like a physical piano.
        let mut preceding_white_keys = 0;
        for note in 0..NOTE_COUNT {
            if !is_black_key(note) {
                preceding_white_keys += 1;
                continue;
            }

            let rect = egui::Rect::from_min_size(
                area.min
                    + Vec2::new(
                        preceding_white_keys as f32 * white_key_width - white_key_width * 0.32,
                        0.0,
                    ),
                Vec2::new(white_key_width * 0.64, height * 0.62),
            );
            let response = ui.interact(
                rect,
                ui.make_persistent_id(("piano-key", note)),
                Sense::click_and_drag(),
            );
            mouse_notes[note] = response.is_pointer_button_down_on();
            let is_pressed = mouse_notes[note] || keyboard_notes[note];
            painter.rect_filled(
                rect,
                2.0,
                if is_pressed {
                    Color32::from_rgb(40, 105, 210)
                } else {
                    Color32::from_rgb(30, 30, 34)
                },
            );
            painter.rect_stroke(
                rect,
                2.0,
                Stroke::new(1.0, Color32::BLACK),
                egui::StrokeKind::Inside,
            );
            painter.text(
                rect.center_bottom() - Vec2::new(0.0, 14.0),
                egui::Align2::CENTER_CENTER,
                keyboard_label(note),
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        }

        mouse_notes
    }
}

fn is_black_key(note: usize) -> bool {
    matches!(note % 12, 1 | 3 | 6 | 8 | 10)
}

fn note_index_for_key(key: Key) -> Option<usize> {
    KEYBOARD_NOTES
        .iter()
        .position(|&mapped_key| mapped_key == key)
}

fn keyboard_label(note: usize) -> &'static str {
    match KEYBOARD_NOTES[note] {
        Key::Z => "Z",
        Key::S => "S",
        Key::X => "X",
        Key::D => "D",
        Key::C => "C",
        Key::V => "V",
        Key::G => "G",
        Key::B => "B",
        Key::H => "H",
        Key::N => "N",
        Key::J => "J",
        Key::M => "M",
        Key::Q => "Q",
        Key::Num2 => "2",
        Key::W => "W",
        Key::Num3 => "3",
        Key::E => "E",
        Key::R => "R",
        Key::Num5 => "5",
        Key::T => "T",
        Key::Num6 => "6",
        Key::Y => "Y",
        Key::Num7 => "7",
        Key::U => "U",
        _ => "",
    }
}
