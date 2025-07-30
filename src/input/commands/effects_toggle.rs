use minifb::{Key, KeyRepeat, Window};
use rodio::Sink;
use crate::effects::AudioEffect;
use crate::state::State;
use super::super::InputCommand;

/// Command for toggling audio effects
pub struct EffectsToggleCommand {
    effect_type: EffectType,
}

#[derive(Debug, Clone, Copy)]
enum EffectType {
    Delay,
    Reverb,
    Flanger,
}

impl EffectsToggleCommand {
    pub fn new_delay() -> Self {
        Self { effect_type: EffectType::Delay }
    }
    
    pub fn new_reverb() -> Self {
        Self { effect_type: EffectType::Reverb }
    }
    
    pub fn new_flanger() -> Self {
        Self { effect_type: EffectType::Flanger }
    }
}

impl InputCommand for EffectsToggleCommand {
    fn execute(&self, state: &mut State, window: &mut Window, sink: &mut Sink) {
        let key = match self.effect_type {
            EffectType::Delay => Key::F10,
            EffectType::Reverb => Key::F11,
            EffectType::Flanger => Key::F12,
        };
        
        if window.is_key_pressed(key, KeyRepeat::No) {
            match self.effect_type {
                EffectType::Delay => {
                    state.toggle_current_track_delay();
                    let current_track_id = state.current_track_id;
                    if !state.tracks[current_track_id].delay_enabled {
                        state.tracks[current_track_id].delay_effect.reset();
                    }
                },
                EffectType::Reverb => {
                    state.toggle_current_track_reverb();
                    let current_track_id = state.current_track_id;
                    if !state.tracks[current_track_id].reverb_enabled {
                        state.tracks[current_track_id].reverb_effect.reset();
                    }
                },
                EffectType::Flanger => {
                    state.toggle_current_track_flanger();
                    let current_track_id = state.current_track_id;
                    if !state.tracks[current_track_id].flanger_enabled {
                        state.tracks[current_track_id].flanger_effect.reset();
                    }
                },
            }
        }
    }
}