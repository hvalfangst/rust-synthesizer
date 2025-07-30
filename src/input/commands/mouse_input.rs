use minifb::{Key, MouseButton, MouseMode, Window};
use rodio::Sink;
use crate::music_theory::note::Note;
use crate::state::State;
use crate::state::utils::{get_key_mappings, handle_musical_note};
use crate::effects::AudioEffect;
use super::super::InputCommand;

/// Command for handling all mouse interactions
pub struct MouseInputCommand;

impl InputCommand for MouseInputCommand {
    fn execute(&self, state: &mut State, window: &mut Window, sink: &mut Sink) {
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

                // Calculate the distance between the current mouse position and the drag start position.
                // If the distance exceeds 3.0 (a small threshold to avoid accidental drags),
                // set the `dragging` flag to true, indicating that a drag operation is in progress.
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
        
        // Handle control button interactions - DISABLED: now using per-track transport
        // handle_control_buttons_mouse(state, sink);
        
        // Handle effects button interactions
        handle_effects_buttons_mouse(state, sink);
        
        // Handle track selection clicks
        handle_track_selection_mouse(state, sink);
    }
}

/// Handle mouse interactions with ADSR faders
pub fn handle_adsr_fader_mouse(state: &mut State, sink: &mut Sink) {
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

                // Update the appropriate ADSR parameter on current track
                match *param {
                    "attack" => {
                        state.tracks[state.current_track_id].attack = new_value;
                        state.attack = new_value; // Sync legacy state
                    },
                    "decay" => {
                        state.tracks[state.current_track_id].decay = new_value;
                        state.decay = new_value; // Sync legacy state
                    },
                    "sustain" => {
                        state.tracks[state.current_track_id].sustain = new_value;
                        state.sustain = new_value; // Sync legacy state
                    },
                    "release" => {
                        state.tracks[state.current_track_id].release = new_value;
                        state.release = new_value; // Sync legacy state
                    },
                    _ => {}
                }
            }
        }
    }
}

/// Handle mouse interactions with tangent (sharp) keys
/// Returns true if a tangent was clicked, false otherwise
pub fn handle_tangent_mouse(state: &mut State, sink: &mut Sink) -> bool {
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

                // Record note if recording - record to current track
                if state.recording_state == crate::state::RecordingState::Recording {
                    // Finish previous note if there was one
                    if let Some((start_time, prev_note, prev_octave)) = state.current_note_start.take() {
                        let duration = start_time.elapsed().as_secs_f32();
                        let timestamp = state.recording_start_time
                            .map(|start| start.elapsed().as_secs_f32() - duration)
                            .unwrap_or(0.0);

                        // Add to current track instead of global recorded_notes
                        state.add_note_to_current_track(crate::state::RecordedNote {
                            note: prev_note,
                            octave: prev_octave,
                            timestamp,
                            duration,
                        });
                    }

                    // Start recording new note using current track's octave
                    let current_track_octave = state.tracks[state.current_track_id].octave;
                    state.current_note_start = Some((std::time::Instant::now(), note, current_track_octave));
                }
                return true; // Return true to indicate a tangent was clicked
            }
        }
    }
    false // Return false if no tangent was clicked
}

/// Handle mouse interactions with keyboard keys
pub fn handle_keyboard_mouse(state: &mut State, sink: &mut Sink) {
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


                // Record note if recording - record to current track
                if state.recording_state == crate::state::RecordingState::Recording {
                    // Finish previous note if there was one
                    if let Some((start_time, prev_note, prev_octave)) = state.current_note_start.take() {
                        let duration = start_time.elapsed().as_secs_f32();
                        let timestamp = state.recording_start_time
                            .map(|start| start.elapsed().as_secs_f32() - duration)
                            .unwrap_or(0.0);

                        // Add to current track instead of global recorded_notes
                        state.add_note_to_current_track(crate::state::RecordedNote {
                            note: prev_note,
                            octave: prev_octave,
                            timestamp,
                            duration,
                        });
                    }

                    // Start recording new note using current track's octave
                    let current_track_octave = state.tracks[state.current_track_id].octave;
                    state.current_note_start = Some((std::time::Instant::now(), note, current_track_octave));
                }
                return; // Exit after handling one key to avoid multiple triggers
            }
        }
    }

}

/// Handle mouse interactions with octave fader
pub fn handle_octave_fader_mouse(state: &mut State) {
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
                state.increase_current_track_octave();
            } else {
                // Clicked in lower part - decrease octave
                state.decrease_current_track_octave();
            }
        }
    }
}

/// Handle mouse interactions with waveform display
pub fn handle_waveform_display_mouse(state: &mut State) {
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
            state.toggle_current_track_waveform();
        }
    }
}

/// Handle mouse interactions with control buttons
pub fn handle_control_buttons_mouse(state: &mut State, sink: &mut Sink) {
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
            // Stop all audio immediately
            sink.stop();

            // Stop recording and playback
            state.stop_recording();
            state.stop_playback();

            // Clear any pressed keys and reset audio state
            state.pressed_key = None;
            state.current_frequency = None;
            state.key_release_time = None;

            // Set glow effect for visual feedback
            state.stop_button_glow_time = Some(std::time::Instant::now());
        }
    }
}

/// Handle mouse interactions with effects buttons
pub fn handle_effects_buttons_mouse(state: &mut State, sink: &mut Sink) {
    // Match the positioning from draw_effects_buttons
    let display_end_x = 164 + 164; // 328
    let adsr_start_x = 164 + 164 + 104; // 432
    let available_width = adsr_start_x - display_end_x; // 104px
    
    let button_width = 30;
    let button_height = 20;
    let button_spacing = (available_width - (3 * button_width)) / 4;
    let base_x = display_end_x + button_spacing;
    let base_y = 4 * 51 + 17 + 15;
    
    // Check each effect button
    for i in 0..3 {
        let button_x = base_x + i * (button_width + button_spacing);
        
        // Check if mouse is over this button
        if state.mouse.x >= button_x as f32 && state.mouse.x <= (button_x + button_width) as f32 &&
           state.mouse.y >= base_y as f32 && state.mouse.y <= (base_y + button_height) as f32 {
            
            if state.mouse.left_clicked {
                // Toggle the appropriate effect on current track
                match i {
                    0 => {
                        // Delay button
                        state.toggle_current_track_delay();
                        let current_track_id = state.current_track_id;
                        if !state.tracks[current_track_id].delay_enabled {
                            state.tracks[current_track_id].delay_effect.reset();
                        }
                    },
                    1 => {
                        // Reverb button
                        state.toggle_current_track_reverb();
                        let current_track_id = state.current_track_id;
                        if !state.tracks[current_track_id].reverb_enabled {
                            state.tracks[current_track_id].reverb_effect.reset();
                        }
                    },
                    2 => {
                        // Flanger button
                        state.toggle_current_track_flanger();
                        let current_track_id = state.current_track_id;
                        if !state.tracks[current_track_id].flanger_enabled {
                            state.tracks[current_track_id].flanger_effect.reset();
                        }
                    },
                    _ => {}
                }
                return; // Exit after handling one button
            }
        }
    }
}

/// Handle mouse interactions with track display, transport controls, and mute/solo buttons
pub fn handle_track_selection_mouse(state: &mut State, sink: &mut Sink) {
    // Track display positions (matching draw_track_info)
    let base_x = 10;
    let base_y = 10;
    let track_height = 25;
    let track_width = 350; // Updated width for transport controls
    
    // Check each track
    for i in 0..state.tracks.len() {
        let track_y = base_y + i * track_height;
        let transport_x = base_x + 80;
        
        // Check record button
        if state.mouse.x >= transport_x as f32 && state.mouse.x <= (transport_x + 16) as f32 &&
           state.mouse.y >= (track_y + 2) as f32 && state.mouse.y <= (track_y + 18) as f32 {
            
            if state.mouse.left_clicked {
                // Switch to this track first
                state.switch_to_track(i);
                
                // Handle record toggle
                match state.recording_state {
                    crate::state::RecordingState::Stopped => {
                        state.start_track_recording();
                        println!("Recording on track {}: {}", i + 1, state.tracks[i].name);
                    },
                    crate::state::RecordingState::Recording => {
                        state.stop_recording();
                        println!("Stopped recording on track {}: {}", i + 1, state.tracks[i].name);
                    },
                    crate::state::RecordingState::Playing => {
                        state.stop_playback();
                        state.start_track_recording();
                        println!("Switched to recording on track {}: {}", i + 1, state.tracks[i].name);
                    },
                }
                return;
            }
        }
        
        // Check play button - now toggles individual track playback
        let play_x = transport_x + 20;
        if state.mouse.x >= play_x as f32 && state.mouse.x <= (play_x + 16) as f32 &&
           state.mouse.y >= (track_y + 2) as f32 && state.mouse.y <= (track_y + 18) as f32 {
            
            if state.mouse.left_clicked {
                // Toggle individual track playback only if track has content
                if !state.tracks[i].recorded_notes.is_empty() {
                    state.tracks[i].playing = !state.tracks[i].playing;
                    println!("Track {} ({}) playing: {}", i + 1, state.tracks[i].name, state.tracks[i].playing);
                    
                    // If any tracks are now playing, switch to playing mode
                    // If no tracks are playing, stop playback mode
                    if state.has_playing_tracks() {
                        if state.recording_state != crate::state::RecordingState::Playing {
                            state.recording_state = crate::state::RecordingState::Playing;
                            state.playback_start_time = Some(std::time::Instant::now());
                        }
                    } else {
                        state.stop_playback();
                    }
                } else {
                    println!("Track {} ({}) has no recorded content to play", i + 1, state.tracks[i].name);
                }
                return;
            }
        }
        
        // Check stop button
        let stop_x = play_x + 20;
        if state.mouse.x >= stop_x as f32 && state.mouse.x <= (stop_x + 16) as f32 &&
           state.mouse.y >= (track_y + 2) as f32 && state.mouse.y <= (track_y + 18) as f32 {
            
            if state.mouse.left_clicked {
                // Stop everything
                sink.stop(); // Stop all audio immediately
                state.stop_recording();
                state.stop_playback();
                state.stop_all_track_playback(); // Stop individual track playback
                
                // Clear any pressed keys and reset audio state
                state.pressed_key = None;
                state.current_frequency = None;
                state.key_release_time = None;
                
                println!("Stopped all transport");
                return;
            }
        }
        
        
        // Check track name area for selection (avoid buttons)
        let name_area_width = 75; // Just the name area
        if state.mouse.x >= base_x as f32 && state.mouse.x <= (base_x + name_area_width) as f32 &&
           state.mouse.y >= track_y as f32 && state.mouse.y <= (track_y + 20) as f32 {
            
            if state.mouse.left_clicked {
                // Switch to this track
                state.switch_to_track(i);
                
                // Update legacy state to match selected track
                let current_track_id = state.current_track_id;
                let track = &state.tracks[current_track_id];
                state.octave = track.octave;
                state.waveform = track.waveform.clone();
                state.attack = track.attack;
                state.decay = track.decay;
                state.sustain = track.sustain;
                state.release = track.release;
                state.delay_enabled = track.delay_enabled;
                state.reverb_enabled = track.reverb_enabled;
                state.flanger_enabled = track.flanger_enabled;
                
                println!("Switched to track {}: {}", i + 1, track.name);
                return; // Exit after handling one track
            }
        }
    }
}