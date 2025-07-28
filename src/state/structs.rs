use std::time::{Duration, Instant};
use minifb::{Key, Window};
use crate::graphics::constants::{WAVEFORM_SINE, WAVEFORM_SQUARE, WAVEFORM_TRIANGLE, WAVEFORM_SAWTOOTH};
use crate::graphics::sprites::SpriteMaps;
use crate::music_theory::note::Note;
use crate::music_theory::{OCTAVE_LOWER_BOUND, OCTAVE_UPPER_BOUND};
use crate::waveforms::Waveform;

pub struct Camera {
    pub x: f32,
    pub y: f32,
}

impl Camera {
    pub fn new(x: f32, y: f32) -> Self {
        Camera {
            x,
            y
        }
    }
}

pub struct GraphicsState<'a> {
    pub camera: Camera, // Camera object
    pub sprites: SpriteMaps, // Sprite maps
    pub window_buffer: &'a mut Vec<u32>, // Window buffer
    pub window_width: usize, // Width of the window
    pub window_height: usize, // Height of the window
    pub window: Option<&'a mut Window>, // Optional window object
    pub scaled_buffer: &'a mut Vec<u32>, // Scaled buffer
    pub art_width: usize, // Width of the game world
    pub art_height: usize, // Height of the game world
}

pub const FRAME_DURATION: Duration = Duration::from_millis(16); // Approximately 60Hz refresh rate

// Synthesizer State Struct
pub struct SynthState {
    pub(crate) octave: i32,
    pub(crate) waveform: Waveform,
    pub(crate) pressed_key: Option<(Key, Note)>,
    waveform_sprite_index: usize,
    pub(crate) filter_factor: f32,
    pub(crate) lpf_active: usize,
    pub(crate) current_frequency: Option<f32>, // Track current playing frequency
}

// Initialize Synthesizer State
impl SynthState {
    pub  fn new() -> Self {
        SynthState {
            octave: 4, // Set default octave to 4
            waveform: Waveform::SINE, // Set default waveform to Sine
            pressed_key: None, // Default is no key
            waveform_sprite_index: WAVEFORM_SINE, // Set default waveform sprite index to Sine
            filter_factor: 1.0, // Set default cutoff to 1.0
            lpf_active: 0, // Default for LPF is deactivated
            current_frequency: None, // No frequency being played initially
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

    /// Cycles through all waveforms (SINE -> SQUARE -> TRIANGLE -> SAWTOOTH -> SINE) and sets the associated sprite index accordingly.
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
            },
        };
    }
}