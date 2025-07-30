use minifb::{Key, KeyRepeat, Window};
use rodio::Sink;
use crate::state::State;
use super::super::InputCommand;

/// Command for controlling ADSR parameters
pub struct ADSRControlCommand {
    parameter: ADSRParameter,
    increase: bool,
}

#[derive(Debug, Clone, Copy)]
enum ADSRParameter {
    Attack,
    Decay,
    Sustain,
    Release,
}

impl ADSRControlCommand {
    pub fn new_attack(increase: bool) -> Self {
        Self { parameter: ADSRParameter::Attack, increase }
    }
    
    pub fn new_decay(increase: bool) -> Self {
        Self { parameter: ADSRParameter::Decay, increase }
    }
    
    pub fn new_sustain(increase: bool) -> Self {
        Self { parameter: ADSRParameter::Sustain, increase }
    }
    
    pub fn new_release(increase: bool) -> Self {
        Self { parameter: ADSRParameter::Release, increase }
    }
}

impl InputCommand for ADSRControlCommand {
    fn execute(&self, state: &mut State, window: &mut Window, sink: &mut Sink) {
        let key = match (self.parameter, self.increase) {
            (ADSRParameter::Attack, false) => Key::F3,
            (ADSRParameter::Attack, true) => Key::F4,
            (ADSRParameter::Decay, false) => Key::F5,
            (ADSRParameter::Decay, true) => Key::F6,
            (ADSRParameter::Sustain, false) => Key::F7,
            (ADSRParameter::Sustain, true) => Key::F8,
            (ADSRParameter::Release, false) => Key::F9,
            (ADSRParameter::Release, true) => Key::Key0,
        };
        
        if window.is_key_pressed(key, KeyRepeat::Yes) {
            match (self.parameter, self.increase) {
                (ADSRParameter::Attack, true) => state.increase_current_track_attack(),
                (ADSRParameter::Attack, false) => state.decrease_current_track_attack(),
                (ADSRParameter::Decay, true) => state.increase_current_track_decay(),
                (ADSRParameter::Decay, false) => state.decrease_current_track_decay(),
                (ADSRParameter::Sustain, true) => state.increase_current_track_sustain(),
                (ADSRParameter::Sustain, false) => state.decrease_current_track_sustain(),
                (ADSRParameter::Release, true) => state.increase_current_track_release(),
                (ADSRParameter::Release, false) => state.decrease_current_track_release(),
            }
        }
    }
}