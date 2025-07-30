use std::time::{Duration, Instant};

use minifb::Key;

use crate::graphics::constants::{WAVEFORM_SAWTOOTH, WAVEFORM_SINE, WAVEFORM_SQUARE, WAVEFORM_TRIANGLE};
use crate::music_theory::{OCTAVE_LOWER_BOUND, OCTAVE_UPPER_BOUND};
use crate::music_theory::note::Note;
use crate::waveforms::Waveform;

// Recording structures
#[derive(Debug, Clone)]
pub struct RecordedNote {
    pub note: Note,
    pub octave: i32,
    pub timestamp: f32, // Time in seconds from recording start
    pub duration: f32,  // How long the note was held
}

#[derive(Debug, Clone)]
pub struct VisualNote {
    pub note: Note,
    pub octave: i32,
    pub spawn_time: Instant,
    pub fade_start_time: Option<Instant>,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordingState {
    Stopped,
    Recording,
    Playing,
}

#[derive(Debug, Clone)]
pub struct MouseState {
    pub x: f32,
    pub y: f32,
    pub left_pressed: bool,
    pub left_clicked: bool,
    pub dragging: bool,
    pub drag_start: Option<(f32, f32)>,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            left_pressed: false,
            left_clicked: false,
            dragging: false,
            drag_start: None,
        }
    }
}

pub mod event_loop;
pub mod utils;
pub mod updaters;

const FRAME_DURATION: Duration = Duration::from_millis(16); // Approximately 60Hz refresh rate

// Synthesizer State Struct
pub struct State {
    pub(crate) octave: i32,
    pub(crate) waveform: Waveform,
    pub(crate) pressed_key: Option<(Key, Note)>,
    waveform_sprite_index: usize,
    pub(crate) filter_factor: f32,
    pub(crate) lpf_active: usize,
    pub(crate) current_frequency: Option<f32>, // Track current playing frequency
    pub(crate) animation_start_time: Instant, // When the animation started
    pub(crate) key_release_time: Option<Instant>, // When the key was released for fade-out
    // ADSR parameters (0 to 99)
    pub attack: u8,  // Attack time (0 = instant, 99 = 2 seconds)
    pub decay: u8,   // Decay time (0 = instant, 99 = 2 seconds)
    pub sustain: u8, // Sustain level (0 = silent, 99 = full volume)
    pub release: u8, // Release time (0 = instant, 99 = 2 seconds)
    
    // Recording state
    pub recording_state: RecordingState,
    pub recorded_notes: Vec<RecordedNote>,
    pub visual_notes: Vec<VisualNote>,
    pub recording_start_time: Option<Instant>,
    pub playback_start_time: Option<Instant>,
    pub current_note_start: Option<(Instant, Note, i32)>, // (start_time, note, octave)
    
    // Mouse state
    pub mouse: MouseState,
    
    // Stop button feedback
    pub stop_button_glow_time: Option<Instant>,
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
            
            // Recording state defaults
            recording_state: RecordingState::Stopped,
            recorded_notes: Vec::new(),
            visual_notes: Vec::new(),
            recording_start_time: None,
            playback_start_time: None,
            current_note_start: None,
            
            // Mouse state defaults
            mouse: MouseState::new(),
            
            // Stop button feedback defaults
            stop_button_glow_time: None,
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

    // Recording control methods
    pub fn start_recording(&mut self) {
        self.recording_state = RecordingState::Recording;
        self.recording_start_time = Some(Instant::now());
        self.recorded_notes.clear();
        self.current_note_start = None;
    }

    pub fn stop_recording(&mut self) {
        // Finish any currently held note
        if let Some((start_time, note, octave)) = self.current_note_start.take() {
            let duration = start_time.elapsed().as_secs_f32();
            let timestamp = self.recording_start_time
                .map(|start| start.elapsed().as_secs_f32() - duration)
                .unwrap_or(0.0);
            
            self.recorded_notes.push(RecordedNote {
                note,
                octave,
                timestamp,
                duration,
            });
        }
        
        self.recording_state = RecordingState::Stopped;
        self.recording_start_time = None;
    }

    pub fn start_playback(&mut self) {
        if !self.recorded_notes.is_empty() {
            self.recording_state = RecordingState::Playing;
            self.playback_start_time = Some(Instant::now());
        }
    }

    pub fn stop_playback(&mut self) {
        self.recording_state = RecordingState::Stopped;
        self.playback_start_time = None;
    }

    pub fn add_visual_note(&mut self, note: Note, octave: i32) {
        // Position notes in a flowing pattern across the screen
        let note_index = self.visual_notes.len() as f32;
        let x = 100.0 + (note_index * 60.0) % 400.0;
        let y = 50.0 + ((note_index * 30.0) % 150.0);
        
        self.visual_notes.push(VisualNote {
            note,
            octave,
            spawn_time: Instant::now(),
            fade_start_time: None,
            x,
            y,
        });
    }

    pub fn update_visual_notes(&mut self) {
        // Start fade for old notes (after 2 seconds)
        let now = Instant::now();
        for visual_note in &mut self.visual_notes {
            if visual_note.fade_start_time.is_none() && now.duration_since(visual_note.spawn_time).as_secs_f32() > 2.0 {
                visual_note.fade_start_time = Some(now);
            }
        }

        // Remove fully faded notes (after 1 second fade)
        self.visual_notes.retain(|note| {
            if let Some(fade_start) = note.fade_start_time {
                now.duration_since(fade_start).as_secs_f32() < 1.0
            } else {
                true
            }
        });
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