use std::collections::HashMap;
use std::sync::Arc;
use minifb::{Key, Window};
use rodio::Sink;

use crate::state::State;
use super::{InputCommand, InputCommandRef};
use super::commands::*;

/// Central input handler that manages and executes input commands
pub struct InputHandler {
    keyboard_commands: HashMap<Key, InputCommandRef>,
    mouse_command: InputCommandRef,
}

impl InputHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            keyboard_commands: HashMap::new(),
            mouse_command: Arc::new(MouseInputCommand),
        };
        
        handler.initialize_keyboard_commands();
        handler
    }
    
    /// Initialize all keyboard command mappings
    fn initialize_keyboard_commands(&mut self) {
        // Musical note commands
        self.register_keyboard_command(Key::Q, Arc::new(KeyboardInputCommand::new(Key::Q)));
        self.register_keyboard_command(Key::Key2, Arc::new(KeyboardInputCommand::new(Key::Key2)));
        self.register_keyboard_command(Key::W, Arc::new(KeyboardInputCommand::new(Key::W)));
        self.register_keyboard_command(Key::Key3, Arc::new(KeyboardInputCommand::new(Key::Key3)));
        self.register_keyboard_command(Key::E, Arc::new(KeyboardInputCommand::new(Key::E)));
        self.register_keyboard_command(Key::R, Arc::new(KeyboardInputCommand::new(Key::R)));
        self.register_keyboard_command(Key::Key5, Arc::new(KeyboardInputCommand::new(Key::Key5)));
        self.register_keyboard_command(Key::T, Arc::new(KeyboardInputCommand::new(Key::T)));
        self.register_keyboard_command(Key::Key6, Arc::new(KeyboardInputCommand::new(Key::Key6)));
        self.register_keyboard_command(Key::Y, Arc::new(KeyboardInputCommand::new(Key::Y)));
        self.register_keyboard_command(Key::Key7, Arc::new(KeyboardInputCommand::new(Key::Key7)));
        self.register_keyboard_command(Key::U, Arc::new(KeyboardInputCommand::new(Key::U)));
        
        // Waveform toggle
        self.register_keyboard_command(Key::S, Arc::new(WaveformToggleCommand));
        
        // Octave controls
        self.register_keyboard_command(Key::F1, Arc::new(OctaveAdjustCommand::new(false))); // decrease
        self.register_keyboard_command(Key::F2, Arc::new(OctaveAdjustCommand::new(true)));  // increase
        
        // ADSR controls
        self.register_keyboard_command(Key::F3, Arc::new(ADSRControlCommand::new_attack(false)));   // decrease attack
        self.register_keyboard_command(Key::F4, Arc::new(ADSRControlCommand::new_attack(true)));    // increase attack
        self.register_keyboard_command(Key::F5, Arc::new(ADSRControlCommand::new_decay(false)));    // decrease decay
        self.register_keyboard_command(Key::F6, Arc::new(ADSRControlCommand::new_decay(true)));     // increase decay
        self.register_keyboard_command(Key::F7, Arc::new(ADSRControlCommand::new_sustain(false)));  // decrease sustain
        self.register_keyboard_command(Key::F8, Arc::new(ADSRControlCommand::new_sustain(true)));   // increase sustain
        self.register_keyboard_command(Key::F9, Arc::new(ADSRControlCommand::new_release(false)));  // decrease release
        self.register_keyboard_command(Key::Key0, Arc::new(ADSRControlCommand::new_release(true))); // increase release
    }
    
    /// Register a keyboard command for a specific key
    fn register_keyboard_command(&mut self, key: Key, command: InputCommandRef) {
        self.keyboard_commands.insert(key, command);
    }
    
    /// Handle all keyboard input by delegating to appropriate commands
    pub fn handle_keyboard_input(&self, state: &mut State, window: &mut Window, sink: &mut Sink) {
        for (key, command) in &self.keyboard_commands {
            if window.is_key_pressed(*key, minifb::KeyRepeat::No) || 
               (matches!(key, Key::F3 | Key::F4 | Key::F5 | Key::F6 | Key::F7 | Key::F8 | Key::F9 | Key::Key0) && 
                window.is_key_pressed(*key, minifb::KeyRepeat::Yes)) {
                command.execute(state, window, sink);
                // For musical note keys, return early to prevent multiple keys being processed
                if matches!(key, Key::Q | Key::Key2 | Key::W | Key::Key3 | Key::E | Key::R | Key::Key5 | Key::T | Key::Key6 | Key::Y | Key::Key7 | Key::U) {
                    return;
                }
            }
        }
    }
    
    /// Handle mouse input
    pub fn handle_mouse_input(&self, state: &mut State, window: &mut Window, sink: &mut Sink) {
        self.mouse_command.execute(state, window, sink);
    }
    
    /// Handle all input types
    pub fn handle_input(&self, state: &mut State, window: &mut Window, sink: &mut Sink) {
        self.handle_keyboard_input(state, window, sink);
        self.handle_mouse_input(state, window, sink);
        
        // Always handle recording control (key release timing, playback, etc.)
        let recording_command = RecordingControlCommand;
        recording_command.execute(state, window, sink);
    }
}