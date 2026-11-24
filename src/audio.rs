use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const NOTE_COUNT: usize = 24;

use crate::instruments::synth::engine::{SynthEngine, SynthSettings};

#[derive(Clone, Copy)]
struct PlaybackState {
    active_notes: [bool; NOTE_COUNT],
    settings: SynthSettings,
}

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
        let output_devices: Vec<String> = host
            .output_devices()
            .into_iter()
            .flatten()
            .filter_map(|device| {
                device
                    .description()
                    .ok()
                    .map(|description| description.name().to_owned())
            })
            .collect();
        let default_name = host.default_output_device().and_then(|device| {
            device
                .description()
                .ok()
                .map(|description| description.name().to_owned())
        });
        let selected_device = default_name
            .and_then(|name| output_devices.iter().position(|device| *device == name))
            .or_else(|| (!output_devices.is_empty()).then_some(0));

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
        let device = host
            .output_devices()
            .map_err(|error| format!("Could not list audio output devices: {error}"))?
            .find(|device| {
                device
                    .description()
                    .ok()
                    .is_some_and(|description| description.name() == selected_name)
            })
            .ok_or_else(|| format!("Selected audio output is unavailable: {selected_name}"))?;
        let supported_config = device
            .default_output_config()
            .map_err(|error| format!("Could not get output configuration: {error}"))?;

        if supported_config.sample_format() != cpal::SampleFormat::F32 {
            return Err(format!(
                "The selected output uses {:?}; this piano needs an f32 output",
                supported_config.sample_format()
            ));
        }

        let config: cpal::StreamConfig = supported_config.into();
        let channels = config.channels as usize;
        let sample_rate = config.sample_rate as f32;
        let playback_state = Arc::clone(&self.playback_state);
        let mut synth = SynthEngine::default();

        let stream = device
            .build_output_stream(
                config,
                move |output: &mut [f32], _| {
                    let Ok(state) = playback_state.lock().map(|state| *state) else {
                        output.fill(0.0);
                        return;
                    };
                    synth.render(
                        output,
                        channels,
                        sample_rate,
                        &state.active_notes,
                        state.settings,
                    );
                },
                |error| eprintln!("Audio stream error: {error}"),
                None,
            )
            .map_err(|error| format!("Could not create audio stream: {error}"))?;

        stream
            .play()
            .map_err(|error| format!("Could not start audio: {error}"))?;
        self.stream = Some(stream);
        Ok(())
    }
}
