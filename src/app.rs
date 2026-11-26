use eframe::egui::{self, Color32};

use crate::{
    audio::SoundPlayer,
    instruments::synth::controls::SynthControls,
    project::Project,
    ui::{
        output_device_selector::OutputDeviceSelector,
        piano::Piano,
        piano_octave_selector::PianoOctaveSelector,
        workspace::{
            arrangement, browser, project_header, synth_editor, track_settings, transport,
        },
    },
};

pub struct App {
    sound: SoundPlayer,
    output_device_selector: OutputDeviceSelector,
    piano: Piano,
    piano_octave_selector: PianoOctaveSelector,
    synth_controls: SynthControls,
    project: Project,
    selected_track: Option<usize>,
    track_settings_track: Option<usize>,
    synth_ui_track: Option<usize>,
    controls_track: Option<usize>,
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
            selected_track: None,
            track_settings_track: None,
            synth_ui_track: None,
            controls_track: None,
            audio_error: None,
        }
    }
}

impl App {
    fn delete_track(&mut self, index: usize) {
        if !self.project.remove_track(index) {
            return;
        }

        self.selected_track = self.selected_track.and_then(|selected| {
            if selected == index {
                if self.project.tracks.is_empty() {
                    None
                } else {
                    Some(index.min(self.project.tracks.len() - 1))
                }
            } else if selected > index {
                Some(selected - 1)
            } else {
                Some(selected)
            }
        });
        self.track_settings_track = adjust_index_after_removal(self.track_settings_track, index);
        self.synth_ui_track = adjust_index_after_removal(self.synth_ui_track, index);
        self.controls_track = adjust_index_after_removal(self.controls_track, index);
    }
}

fn adjust_index_after_removal(index: Option<usize>, removed: usize) -> Option<usize> {
    index.and_then(|index| {
        if index == removed {
            None
        } else if index > removed {
            Some(index - 1)
        } else {
            Some(index)
        }
    })
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("project_header").show(ui, |ui| {
            project_header::show(ui, &self.project.name);
            if let Some(index) = transport::show(
                ui,
                &mut self.output_device_selector,
                self.sound.output_devices(),
            ) {
                self.sound.select_device(index);
            }
        });

        egui::Panel::left("asset_browser")
            .resizable(true)
            .default_size(250.0)
            .show(ui, browser::show);

        let keyboard_track = self.selected_track.filter(|&index| {
            self.project.tracks.get(index).is_some_and(|track| {
                track.kind == crate::project::TrackKind::Instrument
                    && track.instrument == Some(crate::project::InstrumentKind::Synth)
            })
        });
        let mut live_notes = None;
        if keyboard_track.is_some() {
            egui::Panel::bottom("performance_keyboard")
                .resizable(true)
                .default_size(260.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Performance Keyboard");
                        if ui.button("Open Synth UI").clicked() {
                            self.synth_ui_track = keyboard_track;
                        }
                        self.piano_octave_selector.ui(ui);
                    });
                    live_notes = Some(self.piano.ui(
                        ui,
                        self.piano_octave_selector.octave_shift(),
                        self.synth_controls.note_selector_focused(),
                    ));
                });
        } else {
            self.piano.release_all();
        }

        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(action) = arrangement::show(ui, &self.project, self.selected_track) {
                match action {
                    arrangement::ArrangementAction::AddTrack => {
                        let index = self.project.add_track();
                        self.selected_track = Some(index);
                    }
                    arrangement::ArrangementAction::OpenTrackSettings(index) => {
                        if index < self.project.tracks.len() {
                            self.selected_track = Some(index);
                            self.track_settings_track = Some(index);
                            self.synth_ui_track = None;
                        }
                    }
                    arrangement::ArrangementAction::SetMuted { index, muted } => {
                        if let Some(track) = self.project.tracks.get_mut(index) {
                            track.muted = muted;
                        }
                    }
                    arrangement::ArrangementAction::DeleteTrack(index) => {
                        self.delete_track(index);
                    }
                }
            }

            if let Some(error) = &self.audio_error {
                ui.colored_label(Color32::RED, error);
            }
        });

        if let Some(index) = self.track_settings_track {
            if let Some(track) = self.project.tracks.get_mut(index) {
                if self.controls_track != Some(index) {
                    self.synth_controls.set_settings(track.synth_settings);
                    self.controls_track = Some(index);
                }

                let mut is_open = true;
                let settings_result = track_settings::show(ui.ctx(), index, track, &mut is_open);
                if !settings_result.open {
                    self.track_settings_track = None;
                }
            } else {
                self.track_settings_track = None;
            }
        }

        if let Some(index) = self.synth_ui_track {
            if let Some(track) = self.project.tracks.get_mut(index) {
                if track.kind == crate::project::TrackKind::Instrument
                    && track.instrument == Some(crate::project::InstrumentKind::Synth)
                {
                    if self.controls_track != Some(index) {
                        self.synth_controls.set_settings(track.synth_settings);
                        self.controls_track = Some(index);
                    }
                    if !synth_editor::show(ui.ctx(), index, &mut self.synth_controls) {
                        self.synth_ui_track = None;
                    }
                    track.synth_settings = self.synth_controls.settings();
                } else {
                    self.synth_ui_track = None;
                }
            } else {
                self.synth_ui_track = None;
            }
        }

        let selected_synth_track = self.selected_track.filter(|&index| {
            self.project.tracks.get(index).is_some_and(|track| {
                track.kind == crate::project::TrackKind::Instrument
                    && track.instrument == Some(crate::project::InstrumentKind::Synth)
            })
        });
        if let Some(index) = selected_synth_track {
            let track = &self.project.tracks[index];
            let notes = if track.muted || Some(index) != keyboard_track {
                [false; 24]
            } else {
                live_notes.unwrap_or([false; 24])
            };
            let mut settings = track.synth_settings;
            settings.keyboard_octave_shift = self.piano_octave_selector.octave_shift();
            self.sound.set_settings(settings);
            self.audio_error = self.sound.set_active_notes(notes).err();
        } else {
            self.audio_error = self.sound.set_active_notes([false; 24]).err();
        }
    }
}
