use minifb::{Key, KeyRepeat, Window};
use rodio::Sink;
use crate::state::State;
use super::super::InputCommand;

/// Command for toggling waveform types
pub struct WaveformToggleCommand;

impl InputCommand for WaveformToggleCommand {
    fn execute(&self, state: &mut State, window: &mut Window, sink: &mut Sink) {
        if window.is_key_pressed(Key::Tab, KeyRepeat::No) {
            state.toggle_current_track_waveform();
        }
    }
}