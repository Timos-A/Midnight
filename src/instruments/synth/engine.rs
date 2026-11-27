use std::f32::consts::{PI, TAU};

const NOTE_COUNT: usize = 24;
pub const MAX_OSCILLATORS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Triangle,
    Sawtooth,
    Square,
}

impl Waveform {
    pub const ALL: [Self; 4] = [Self::Sine, Self::Triangle, Self::Sawtooth, Self::Square];

    pub fn name(self) -> &'static str {
        match self {
            Self::Sine => "Sine",
            Self::Triangle => "Triangle",
            Self::Sawtooth => "Sawtooth",
            Self::Square => "Square",
        }
    }

    fn sample(self, phase: f32) -> f32 {
        match self {
            Self::Sine => phase.sin(),
            Self::Triangle => 1.0 - 4.0 * ((phase / TAU) - 0.5).abs(),
            Self::Sawtooth => 2.0 * phase / TAU - 1.0,
            Self::Square => {
                if phase < PI {
                    1.0
                } else {
                    -1.0
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct OscillatorSettings {
    pub enabled: bool,
    pub waveform: Waveform,
    pub octave_shift: i8,
    pub tune_cents: f32,
    pub level: f32,
}

impl OscillatorSettings {
    const fn new(
        enabled: bool,
        waveform: Waveform,
        octave_shift: i8,
        tune_cents: f32,
        level: f32,
    ) -> Self {
        Self {
            enabled,
            waveform,
            octave_shift,
            tune_cents,
            level,
        }
    }
}

impl Default for OscillatorSettings {
    fn default() -> Self {
        Self::new(false, Waveform::Sine, 0, 0.0, 0.5)
    }
}

#[derive(Clone, Copy)]
pub struct SynthSettings {
    pub volume: f32,
    pub keyboard_octave_shift: i8,
    pub oscillators: [OscillatorSettings; MAX_OSCILLATORS],
    pub oscillator_count: usize,
    pub delay: f32,
    pub attack: f32,
    pub hold: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub per_note_enabled: bool,
    pub selected_note: usize,
    pub note_settings: [Option<NoteParameters>; NOTE_COUNT],
}

#[derive(Clone, Copy)]
pub struct NoteParameters {
    pub oscillators: [OscillatorSettings; MAX_OSCILLATORS],
    pub oscillator_count: usize,
    pub delay: f32,
    pub attack: f32,
    pub hold: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub cutoff_hz: f32,
    pub resonance: f32,
}

impl Default for SynthSettings {
    fn default() -> Self {
        let mut oscillators = [OscillatorSettings::default(); MAX_OSCILLATORS];
        oscillators[0] = OscillatorSettings::new(true, Waveform::Sine, 0, 0.0, 1.0);
        oscillators[1] = OscillatorSettings::new(false, Waveform::Triangle, 0, -7.0, 0.5);
        oscillators[2] = OscillatorSettings::new(false, Waveform::Sawtooth, -1, 7.0, 0.35);

        Self {
            volume: 0.7,
            keyboard_octave_shift: 0,
            oscillators,
            oscillator_count: 3,
            delay: 0.0,
            attack: 0.01,
            hold: 0.0,
            decay: 0.15,
            sustain: 0.75,
            release: 0.25,
            cutoff_hz: 12_000.0,
            resonance: 0.1,
            per_note_enabled: false,
            selected_note: 24,
            note_settings: [None; NOTE_COUNT],
        }
    }
}

impl NoteParameters {
    pub fn from_settings(s: &SynthSettings) -> Self {
        Self {
            oscillators: s.oscillators,
            oscillator_count: s.oscillator_count,
            delay: s.delay,
            attack: s.attack,
            hold: s.hold,
            decay: s.decay,
            sustain: s.sustain,
            release: s.release,
            cutoff_hz: s.cutoff_hz,
            resonance: s.resonance,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnvelopeStage {
    Off,
    Delay,
    Attack,
    Hold,
    Decay,
    Sustain,
    Release,
}

struct Voice {
    phases: [f32; MAX_OSCILLATORS],
    level: f32,
    stage_start: f32,
    stage_elapsed: f32,
    release_start: f32,
    gate: bool,
    stage: EnvelopeStage,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            phases: [0.0; MAX_OSCILLATORS],
            level: 0.0,
            stage_start: 0.0,
            stage_elapsed: 0.0,
            release_start: 0.0,
            gate: false,
            stage: EnvelopeStage::Off,
        }
    }
}

impl Voice {
    fn update_envelope(&mut self, gate: bool, parameters: NoteParameters, sample_rate: f32) {
        if gate && !self.gate {
            self.stage_elapsed = 0.0;
            if parameters.delay > 0.0 {
                self.stage = EnvelopeStage::Delay;
                self.level = 0.0;
            } else {
                self.stage_start = self.level;
                self.stage = EnvelopeStage::Attack;
            }
        } else if !gate && self.gate && self.stage != EnvelopeStage::Off {
            self.release_start = self.level;
            self.stage_elapsed = 0.0;
            self.stage = if parameters.release <= 0.0 {
                self.level = 0.0;
                EnvelopeStage::Off
            } else {
                EnvelopeStage::Release
            };
        }
        self.gate = gate;

        match self.stage {
            EnvelopeStage::Off => self.level = 0.0,
            EnvelopeStage::Delay => {
                self.level = 0.0;
                self.stage_elapsed += 1.0 / sample_rate;
                if self.stage_elapsed >= parameters.delay {
                    self.stage_elapsed = 0.0;
                    self.stage_start = 0.0;
                    self.stage = EnvelopeStage::Attack;
                }
            }
            EnvelopeStage::Attack => {
                if parameters.attack <= 0.0 {
                    self.level = 1.0;
                    self.begin_hold_or_decay(parameters);
                } else {
                    self.stage_elapsed += 1.0 / sample_rate;
                    let progress = (self.stage_elapsed / parameters.attack).clamp(0.0, 1.0);
                    self.level = self.stage_start + (1.0 - self.stage_start) * progress;
                    if progress >= 1.0 {
                        self.level = 1.0;
                        self.begin_hold_or_decay(parameters);
                    }
                }
            }
            EnvelopeStage::Hold => {
                self.level = 1.0;
                self.stage_elapsed += 1.0 / sample_rate;
                if self.stage_elapsed >= parameters.hold {
                    self.stage = EnvelopeStage::Decay;
                    self.stage_elapsed = 0.0;
                }
            }
            EnvelopeStage::Decay => {
                if parameters.decay <= 0.001 || self.level <= parameters.sustain {
                    self.level = parameters.sustain;
                    self.stage = EnvelopeStage::Sustain;
                } else {
                    self.stage_elapsed += 1.0 / sample_rate;
                    let progress = (self.stage_elapsed / parameters.decay).clamp(0.0, 1.0);
                    self.level = 1.0 + (parameters.sustain - 1.0) * progress;
                    if progress >= 1.0 {
                        self.level = parameters.sustain;
                        self.stage = EnvelopeStage::Sustain;
                    }
                }
            }
            EnvelopeStage::Sustain => self.level = parameters.sustain,
            EnvelopeStage::Release => {
                self.stage_elapsed += 1.0 / sample_rate;
                let progress = (self.stage_elapsed / parameters.release).clamp(0.0, 1.0);
                self.level = self.release_start * (1.0 - progress);
                if progress >= 1.0 {
                    self.stage = EnvelopeStage::Off;
                }
            }
        }
    }

    fn begin_hold_or_decay(&mut self, parameters: NoteParameters) {
        self.stage_elapsed = 0.0;
        self.stage = if parameters.hold > 0.0 {
            EnvelopeStage::Hold
        } else {
            EnvelopeStage::Decay
        };
    }
}

#[derive(Default)]
struct LowPassFilter {
    integrator_one: f32,
    integrator_two: f32,
}

impl LowPassFilter {
    fn process(&mut self, input: f32, g: f32, damping: f32) -> f32 {
        let a1 = 1.0 / (1.0 + g * (g + damping));
        let a2 = g * a1;
        let a3 = g * a2;
        let v3 = input - self.integrator_two;
        let v1 = a1 * self.integrator_one + a2 * v3;
        let v2 = self.integrator_two + a2 * self.integrator_one + a3 * v3;
        self.integrator_one = 2.0 * v1 - self.integrator_one;
        self.integrator_two = 2.0 * v2 - self.integrator_two;
        v2
    }
}

/// Audio-thread synthesizer: three oscillators, 24 note voices, ADSR envelopes and a resonant low-pass filter.
#[derive(Default)]
pub struct SynthEngine {
    voices: [Voice; NOTE_COUNT],
    filter: LowPassFilter,
}

impl SynthEngine {
    pub fn render(
        &mut self,
        output: &mut [f32],
        channels: usize,
        sample_rate: f32,
        active_notes: &[bool; NOTE_COUNT],
        settings: SynthSettings,
    ) {
        let global_parameters = NoteParameters::from_settings(&settings);
        let note_parameters: [NoteParameters; NOTE_COUNT] = std::array::from_fn(|index| {
            if settings.per_note_enabled {
                settings.note_settings[index].unwrap_or(global_parameters)
            } else {
                global_parameters
            }
        });

        let safe_cutoff = settings.cutoff_hz.clamp(20.0, sample_rate * 0.45);
        let g = (PI * safe_cutoff / sample_rate).tan();
        let damping = 2.0 - 1.95 * settings.resonance.clamp(0.0, 1.0);
        let phase_steps: [[f32; MAX_OSCILLATORS]; NOTE_COUNT] = std::array::from_fn(|note_index| {
            let parameters = note_parameters[note_index];
            let midi_note = 60 + note_index as i32 + settings.keyboard_octave_shift as i32 * 12;
            let frequency = 440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0);
            std::array::from_fn(|oscillator_index| {
                if oscillator_index >= parameters.oscillator_count.clamp(1, MAX_OSCILLATORS) {
                    return 0.0;
                }
                let oscillator = parameters.oscillators[oscillator_index];
                let octave_multiplier = 2.0_f32.powi(oscillator.octave_shift.clamp(-2, 2) as i32);
                let tune_multiplier =
                    2.0_f32.powf(oscillator.tune_cents.clamp(-100.0, 100.0) / 1200.0);
                if oscillator.enabled {
                    TAU * frequency * octave_multiplier * tune_multiplier / sample_rate
                } else {
                    0.0
                }
            })
        });

        for frame in output.chunks_mut(channels) {
            let mut mixed = 0.0;
            let mut sounding_voices = 0;
            for index in 0..NOTE_COUNT {
                let voice = &mut self.voices[index];
                let parameters = note_parameters[index];
                voice.update_envelope(active_notes[index], parameters, sample_rate);
                if voice.level > 0.0 {
                    sounding_voices += 1;
                    for oscillator_index in 0..parameters.oscillator_count.clamp(1, MAX_OSCILLATORS)
                    {
                        let oscillator = parameters.oscillators[oscillator_index];
                        if oscillator.enabled {
                            mixed += oscillator.waveform.sample(voice.phases[oscillator_index])
                                * oscillator.level
                                * voice.level;
                            voice.phases[oscillator_index] = (voice.phases[oscillator_index]
                                + phase_steps[index][oscillator_index])
                                % TAU;
                        }
                    }
                }
            }

            let enabled_oscillators = note_parameters
                .iter()
                .map(|parameters| {
                    parameters
                        .oscillators
                        .iter()
                        .take(parameters.oscillator_count.clamp(1, MAX_OSCILLATORS))
                        .filter(|oscillator| oscillator.enabled && oscillator.level > 0.0)
                        .count()
                })
                .max()
                .unwrap_or(0);
            let voice_gain = if sounding_voices == 0 || enabled_oscillators == 0 {
                0.0
            } else {
                0.16 * settings.volume.clamp(0.0, 1.0)
                    / ((sounding_voices * enabled_oscillators) as f32).sqrt()
            };
            let sample = self
                .filter
                .process(mixed * voice_gain, g, damping)
                .clamp(-1.0, 1.0);
            for channel in frame {
                *channel = sample;
            }
        }
    }
}
