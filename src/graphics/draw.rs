use std::collections::HashMap;
use minifb::Window;
use crate::graphics::constants::{KEY_IDLE, KEY_PRESSED, TANGENT_IDLE, TANGENT_PRESSED, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::graphics::sprites::{draw_sprite, Sprite, Sprites};
use crate::state::State;

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

    // Stop button with enhanced feedback
    let stop_x = play_x + button_width + 10;
    let stop_color = if let Some(glow_start) = state.stop_button_glow_time {
        // Show intense red glow for 0.5 seconds after clicking
        if glow_start.elapsed().as_secs_f32() < 0.5 {
            0xFFFF0000 // Intense red glow
        } else {
            // After glow expires, show normal color based on state
            match state.recording_state {
                crate::state::RecordingState::Recording | crate::state::RecordingState::Playing => 0xFFFF4444, // Bright red when there's something to stop
                _ => 0xFF666666, // Gray when idle
            }
        }
    } else {
        // Normal color based on state
        match state.recording_state {
            crate::state::RecordingState::Recording | crate::state::RecordingState::Playing => 0xFFFF4444, // Bright red when there's something to stop
            _ => 0xFF666666, // Gray when idle
        }
    };
    draw_button(stop_x, button_y, button_width, button_height, stop_color, "STOP", buffer);
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