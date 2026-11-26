use eframe::egui::Ui;

use crate::{
    instruments::synth::{controls::SynthControls, engine::SynthSettings},
    project::TrackKind,
    ui::{piano::Piano, piano_octave_selector::PianoOctaveSelector},
};

pub struct SynthInput {
    pub active_notes: [bool; 24],
    pub settings: SynthSettings,
}

pub fn show(
    ui: &mut Ui,
    track_kind: Option<TrackKind>,
    muted: bool,
    synth_controls: &mut SynthControls,
    piano: &mut Piano,
    octave_selector: &mut PianoOctaveSelector,
) -> Option<SynthInput> {
    match track_kind {
        Some(TrackKind::Synth) => {
            ui.horizontal(|ui| {
                ui.heading("Synth instrument");
                synth_controls.ui(ui);
                octave_selector.ui(ui);
            });
            ui.label("Play notes live. MIDI clip playback and recording are not connected yet.");

            let octave_shift = octave_selector.octave_shift();
            let notes = piano.ui(ui, octave_shift, synth_controls.note_selector_focused());
            let mut settings = synth_controls.settings();
            settings.keyboard_octave_shift = octave_shift;

            Some(SynthInput {
                active_notes: if muted { [false; 24] } else { notes },
                settings,
            })
        }
        Some(TrackKind::Midi) => {
            ui.heading("MIDI track");
            ui.label(
                "Add clips, record input, and edit notes here as the MIDI workflow is implemented.",
            );
            None
        }
        Some(TrackKind::Audio) => {
            ui.heading("Audio track");
            ui.label("Record or import audio here as the audio workflow is implemented.");
            None
        }
        None => {
            ui.heading("No track selected");
            None
        }
    }
}
