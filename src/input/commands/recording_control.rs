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

/// Handle playback of recorded notes during playback mode
pub fn handle_playback(state: &mut State, sink: &mut Sink) {
    if state.recording_state != crate::state::RecordingState::Playing {
        return;
    }

    let Some(playback_start) = state.playback_start_time else {
        return;
    };

    if state.recorded_notes.is_empty() {
        return;
    }

    let current_time = playback_start.elapsed().as_secs_f32();

    // Clone the recorded notes to avoid borrowing issues
    let recorded_notes = state.recorded_notes.clone();

    // Find the total duration of the recording
    let max_end_time = recorded_notes.iter()
        .map(|note| note.timestamp + note.duration)
        .fold(0.0f32, f32::max);

    // Loop the playback - restart if we've reached the end
    let loop_time = if max_end_time > 0.0 {
        current_time % max_end_time
    } else {
        current_time
    };

    // Find notes that should start playing now (within a small time window)
    static mut LAST_LOOP_TIME: f32 = -1.0;
    let frame_time_threshold = 0.05; // 50ms threshold for frame timing

    unsafe {
        // Check if we've looped back to the beginning
        if loop_time < LAST_LOOP_TIME {
            LAST_LOOP_TIME = -1.0; // Reset to catch notes at the beginning of the loop
        }

        for recorded_note in &recorded_notes {
            let note_start = recorded_note.timestamp;

            // Check if this note should start playing now
            // Either: 1) We just crossed the note start time, or 2) We're at the beginning of a new loop
            let should_trigger = (LAST_LOOP_TIME < note_start && loop_time >= note_start) ||
                (LAST_LOOP_TIME < 0.0 && loop_time >= note_start && loop_time < note_start + frame_time_threshold);

            if should_trigger {
                // Store note and octave to play
                let note_to_play = recorded_note.note;
                let octave_to_use = recorded_note.octave;

                // Set the octave temporarily for playback
                let original_octave = state.octave;
                state.octave = octave_to_use;

                // Play the note
                handle_musical_note(state, sink, note_to_play);
                state.pressed_key = Some((Key::Q, note_to_play)); // Use Q as placeholder key for playback

                // Restore original octave
                state.octave = original_octave;
            }
        }

        LAST_LOOP_TIME = loop_time;
    }
}