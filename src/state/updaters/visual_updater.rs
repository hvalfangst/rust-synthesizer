use crate::state::State;

/// Handles visual-related state updates
pub struct VisualStateUpdater;

impl VisualStateUpdater {
    pub fn new() -> Self {
        Self
    }
    
    /// Update visual state elements
    pub fn update(&self, state: &mut State) {
        // Update visual notes for recording playback display
        state.update_visual_notes();
        
        // Update stop button glow effect
        self.update_stop_button_glow(state);
    }
    
    /// Update stop button glow timing
    fn update_stop_button_glow(&self, state: &mut State) {
        // The glow effect is handled in the drawing code based on the timestamp
        // This could be expanded for more complex visual effects
        if let Some(glow_start) = state.stop_button_glow_time {
            // Clear glow after a reasonable time to prevent memory leaks
            if glow_start.elapsed().as_secs_f32() > 1.0 {
                // Keep the timestamp for drawing logic, but could add cleanup here if needed
            }
        }
    }
}