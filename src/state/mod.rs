use std::time::{Duration, Instant};

use minifb::Key;

use crate::graphics::constants::{WAVEFORM_SAWTOOTH, WAVEFORM_SINE, WAVEFORM_SQUARE, WAVEFORM_TRIANGLE};
use crate::music_theory::{OCTAVE_LOWER_BOUND, OCTAVE_UPPER_BOUND};
use crate::music_theory::note::Note;
use crate::waveforms::Waveform;
use crate::effects::{DelayEffect, ReverbEffect, FlangerEffect};

// DAW Track System
#[derive(Debug, Clone)]
pub struct Track {
    pub id: usize,
    pub name: String,
    pub recorded_notes: Vec<RecordedNote>,
    pub volume: f32,        // 0.0 - 1.0
    pub pan: f32,           // -1.0 (left) to 1.0 (right)
    pub playing: bool,      // Whether this track's loop is currently playing
    pub waveform: Waveform,
    pub octave: i32,
    // Track-specific effects
    pub delay_enabled: bool,
    pub reverb_enabled: bool,
    pub flanger_enabled: bool,
    pub delay_effect: DelayEffect,
    pub reverb_effect: ReverbEffect,
    pub flanger_effect: FlangerEffect,
    // Track-specific ADSR
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
}

impl Track {
    pub fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            recorded_notes: Vec::new(),
            volume: 0.8,
            pan: 0.0,
            playing: false,
            waveform: Waveform::SQUARE,
            octave: 4,
            delay_enabled: false,
            reverb_enabled: false,
            flanger_enabled: false,
            delay_effect: DelayEffect::new(300.0, 0.55, 0.5, 44100),
            reverb_effect: ReverbEffect::new(0.7, 0.4, 0.6, 44100),
            flanger_effect: FlangerEffect::new(0.5, 0.7, 0.1, 0.5, 44100),
            attack: 0,
            decay: 0,
            sustain: 50,
            release: 20,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MasterTrack {
    pub volume: f32,        // Master volume 0.0 - 1.0
    pub delay_enabled: bool,
    pub reverb_enabled: bool,
    pub flanger_enabled: bool,
    pub delay_effect: DelayEffect,
    pub reverb_effect: ReverbEffect,
    pub flanger_effect: FlangerEffect,
}

impl MasterTrack {
    pub fn new() -> Self {
        Self {
            volume: 0.9,
            delay_enabled: false,
            reverb_enabled: false,
            flanger_enabled: false,
            delay_effect: DelayEffect::new(400.0, 0.4, 0.3, 44100),
            reverb_effect: ReverbEffect::new(0.8, 0.3, 0.4, 44100),
            flanger_effect: FlangerEffect::new(0.3, 0.5, 0.05, 0.3, 44100),
        }
    }
}

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

// DAW State Struct - Multi-track Digital Audio Workstation
pub struct State {
    // DAW Core
    pub tracks: Vec<Track>,          // 4 individual tracks
    pub master_track: MasterTrack,   // Master mix bus
    pub current_track_id: usize,     // Currently selected track (0-3)
    
    // Legacy single-track compatibility (will be removed later)
    pub(crate) octave: i32,
    pub(crate) waveform: Waveform,
    pub(crate) pressed_key: Option<(Key, Note)>,
    waveform_sprite_index: usize,
    pub(crate) filter_factor: f32,
    pub(crate) lpf_active: usize,
    pub(crate) current_frequency: Option<f32>, // Track current playing frequency
    pub(crate) animation_start_time: Instant, // When the animation started
    pub(crate) key_release_time: Option<Instant>, // When the key was released for fade-out
    
    // Legacy ADSR (will use track-specific ADSR later)
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
    
    // Recording state
    pub recording_state: RecordingState,
    pub recorded_notes: Vec<RecordedNote>, // Legacy - will use track-specific
    pub visual_notes: Vec<VisualNote>,
    pub recording_start_time: Option<Instant>,
    pub playback_start_time: Option<Instant>,
    pub current_note_start: Option<(Instant, Note, i32)>, // (start_time, note, octave)
    
    // Mouse state
    pub mouse: MouseState,
    
    // Stop button feedback
    pub stop_button_glow_time: Option<Instant>,
    
    // Audio effects
    pub delay_enabled: bool,
    pub reverb_enabled: bool,
    pub flanger_enabled: bool,
    pub delay_effect: DelayEffect,
    pub reverb_effect: ReverbEffect,
    pub flanger_effect: FlangerEffect,
}

// Initialize DAW State
impl State {
    pub(crate) fn new() -> Self {
        // Create 4 tracks with different default settings
        let tracks = vec![
            Track::new(0, "Lead".to_string()),
            Track::new(1, "Bass".to_string()),
            Track::new(2, "Drums".to_string()),
            Track::new(3, "Pads".to_string()),
        ];
        
        State {
            // DAW Core initialization
            tracks,
            master_track: MasterTrack::new(),
            current_track_id: 0, // Start with track 0 (Lead)
            octave: 4, // Set default octave to 4
            waveform: Waveform::SQUARE, // Set default waveform to Square
            pressed_key: None, // Default is no key
            waveform_sprite_index: WAVEFORM_SQUARE, // Set default waveform sprite index to Square
            filter_factor: 1.0, // Set default cutoff to 1.0
            lpf_active: 0, // Default for LPF is deactivated
            current_frequency: None, // No frequency being played initially
            animation_start_time: Instant::now(), // Initialize animation time
            key_release_time: None, // No key released initially
            // ADSR defaults for pluck-like instant sound
            attack: 0,   // Instant attack (no delayed PAD effect)
            decay: 0,    // No decay (sounds does not fade to sustain level)
            sustain: 50, // Half sustain level (sound stays at half volume while key held)
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
            
            // Audio effects defaults
            delay_enabled: false,
            reverb_enabled: false,
            flanger_enabled: false,
            delay_effect: DelayEffect::new(300.0, 0.55, 0.5, 44100), // 300ms delay, 55% feedback, 50% mix
            reverb_effect: ReverbEffect::new(0.7, 0.4, 0.6, 44100), // Large room, light damping, 60% mix  
            flanger_effect: FlangerEffect::new(0.5, 0.7, 0.1, 0.5, 44100), // 0.5Hz LFO, 70% depth, 10% feedback, 50% mix
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
    
    // === DAW TRACK MANAGEMENT METHODS ===
    
    /// Switch to a specific track (0-3)
    pub fn switch_to_track(&mut self, track_id: usize) {
        if track_id < self.tracks.len() {
            self.current_track_id = track_id;
        }
    }
    
    /// Get the currently active track
    pub fn current_track(&self) -> &Track {
        &self.tracks[self.current_track_id]
    }
    
    /// Get the currently active track (mutable)
    pub fn current_track_mut(&mut self) -> &mut Track {
        &mut self.tracks[self.current_track_id]
    }
    
    /// Toggle playback on current track
    pub fn toggle_current_track_playback(&mut self) {
        self.tracks[self.current_track_id].playing = !self.tracks[self.current_track_id].playing;
    }
    
    /// Stop all track playback
    pub fn stop_all_track_playback(&mut self) {
        for track in &mut self.tracks {
            track.playing = false;
        }
    }
    
    /// Adjust volume of current track
    pub fn adjust_current_track_volume(&mut self, delta: f32) {
        let track = &mut self.tracks[self.current_track_id];
        track.volume = (track.volume + delta).clamp(0.0, 1.0);
    }
    
    /// Adjust pan of current track
    pub fn adjust_current_track_pan(&mut self, delta: f32) {
        let track = &mut self.tracks[self.current_track_id];
        track.pan = (track.pan + delta).clamp(-1.0, 1.0);
    }
    
    /// Get list of tracks that are currently playing
    pub fn playing_tracks(&self) -> Vec<usize> {
        self.tracks.iter()
            .enumerate()
            .filter(|(_, track)| track.playing && !track.recorded_notes.is_empty())
            .map(|(i, _)| i)
            .collect()
    }
    
    /// Check if any tracks are currently playing
    pub fn has_playing_tracks(&self) -> bool {
        self.tracks.iter().any(|track| track.playing && !track.recorded_notes.is_empty())
    }
    
    /// Start recording on current track
    pub fn start_track_recording(&mut self) {
        self.recording_state = RecordingState::Recording;
        self.recording_start_time = Some(Instant::now());
        // Clear current track's recorded notes
        self.tracks[self.current_track_id].recorded_notes.clear();
        self.current_note_start = None;
    }
    
    /// Add recorded note to current track
    pub fn add_note_to_current_track(&mut self, note: RecordedNote) {
        self.tracks[self.current_track_id].recorded_notes.push(note);
    }
    
    // === TRACK-SPECIFIC ADSR CONTROLS ===
    
    /// Increase attack on current track
    pub fn increase_current_track_attack(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.attack = (track.attack + 1).min(99);
        // Sync with legacy state
        self.attack = track.attack;
    }
    
    /// Decrease attack on current track
    pub fn decrease_current_track_attack(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.attack = track.attack.saturating_sub(1);
        // Sync with legacy state
        self.attack = track.attack;
    }
    
    /// Increase decay on current track
    pub fn increase_current_track_decay(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.decay = (track.decay + 1).min(99);
        // Sync with legacy state
        self.decay = track.decay;
    }
    
    /// Decrease decay on current track
    pub fn decrease_current_track_decay(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.decay = track.decay.saturating_sub(1);
        // Sync with legacy state
        self.decay = track.decay;
    }
    
    /// Increase sustain on current track
    pub fn increase_current_track_sustain(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.sustain = (track.sustain + 1).min(99);
        // Sync with legacy state
        self.sustain = track.sustain;
    }
    
    /// Decrease sustain on current track
    pub fn decrease_current_track_sustain(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.sustain = track.sustain.saturating_sub(1);
        // Sync with legacy state
        self.sustain = track.sustain;
    }
    
    /// Increase release on current track
    pub fn increase_current_track_release(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.release = (track.release + 1).min(99);
        // Sync with legacy state
        self.release = track.release;
    }
    
    /// Decrease release on current track
    pub fn decrease_current_track_release(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.release = track.release.saturating_sub(1);
        // Sync with legacy state
        self.release = track.release;
    }
    
    // === TRACK-SPECIFIC OCTAVE CONTROLS ===
    
    /// Increase octave on current track
    pub fn increase_current_track_octave(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        if track.octave < OCTAVE_UPPER_BOUND {
            track.octave += 1;
            // Sync with legacy state
            self.octave = track.octave;
        }
    }
    
    /// Decrease octave on current track
    pub fn decrease_current_track_octave(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        if track.octave > OCTAVE_LOWER_BOUND {
            track.octave -= 1;
            // Sync with legacy state
            self.octave = track.octave;
        }
    }
    
    // === TRACK-SPECIFIC EFFECTS CONTROLS ===
    
    /// Toggle delay on current track
    pub fn toggle_current_track_delay(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.delay_enabled = !track.delay_enabled;
        // Sync with legacy state
        self.delay_enabled = track.delay_enabled;
    }
    
    /// Toggle reverb on current track
    pub fn toggle_current_track_reverb(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.reverb_enabled = !track.reverb_enabled;
        // Sync with legacy state
        self.reverb_enabled = track.reverb_enabled;
    }
    
    /// Toggle flanger on current track
    pub fn toggle_current_track_flanger(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.flanger_enabled = !track.flanger_enabled;
        // Sync with legacy state
        self.flanger_enabled = track.flanger_enabled;
    }
    
    /// Toggle waveform on current track
    pub fn toggle_current_track_waveform(&mut self) {
        let track = &mut self.tracks[self.current_track_id];
        track.waveform = match track.waveform {
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
        // Sync with legacy state
        self.waveform = track.waveform.clone();
    }
}