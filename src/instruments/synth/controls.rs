use eframe::egui::{self, Context, Id, Slider, Ui};

use crate::instruments::synth::engine::{MAX_OSCILLATORS, NoteParameters, SynthSettings, Waveform};
use crate::ui::knob::{self, Scale as KnobScale};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Oscillator,
    Envelope,
    Filter,
    Output,
}

pub struct SynthControls {
    settings: SynthSettings,
    selected_tab: Tab,
    confirm_more_oscillators: bool,
    allow_more_oscillators: bool,
    last_note: usize,
    note_query: String,
    note_selector_focused: bool,
    global_parameters: Option<NoteParameters>,
}

impl Default for SynthControls {
    fn default() -> Self {
        Self {
            settings: SynthSettings::default(),
            selected_tab: Tab::Oscillator,
            confirm_more_oscillators: false,
            allow_more_oscillators: false,
            last_note: 0,
            note_query: String::new(),
            note_selector_focused: false,
            global_parameters: None,
        }
    }
}

impl SynthControls {
    pub fn settings(&self) -> SynthSettings {
        self.settings
    }

    pub fn set_settings(&mut self, settings: SynthSettings) {
        self.settings = settings;
        self.global_parameters = Some(NoteParameters::from_settings(&settings));
        self.last_note = settings.selected_note;
        self.note_query.clear();
        self.note_selector_focused = false;
    }

    pub fn note_selector_focused(&self) -> bool {
        self.note_selector_focused
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        if self.global_parameters.is_none() {
            self.global_parameters = Some(NoteParameters::from_settings(&self.settings));
        }
        ui.heading("Synth controls");
        ui.label("Adjust the sound while playing; changes apply immediately.");
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.settings.per_note_enabled,
                "Per-note parameter locks",
            );
            ui.add_enabled_ui(self.settings.per_note_enabled, |ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.note_query)
                        .hint_text(if self.settings.selected_note < 24 {
                            format!("Select note: {}", note_name(self.settings.selected_note))
                        } else {
                            "All notes".to_owned()
                        })
                        .desired_width(110.0),
                );
                if ui.small_button("All notes").clicked() {
                    self.settings.selected_note = 24;
                    self.note_query.clear();
                }
                self.note_selector_focused = response.has_focus();
                if response.has_focus() {
                    let events = ui.input(|input| input.events.clone());
                    if let Some(note) = events.iter().find_map(key_note) {
                        self.settings.selected_note = note;
                        self.note_query = note_name(note);
                    }
                } else {
                    self.note_selector_focused = false;
                }
                if self.settings.per_note_enabled && !self.note_query.is_empty() {
                    let query = self.note_query.to_lowercase();
                    ui.horizontal_wrapped(|ui| {
                        for note in 0..24 {
                            let label = note_name(note);
                            if label.to_lowercase().contains(&query)
                                && ui.small_button(&label).clicked()
                            {
                                self.settings.selected_note = note;
                                self.note_query.clear();
                            }
                        }
                    });
                }
            });
        });
        if self.settings.per_note_enabled && self.settings.selected_note != self.last_note {
            if self.last_note < 24 {
                self.store_selected_note_at(self.last_note);
            }
            if self.settings.selected_note < 24 {
                self.load_selected_note();
            } else {
                self.load_global_parameters();
            }
            self.last_note = self.settings.selected_note;
        }
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            tab_button(ui, &mut self.selected_tab, Tab::Oscillator, "Oscillator");
            tab_button(ui, &mut self.selected_tab, Tab::Envelope, "Envelope");
            tab_button(ui, &mut self.selected_tab, Tab::Filter, "Filter");
            tab_button(ui, &mut self.selected_tab, Tab::Output, "Output");
        });
        ui.separator();
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| match self.selected_tab {
                Tab::Oscillator => self.oscillator_ui(ui),
                Tab::Envelope => self.envelope_ui(ui),
                Tab::Filter => self.filter_ui(ui),
                Tab::Output => self.output_ui(ui),
            });

        if self.settings.per_note_enabled && self.settings.selected_note < 24 {
            self.store_selected_note();
        }
        if self.settings.selected_note >= 24 {
            self.global_parameters = Some(NoteParameters::from_settings(&self.settings));
        }

        ui.add_space(8.0);
        if ui.button("Reset defaults").clicked() {
            self.settings = SynthSettings::default();
            self.allow_more_oscillators = false;
            self.confirm_more_oscillators = false;
        }

        if self.confirm_more_oscillators {
            self.confirm_more_oscillators_ui(&ctx);
        }
    }

    fn store_selected_note(&mut self) {
        self.store_selected_note_at(self.settings.selected_note);
    }

    fn store_selected_note_at(&mut self, i: usize) {
        let i = i.min(23);
        self.settings.note_settings[i] = Some(NoteParameters {
            oscillators: self.settings.oscillators,
            oscillator_count: self.settings.oscillator_count,
            delay: self.settings.delay,
            attack: self.settings.attack,
            hold: self.settings.hold,
            decay: self.settings.decay,
            sustain: self.settings.sustain,
            release: self.settings.release,
            cutoff_hz: self.settings.cutoff_hz,
            resonance: self.settings.resonance,
        });
    }

    fn load_global_parameters(&mut self) {
        if let Some(p) = self.global_parameters {
            self.settings.oscillators = p.oscillators;
            self.settings.oscillator_count = p.oscillator_count;
            self.settings.delay = p.delay;
            self.settings.attack = p.attack;
            self.settings.hold = p.hold;
            self.settings.decay = p.decay;
            self.settings.sustain = p.sustain;
            self.settings.release = p.release;
            self.settings.cutoff_hz = p.cutoff_hz;
            self.settings.resonance = p.resonance;
        }
    }

    fn load_selected_note(&mut self) {
        let p = self.settings.note_settings[self.settings.selected_note.min(23)]
            .or(self.global_parameters)
            .unwrap_or_else(|| NoteParameters::from_settings(&self.settings));
        self.settings.oscillators = p.oscillators;
        self.settings.oscillator_count = p.oscillator_count;
        self.settings.delay = p.delay;
        self.settings.attack = p.attack;
        self.settings.hold = p.hold;
        self.settings.decay = p.decay;
        self.settings.sustain = p.sustain;
        self.settings.release = p.release;
        self.settings.cutoff_hz = p.cutoff_hz;
        self.settings.resonance = p.resonance;
    }

    fn oscillator_ui(&mut self, ui: &mut Ui) {
        ui.label("Enable and tune each oscillator independently.");
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(format!(
                "Oscillators: {}/{}",
                self.settings.oscillator_count, MAX_OSCILLATORS
            ));
            if ui
                .add_enabled(
                    self.settings.oscillator_count < MAX_OSCILLATORS,
                    egui::Button::new("Add oscillator"),
                )
                .clicked()
            {
                if self.settings.oscillator_count >= 8 && !self.allow_more_oscillators {
                    self.confirm_more_oscillators = true;
                } else {
                    self.add_oscillator();
                }
            }
            if ui
                .add_enabled(
                    self.settings.oscillator_count > 1,
                    egui::Button::new("Remove last"),
                )
                .clicked()
            {
                self.remove_oscillator();
            }
        });
        ui.add_space(6.0);

        let oscillator_count = self.settings.oscillator_count;
        egui::Grid::new("oscillator_grid")
            .num_columns(3)
            .spacing(egui::vec2(8.0, 8.0))
            .show(ui, |ui| {
                for index in 0..oscillator_count {
                    let oscillator = &mut self.settings.oscillators[index];
                    ui.group(|ui| {
                        ui.set_min_width(210.0);
                        ui.set_max_width(230.0);
                        ui.vertical(|ui| {
                            ui.checkbox(
                                &mut oscillator.enabled,
                                format!("Oscillator {}", index + 1),
                            );
                            ui.add_enabled_ui(oscillator.enabled, |ui| {
                                ui.label("Waveform");
                                egui::ComboBox::from_id_salt(("oscillator_waveform", index))
                                    .selected_text(oscillator.waveform.name())
                                    .width(170.0)
                                    .show_ui(ui, |ui| {
                                        for waveform in Waveform::ALL {
                                            ui.selectable_value(
                                                &mut oscillator.waveform,
                                                waveform,
                                                waveform.name(),
                                            );
                                        }
                                    });
                                ui.add(
                                    Slider::new(&mut oscillator.octave_shift, -2..=2)
                                        .text("Octave")
                                        .step_by(1.0),
                                );
                                ui.add(
                                    Slider::new(&mut oscillator.tune_cents, -100.0..=100.0)
                                        .text("Detune")
                                        .suffix(" cents"),
                                );
                                ui.add(Slider::new(&mut oscillator.level, 0.0..=1.0).text("Level"));
                            });
                        });
                    });
                    if (index + 1) % 3 == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    fn add_oscillator(&mut self) {
        let count = self.settings.oscillator_count;
        if count < MAX_OSCILLATORS {
            self.settings.oscillators[count] = Default::default();
            self.settings.oscillator_count += 1;
        }
    }

    fn remove_oscillator(&mut self) {
        if self.settings.oscillator_count > 1 {
            self.settings.oscillator_count -= 1;
            self.settings.oscillators[self.settings.oscillator_count] = Default::default();
        }
    }

    fn confirm_more_oscillators_ui(&mut self, ctx: &Context) {
        let mut confirmed = false;
        let mut cancelled = false;
        let response = egui::Modal::new(Id::unique("confirm_extra_oscillators")).show(ctx, |ui| {
            ui.set_min_width(360.0);
            ui.heading("Use more than 8 oscillators?");
            ui.label(
                "Higher oscillator counts can increase CPU use, especially with many notes held.",
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                confirmed = ui.button("Continue").clicked();
                cancelled = ui.button("Cancel").clicked();
            });
        });

        if confirmed {
            self.allow_more_oscillators = true;
            self.add_oscillator();
            self.confirm_more_oscillators = false;
        } else if cancelled || response.backdrop_response.clicked() {
            self.confirm_more_oscillators = false;
        }
    }

    fn envelope_ui(&mut self, ui: &mut Ui) {
        ui.label("Delay, attack, hold, decay, sustain and release envelope");
        crate::ui::charts::envelope_chart::show(ui, self.settings);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            knob::show(
                ui,
                "Delay",
                &mut self.settings.delay,
                0.0,
                0.0..=2.0,
                KnobScale::Squared,
                |value| format!("{value:.2}s"),
            );
            knob::show(
                ui,
                "Attack",
                &mut self.settings.attack,
                0.01,
                0.001..=5.0,
                KnobScale::Logarithmic,
                |value| format!("{value:.2}s"),
            );
            knob::show(
                ui,
                "Hold",
                &mut self.settings.hold,
                0.0,
                0.0..=2.0,
                KnobScale::Squared,
                |value| format!("{value:.2}s"),
            );
            knob::show(
                ui,
                "Decay",
                &mut self.settings.decay,
                0.15,
                0.001..=5.0,
                KnobScale::Logarithmic,
                |value| format!("{value:.2}s"),
            );
            knob::show(
                ui,
                "Sustain",
                &mut self.settings.sustain,
                0.75,
                0.0..=1.0,
                KnobScale::Linear,
                |value| format!("{:.0}%", value * 100.0),
            );
            knob::show(
                ui,
                "Release",
                &mut self.settings.release,
                0.25,
                0.001..=8.0,
                KnobScale::Logarithmic,
                |value| format!("{value:.2}s"),
            );
        });
    }

    fn filter_ui(&mut self, ui: &mut Ui) {
        ui.label("Resonant low-pass filter");
        ui.add(
            Slider::new(&mut self.settings.cutoff_hz, 40.0..=18_000.0)
                .text("Cutoff")
                .logarithmic(true)
                .suffix(" Hz"),
        );
        ui.add(Slider::new(&mut self.settings.resonance, 0.0..=1.0).text("Resonance"));
    }

    fn output_ui(&mut self, ui: &mut Ui) {
        ui.label("Master output level");
        ui.add(
            Slider::new(&mut self.settings.volume, 0.0..=1.0)
                .text("Volume")
                .show_value(false),
        );
        ui.label(format!(
            "{}%",
            (self.settings.volume * 100.0).round() as u32
        ));
    }
}

fn tab_button(ui: &mut Ui, selected: &mut Tab, tab: Tab, label: &str) {
    ui.selectable_value(selected, tab, label);
}

fn note_name(note: usize) -> String {
    if note >= 24 {
        return "All notes".to_owned();
    }
    const NAMES: [&str; 12] = [
        "C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B",
    ];
    format!("{}{}", NAMES[note % 12], 4 + note / 12)
}

fn key_note(event: &egui::Event) -> Option<usize> {
    let egui::Event::Key {
        key,
        physical_key,
        pressed: true,
        repeat: false,
        ..
    } = event
    else {
        return None;
    };
    let key = physical_key.unwrap_or(*key);
    let keys = [
        egui::Key::Z,
        egui::Key::S,
        egui::Key::X,
        egui::Key::D,
        egui::Key::C,
        egui::Key::V,
        egui::Key::G,
        egui::Key::B,
        egui::Key::H,
        egui::Key::N,
        egui::Key::J,
        egui::Key::M,
        egui::Key::Q,
        egui::Key::Num2,
        egui::Key::W,
        egui::Key::Num3,
        egui::Key::E,
        egui::Key::R,
        egui::Key::Num5,
        egui::Key::T,
        egui::Key::Num6,
        egui::Key::Y,
        egui::Key::Num7,
        egui::Key::U,
    ];
    keys.iter().position(|candidate| *candidate == key)
}
