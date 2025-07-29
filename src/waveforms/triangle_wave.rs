use rodio::Source;
use std::time::Duration;
use crate::{
    waveforms::{MONO, SAMPLE_RATE}
};

#[derive(Debug)]
pub struct TriangleWave {
    freq: f32,
    num_sample: usize
}

impl TriangleWave {
    pub fn new(freq: f32) -> TriangleWave {
        TriangleWave { freq, num_sample: 0}
    }
    pub fn generate_triangle_wave(&mut self) -> f32 {
        calculate_triangle(self.freq, self.num_sample)
    }
}

/// Implementation of the [Iterator] trait for the [TriangleWave]
impl Iterator for TriangleWave {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        // increment sample counter by 1
        self.num_sample = self.num_sample.wrapping_add(1);

        // Generates a triangle wave
        let triangle_wave = self.generate_triangle_wave();

        Some(triangle_wave)
    }
}

/// Implementation of the [Source] trait for the [TriangleWave]
impl Source for TriangleWave {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        MONO
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE as u32
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Calculates a triangle wave value for a given frequency and sample number.
/// Triangle wave oscillates linearly between -1 and 1, creating a triangular shape.
pub fn calculate_triangle(frequency: f32, num_sample: usize) -> f32 {
    // Calculate time in seconds based on the sample number and the sample rate
    let time: f32 = num_sample as f32 / SAMPLE_RATE;
    // Calculate the period of the wave
    let period: f32 = 1.0 / frequency;
    
    // Calculate position within the current period (0 to 1)
    let phase = (time % period) / period;
    
    // Generate triangle wave: rises from -1 to 1 in first half, falls from 1 to -1 in second half
    if phase < 0.5 {
        4.0 * phase - 1.0  // Rising edge: -1 to 1
    } else {
        3.0 - 4.0 * phase  // Falling edge: 1 to -1
    }
}