use super::AudioEffect;

/// Enhanced delay effect with multiple taps and filtering
#[derive(Debug, Clone)]
pub struct DelayEffect {
    buffer: Vec<f32>,
    write_index: usize,
    delay_samples: usize,
    feedback: f32,    // Amount of delayed signal fed back (0.0 - 0.99)
    mix: f32,         // Dry/wet mix (0.0 = dry only, 1.0 = wet only)
    sample_rate: u32,
    // Multiple delay taps for richer sound
    tap1_samples: usize,
    tap2_samples: usize,
    // High-frequency damping filter
    damping_filter: f32,
    damping_coefficient: f32,
}

impl DelayEffect {
    /// Create a new delay effect
    /// 
    /// # Parameters
    /// - `delay_time_ms`: Delay time in milliseconds
    /// - `feedback`: Feedback amount (0.0 - 0.99)
    /// - `mix`: Dry/wet mix (0.0 - 1.0)
    /// - `sample_rate`: Audio sample rate (e.g., 44100)
    pub fn new(delay_time_ms: f32, feedback: f32, mix: f32, sample_rate: u32) -> Self {
        let delay_samples = ((delay_time_ms / 1000.0) * sample_rate as f32) as usize;
        let buffer_size = delay_samples.max(1024); // Ensure minimum buffer size
        
        // Create multiple delay taps for stereo width and richness
        let tap1_samples = (delay_samples as f32 * 0.618) as usize; // Golden ratio for musicality
        let tap2_samples = (delay_samples as f32 * 0.382) as usize; // Complementary ratio
        
        Self {
            buffer: vec![0.0; buffer_size],
            write_index: 0,
            delay_samples,
            feedback: feedback.clamp(0.0, 0.95), // Slightly higher max feedback
            mix: mix.clamp(0.0, 1.0),
            sample_rate,
            tap1_samples,
            tap2_samples,
            damping_filter: 0.0,
            damping_coefficient: 0.3, // Gentle high-frequency roll-off
        }
    }
    
    /// Update delay time in milliseconds
    pub fn set_delay_time(&mut self, delay_time_ms: f32) {
        let new_delay_samples = ((delay_time_ms / 1000.0) * self.sample_rate as f32) as usize;
        
        if new_delay_samples != self.delay_samples {
            self.delay_samples = new_delay_samples;
            
            // Update taps
            self.tap1_samples = (new_delay_samples as f32 * 0.618) as usize;
            self.tap2_samples = (new_delay_samples as f32 * 0.382) as usize;
            
            // Resize buffer if needed
            if new_delay_samples >= self.buffer.len() {
                self.buffer.resize(new_delay_samples + 1024, 0.0);
            }
        }
    }
    
    /// Set feedback amount (0.0 - 0.99)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.99);
    }
    
    /// Set dry/wet mix (0.0 - 1.0)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
    
    /// Read a sample from the delay buffer at a specific tap position
    fn read_tap(&self, tap_samples: usize) -> f32 {
        if tap_samples == 0 || tap_samples >= self.buffer.len() {
            return 0.0;
        }
        
        let read_index = if self.write_index >= tap_samples {
            self.write_index - tap_samples
        } else {
            self.buffer.len() - (tap_samples - self.write_index)
        };
        
        self.buffer[read_index % self.buffer.len()]
    }
}

impl AudioEffect for DelayEffect {
    fn process_sample(&mut self, input: f32) -> f32 {
        // Read from multiple delay taps for richer sound
        let main_tap = self.read_tap(self.delay_samples);
        let tap1 = self.read_tap(self.tap1_samples);
        let tap2 = self.read_tap(self.tap2_samples);
        
        // Mix the taps with different amplitudes
        let wet_signal = main_tap * 0.6 + tap1 * 0.25 + tap2 * 0.15;
        
        // Apply high-frequency damping to feedback
        self.damping_filter = wet_signal * (1.0 - self.damping_coefficient) + 
                            self.damping_filter * self.damping_coefficient;
        
        // Write new sample with damped feedback
        self.buffer[self.write_index] = input + (self.damping_filter * self.feedback);
        
        // Advance write index (circular)
        self.write_index = (self.write_index + 1) % self.buffer.len();
        
        // Mix dry and wet signals
        input * (1.0 - self.mix) + wet_signal * self.mix
    }
    
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_index = 0;
        self.damping_filter = 0.0;
    }
    
    fn name(&self) -> &str {
        "Delay"
    }
}