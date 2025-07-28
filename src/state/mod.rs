use std::time::{Duration, Instant};

use minifb::Key;

use crate::graphics::constants::{WAVEFORM_SAWTOOTH, WAVEFORM_SINE, WAVEFORM_SQUARE, WAVEFORM_TRIANGLE};
use crate::music_theory::{OCTAVE_LOWER_BOUND, OCTAVE_UPPER_BOUND};
use crate::music_theory::note::Note;
use crate::waveforms::Waveform;

pub mod event_loop;
mod utils;

const FRAME_DURATION: Duration = Duration::from_millis(16); // Approximately 60Hz refresh rate

// Synthesizer State Struct
pub struct State {
    octave: i32,
    waveform: Waveform,
    pressed_key: Option<(Key, Note)>,
    waveform_sprite_index: usize,
    filter_factor: f32,
    lpf_active: usize,
    current_frequency: Option<f32>, // Track current playing frequency
    animation_start_time: Instant, // When the animation started
    key_release_time: Option<Instant>, // When the key was released for fade-out
    // ADSR parameters (0 to 99)
    pub attack: u8,  // Attack time (0 = instant, 99 = 2 seconds)
    pub decay: u8,   // Decay time (0 = instant, 99 = 2 seconds)
    pub sustain: u8, // Sustain level (0 = silent, 99 = full volume)
    pub release: u8, // Release time (0 = instant, 99 = 2 seconds)
}

// Initialize Synthesizer State
impl State {
    pub(crate) fn new() -> Self {
        State {
            octave: 4, // Set default octave to 4
            waveform: Waveform::SINE, // Set default waveform to Sine
            pressed_key: None, // Default is no key
            waveform_sprite_index: WAVEFORM_SINE, // Set default waveform sprite index to Sine
            filter_factor: 1.0, // Set default cutoff to 1.0
            lpf_active: 0, // Default for LPF is deactivated
            current_frequency: None, // No frequency being played initially
            animation_start_time: Instant::now(), // Initialize animation time
            key_release_time: None, // No key released initially
            // ADSR defaults for immediate sound
            attack: 0,   // Instant attack
            decay: 0,    // No decay
            sustain: 99, // Full sustain level
            release: 20, // Quick release
        }
    }

    /// Multiplies the sample frequency with that of the filter cutoff coefficient
    pub fn apply_lpf(&mut self, sample: f32) -> f32 {
        sample * self.filter_factor
    }

    /// Increases the octave by one step, ensuring it does not exceed the upper bound.
    pub fn increase_octave(&mut self) {
        if self.octave < OCTAVE_UPPER_BOUND {
            self.octave += 1;
        }
    }

    /// Decreases the octave by one step, ensuring it does not go below the lower bound.
    pub fn decrease_octave(&mut self) {
        if self.octave > OCTAVE_LOWER_BOUND {
            self.octave -= 1;
        }
    }

    /// Toggle LPF on/off
    pub fn toggle_lpf(&mut self) {
        self.lpf_active ^= 1;
        self.filter_factor = 1.0;
    }

    /// Increases the filter cutoff
    pub fn increase_filter_cutoff(&mut self) {
        if self.lpf_active == 1 && self.filter_factor <= 0.9 {
            self.filter_factor += 0.142857;
        }
    }

    /// Decreases the filter cutoff
    pub fn decrease_filter_cutoff(&mut self) {
        if self.lpf_active == 1 && self.filter_factor >= 0.15 {
            self.filter_factor -= 0.142857;
        }
    }

    /// Returns the current octave value.
    pub fn get_current_octave(&self) -> i32 {
        self.octave
    }

    /// Toggles the waveform between SINE and SQUARE and sets the associated sprite index accordingly.
    pub fn toggle_waveform(&mut self) {
        self.waveform = match self.waveform {
            Waveform::SINE => {
                self.waveform_sprite_index = WAVEFORM_SQUARE;
                Waveform::SQUARE
            },
            Waveform::SQUARE => {
                self.waveform_sprite_index = WAVEFORM_TRIANGLE;
                Waveform::TRIANGLE
            },
            Waveform::TRIANGLE => {
                self.waveform_sprite_index = WAVEFORM_SAWTOOTH;
                Waveform::SAWTOOTH
            },
            Waveform::SAWTOOTH => {
                self.waveform_sprite_index = WAVEFORM_SINE;
                Waveform::SINE
            }
        };
    }

    // ADSR control methods (0-99 range)
    pub fn increase_attack(&mut self) {
        self.attack = (self.attack + 1).min(99);
    }

    pub fn decrease_attack(&mut self) {
        self.attack = self.attack.saturating_sub(1);
    }

    pub fn increase_decay(&mut self) {
        self.decay = (self.decay + 1).min(99);
    }

    pub fn decrease_decay(&mut self) {
        self.decay = self.decay.saturating_sub(1);
    }

    pub fn increase_sustain(&mut self) {
        self.sustain = (self.sustain + 1).min(99);
    }

    pub fn decrease_sustain(&mut self) {
        self.sustain = self.sustain.saturating_sub(1);
    }

    pub fn increase_release(&mut self) {
        self.release = (self.release + 1).min(99);
    }

    pub fn decrease_release(&mut self) {
        self.release = self.release.saturating_sub(1);
    }

    // Helper methods to convert 0-99 values to 0.0-1.0 range for calculations
    pub fn attack_normalized(&self) -> f32 {
        self.attack as f32 / 99.0
    }

    pub fn decay_normalized(&self) -> f32 {
        self.decay as f32 / 99.0
    }

    pub fn sustain_normalized(&self) -> f32 {
        self.sustain as f32 / 99.0
    }

    pub fn release_normalized(&self) -> f32 {
        self.release as f32 / 99.0
    }

    /// Calculate ADSR envelope amplitude at a given time since note start
    pub fn calculate_adsr_amplitude(&self, time_since_start: f32, is_key_pressed: bool, time_since_release: Option<f32>) -> f32 {
        if let Some(release_time) = time_since_release {
            // Release phase
            let release_duration = self.release_normalized() * 2.0; // Scale to 2 seconds max
            if release_duration == 0.0 {
                return 0.0;
            }
            let release_progress = (release_time / release_duration).min(1.0);
            return self.sustain_normalized() * (1.0 - release_progress);
        }

        if !is_key_pressed {
            return 0.0;
        }

        let attack_duration = self.attack_normalized() * 2.0; // Scale to 2 seconds max
        let decay_duration = self.decay_normalized() * 2.0;

        if time_since_start <= attack_duration {
            // Attack phase
            if attack_duration == 0.0 {
                return 1.0;
            }
            return time_since_start / attack_duration;
        } else if time_since_start <= attack_duration + decay_duration {
            // Decay phase
            if decay_duration == 0.0 {
                return self.sustain_normalized();
            }
            let decay_time = time_since_start - attack_duration;
            let decay_progress = decay_time / decay_duration;
            return 1.0 - (1.0 - self.sustain_normalized()) * decay_progress;
        } else {
            // Sustain phase
            return self.sustain_normalized();
        }
    }
}