use std::collections::HashMap;

use minifb::Key;
use rodio::{Sink, Source};
use crate::audio::MultiTrackMixer;
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
    // Get current track info without borrowing
    let current_track_id = state.current_track_id;
    let base_frequency = note.frequency(state.tracks[current_track_id].octave);

    // Store the current frequency for display purposes and reset animation timing
    state.current_frequency = Some(base_frequency);
    state.animation_start_time = std::time::Instant::now();
    state.key_release_time = None; // Clear any previous release time

    // Stop any currently playing audio to prevent queueing
    sink.stop();

    // Create mixer and play note on current track
    let mixer = MultiTrackMixer::new(44100);
    let current_track = &state.tracks[current_track_id];
    mixer.play_note_on_track(current_track, note, sink);
    
    // Return early - mixer handles everything now
    return;

    /* LEGACY CODE - COMMENTED OUT
    // For backwards compatibility, also create the original synth
    // TODO: Remove this once full multi-track system is implemented
    let synth = match current_track.waveform {
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
    */
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
    
    // Draw control buttons - DISABLED: now using per-track transport
    // draw_control_buttons(state, window_buffer);
    
    // Draw effects buttons
    draw_effects_buttons(state, window_buffer);
    
    // Draw track information
    draw_track_info(state, window_buffer);

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
    // Use current track's waveform
    let current_track_waveform = state.tracks[state.current_track_id].waveform.clone();
    let waveform_sprite = generate_waveform_display(frequency, current_track_waveform, animation_time, amplitude);
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
    
    // Show current track's effects
    let current_track = &state.tracks[state.current_track_id];
    let effects = [
        ("DLY", current_track.delay_enabled, 0xFF4444FF), // Blue for delay
        ("REV", current_track.reverb_enabled, 0xFF44FF44), // Green for reverb  
        ("FLG", current_track.flanger_enabled, 0xFFFF4444), // Red for flanger
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

/// Draw track information display with per-track transport controls
pub fn draw_track_info(state: &State, buffer: &mut Vec<u32>) {
    let base_x = 10;
    let base_y = 10;
    let track_height = 25;
    let track_width = 250; // Reduced width since we removed mute/solo
    
    // Draw all 4 tracks
    for (i, track) in state.tracks.iter().enumerate() {
        let y = base_y + i * track_height;
        let is_current = i == state.current_track_id;
        let is_recording = state.recording_state == crate::state::RecordingState::Recording && is_current;
        let is_track_playing = track.playing;
        
        // Choose colors based on track state
        let (bg_color, text_color) = if is_current {
            (0xFF444444, 0xFFFFFFFF) // Bright background for current track
        } else {
            (0xFF222222, 0xFF888888) // Dark background for other tracks
        };
        
        // Draw track background
        draw_track_bar(base_x, y, track_width, 20, bg_color, buffer);
        
        // Draw track number and name
        let track_text = format!("{}: {}", i + 1, track.name);
        draw_simple_text(base_x + 5, y + 5, &track_text, text_color, buffer);
        
        // Transport controls start after track name
        let transport_x = base_x + 80;
        
        // Record button (red circle ●)
        let rec_color = if is_recording { 0xFFFF0000 } else { 0xFF660000 };
        draw_transport_button(transport_x, y + 2, 16, 16, rec_color, buffer);
        draw_record_symbol(transport_x + 5, y + 7, if is_recording { 0xFFFFFFFF } else { 0xFF888888 }, buffer);
        
        // Play button (triangle ▶) - now shows individual track play state
        let play_x = transport_x + 20;
        let has_content = !track.recorded_notes.is_empty();
        let play_color = if is_track_playing && has_content { 0xFF00AA00 } else { 0xFF006600 };
        draw_transport_button(play_x, y + 2, 16, 16, play_color, buffer);
        draw_play_symbol(play_x + 4, y + 5, if is_track_playing { 0xFFFFFFFF } else { 0xFF888888 }, buffer);
        
        // Stop button (square ■)
        let stop_x = play_x + 20;
        let stop_color = 0xFF666666;
        draw_transport_button(stop_x, y + 2, 16, 16, stop_color, buffer);
        draw_stop_symbol(stop_x + 4, y + 6, 0xFF888888, buffer);
        
        // Loop indicator
        let loop_x = stop_x + 25;
        let has_loop = !track.recorded_notes.is_empty();
        let loop_color = if has_loop { 0xFF00AAFF } else { 0xFF333333 };
        draw_button(loop_x, y + 2, 20, 16, loop_color, buffer);
        draw_simple_text(loop_x + 2, y + 7, "♪", if has_loop { 0xFFFFFFFF } else { 0xFF666666 }, buffer);
        
        // Volume indicator  
        let vol_x = loop_x + 25;
        let vol_width = (track.volume * 25.0) as usize;
        draw_volume_bar(vol_x, y + 8, vol_width, 4, 0xFF0088FF, buffer);
    }
}

/// Draw a simple track background bar
fn draw_track_bar(x: usize, y: usize, width: usize, height: usize, color: u32, buffer: &mut Vec<u32>) {
    for dy in 0..height {
        for dx in 0..width {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() {
                buffer[index] = color;
            }
        }
    }
}

/// Draw simple text using a basic bitmap font
fn draw_simple_text(x: usize, y: usize, text: &str, color: u32, buffer: &mut Vec<u32>) {
    // Simple 3x5 bitmap font (limited character set)
    let font_patterns = std::collections::HashMap::from([
        ('1', vec![0b010, 0b110, 0b010, 0b010, 0b111]),
        ('2', vec![0b111, 0b001, 0b111, 0b100, 0b111]),
        ('3', vec![0b111, 0b001, 0b111, 0b001, 0b111]),
        ('4', vec![0b101, 0b101, 0b111, 0b001, 0b001]),
        ('L', vec![0b100, 0b100, 0b100, 0b100, 0b111]),
        ('e', vec![0b000, 0b111, 0b101, 0b110, 0b111]),
        ('a', vec![0b000, 0b011, 0b101, 0b101, 0b011]),
        ('d', vec![0b001, 0b011, 0b101, 0b101, 0b011]),
        ('B', vec![0b110, 0b101, 0b110, 0b101, 0b110]),
        ('s', vec![0b000, 0b111, 0b100, 0b001, 0b111]),
        ('r', vec![0b000, 0b110, 0b100, 0b100, 0b100]),
        ('u', vec![0b000, 0b101, 0b101, 0b101, 0b011]),
        ('m', vec![0b000, 0b110, 0b111, 0b101, 0b101]),
        ('D', vec![0b110, 0b101, 0b101, 0b101, 0b110]),
        ('P', vec![0b111, 0b101, 0b111, 0b100, 0b100]),
        (':', vec![0b000, 0b010, 0b000, 0b010, 0b000]),
        (' ', vec![0b000, 0b000, 0b000, 0b000, 0b000]),
        ('M', vec![0b101, 0b111, 0b101, 0b101, 0b101]),
        ('S', vec![0b111, 0b100, 0b111, 0b001, 0b111]),
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

/// Draw a volume level bar
fn draw_volume_bar(x: usize, y: usize, width: usize, height: usize, color: u32, buffer: &mut Vec<u32>) {
    for dy in 0..height {
        for dx in 0..width {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() {
                buffer[index] = color;
            }
        }
    }
}

/// Draw a clickable button
fn draw_button(x: usize, y: usize, width: usize, height: usize, color: u32, buffer: &mut Vec<u32>) {
    // Draw button background
    for dy in 1..height-1 {
        for dx in 1..width-1 {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() {
                buffer[index] = color;
            }
        }
    }
    
    // Draw border
    let border_color = 0xFF888888;
    for dy in 0..height {
        for dx in 0..width {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() && (dx == 0 || dx == width - 1 || dy == 0 || dy == height - 1) {
                buffer[index] = border_color;
            }
        }
    }
}

/// Draw a transport button (rounded style)
fn draw_transport_button(x: usize, y: usize, width: usize, height: usize, color: u32, buffer: &mut Vec<u32>) {
    // Draw rounded button background
    for dy in 0..height {
        for dx in 0..width {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() {
                // Skip corners for rounded effect
                let is_corner = (dx <= 1 || dx >= width - 2) && (dy <= 1 || dy >= height - 2);
                if !is_corner {
                    buffer[index] = color;
                }
            }
        }
    }
}

/// Draw record symbol (filled circle)
fn draw_record_symbol(x: usize, y: usize, color: u32, buffer: &mut Vec<u32>) {
    // Draw a 6x6 filled circle
    let circle = [
        0b011110,
        0b111111,
        0b111111,
        0b111111,
        0b111111,
        0b011110,
    ];
    
    for (row, &bits) in circle.iter().enumerate() {
        for col in 0..6 {
            if (bits >> (5 - col)) & 1 == 1 {
                let pixel_x = x + col;
                let pixel_y = y + row;
                let index = pixel_y * WINDOW_WIDTH + pixel_x;
                
                if index < buffer.len() {
                    buffer[index] = color;
                }
            }
        }
    }
}

/// Draw play symbol (triangle pointing right)
fn draw_play_symbol(x: usize, y: usize, color: u32, buffer: &mut Vec<u32>) {
    // Draw a right-pointing triangle
    let triangle = [
        0b100000,
        0b110000,
        0b111000,
        0b111100,
        0b111110,
        0b111100,
        0b111000,
        0b110000,
        0b100000,
    ];
    
    for (row, &bits) in triangle.iter().enumerate() {
        for col in 0..6 {
            if (bits >> (5 - col)) & 1 == 1 {
                let pixel_x = x + col;
                let pixel_y = y + row;
                let index = pixel_y * WINDOW_WIDTH + pixel_x;
                
                if index < buffer.len() {
                    buffer[index] = color;
                }
            }
        }
    }
}

/// Draw stop symbol (filled square)
fn draw_stop_symbol(x: usize, y: usize, color: u32, buffer: &mut Vec<u32>) {
    // Draw a 8x8 filled square
    for dy in 0..8 {
        for dx in 0..8 {
            let pixel_x = x + dx;
            let pixel_y = y + dy;
            let index = pixel_y * WINDOW_WIDTH + pixel_x;
            
            if index < buffer.len() {
                buffer[index] = color;
            }
        }
    }
}