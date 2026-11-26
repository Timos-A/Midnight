use crate::instruments::synth::engine::SynthSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackKind {
    Blank,
    Instrument,
    Midi,
    Audio,
}

impl TrackKind {
    pub const ALL: [Self; 4] = [Self::Blank, Self::Instrument, Self::Midi, Self::Audio];

    pub fn label(self) -> &'static str {
        match self {
            Self::Blank => "Blank",
            Self::Instrument => "Instrument",
            Self::Midi => "MIDI",
            Self::Audio => "Audio",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstrumentKind {
    Synth,
}

impl InstrumentKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Synth => "Built-in Synth",
        }
    }
}

#[derive(Clone)]
pub struct Track {
    pub name: String,
    pub kind: TrackKind,
    pub instrument: Option<InstrumentKind>,
    pub synth_settings: SynthSettings,
    pub muted: bool,
}

impl Track {
    fn new(name: String) -> Self {
        Self {
            name,
            kind: TrackKind::Blank,
            instrument: None,
            synth_settings: SynthSettings::default(),
            muted: false,
        }
    }
}

pub struct Project {
    pub name: String,
    pub tracks: Vec<Track>,
    next_track_number: usize,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "Untitled Session".to_owned(),
            tracks: Vec::new(),
            next_track_number: 1,
        }
    }
}

impl Project {
    pub fn add_track(&mut self) -> usize {
        let name = format!("Track {}", self.next_track_number);
        self.next_track_number += 1;
        self.tracks.push(Track::new(name));
        self.tracks.len() - 1
    }

    pub fn remove_track(&mut self, index: usize) -> bool {
        if index < self.tracks.len() {
            self.tracks.remove(index);
            true
        } else {
            false
        }
    }
}
