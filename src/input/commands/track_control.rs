use minifb::{Key, Window};
use rodio::Sink;
use crate::state::State;
use super::super::InputCommand;

/// Command for handling track selection and control
pub struct TrackControlCommand {
    action: TrackAction,
}

#[derive(Debug, Clone)]
pub enum TrackAction {
    SwitchToTrack(usize),
    ToggleMute,
    ToggleSolo,
    VolumeUp,
    VolumeDown,
    PanLeft,
    PanRight,
}

impl TrackControlCommand {
    pub fn new(action: TrackAction) -> Self {
        Self { action }
    }
}

impl InputCommand for TrackControlCommand {
    fn execute(&self, state: &mut State, _window: &mut Window, _sink: &mut Sink) {
        match &self.action {
            TrackAction::SwitchToTrack(track_id) => {
                state.switch_to_track(*track_id);
                
                // Update legacy state to match current track (avoid borrowing)
                let current_track_id = state.current_track_id;
                let track = &state.tracks[current_track_id];
                // state.octave = track.octave;
                state.waveform = track.waveform.clone();
                state.attack = track.attack;
                state.decay = track.decay;
                state.sustain = track.sustain;
                state.release = track.release;
                
                println!("Switched to track {}: {}", track_id, track.name);
            },
            TrackAction::ToggleMute => {
                // state.toggle_current_track_mute();
                let current_track_id = state.current_track_id;
                let track = &state.tracks[current_track_id];
                // println!("Track {} ({}) mute: {}", track.id, track.name, track.muted);
            },
            TrackAction::ToggleSolo => {
                state.current_track();
                let current_track_id = state.current_track_id;
                let track = &state.tracks[current_track_id];
                // println!("Track {} ({}) solo: {}", track.id, track.name, track.soloed);
            },
            TrackAction::VolumeUp => {
                state.adjust_current_track_volume(0.1);
                let current_track_id = state.current_track_id;
                let track = &state.tracks[current_track_id];
                println!("Track {} volume: {:.1}%", track.id, track.volume * 100.0);
            },
            TrackAction::VolumeDown => {
                state.adjust_current_track_volume(-0.1);
                let current_track_id = state.current_track_id;
                let track = &state.tracks[current_track_id];
                println!("Track {} volume: {:.1}%", track.id, track.volume * 100.0);
            },
            TrackAction::PanLeft => {
                state.adjust_current_track_pan(-0.2);
                let current_track_id = state.current_track_id;
                let track = &state.tracks[current_track_id];
                let pan_desc = if track.pan < -0.1 { "Left" } 
                              else if track.pan > 0.1 { "Right" } 
                              else { "Center" };
                println!("Track {} pan: {} ({:.1})", track.id, pan_desc, track.pan);
            },
            TrackAction::PanRight => {
                state.adjust_current_track_pan(0.2);
                let current_track_id = state.current_track_id;
                let track = &state.tracks[current_track_id];
                let pan_desc = if track.pan < -0.1 { "Left" } 
                              else if track.pan > 0.1 { "Right" } 
                              else { "Center" };
                println!("Track {} pan: {} ({:.1})", track.id, pan_desc, track.pan);
            },
        }
    }
}