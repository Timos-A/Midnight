use crate::instruments::synth::engine::SynthSettings;

pub const NOTE_COUNT: usize = 24;

#[derive(Clone, Copy)]
pub struct PlaybackState {
    pub active_notes: [bool; NOTE_COUNT],
    pub settings: SynthSettings,
}
