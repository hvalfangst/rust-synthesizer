use rodio::Source;
use std::time::Duration;
use crate::{
    waveforms::{MONO, SAMPLE_RATE}
};

#[derive(Debug)]
pub struct SawtoothWave {
    freq: f32,
    num_sample: usize
}

impl SawtoothWave {
    pub fn new(freq: f32) -> SawtoothWave {
        SawtoothWave { freq, num_sample: 0}
    }
    pub fn generate_sawtooth_wave(&mut self) -> f32 {
        calculate_sawtooth(self.freq, self.num_sample)
    }
}

/// Implementation of the [Iterator] trait for the [SawtoothWave]
impl Iterator for SawtoothWave {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        // increment sample counter by 1
        self.num_sample = self.num_sample.wrapping_add(1);

        // Generates a sawtooth wave
        let sawtooth_wave = self.generate_sawtooth_wave();

        Some(sawtooth_wave)
    }
}

/// Implementation of the [Source] trait for the [SawtoothWave]
impl Source for SawtoothWave {
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

/// Calculates a sawtooth wave value for a given frequency and sample number.
/// Sawtooth wave rises linearly from -1 to 1 then drops immediately back to -1.
pub fn calculate_sawtooth(frequency: f32, num_sample: usize) -> f32 {
    // Calculate time in seconds based on the sample number and the sample rate
    let time: f32 = num_sample as f32 / SAMPLE_RATE;
    // Calculate the period of the wave
    let period: f32 = 1.0 / frequency;
    
    // Calculate position within the current period (0 to 1)
    let phase = (time % period) / period;
    
    // Generate sawtooth wave: linear rise from -1 to 1
    2.0 * phase - 1.0
}