use std::sync::{Arc, Mutex};

use crate::instruments::synth::engine::SynthSettings;

use super::{audio_commands::PlaybackState, audio_config, audio_thread};

const NOTE_COUNT: usize = 24;

/// Owns the output stream and the notes currently being played.
pub struct SoundPlayer {
    stream: Option<cpal::Stream>,
    playback_state: Arc<Mutex<PlaybackState>>,
    output_devices: Vec<String>,
    selected_device: Option<usize>,
}

impl Default for SoundPlayer {
    fn default() -> Self {
        let host = cpal::default_host();
        let output_devices = audio_config::output_devices(&host);
        let selected_device = audio_config::default_output_index(&host, &output_devices);

        Self {
            stream: None,
            playback_state: Arc::new(Mutex::new(PlaybackState {
                active_notes: [false; NOTE_COUNT],
                settings: SynthSettings::default(),
            })),
            output_devices,
            selected_device,
        }
    }
}

impl SoundPlayer {
    pub fn output_devices(&self) -> &[String] {
        &self.output_devices
    }

    pub fn selected_device(&self) -> Option<usize> {
        self.selected_device
    }

    pub fn select_device(&mut self, index: usize) {
        if index < self.output_devices.len() && self.selected_device != Some(index) {
            self.selected_device = Some(index);
            self.stream = None;
        }
    }

    pub fn set_active_notes(&mut self, notes: [bool; NOTE_COUNT]) -> Result<(), String> {
        if notes.iter().any(|&active| active) && self.stream.is_none() {
            self.start_stream()?;
        }

        if let Ok(mut state) = self.playback_state.lock() {
            state.active_notes = notes;
        }

        Ok(())
    }

    pub fn set_settings(&mut self, settings: SynthSettings) {
        if let Ok(mut state) = self.playback_state.lock() {
            state.settings = settings;
        }
    }

    fn start_stream(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let selected_name = self
            .selected_device
            .and_then(|index| self.output_devices.get(index))
            .ok_or_else(|| "No audio output device is available".to_owned())?;
        let device = audio_config::find_output_device(&host, selected_name)?;
        let stream = audio_thread::build_stream(&device, Arc::clone(&self.playback_state))?;
        self.stream = Some(stream);
        Ok(())
    }
}
