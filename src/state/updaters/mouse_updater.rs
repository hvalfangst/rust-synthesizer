use crate::state::State;

/// Handles mouse state updates
pub struct MouseStateUpdater;

impl MouseStateUpdater {
    pub fn new() -> Self {
        Self
    }
    
    /// Update mouse-related state
    pub fn update(&self, state: &mut State) {
        // Handle mouse state cleanup and validation
        self.validate_mouse_state(state);
        
        // Handle drag state consistency
        self.update_drag_state(state);
    }
    
    /// Validate mouse state consistency
    fn validate_mouse_state(&self, state: &mut State) {
        // Ensure drag state is consistent with button state
        if !state.mouse.left_pressed {
            state.mouse.dragging = false;
            state.mouse.drag_start = None;
        }
    }
    
    /// Update dragging state based on mouse movement
    fn update_drag_state(&self, state: &mut State) {
        // Drag detection is handled in the mouse command
        // This could be expanded for more complex mouse state management
        
        // Reset click state after processing
        if state.mouse.left_clicked {
            // Click state should be consumed by the mouse command
            // This prevents multiple commands from processing the same click
        }
    }
}