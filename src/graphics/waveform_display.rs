use crate::graphics::sprites::Sprite;
use crate::waveforms::sine_wave::calculate_sine;
use crate::waveforms::triangle_wave::calculate_triangle;
use crate::waveforms::sawtooth_wave::calculate_sawtooth;
use crate::waveforms::{Waveform, SAMPLE_RATE};

const DISPLAY_WIDTH: u32 = 164;
const DISPLAY_HEIGHT: u32 = 51;
const DISPLAY_CENTER_Y: u32 = DISPLAY_HEIGHT / 2;

/// Generates a real-time animated waveform visualization sprite for the given frequency and waveform type.
/// The animation_time parameter creates a phase shift that makes the wave appear to oscillate.
/// The amplitude parameter controls the fade-out effect (0.0 = invisible, 1.0 = full brightness).
pub fn generate_waveform_display(frequency: f32, waveform: Waveform, animation_time: f32, amplitude: f32) -> Sprite {
    let mut pixel_data = vec![0x00000000u32; (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize]; // Transparent background
    
    // Draw display frame
    draw_display_frame(&mut pixel_data);
    
    // Calculate how many samples to show across the display width
    let samples_per_cycle = SAMPLE_RATE / frequency;
    let cycles_to_show = 2.0; // Show 2 complete cycles
    let total_samples = (samples_per_cycle * cycles_to_show) as usize;
    
    // Only draw waveform if amplitude > 0
    if amplitude > 0.0 {
        // Calculate phase offset for animation (makes the wave appear to move)
        let phase_offset = (animation_time * frequency * 2.0 * std::f32::consts::PI) as usize;
        
        // Generate waveform points
        let mut previous_y = DISPLAY_CENTER_Y;
        
        for x in 0..DISPLAY_WIDTH {
            let sample_index = (x as f32 / DISPLAY_WIDTH as f32 * total_samples as f32) as usize + phase_offset;
            
            // Calculate waveform value (-1.0 to 1.0)
            let waveform_value = match waveform {
                Waveform::SINE => calculate_sine(frequency, sample_index),
                Waveform::SQUARE => {
                    let sine_val = calculate_sine(frequency, sample_index);
                    sine_val.signum() // Convert to square wave
                },
                Waveform::TRIANGLE => calculate_triangle(frequency, sample_index),
                Waveform::SAWTOOTH => calculate_sawtooth(frequency, sample_index),
            };
            
            // Convert waveform value to y coordinate (flip because screen coordinates)
            let y = (DISPLAY_CENTER_Y as f32 - (waveform_value * (DISPLAY_HEIGHT as f32 / 2.0) * 0.8)) as u32;
            let y = y.clamp(0, DISPLAY_HEIGHT - 1);
            
            // Apply amplitude fading and draw waveform point/line
            let green_intensity = (255.0 * amplitude).clamp(0.0, 255.0) as u32;
            let waveform_color = 0xFF000000 | (green_intensity << 8); // Green with alpha
            
            // Draw a smoother line by connecting points
            if x > 0 {
                draw_line(&mut pixel_data, x - 1, previous_y, x, y, waveform_color);
            } else {
                // First pixel - just draw the point
                draw_pixel(&mut pixel_data, x, y, waveform_color);
            }
            
            previous_y = y;
        }
    }
    
    // Don't draw center line - keep it clean with just the waveform
    
    Sprite::new(DISPLAY_WIDTH, DISPLAY_HEIGHT, pixel_data)
}

/// Draws a single pixel at the given coordinates
fn draw_pixel(pixel_data: &mut [u32], x: u32, y: u32, color: u32) {
    if x < DISPLAY_WIDTH && y < DISPLAY_HEIGHT {
        let index = (y * DISPLAY_WIDTH + x) as usize;
        if index < pixel_data.len() {
            pixel_data[index] = color;
        }
    }
}

/// Draws a line between two points using Bresenham's line algorithm
fn draw_line(pixel_data: &mut [u32], x0: u32, y0: u32, x1: u32, y1: u32, color: u32) {
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = (y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    
    let mut x = x0 as i32;
    let mut y = y0 as i32;
    
    loop {
        draw_pixel(pixel_data, x as u32, y as u32, color);
        
        if x == x1 as i32 && y == y1 as i32 {
            break;
        }
        
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Draws the display frame with outer border (#c7c7c7) and inner background (#141515)
fn draw_display_frame(pixel_data: &mut [u32]) {
    const OUTER_COLOR: u32 = 0xFFc7c7c7; // #c7c7c7
    const INNER_COLOR: u32 = 0xFF141515; // #141515
    
    // Fill the entire area with the inner color first
    for y in 0..DISPLAY_HEIGHT {
        for x in 0..DISPLAY_WIDTH {
            let index = (y * DISPLAY_WIDTH + x) as usize;
            if index < pixel_data.len() {
                pixel_data[index] = INNER_COLOR;
            }
        }
    }
    
    // Draw outer border (1 pixel border around the entire display)
    // Top border
    for x in 0..DISPLAY_WIDTH {
        draw_pixel(pixel_data, x, 0, OUTER_COLOR);
    }
    // Bottom border
    for x in 0..DISPLAY_WIDTH {
        draw_pixel(pixel_data, x, DISPLAY_HEIGHT - 1, OUTER_COLOR);
    }
    // Left border
    for y in 0..DISPLAY_HEIGHT {
        draw_pixel(pixel_data, 0, y, OUTER_COLOR);
    }
    // Right border
    for y in 0..DISPLAY_HEIGHT {
        draw_pixel(pixel_data, DISPLAY_WIDTH - 1, y, OUTER_COLOR);
    }
}

