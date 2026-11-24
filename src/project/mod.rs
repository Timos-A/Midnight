#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackKind {
    Synth,
    Midi,
    Audio,
}

impl TrackKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Synth => "Synth",
            Self::Midi => "MIDI",
            Self::Audio => "Audio",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub name: String,
    pub kind: TrackKind,
    pub muted: bool,
}

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub tracks: Vec<Track>,
    next_track_number: usize,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "Untitled Session".to_owned(),
            tracks: vec![Track {
                name: "Synth 1".to_owned(),
                kind: TrackKind::Synth,
                muted: false,
            }],
            next_track_number: 2,
        }
    }
}

impl Project {
    pub fn add_track(&mut self, kind: TrackKind) -> usize {
        let name = format!("{} {}", kind.label(), self.next_track_number);
        self.next_track_number += 1;
        self.tracks.push(Track { name, kind, muted: false });
        self.tracks.len() - 1
    }
}
