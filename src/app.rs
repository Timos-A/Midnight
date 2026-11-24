use eframe::egui::{self, Color32};

use crate::{
    audio::SoundPlayer,
    instruments::synth::controls::SynthControls,
    project::{Project, TrackKind},
    ui::{output_device_selector::OutputDeviceSelector, piano::Piano, piano_octave_selector::PianoOctaveSelector},
};

pub struct App {
    sound: SoundPlayer,
    output_device_selector: OutputDeviceSelector,
    piano: Piano,
    piano_octave_selector: PianoOctaveSelector,
    synth_controls: SynthControls,
    project: Project,
    selected_track: usize,
    audio_error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let sound = SoundPlayer::default();
        let output_device_selector = OutputDeviceSelector::new(sound.selected_device());
        Self {
            sound,
            output_device_selector,
            piano: Piano::default(),
            piano_octave_selector: PianoOctaveSelector::default(),
            synth_controls: SynthControls::default(),
            project: Project::default(),
            selected_track: 0,
            audio_error: None,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("project_header").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Midnight");
                ui.separator();
                ui.label(&self.project.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_enabled(false, egui::Button::new("Save"))
                        .on_hover_text("Project saving is not available yet");
                    ui.add_enabled(false, egui::Button::new("Export WAV"))
                        .on_hover_text("Audio export is not available yet");
                });
            });
            ui.horizontal(|ui| {
                ui.label("Transport");
                ui.add_enabled(false, egui::Button::new("⏮"));
                ui.add_enabled(false, egui::Button::new("▶"));
                ui.add_enabled(false, egui::Button::new("■"));
                ui.add_enabled(false, egui::Button::new("● Record"));
                ui.separator();
                ui.label("00:00:00");
                ui.separator();
                if let Some(index) = self.output_device_selector.ui(ui, self.sound.output_devices()) {
                    self.sound.select_device(index);
                }
            });
        });

        egui::Panel::left("track_list").resizable(true).default_size(220.0).show(ui, |ui| {
            ui.heading("Tracks");
            ui.horizontal(|ui| {
                if ui.button("+ Synth").clicked() {
                    self.selected_track = self.project.add_track(TrackKind::Synth);
                }
                if ui.button("+ MIDI").clicked() {
                    self.selected_track = self.project.add_track(TrackKind::Midi);
                }
                if ui.button("+ Audio").clicked() {
                    self.selected_track = self.project.add_track(TrackKind::Audio);
                }
            });
            ui.separator();
            for (index, track) in self.project.tracks.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_track, index, format!("{}  ·  {}", track.name, track.kind.label()));
                    ui.checkbox(&mut track.muted, "M");
                });
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label("Tracks are the building blocks of a session.");
            });
        });

        let selected_kind = self.project.tracks.get(self.selected_track).map(|track| track.kind);
        egui::Panel::bottom("instrument_editor")
            .resizable(true)
            .default_size(330.0)
            .show(ui, |ui| {
                match selected_kind {
                    Some(TrackKind::Synth) => {
                        ui.horizontal(|ui| {
                            ui.heading("Synth instrument");
                            self.synth_controls.ui(ui);
                            self.piano_octave_selector.ui(ui);
                        });
                        ui.label("Play notes live. MIDI clip playback and recording are not connected yet.");
                        let octave_shift = self.piano_octave_selector.octave_shift();
                        let active_notes = self.piano.ui(
                            ui,
                            octave_shift,
                            self.synth_controls.note_selector_focused(),
                        );
                        let mut settings = self.synth_controls.settings();
                        settings.keyboard_octave_shift = octave_shift;
                        self.sound.set_settings(settings);
                        let muted = self.project.tracks[self.selected_track].muted;
                        if let Err(error) = self
                            .sound
                            .set_active_notes(if muted { [false; 24] } else { active_notes })
                        {
                            self.audio_error = Some(error);
                        } else {
                            self.audio_error = None;
                        }
                    }
                    Some(TrackKind::Midi) => {
                        ui.heading("MIDI track");
                        ui.label("Add clips, record input, and edit notes here as the MIDI workflow is implemented.");
                        let _ = self.sound.set_active_notes([false; 24]);
                    }
                    Some(TrackKind::Audio) => {
                        ui.heading("Audio track");
                        ui.label("Record or import audio here as the audio workflow is implemented.");
                        let _ = self.sound.set_active_notes([false; 24]);
                    }
                    None => {}
                }

                if let Some(error) = &self.audio_error {
                    ui.colored_label(Color32::RED, error);
                }
            });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Arrangement");
            ui.label("Timeline workspace");
            ui.add_space(8.0);
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                ui.set_min_width(ui.available_width().max(700.0));
                ui.horizontal(|ui| {
                    ui.add_sized([180.0, 24.0], egui::Label::new("Track / bars"));
                    for bar in 1..=9 {
                        ui.add_sized([96.0, 24.0], egui::Label::new(format!("{bar}")));
                    }
                });
                ui.separator();
                for (index, track) in self.project.tracks.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let track_button = egui::Button::new(format!(
                            "{}{}",
                            track.name,
                            if track.muted { " (muted)" } else { "" }
                        ))
                        .selected(self.selected_track == index);
                        if ui.add_sized([180.0, 44.0], track_button).clicked() {
                            self.selected_track = index;
                        }
                        ui.allocate_ui_with_layout(
                            egui::vec2(96.0 * 8.0, 44.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.painter().rect_stroke(
                                    ui.max_rect(), 2.0,
                                    egui::Stroke::new(1.0, Color32::from_gray(65)),
                                    egui::StrokeKind::Inside,
                                );
                                let message = match track.kind {
                                    TrackKind::Synth => "Play this instrument below",
                                    TrackKind::Midi => "MIDI clip lane · recording and editing coming next",
                                    TrackKind::Audio => "Audio lane · recording and editing coming next",
                                };
                                ui.centered_and_justified(|ui| { ui.weak(message); });
                            },
                        );
                    });
                }
            });
            ui.separator();
        });
    }
}
