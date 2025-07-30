use rodio::Source;
use std::time::Duration;

/// ADSR envelope wrapper that applies envelope shaping to any source
pub struct ADSREnvelope<S>
where
    S: Source<Item = f32>,
{
    source: S,
    sample_count: usize,
    attack_samples: usize,
    decay_samples: usize,
    sustain_level: f32,
    release_samples: usize,
    release_start_sample: Option<usize>,
    is_released: bool,
    max_sustain_samples: usize, // Maximum time to hold sustain before auto-release
}

impl<S> ADSREnvelope<S>
where
    S: Source<Item = f32>,
{
    pub fn new(
        source: S,
        attack: f32,    // Attack time in seconds
        decay: f32,     // Decay time in seconds
        sustain: f32,   // Sustain level (0.0 to 1.0)
        release: f32,   // Release time in seconds
    ) -> Self {
        let sample_rate = source.sample_rate() as f32;
        
        Self {
            source,
            sample_count: 0,
            attack_samples: (attack * sample_rate) as usize,
            decay_samples: (decay * sample_rate) as usize,
            sustain_level: sustain,
            release_samples: ((release * sample_rate) as usize).max(1), // Minimum 1 sample for release
            release_start_sample: None,
            is_released: false,
            max_sustain_samples: ((release * 0.5 + 0.05) * sample_rate) as usize, // Shorter auto-release based on release setting
        }
    }

    pub fn release(&mut self) {
        if !self.is_released {
            self.release_start_sample = Some(self.sample_count);
            self.is_released = true;
        }
    }

    fn calculate_envelope_amplitude(&self) -> f32 {
        if let Some(release_start) = self.release_start_sample {
            // Release phase
            let release_progress = self.sample_count - release_start;
            if release_progress >= self.release_samples {
                return 0.0; // Envelope finished
            }
            
            if self.release_samples == 0 {
                return 0.0;
            }
            
            let release_factor = 1.0 - (release_progress as f32 / self.release_samples as f32);
            return self.sustain_level * release_factor;
        }

        if self.sample_count <= self.attack_samples {
            // Attack phase
            if self.attack_samples == 0 {
                return 1.0;
            }
            let attack_progress = self.sample_count as f32 / self.attack_samples as f32;
            return attack_progress.min(1.0);
        } else if self.sample_count <= self.attack_samples + self.decay_samples {
            // Decay phase
            if self.decay_samples == 0 {
                return self.sustain_level;
            }
            let decay_progress = (self.sample_count - self.attack_samples) as f32 / self.decay_samples as f32;
            return 1.0 - (1.0 - self.sustain_level) * decay_progress;
        } else {
            // Sustain phase 
            return self.sustain_level;
        }
    }
}

impl<S> Iterator for ADSREnvelope<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.source.next()?;
        let envelope_amplitude = self.calculate_envelope_amplitude();
        
        self.sample_count += 1;
        
        // Auto-release after max sustain time
        if self.sample_count > self.attack_samples + self.decay_samples + self.max_sustain_samples && !self.is_released {
            self.release_start_sample = Some(self.sample_count);
            self.is_released = true;
        }
        
        // If we're in release phase and envelope is finished, return None to end the sound
        if let Some(release_start) = self.release_start_sample {
            if self.sample_count - release_start >= self.release_samples {
                return None;
            }
        }
        
        // Only end sound if we're in release phase and amplitude is effectively zero
        if envelope_amplitude < 0.0001 && self.release_start_sample.is_some() {
            return None;
        }
        
        Some(sample * envelope_amplitude)
    }
}

impl<S> Source for ADSREnvelope<S>
where
    S: Source<Item = f32>,
{
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
        None // ADSR envelope can vary in duration
    }
}