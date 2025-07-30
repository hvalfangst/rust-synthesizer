use crate::state::{State, RecordingState};

/// Handles recording and playback state updates
pub struct RecordingStateUpdater;

impl RecordingStateUpdater {
    pub fn new() -> Self {
        Self
    }
    
    /// Update recording-related state
    pub fn update(&self, state: &mut State) {
        // Handle recording state transitions and cleanup
        self.handle_recording_cleanup(state);
        
        // Update playback timing if needed
        self.handle_playback_timing(state);
    }
    
    /// Handle cleanup of recording state
    fn handle_recording_cleanup(&self, state: &mut State) {
        // Finish any held notes when stopping recording
        if state.recording_state == RecordingState::Stopped && state.current_note_start.is_some() {
            if let Some((start_time, note, octave)) = state.current_note_start.take() {
                let duration = start_time.elapsed().as_secs_f32();
                let timestamp = state.recording_start_time
                    .map(|start| start.elapsed().as_secs_f32() - duration)
                    .unwrap_or(0.0);
                
                state.recorded_notes.push(crate::state::RecordedNote {
                    note,
                    octave,
                    timestamp,
                    duration,
                });
            }
        }
    }
    
    /// Handle playback timing updates
    fn handle_playback_timing(&self, state: &mut State) {
        // Playback timing is handled in the main playback function
        // This could be expanded for more complex playback state management
    }
}