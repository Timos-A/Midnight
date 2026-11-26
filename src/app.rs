use eframe::egui::{self, Color32};

use crate::{
    audio::SoundPlayer,
    instruments::synth::controls::SynthControls,
    project::Project,
    ui::{
        output_device_selector::OutputDeviceSelector,
        piano::Piano,
        piano_octave_selector::PianoOctaveSelector,
        workspace::{arrangement, instrument_editor, project_header, sidebar, transport},
    },
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
            project_header::show(ui, &self.project.name);
            if let Some(index) = transport::show(
                ui,
                &mut self.output_device_selector,
                self.sound.output_devices(),
            ) {
                self.sound.select_device(index);
            }
        });

        let sidebar_action = egui::Panel::left("track_list")
            .resizable(true)
            .default_size(220.0)
            .show(ui, |ui| {
                sidebar::show(ui, &self.project, self.selected_track)
            })
            .inner;
        match sidebar_action {
            Some(sidebar::SidebarAction::AddTrack(kind)) => {
                self.selected_track = self.project.add_track(kind);
            }
            Some(sidebar::SidebarAction::SelectTrack(index)) => {
                self.selected_track = index;
            }
            Some(sidebar::SidebarAction::SetMuted { index, muted }) => {
                if let Some(track) = self.project.tracks.get_mut(index) {
                    track.muted = muted;
                }
            }
            None => {}
        }

        let selected_kind = self
            .project
            .tracks
            .get(self.selected_track)
            .map(|track| track.kind);
        let selected_muted = self
            .project
            .tracks
            .get(self.selected_track)
            .is_some_and(|track| track.muted);
        let synth_input = egui::Panel::bottom("instrument_editor")
            .resizable(true)
            .default_size(330.0)
            .show(ui, |ui| {
                instrument_editor::show(
                    ui,
                    selected_kind,
                    selected_muted,
                    &mut self.synth_controls,
                    &mut self.piano,
                    &mut self.piano_octave_selector,
                )
            })
            .inner;

        if let Some(synth_input) = synth_input {
            self.sound.set_settings(synth_input.settings);
            self.audio_error = self.sound.set_active_notes(synth_input.active_notes).err();
        } else {
            self.audio_error = self.sound.set_active_notes([false; 24]).err();
        }

        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(index) = arrangement::show(ui, &self.project, self.selected_track) {
                self.selected_track = index;
            }
            if let Some(error) = &self.audio_error {
                ui.colored_label(Color32::RED, error);
            }
        });
    }
}
