use std::collections::HashMap;

use minifb::Key;
use rodio::{Sink, Source};
use crate::effects::{EffectWrapper, AudioEffect, DelayEffect, ReverbEffect, FlangerEffect};
use std::time::Duration;

use crate::graphics::draw::{draw_adsr_faders, draw_control_buttons, draw_display_sprite_single, draw_idle_key_sprites, draw_idle_tangent_sprites, draw_note_sprite, draw_octave_fader_sprite, draw_pressed_key_sprite, draw_rack_sprite, draw_tangent_sprites};
use crate::graphics::sprites::Sprites;
use crate::music_theory::note::Note;
use crate::state::State;
use crate::waveforms::adsr_envelope::ADSREnvelope;
use crate::waveforms::sawtooth_wave::SawtoothWave;
use crate::waveforms::sine_wave::SineWave;
use crate::waveforms::square_wave::SquareWave;
use crate::waveforms::triangle_wave::TriangleWave;
use crate::waveforms::{Waveform, AMPLITUDE};

/// Effects processor that applies enabled effects to an audio source
struct EffectsProcessor<S: Source<Item = f32>> {
    source: S,
    delay_effect: DelayEffect,
    reverb_effect: ReverbEffect,
    flanger_effect: FlangerEffect,
    delay_enabled: bool,
    reverb_enabled: bool,
    flanger_enabled: bool,
}

impl<S: Source<Item = f32>> EffectsProcessor<S> {
    fn new(source: S, state: &State) -> Self {
        Self {
            source,
            delay_effect: DelayEffect::new(300.0, 0.55, 0.5, 44100), // Enhanced parameters
            reverb_effect: ReverbEffect::new(0.7, 0.4, 0.6, 44100),  // Larger room, more wet
            flanger_effect: FlangerEffect::new(0.5, 0.7, 0.1, 0.5, 44100),
            delay_enabled: state.delay_enabled,
            reverb_enabled: state.reverb_enabled,
            flanger_enabled: state.flanger_enabled,
        }
    }
}

impl<S: Source<Item = f32>> Iterator for EffectsProcessor<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next().map(|mut sample| {
            // Apply effects in series: Delay -> Reverb -> Flanger
            if self.delay_enabled {
                sample = self.delay_effect.process_sample(sample);
            }
            if self.reverb_enabled {
                sample = self.reverb_effect.process_sample(sample);
            }
            if self.flanger_enabled {
                sample = self.flanger_effect.process_sample(sample);
            }
            sample
        })
    }
}

impl<S: Source<Item = f32>> Source for EffectsProcessor<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
}
use crate::{
    graphics::constants::*,
    graphics::waveform_display::generate_waveform_display
};

// Handles playing a musical note with a specified octave, waveform, and duration.
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
    let mut source = synth.amplify(AMPLITUDE);

    // Apply effects chain if any are enabled
    let source_with_effects: Box<dyn Source<Item=f32> + Send> = if state.delay_enabled || state.reverb_enabled || state.flanger_enabled {
        // Create an effects-processing source
        Box::new(EffectsProcessor::new(source, state))
    } else {
        Box::new(source)
    };

    // Play the sound source immediately, replacing any queued audio
    let _result = sink.append(source_with_effects);
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
    
    // Draw effects buttons
    draw_effects_buttons(state, window_buffer);

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

/// Draws effects buttons (Delay, Reverb, Flanger) between waveform display and ADSR faders
pub fn draw_effects_buttons(state: &State, buffer: &mut Vec<u32>) {
    // Position between waveform display and ADSR faders
    let display_end_x = 164 + 164; // 328
    let adsr_start_x = 164 + 164 + 104; // 432
    let available_width = adsr_start_x - display_end_x; // 104px
    
    let button_width = 30;
    let button_height = 20;
    let button_spacing = (available_width - (3 * button_width)) / 4; // Equal spacing
    let base_x = display_end_x + button_spacing;
    let base_y = 4 * 51 + 17 + 15; // Same Y as display + offset
    
    let effects = [
        ("DLY", state.delay_enabled, 0xFF4444FF), // Blue for delay
        ("REV", state.reverb_enabled, 0xFF44FF44), // Green for reverb  
        ("FLG", state.flanger_enabled, 0xFFFF4444), // Red for flanger
    ];
    
    for (i, (label, enabled, base_color)) in effects.iter().enumerate() {
        let x = base_x + i * (button_width + button_spacing);
        
        // Choose colors based on state
        let (bg_color, border_color, text_color) = if *enabled {
            (*base_color, 0xFFFFFFFF, 0xFFFFFFFF) // Bright when enabled
        } else {
            (0xFF333333, 0xFF666666, 0xFF999999) // Dark when disabled
        };
        
        // Draw button background and border with rounded corners effect
        draw_effects_button_shape(x, base_y, button_width, button_height, bg_color, border_color, buffer);
        
        // Draw label text centered
        let text_x = x + button_width / 2 - (label.len() * 2); // Rough centering
        let text_y = base_y + button_height / 2 - 3;
        draw_effects_button_text(text_x, text_y, label, text_color, buffer);
    }
}

/// Draw a button shape with rounded corners effect and glow
fn draw_effects_button_shape(x: usize, y: usize, width: usize, height: usize, bg_color: u32, border_color: u32, buffer: &mut Vec<u32>) {
    // Draw main button body
    for dy in 1..height-1 {
        for dx in 1..width-1 {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() {
                buffer[index] = bg_color;
            }
        }
    }
    
    // Draw border with rounded corner effect
    for dy in 0..height {
        for dx in 0..width {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() {
                // Skip corners for rounded effect
                let is_corner = (dx == 0 || dx == width - 1) && (dy == 0 || dy == height - 1);
                if !is_corner && (dx == 0 || dx == width - 1 || dy == 0 || dy == height - 1) {
                    buffer[index] = border_color;
                }
            }
        }
    }
    
    // Add subtle highlight on top edge
    for dx in 2..width-2 {
        let pixel_x = x + dx;
        let pixel_y = y + 1;
        let index = pixel_y * WINDOW_WIDTH + pixel_x;
        
        if index < buffer.len() {
            let highlight = blend_colors(bg_color, 0xFFFFFFFF, 0.3);
            buffer[index] = highlight;
        }
    }
}

/// Draw text for effects buttons using a simple bitmap font
fn draw_effects_button_text(x: usize, y: usize, text: &str, color: u32, buffer: &mut Vec<u32>) {
    // Simple 3x5 bitmap font patterns for effect labels
    let font_patterns = std::collections::HashMap::from([
        ('D', vec![0b111, 0b101, 0b101, 0b101, 0b111]),
        ('L', vec![0b100, 0b100, 0b100, 0b100, 0b111]),
        ('Y', vec![0b101, 0b101, 0b010, 0b010, 0b010]),
        ('R', vec![0b111, 0b101, 0b111, 0b110, 0b101]),
        ('E', vec![0b111, 0b100, 0b111, 0b100, 0b111]),
        ('V', vec![0b101, 0b101, 0b101, 0b101, 0b010]),
        ('F', vec![0b111, 0b100, 0b111, 0b100, 0b100]),
        ('G', vec![0b111, 0b100, 0b101, 0b101, 0b111]),
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

/// Blend two colors together
fn blend_colors(color1: u32, color2: u32, factor: f32) -> u32 {
    let r1 = ((color1 >> 16) & 0xFF) as f32;
    let g1 = ((color1 >> 8) & 0xFF) as f32;
    let b1 = (color1 & 0xFF) as f32;
    
    let r2 = ((color2 >> 16) & 0xFF) as f32;
    let g2 = ((color2 >> 8) & 0xFF) as f32;
    let b2 = (color2 & 0xFF) as f32;
    
    let r = (r1 * (1.0 - factor) + r2 * factor) as u32;
    let g = (g1 * (1.0 - factor) + g2 * factor) as u32;
    let b = (b1 * (1.0 - factor) + b2 * factor) as u32;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}