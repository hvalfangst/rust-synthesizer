use minifb::{Key, Window};
use rodio::Sink;
use crate::state::State;
use crate::state::utils::{handle_musical_note};
use super::super::InputCommand;

/// Command for handling recording and playback controls
pub struct RecordingControlCommand;

impl InputCommand for RecordingControlCommand {
    fn execute(&self, state: &mut State, window: &mut Window, sink: &mut Sink) {
        // Handle playback logic
        handle_playback(state, sink);
        
        // Handle key release timing and fade effects
        let mut key_pressed = false;
        
        // Check if any musical key is currently pressed
        if let Some(_) = state.pressed_key {
            for (key, _, _, _) in crate::state::utils::get_key_mappings() {
                if window.is_key_pressed(key, minifb::KeyRepeat::No) {
                    key_pressed = true;
                    break;
                }
            }
        }
        
        // If no musical key is pressed, handle key release based on ADSR settings
        if !key_pressed && state.pressed_key.is_some() && state.key_release_time.is_none() {
            // For very quick release settings (0-10), stop immediately
            if state.release <= 10 {
                sink.stop(); // Immediate stop for instant release
            }
            // For other settings, let ADSR envelope handle the release naturally
            // The ADSR envelope will auto-release after max_sustain_samples 
            state.key_release_time = Some(std::time::Instant::now());
        }
        
        // Clear visual display quickly after audio has stopped
        if let Some(release_time) = state.key_release_time {
            let visual_clear_time = (state.release_normalized() * 2.0).max(0.1); // Minimum 100ms for visual feedback
            if release_time.elapsed().as_secs_f32() > visual_clear_time {
                state.current_frequency = None;
                state.key_release_time = None;
            }
        }
    }
}

/// Handle multi-track playback of recorded loops during playback mode
pub fn handle_playback(state: &mut State, sink: &mut Sink) {
    if state.recording_state != crate::state::RecordingState::Playing {
        return;
    }

    let Some(playback_start) = state.playback_start_time else {
        return;
    };

    let current_time = playback_start.elapsed().as_secs_f32();
    
    // Get all tracks that are currently set to playing
    let playing_tracks = state.playing_tracks();
    
    // Check if any playing tracks have recorded notes
    let has_recorded_content = playing_tracks.iter()
        .any(|&track_id| !state.tracks[track_id].recorded_notes.is_empty());
        
    if !has_recorded_content {
        return;
    }

    // Find the maximum loop duration across all playing tracks
    let max_loop_duration = playing_tracks.iter()
        .map(|&track_id| {
            let track = &state.tracks[track_id];
            if track.recorded_notes.is_empty() {
                0.0
            } else {
                track.recorded_notes.iter()
                    .map(|note| note.timestamp + note.duration)
                    .fold(0.0f32, f32::max)
            }
        })
        .fold(0.0f32, f32::max);

    // Calculate loop time
    let loop_time = if max_loop_duration > 0.0 {
        current_time % max_loop_duration
    } else {
        current_time
    };

    // Track timing for note triggering
    static mut LAST_LOOP_TIME: f32 = -1.0;
    let frame_time_threshold = 0.05; // 50ms threshold for frame timing

    unsafe {
        // Check if we've looped back to the beginning
        if loop_time < LAST_LOOP_TIME {
            LAST_LOOP_TIME = -1.0; // Reset to catch notes at the beginning of the loop
        }

        // Play notes from all playing tracks
        for &track_id in &playing_tracks {
            let track = &state.tracks[track_id];
            
            for recorded_note in &track.recorded_notes {
                let note_start = recorded_note.timestamp;

                // Check if this note should start playing now
                let should_trigger = (LAST_LOOP_TIME < note_start && loop_time >= note_start) ||
                    (LAST_LOOP_TIME < 0.0 && loop_time >= note_start && loop_time < note_start + frame_time_threshold);

                if should_trigger {
                    // Create mixer and play note on this specific track
                    let mixer = crate::audio::MultiTrackMixer::new(44100);
                    mixer.play_note_on_track(track, recorded_note.note, sink);
                    
                    // Set visual feedback for any playing track
                    state.pressed_key = Some((Key::Q, recorded_note.note));
                    state.current_frequency = Some(recorded_note.note.frequency(recorded_note.octave));
                    state.animation_start_time = std::time::Instant::now();
                }
            }
        }

        LAST_LOOP_TIME = loop_time;
    }
}