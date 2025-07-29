use std::collections::HashMap;
use std::time::Duration;

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};
use rodio::{Sink, Source};

use crate::{
    graphics::constants::*,
    graphics::waveform_display::generate_waveform_display
};
use crate::graphics::sprites::{draw_sprite, Sprite, Sprites};
use crate::music_theory::{OCTAVE_LOWER_BOUND, OCTAVE_UPPER_BOUND};
use crate::music_theory::note::Note;
use crate::state::State;
use crate::waveforms::{AMPLITUDE, DURATION, Waveform};
use crate::waveforms::sine_wave::SineWave;
use crate::waveforms::square_wave::SquareWave;
use crate::waveforms::triangle_wave::TriangleWave;
use crate::waveforms::sawtooth_wave::SawtoothWave;
use crate::waveforms::adsr_envelope::ADSREnvelope;

/// Handles key presses for musical notes, waveform toggling, and octave adjustments.
///
/// # Parameters
/// - `state`: Mutable reference to the synthesizer state which holds current octave, waveform, and pressed key.
/// - `window`: Mutable reference to the window object used to detect key presses.
/// - `sink`: Mutable reference to the audio sink where musical notes are played.
///
/// # Key Handling Logic
/// - It iterates over predefined key mappings and triggers musical note generation when a corresponding key is pressed.
/// - Toggles between SINE and SQUARE waveform when the 'F' key is pressed.
/// - Increases the octave when 'F2' key is pressed and the current octave is below the upper bound.
/// - Decreases the octave when 'F1' key is pressed and the current octave is above the lower bound.
    pub fn handle_key_presses(state: &mut State, window: &mut Window, sink: &mut Sink) {
        // Check for musical note key presses
        let mut key_pressed = false;
        for (key, note, _, _) in get_key_mappings() {
            if window.is_key_pressed(key, KeyRepeat::No) {
                handle_musical_note(state, sink, note);
                state.pressed_key = Some((key, note));
                

                // Record note if recording
                if state.recording_state == crate::state::RecordingState::Recording {
                    // Finish previous note if there was one
                    if let Some((start_time, prev_note, prev_octave)) = state.current_note_start.take() {
                        let duration = start_time.elapsed().as_secs_f32();
                        let timestamp = state.recording_start_time
                            .map(|start| start.elapsed().as_secs_f32() - duration)
                            .unwrap_or(0.0);

                        state.recorded_notes.push(crate::state::RecordedNote {
                            note: prev_note,
                            octave: prev_octave,
                            timestamp,
                            duration,
                        });
                    }

                    // Start recording new note
                    state.current_note_start = Some((std::time::Instant::now(), note, state.octave));
                }
                
                key_pressed = true;
                return;
            }
        }
        
        // If no musical key is pressed, start the fade-out effect but keep the visual key displayed
        if !key_pressed && state.pressed_key.is_some() && state.key_release_time.is_none() {
            state.key_release_time = Some(std::time::Instant::now());
            // Don't clear pressed_key here - keep it for visual display until next key press
        }
        
        // Clear frequency after fade-out is complete
        if let Some(release_time) = state.key_release_time {
            if release_time.elapsed().as_secs_f32() > 2.0 {
                state.current_frequency = None;
                state.key_release_time = None;
            }
        }

    // Toggle the waveform between SINE and SQUARE when 'S' key is pressed
    if window.is_key_pressed(Key::S, KeyRepeat::No) {
        state.toggle_waveform();
    }

    // Increase the octave when 'F2' key is pressed and the current octave is below the upper bound
    if window.is_key_pressed(Key::F2, KeyRepeat::No) && state.get_current_octave() < OCTAVE_UPPER_BOUND {
        state.increase_octave();
    }

    // Decrease the octave when 'F1' key is pressed and the current octave is above the lower bound
    if window.is_key_pressed(Key::F1, KeyRepeat::No) && state.get_current_octave() > OCTAVE_LOWER_BOUND {
        state.decrease_octave();
    }

    // // Activate/Deactivate LPF (low pass filter) when 'F' key is pressed
    // if window.is_key_pressed(Key::F, KeyRepeat::No) {
    //     state.toggle_lpf();
    // }

    // // Increase the filter cutoff coefficient when 'F4' key is pressed
    // if window.is_key_pressed(Key::F4, KeyRepeat::No) {
    //     state.increase_filter_cutoff();
    // }

    // // Decrease the filter cutoff coefficient when 'F3' key is pressed
    // if window.is_key_pressed(Key::F3, KeyRepeat::No) {
    //     state.decrease_filter_cutoff();
    // }

    // ADSR control key bindings
    if window.is_key_pressed(Key::F4, KeyRepeat::Yes) {
        state.increase_attack();
    }
    if window.is_key_pressed(Key::F3, KeyRepeat::Yes) {
        state.decrease_attack();
    }

    // Decay controls
    if window.is_key_pressed(Key::F6, KeyRepeat::Yes) {
        state.increase_decay();
    }
    if window.is_key_pressed(Key::F5, KeyRepeat::Yes) {
        state.decrease_decay();
    }

    // Sustain controls
    if window.is_key_pressed(Key::F8, KeyRepeat::Yes) {
        state.increase_sustain();
    }
    if window.is_key_pressed(Key::F7, KeyRepeat::Yes) {
        state.decrease_sustain();
    }

    // Release controls
    if window.is_key_pressed(Key::Key0, KeyRepeat::Yes) {
        state.increase_release();
    }
    if window.is_key_pressed(Key::F9, KeyRepeat::Yes) {
        state.decrease_release();
    }

}

/// Handles mouse input for all interactive elements
pub fn handle_mouse_input(state: &mut State, window: &mut Window, sink: &mut Sink) {
    // Update mouse position
    if let Some((x, y)) = window.get_mouse_pos(MouseMode::Clamp) {
        state.mouse.x = x;
        state.mouse.y = y;
    }

    // Update mouse button state
    let mouse_pressed = window.get_mouse_down(MouseButton::Left);
    let mouse_clicked = mouse_pressed && !state.mouse.left_pressed;
    
    state.mouse.left_clicked = mouse_clicked;
    state.mouse.left_pressed = mouse_pressed;

    // Handle dragging
    if mouse_clicked {
        state.mouse.drag_start = Some((state.mouse.x, state.mouse.y));
        state.mouse.dragging = false;
    } else if mouse_pressed && state.mouse.drag_start.is_some() {
        if let Some((start_x, start_y)) = state.mouse.drag_start {
            let distance = ((state.mouse.x - start_x).powi(2) + (state.mouse.y - start_y).powi(2)).sqrt();
            if distance > 3.0 {
                state.mouse.dragging = true;
            }
        }
    } else if !mouse_pressed {
        state.mouse.drag_start = None;
        state.mouse.dragging = false;
    }

    // Handle ADSR fader interactions
    handle_adsr_fader_mouse(state, sink);
    
    // Handle tangent (sharp) key interactions FIRST (they have priority over regular keys)
    if handle_tangent_mouse(state, sink) {
        return; // Exit if a tangent was clicked
    }
    
    // Handle regular keyboard key interactions
    handle_keyboard_mouse(state, sink);
    
    // Handle octave fader interactions
    handle_octave_fader_mouse(state);
    
    // Handle waveform display interactions
    handle_waveform_display_mouse(state);
    
    // Handle control button interactions
    handle_control_buttons_mouse(state);
}

/// Handle mouse interactions with ADSR faders
fn handle_adsr_fader_mouse(state: &mut State, sink: &mut Sink) {
    // ADSR fader positions (matching the draw_adsr_faders function)
    let display_x = 164;
    let display_width = 164;
    let display_y = 4 * 51 + 17;
    let base_x = display_x + display_width + 104;
    let base_y = display_y;
    
    let fader_width = 25;
    let fader_height = 50;
    let fader_spacing = 30;

    let adsr_params = ["attack", "decay", "sustain", "release"];
    
    for (i, param) in adsr_params.iter().enumerate() {
        let fader_x = base_x + i * fader_spacing;
        let fader_y = base_y;
        
        // Check if mouse is over this fader
        if state.mouse.x >= fader_x as f32 && state.mouse.x <= (fader_x + fader_width) as f32 &&
           state.mouse.y >= fader_y as f32 && state.mouse.y <= (fader_y + fader_height) as f32 {
            
            if state.mouse.left_clicked || state.mouse.dragging {
                // Calculate new value based on mouse Y position
                let relative_y = state.mouse.y - fader_y as f32;
                let normalized_value = 1.0 - (relative_y / fader_height as f32).clamp(0.0, 1.0);
                let new_value = (normalized_value * 99.0) as u8;
                
                // Update the appropriate ADSR parameter
                match *param {
                    "attack" => state.attack = new_value,
                    "decay" => state.decay = new_value,
                    "sustain" => state.sustain = new_value,
                    "release" => state.release = new_value,
                    _ => {}
                }
            }
        }
    }
}

/// Handle mouse interactions with keyboard keys
fn handle_keyboard_mouse(state: &mut State, sink: &mut Sink) {
    // Virtual keyboard positioning (matching draw_idle_key_sprites exactly)
    // Keys are drawn from i=1 to i=7, at positions i * key_width
    let key_width = 64; // sprites.keys[KEY_IDLE].width
    let key_height = 144; // sprites.keys[KEY_IDLE].height  
    let key_y = 2 * key_height; // Same as drawing: 2 * sprites.keys[KEY_IDLE].height
    
    for (key, note, position, _) in get_key_mappings() {
        // Match the exact drawing position: i * sprites.keys[KEY_IDLE].width where i = position
        let key_x = position * key_width;
        
        // Check if mouse is over this key
        if state.mouse.x >= key_x as f32 && state.mouse.x <= (key_x + key_width) as f32 &&
           state.mouse.y >= key_y as f32 && state.mouse.y <= (key_y + key_height) as f32 {
            
            if state.mouse.left_clicked {
                // Trigger the note
                handle_musical_note(state, sink, note);
                state.pressed_key = Some((key, note));
                
                
                // Record note if recording
                if state.recording_state == crate::state::RecordingState::Recording {
                    // Finish previous note if there was one
                    if let Some((start_time, prev_note, prev_octave)) = state.current_note_start.take() {
                        let duration = start_time.elapsed().as_secs_f32();
                        let timestamp = state.recording_start_time
                            .map(|start| start.elapsed().as_secs_f32() - duration)
                            .unwrap_or(0.0);
                        
                        state.recorded_notes.push(crate::state::RecordedNote {
                            note: prev_note,
                            octave: prev_octave,
                            timestamp,
                            duration,
                        });
                    }
                    
                    // Start recording new note
                    state.current_note_start = Some((std::time::Instant::now(), note, state.octave));
                }
                return; // Exit after handling one key to avoid multiple triggers
            }
        }
    }
    
}

/// Handle mouse interactions with tangent (sharp) keys
/// Returns true if a tangent was clicked, false otherwise
fn handle_tangent_mouse(state: &mut State, sink: &mut Sink) -> bool {
    let key_width = 64; // sprites.keys[KEY_IDLE].width as i32
    let key_height = 144; // sprites.keys[KEY_IDLE].height
    let tangent_width = 30; // sprites.tangents[TANGENT_IDLE].width as i32
    let tangent_height = 96; // sprites.tangents[TANGENT_IDLE].height
    let key_y = 2 * key_height; // Same as drawing: 2 * sprites.keys[KEY_IDLE].height
    
    // Tangent mappings (position -> note) - matching the tangent_map in create_tangent_map()
    let tangent_mappings = [
        (2, Note::CSharp, Key::Key2),   // Between keys C and D
        (3, Note::DSharp, Key::Key3),   // Between keys D and E
        (5, Note::FSharp, Key::Key5),   // Between keys F and G
        (6, Note::GSharp, Key::Key6),   // Between keys G and A
        (7, Note::ASharp, Key::Key7),   // Between keys A and B
    ];
    
    for &(position, note, key) in &tangent_mappings {
        // Calculate tangent position exactly like draw_idle_tangent_sprites:
        // let x = (pos * key_width) - (tangent_width / 2);
        let tangent_x = (position * key_width) - (tangent_width / 2);
        
        // Ensure x position is valid (same bounds check as drawing)
        let tangent_x_final = if tangent_x >= 0 { tangent_x as usize } else { 0 };
        
        // Check if mouse is over this tangent
        if state.mouse.x >= tangent_x_final as f32 && 
           state.mouse.x <= (tangent_x_final + tangent_width as usize) as f32 &&
           state.mouse.y >= key_y as f32 && 
           state.mouse.y <= (key_y + tangent_height as usize) as f32 {
            
            if state.mouse.left_clicked {
                // Trigger the note
                handle_musical_note(state, sink, note);
                state.pressed_key = Some((key, note));
                
                // Record note if recording
                if state.recording_state == crate::state::RecordingState::Recording {
                    // Finish previous note if there was one
                    if let Some((start_time, prev_note, prev_octave)) = state.current_note_start.take() {
                        let duration = start_time.elapsed().as_secs_f32();
                        let timestamp = state.recording_start_time
                            .map(|start| start.elapsed().as_secs_f32() - duration)
                            .unwrap_or(0.0);
                        
                        state.recorded_notes.push(crate::state::RecordedNote {
                            note: prev_note,
                            octave: prev_octave,
                            timestamp,
                            duration,
                        });
                    }
                    
                    // Start recording new note
                    state.current_note_start = Some((std::time::Instant::now(), note, state.octave));
                }
                return true; // Return true to indicate a tangent was clicked
            }
        }
    }
    false // Return false if no tangent was clicked
}

/// Handle mouse interactions with octave fader
fn handle_octave_fader_mouse(state: &mut State) {
    // Octave fader position (matching draw_octave_fader_sprite exactly)
    let key_width = 64; // sprites.keys[0].width
    let key_height = 144; // sprites.keys[0].height
    let fader_x = 8 * key_width + 5; // Same as drawing: 8 * sprites.keys[0].width + 5
    let fader_y = 2 * key_height; // Same as drawing: 2 * sprites.keys[0].height
    
    // Octave fader dimensions (from sprites.octave_fader)
    let fader_width = 28; // sprites.octave_fader width
    let fader_height = 143; // sprites.octave_fader height
    
    // Check if mouse is over the octave fader
    if state.mouse.x >= fader_x as f32 && state.mouse.x <= (fader_x + fader_width) as f32 &&
       state.mouse.y >= fader_y as f32 && state.mouse.y <= (fader_y + fader_height) as f32 {
        
        if state.mouse.left_clicked {
            // Calculate relative Y position within the fader
            let relative_y = state.mouse.y - fader_y as f32;
            let fader_center_y = fader_height as f32 / 2.0;
            
            // If clicked in upper half, increase octave; if lower half, decrease octave
            if relative_y < fader_center_y {
                // Clicked in upper part - increase octave
                state.increase_octave();
            } else {
                // Clicked in lower part - decrease octave
                state.decrease_octave();
            }
        }
    }
}

/// Handle mouse interactions with waveform display
fn handle_waveform_display_mouse(state: &mut State) {
    // Waveform display position (matching draw_display_sprite_single exactly)
    let display_width = 164; // sprite.width (from display sprites)
    let display_height = 51; // sprite.height (from display sprites)
    let display_x = 1 * display_width; // Same as drawing: 1 * sprite.width
    let display_y = 4 * display_height + 17; // Same as drawing: 4 * sprite.height + 17
    
    // Check if mouse is over the waveform display
    if state.mouse.x >= display_x as f32 && state.mouse.x <= (display_x + display_width) as f32 &&
       state.mouse.y >= display_y as f32 && state.mouse.y <= (display_y + display_height) as f32 {
        
        if state.mouse.left_clicked {
            // Toggle to next waveform (cycles through SINE -> SQUARE -> TRIANGLE -> SAWTOOTH -> SINE)
            state.toggle_waveform();
        }
    }
}

/// Handle mouse interactions with control buttons
fn handle_control_buttons_mouse(state: &mut State) {
    // Control button positions - aligned with note display terminal (top left area)
    let button_width = 60;
    let button_height = 30;
    let button_y = 180;
    
    // Align with note display X position: 1 * 64 = 64
    let base_x = 66; // Same X as note display terminal
    
    // Record button
    let record_x = base_x;
    if state.mouse.x >= record_x as f32 && state.mouse.x <= (record_x + button_width) as f32 &&
       state.mouse.y >= button_y as f32 && state.mouse.y <= (button_y + button_height) as f32 {
        
        if state.mouse.left_clicked {
            match state.recording_state {
                crate::state::RecordingState::Stopped => state.start_recording(),
                crate::state::RecordingState::Recording => state.stop_recording(),
                crate::state::RecordingState::Playing => state.stop_playback(),
            }
        }
    }
    
    // Play button
    let play_x = record_x + button_width + 10;
    if state.mouse.x >= play_x as f32 && state.mouse.x <= (play_x + button_width) as f32 &&
       state.mouse.y >= button_y as f32 && state.mouse.y <= (button_y + button_height) as f32 {
        
        if state.mouse.left_clicked {
            match state.recording_state {
                crate::state::RecordingState::Stopped => state.start_playback(),
                crate::state::RecordingState::Playing => state.stop_playback(),
                _ => {},
            }
        }
    }
    
    // Stop button
    let stop_x = play_x + button_width + 10;
    if state.mouse.x >= stop_x as f32 && state.mouse.x <= (stop_x + button_width) as f32 &&
       state.mouse.y >= button_y as f32 && state.mouse.y <= (button_y + button_height) as f32 {
        
        if state.mouse.left_clicked {
            state.stop_recording();
            state.stop_playback();
        }
    }
}


/// Handles playing a musical note with a specified octave, waveform, and duration.
///
/// # Parameters
/// - `octave`: A mutable reference to the current octave of the synthesizer.
/// - `sink`: A mutable reference to the audio sink where the sound will be played.
/// - `current_waveform`: The waveform enum representing the type of waveform to use for synthesizing the sound.
/// - `note`: The musical note (pitch) to be played.
pub fn handle_musical_note(state: &mut State, sink: &mut Sink, note: Note) {

    // Compute the base frequency association with the note and octave
    let base_frequency = note.frequency(state.octave);
    
    // Store the current frequency for display purposes and reset animation timing
    state.current_frequency = Some(base_frequency);
    state.animation_start_time = std::time::Instant::now();
    state.key_release_time = None; // Clear any previous release time

    // Stop any currently playing audio to prevent queueing
    sink.stop();

    // Initialize Synth implementation based on Waveform enum with ADSR envelope
    let synth = match state.waveform {
        Waveform::SINE => {
            let filtered_frequency = state.apply_lpf(base_frequency);
            let sine_wave = SineWave::new(filtered_frequency);
            let adsr_envelope = ADSREnvelope::new(
                sine_wave, 
                state.attack_normalized() * 2.0,    // Convert 0-99 to 0-2 seconds
                state.decay_normalized() * 2.0, 
                state.sustain_normalized(), 
                state.release_normalized() * 2.0
            );
            Box::new(adsr_envelope) as Box<dyn Source<Item=f32> + 'static + Send>
        }
        Waveform::SQUARE => {
            let filtered_frequency = state.apply_lpf(base_frequency);
            let square_wave = SquareWave::new(filtered_frequency);
            let adsr_envelope = ADSREnvelope::new(
                square_wave, 
                state.attack_normalized() * 2.0, 
                state.decay_normalized() * 2.0, 
                state.sustain_normalized(), 
                state.release_normalized() * 2.0
            );
            Box::new(adsr_envelope) as Box<dyn Source<Item=f32> + 'static + Send>
        }
        Waveform::TRIANGLE => {
            let filtered_frequency = state.apply_lpf(base_frequency);
            let triangle_wave = TriangleWave::new(filtered_frequency);
            let adsr_envelope = ADSREnvelope::new(
                triangle_wave, 
                state.attack_normalized() * 2.0, 
                state.decay_normalized() * 2.0, 
                state.sustain_normalized(), 
                state.release_normalized() * 2.0
            );
            Box::new(adsr_envelope) as Box<dyn Source<Item=f32> + 'static + Send>
        }
        Waveform::SAWTOOTH => {
            let filtered_frequency = state.apply_lpf(base_frequency);
            let sawtooth_wave = SawtoothWave::new(filtered_frequency);
            let adsr_envelope = ADSREnvelope::new(
                sawtooth_wave, 
                state.attack_normalized() * 2.0, 
                state.decay_normalized() * 2.0, 
                state.sustain_normalized(), 
                state.release_normalized() * 2.0
            );
            Box::new(adsr_envelope) as Box<dyn Source<Item=f32> + 'static + Send>
        }
    };

    // Create Source from our Synth with ADSR envelope - envelope handles its own termination
    let source = synth.amplify(AMPLITUDE);

    // Play the sound source immediately, replacing any queued audio
    let _result = sink.append(source);
}

/// Draws the current state of the synthesizer on the window buffer.
///
/// # Parameters
/// - `state`: Reference to the current `State` containing the state of the synthesizer.
/// - `sprites`: Reference to the `Sprites` struct containing all sprite data needed for drawing.
/// - `window_buffer`: Mutable reference to the window buffer where pixels are drawn.
/// - `grid_width`: Width of the grid in tiles.
/// - `grid_height`: Height of the grid in tiles.
pub fn update_buffer_with_state(state: &State, sprites: &Sprites, window_buffer: &mut Vec<u32>, rack_index: usize) {

    // Draw rack
    draw_rack_sprite(sprites, window_buffer, rack_index);

    // Draw all idle keys first
    draw_idle_key_sprites(sprites, window_buffer);

    // Create a map for tangent positions and their corresponding note constants
    let tangent_map = create_tangent_map();

    // Draw all tangents as overlay on key sprites in their idle state first
    draw_idle_tangent_sprites(sprites, window_buffer, &tangent_map);

    // Draw the bulb
    // draw_bulb_sprite(state, sprites, window_buffer);

    // Draw the cutoff knob for LPF
    // draw_filter_cutoff_knob_sprite(state, sprites, window_buffer);

    // Draw the idle knob to the left of the cutoff knob for LPF
    // draw_idle_knob_sprite(sprites, window_buffer);

    // Draw ADSR faders
    draw_adsr_faders(state, sprites, window_buffer);
    
    // Draw control buttons
    draw_control_buttons(state, window_buffer);

    // Draw octave fader, which display the current octave controlled by keys F1/F2
    draw_octave_fader_sprite(state.octave, sprites, window_buffer);

    // Calculate animation time and amplitude for waveform display
    let animation_time = state.animation_start_time.elapsed().as_secs_f32();
    
    // Always show the display frame, but only show waveform when playing or fading
    let (frequency, amplitude) = if state.current_frequency.is_some() || state.key_release_time.is_some() {
        // Calculate amplitude based on whether key is pressed or released
        let amplitude = if let Some(release_time) = state.key_release_time {
            // Fade out over 2 seconds after key release
            let fade_duration = release_time.elapsed().as_secs_f32();
            let fade_factor = (1.0 - fade_duration / 2.0).max(0.0);
            fade_factor
        } else {
            1.0 // Full brightness when key is pressed
        };
        
        // Use last played frequency during fade
        let frequency = state.current_frequency.unwrap_or(440.0);
        (frequency, amplitude)
    } else {
        // No waveform - just show empty display
        (440.0, 0.0) // Amplitude 0 means no waveform will be drawn
    };
    
    // Always generate display (frame always visible, waveform only when amplitude > 0)
    let waveform_sprite = generate_waveform_display(frequency, state.waveform, animation_time, amplitude);
    draw_display_sprite_single(&waveform_sprite, window_buffer);
    

    // Check if a key is pressed
    if let Some((_, note)) = &state.pressed_key {

        // Get sprite index associated with the note to be drawn (A, C# etc.)
        let note_sprite_index = get_note_sprite_index(note).unwrap_or_default();

        // Get key position on the keyboard (0 would be the first key, 7 the last etc.)
        let key_position = get_key_position(note).unwrap_or(0);

        // Draw sprites note, knobs and the waveform display
        draw_note_sprite(sprites, window_buffer, note_sprite_index);

        // Draw pressed key sprite if the note is not a sharp
        if matches!(note, Note::A | Note::B | Note::C | Note::D | Note::E | Note::F | Note::G) {
            draw_pressed_key_sprite(sprites, window_buffer, key_position);
        }

        // Draw idle and pressed tangents as overlay on key sprites
        draw_tangent_sprites(note_sprite_index, &tangent_map, sprites, window_buffer);
    }
    
}

/// Returns the position of the given musical note on the keyboard.
///
/// # Arguments
///
/// * `note` - A reference to the `Note` whose position is to be found.
///
/// # Returns
///
/// * `Some(usize)` - The position of the note on the keyboard if it exists.
/// * `None` - If the note is not found in the key mappings.
pub fn get_key_position(note: &Note) -> Option<usize> {
    for (_, mapped_note, position, _) in get_key_mappings() {
        if mapped_note == *note {
            return Some(position);
        }
    }
    None
}

/// Returns the sprite index for the given musical note.
///
/// # Arguments
///
/// * `note` - A reference to the `Note` whose sprite index is to be found.
///
/// # Returns
///
/// * `Some(usize)` - The sprite index for the note if it exists.
/// * `None` - If the note is not found in the key mappings.
pub fn get_note_sprite_index(note: &Note) -> Option<usize> {
    for (_, mapped_note, _, sprite_index) in get_key_mappings() {
        if mapped_note == *note {
            return Some(sprite_index);
        }
    }
    None
}

/// Returns a vector of tuples representing key mappings.
///
/// Each tuple contains the following elements:
/// - `Key`: The key that is pressed.
/// - `Note`: The musical note associated with the key.
/// - `usize`: The position of the key on the keyboard.
/// - `usize`: The sprite index for the note.
pub fn get_key_mappings() -> Vec<(Key, Note, usize, usize)> {
    vec![
        (Key::Q, Note::C, 1, NOTE_C),
        (Key::Key2, Note::CSharp, 1, NOTE_C_SHARP),
        (Key::W, Note::D, 2, NOTE_D),
        (Key::Key3, Note::DSharp, 2, NOTE_D_SHARP),
        (Key::E, Note::E, 3, NOTE_E),
        (Key::R, Note::F, 4, NOTE_F),
        (Key::Key5, Note::FSharp, 4, NOTE_F_SHARP),
        (Key::T, Note::G, 5, NOTE_G),
        (Key::Key6, Note::GSharp, 5, NOTE_G_SHARP),
        (Key::Y, Note::A, 6, NOTE_A),
        (Key::Key7, Note::ASharp, 6, NOTE_A_SHARP),
        (Key::U, Note::B, 7, NOTE_B),
    ]
}

/// Creates a map for tangent positions and their corresponding note sprite indices.
///
/// # Returns
/// A `HashMap` where the keys are positions on the keyboard and the values are note sprite indices
/// for the corresponding tangent (sharp) keys.
pub fn create_tangent_map() -> HashMap<i32, usize> {
    let tangent_map: HashMap<i32, usize> = [
        (2, NOTE_C_SHARP),   // Between keys C and D
        (3, NOTE_D_SHARP),   // Between keys D and E
        (5, NOTE_F_SHARP),   // Between keys F and G
        (6, NOTE_G_SHARP),   // Between keys G and A
        (7, NOTE_A_SHARP),   // Between keys A and B
    ].iter().cloned().collect();
    tangent_map
}

/// Draws the text sprite.
///
/// # Parameters
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_rack_sprite(sprites: &Sprites, buffer: &mut [u32], rack_index: usize) {
    draw_sprite(0 * sprites.rack[0].width as usize,
                0 * sprites.rack[0].height as usize,
                &sprites.rack[rack_index], buffer, WINDOW_WIDTH);
}

/// Draws the sine wave sprite.
///
/// # Parameters
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_display_sprite(sprite: &Vec<Sprite>, buffer: &mut [u32], display_index: usize) {
    draw_sprite(1 * sprite[0].width as usize,
                4 * sprite[0].height as usize + 17,
                &sprite[display_index], buffer, WINDOW_WIDTH);
}

/// Draws a single waveform display sprite.
///
/// # Parameters
/// - `sprite`: A reference to the single `Sprite` to be drawn.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_display_sprite_single(sprite: &Sprite, buffer: &mut [u32]) {
    draw_sprite(1 * sprite.width as usize,
                4 * sprite.height as usize + 17,
                sprite, buffer, WINDOW_WIDTH);
}

/// Draws the pressed key sprite.
///
/// # Parameters
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_pressed_key_sprite(sprites: &Sprites, window_buffer: &mut Vec<u32>, key_position: usize) {
    draw_sprite(key_position * sprites.keys[KEY_PRESSED].width as usize,
                2 * sprites.keys[KEY_PRESSED].height as usize,
                &sprites.keys[KEY_PRESSED], window_buffer, WINDOW_WIDTH);
}


/// Draws the octave fader sprite.
///
/// # Parameters
/// - `octave`: The current octave.
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_octave_fader_sprite(octave: i32, sprites: &Sprites, window_buffer: &mut Vec<u32>) {
    draw_sprite(8 * sprites.keys[0].width as usize + 5,
                2 * sprites.keys[0].height as usize,
                &sprites.octave_fader[octave as usize], window_buffer, WINDOW_WIDTH);
}


/// Draws the current window with the provided pixel buffer.
///
/// # Parameters
/// - `window`: Mutable reference to the `Window` object where the visuals are displayed.
/// - `window_buffer`: Mutable reference to a vector of `u32` representing the pixel data to be displayed.
pub fn draw_buffer(window: &mut Window, window_buffer: &mut Vec<u32>) {
    window.update_with_buffer(&window_buffer, WINDOW_WIDTH, WINDOW_HEIGHT).unwrap();
}

/// Draws idle knobs.
///
/// # Parameters
/// - `state`: Reference to the current `State` containing the state of the synthesizer.
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_bulb_sprite(state: &State, sprites: &Sprites, window_buffer: &mut Vec<u32>) {
    draw_sprite(6 * sprites.knob[0].width as usize,
                5 * sprites.knob[0].height as usize + 10,
                &sprites.bulb[state.lpf_active], window_buffer, WINDOW_WIDTH);
}

/// Draws idle knobs.
///
/// # Parameters
/// - `state`: Reference to the current `State` containing the state of the synthesizer.
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_filter_cutoff_knob_sprite(state: &State, sprites: &Sprites, window_buffer: &mut Vec<u32>) {
    let filter_cutoff = state.filter_factor;

    // Assigns the appropriate sprite index based on cutoff float value threshold
    let knob_sprite_index = match filter_cutoff {
        v if (0.0..=0.14).contains(&v) => 0,
        v if (0.14..=0.28).contains(&v) => 1,
        v if (0.28..=0.42).contains(&v) => 2,
        v if (0.42..=0.57).contains(&v) => 3,
        v if (0.57..=0.71).contains(&v) => 4,
        v if (0.71..=0.85).contains(&v) => 5,
        v if (0.85..=0.99).contains(&v) => 6,
        _ => 7 // Last knob for ~0.99
    };

    draw_sprite(6 * sprites.knob[0].width as usize,
                5 * sprites.knob[0].height as usize - 10,
                &sprites.knob[knob_sprite_index], window_buffer, WINDOW_WIDTH);
}

/// Draws idle knob.
///
/// # Parameters
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_idle_knob_sprite(sprites: &Sprites, window_buffer: &mut Vec<u32>) {
    draw_sprite(7 * sprites.knob[0].width as usize,
                5 * sprites.knob[0].height as usize - 10,
                &sprites.knob[0], window_buffer, WINDOW_WIDTH);
}

/// Draws the note sprite for the given note sprite index.
///
/// # Parameters
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
/// - `note_sprite_index`: The index of the note sprite to be drawn.
pub fn draw_note_sprite(sprites: &Sprites, window_buffer: &mut Vec<u32>, note_sprite_index: usize) {
    draw_sprite(1 * sprites.notes[0].width as usize,
                5 * sprites.notes[0].height as usize - 15,
                &sprites.notes[note_sprite_index], window_buffer, WINDOW_WIDTH);
}

/// Draws all idle tangents (sharp keys).
///
/// # Parameters
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
/// - `tangent_map`: A hashmap mapping positions to the corresponding tangent note sprite indices.
pub fn draw_idle_tangent_sprites(sprites: &Sprites, window_buffer: &mut Vec<u32>, tangent_map: &HashMap<i32, usize>) {
    let key_width = sprites.keys[KEY_IDLE].width as i32;
    let key_height = sprites.keys[KEY_IDLE].height as usize;
    let tangent_width = sprites.tangents[TANGENT_IDLE].width as i32;

    for &pos in tangent_map.keys() {
        // Calculate the x-coordinate of the tangent's center position
        let x = (pos * key_width) - (tangent_width / 2);

        // Ensure the x position is within bounds
        let x_usize = if x >= 0 { usize::try_from(x).unwrap_or(0) } else { 0 };

        draw_sprite(
            x_usize,
            2 * key_height,
            &sprites.tangents[TANGENT_IDLE],
            window_buffer,
            WINDOW_WIDTH,
        );
    }
}

/// Draws all idle keys.
///
/// # Parameters
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_idle_key_sprites(sprites: &Sprites, window_buffer: &mut Vec<u32>) {
    for i in 1..8 {
        draw_sprite(
            i * sprites.keys[KEY_IDLE].width as usize,
            2 * sprites.keys[KEY_IDLE].height as usize,
            &sprites.keys[KEY_IDLE],
            window_buffer,
            WINDOW_WIDTH
        );
    }
}

/// Draws ADSR faders with custom vertical bars and numerical values (0-99).
///
/// # Parameters
/// - `state`: Reference to the current `State` containing ADSR values.
/// - `sprites`: A reference to the `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_adsr_faders(state: &State, sprites: &Sprites, window_buffer: &mut Vec<u32>) {
    // Compact fader dimensions to fit all 4 ADSR faders
    let fader_width = 25;
    let fader_height = 50;
    let fader_spacing = 30; // Minimal spacing between faders
    
    // Position faders directly to the right of waveform visualizer
    // Display is positioned at: x = 1 * 164 = 164, y = 4 * 51 + 17 = 221
    let display_x = 164; // Display x position
    let display_width = 164; // Display width (from DISPLAY_WIDTH constant)
    let display_y = 4 * 51 + 17; // Display y position

    let base_x = display_x + display_width + 104; // Start right after display (164 + 164 + 5 = 333px)
    let base_y = display_y; // Same y as display
    
    // ADSR values
    let adsr_values = [state.attack, state.decay, state.sustain, state.release];
    let labels = ["A", "D", "S", "R"];
    
    // Draw each ADSR fader
    for (i, (&value, &label)) in adsr_values.iter().zip(labels.iter()).enumerate() {
        let x = base_x + i * fader_spacing;
        let y = base_y;
        
        // Draw fader background (dark gray border)
        draw_fader_background(x, y, fader_width, fader_height, window_buffer);
        
        // Draw fader fill (based on value 0-99)
        let fill_height = (value as f32 / 99.0 * (fader_height - 4) as f32) as usize;
        draw_fader_fill(x + 2, y + (fader_height - 2 - fill_height), fader_width - 4, fill_height, window_buffer);

        // Draw label below fader (A, D, S, R) - centered for smaller width
        draw_fader_label(x + fader_width / 2 - 2, y + fader_height + 3, label, window_buffer);
    }
}

/// Draws a fader background rectangle
fn draw_fader_background(x: usize, y: usize, width: usize, height: usize, buffer: &mut Vec<u32>) {
    let border_color = 0xFF404040; // Dark gray
    let bg_color = 0xFF202020;     // Very dark gray
    
    for dy in 0..height {
        for dx in 0..width {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() {
                // Draw border
                if dx == 0 || dx == width - 1 || dy == 0 || dy == height - 1 {
                    buffer[index] = border_color;
                } else {
                    buffer[index] = bg_color;
                }
            }
        }
    }
}

/// Draws the fader fill based on value
fn draw_fader_fill(x: usize, y: usize, width: usize, height: usize, buffer: &mut Vec<u32>) {
    let fill_color = 0xFF00AA00; // Green
    
    for dy in 0..height {
        for dx in 0..width {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() {
                buffer[index] = fill_color;
            }
        }
    }
}

/// Draws a numerical value using number sprites
fn draw_number_value(x: usize, y: usize, value: u8, sprites: &Sprites, buffer: &mut Vec<u32>) {
    if value < 10 {
        // Single digit
        if value < sprites.numbers.len() as u8 {
            draw_sprite(x, y, &sprites.numbers[value as usize], buffer, WINDOW_WIDTH);
        }
    } else {
        // Two digits
        let tens = value / 10;
        let ones = value % 10;
        
        if tens < sprites.numbers.len() as u8 {
            draw_sprite(x - 5, y, &sprites.numbers[tens as usize], buffer, WINDOW_WIDTH);
        }
        if ones < sprites.numbers.len() as u8 {
            draw_sprite(x + 15, y, &sprites.numbers[ones as usize], buffer, WINDOW_WIDTH);
        }
    }
}

/// Draws control buttons for recording functionality
pub fn draw_control_buttons(state: &State, buffer: &mut Vec<u32>) {
    let button_width = 60;
    let button_height = 30;
    let button_y = 180; // Directly above the displays (display_y - 20)
    
    // Align with note display X position: 1 * 64 = 64
    let base_x = 66; // Same X as note display terminal
    
    // Record button
    let record_x = base_x;
    let record_color = match state.recording_state {
        crate::state::RecordingState::Recording => 0xFFFF0000, // Red when recording
        _ => 0xFF666666, // Gray when not recording
    };
    draw_button(record_x, button_y, button_width, button_height, record_color, "REC", buffer);
    
    // Play button
    let play_x = record_x + button_width + 10;
    let play_color = match state.recording_state {
        crate::state::RecordingState::Playing => 0xFF00FF00, // Green when playing
        _ => 0xFF666666, // Gray when not playing
    };
    draw_button(play_x, button_y, button_width, button_height, play_color, "PLAY", buffer);
    
    // Stop button
    let stop_x = play_x + button_width + 10;
    draw_button(stop_x, button_y, button_width, button_height, 0xFF666666, "STOP", buffer);
}

/// Draws a single button with text
fn draw_button(x: usize, y: usize, width: usize, height: usize, color: u32, text: &str, buffer: &mut Vec<u32>) {
    // Draw button background
    for dy in 0..height {
        for dx in 0..width {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() {
                // Draw border
                if dx == 0 || dx == width - 1 || dy == 0 || dy == height - 1 {
                    buffer[index] = 0xFFFFFFFF; // White border
                } else {
                    buffer[index] = color;
                }
            }
        }
    }
    
    // Draw text (simplified - just draw the text in the center)
    let text_x = x + width / 2 - (text.len() * 3);
    let text_y = y + height / 2 - 3;
    draw_simple_text(text_x, text_y, text, 0xFFFFFFFF, buffer);
}

/// Draws simple text
fn draw_simple_text(x: usize, y: usize, text: &str, color: u32, buffer: &mut Vec<u32>) {
    // Very simple 3x5 font for button labels
    let font_patterns = std::collections::HashMap::from([
        ('R', vec![0b111, 0b101, 0b111, 0b110, 0b101]),
        ('E', vec![0b111, 0b100, 0b111, 0b100, 0b111]),
        ('C', vec![0b111, 0b100, 0b100, 0b100, 0b111]),
        ('P', vec![0b111, 0b101, 0b111, 0b100, 0b100]),
        ('L', vec![0b100, 0b100, 0b100, 0b100, 0b111]),
        ('A', vec![0b111, 0b101, 0b111, 0b101, 0b101]),
        ('Y', vec![0b101, 0b101, 0b111, 0b010, 0b010]),
        ('S', vec![0b111, 0b100, 0b111, 0b001, 0b111]),
        ('T', vec![0b111, 0b010, 0b010, 0b010, 0b010]),
        ('O', vec![0b111, 0b101, 0b101, 0b101, 0b111]),
    ]);
    
    for (i, ch) in text.chars().enumerate() {
        if let Some(pattern) = font_patterns.get(&ch) {
            for (row, &bits) in pattern.iter().enumerate() {
                for col in 0..3 {
                    if (bits >> (2 - col)) & 1 == 1 {
                        let pixel_x = x + i * 4 + col;
                        let pixel_y = y + row;
                        let index = pixel_y * WINDOW_WIDTH + pixel_x;
                        
                        if index < buffer.len() {
                            buffer[index] = color;
                        }
                    }
                }
            }
        }
    }
}


/// Draws a simple text label for the fader
fn draw_fader_label(x: usize, y: usize, label: &str, buffer: &mut Vec<u32>) {
    let text_color = 0xFFFFFFFF; // White
    
    // Simple 5x7 pixel font for A, D, S, R
    let patterns = match label {
        "A" => vec![ // A
            0b01110,
            0b10001,
            0b10001,
            0b11111,
            0b10001,
            0b10001,
            0b10001,
        ],
        "D" => vec![ // D
            0b11110,
            0b10001,
            0b10001,
            0b10001,
            0b10001,
            0b10001,
            0b11110,
        ],
        "S" => vec![ // S
            0b01111,
            0b10000,
            0b10000,
            0b01110,
            0b00001,
            0b00001,
            0b11110,
        ],
        "R" => vec![ // R
            0b11110,
            0b10001,
            0b10001,
            0b11110,
            0b10100,
            0b10010,
            0b10001,
        ],
        _ => return,
    };
    
    for (row, &pattern) in patterns.iter().enumerate() {
        for col in 0..5 {
            if (pattern >> (4 - col)) & 1 == 1 {
                let pixel_x = x + col;
                let pixel_y = y + row;
                let index = pixel_y * WINDOW_WIDTH + pixel_x;
                
                if index < buffer.len() {
                    buffer[index] = text_color;
                }
            }
        }
    }
}

/// Draws the tangents (sharp keys).
///
/// # Parameters
/// - `note_sprite_index`: The index of the sprite representing the current note being pressed.
/// - `tangent_map`: A hashmap mapping positions to the corresponding tangent note sprite indices.
/// - `sprites`: The `Sprites` struct containing all the sprite images.
/// - `window_buffer`: A mutable reference to the buffer representing the window's pixels.
pub fn draw_tangent_sprites(note_sprite_index: usize, tangent_map: &HashMap<i32, usize>, sprites: &Sprites, window_buffer: &mut Vec<u32>) {
    let key_width = sprites.keys[KEY_IDLE].width as i32;
    let key_height = sprites.keys[KEY_IDLE].height as usize;

    for (&pos, &tangent) in tangent_map {
        let tangent_sprite_index = if note_sprite_index == tangent {
            TANGENT_PRESSED
        } else {
            TANGENT_IDLE
        };

        let tangent_width = sprites.tangents[tangent_sprite_index].width as i32;

        // Calculate the x-coordinate of the tangent's center position
        let x = (pos * key_width) - (tangent_width / 2);

        // Ensure the x position is within bounds
        let x_usize = if x >= 0 { usize::try_from(x).unwrap_or(0) } else { 0 };

        draw_sprite(
            x_usize,
            2 * key_height,
            &sprites.tangents[tangent_sprite_index],
            window_buffer,
            WINDOW_WIDTH,
        );
    }
}