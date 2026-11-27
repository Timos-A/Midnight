use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, StreamTrait};

use crate::instruments::synth::engine::SynthEngine;

use super::audio_commands::PlaybackState;

pub fn build_stream(
    device: &cpal::Device,
    playback_state: Arc<Mutex<PlaybackState>>,
) -> Result<cpal::Stream, String> {
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
    Ok(stream)
}
