use minifb::{Key, KeyRepeat, Window};
use rodio::Sink;
use crate::state::State;
use crate::music_theory::{OCTAVE_LOWER_BOUND, OCTAVE_UPPER_BOUND};
use super::super::InputCommand;

/// Command for adjusting octave up or down
pub struct OctaveAdjustCommand {
    increase: bool,
}

impl OctaveAdjustCommand {
    pub fn new(increase: bool) -> Self {
        Self { increase }
    }
}

impl InputCommand for OctaveAdjustCommand {
    fn execute(&self, state: &mut State, window: &mut Window, sink: &mut Sink) {
        let key = if self.increase { Key::F2 } else { Key::F1 };
        
        if window.is_key_pressed(key, KeyRepeat::No) {
            if self.increase && state.get_current_octave() < OCTAVE_UPPER_BOUND {
                state.increase_octave();
            } else if !self.increase && state.get_current_octave() > OCTAVE_LOWER_BOUND {
                state.decrease_octave();
            }
        }
    }
}