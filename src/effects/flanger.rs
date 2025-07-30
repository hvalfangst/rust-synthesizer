use super::AudioEffect;
use std::f32::consts::PI;

/// Flanger effect using modulated delay line
#[derive(Debug, Clone)]
pub struct FlangerEffect {
    buffer: Vec<f32>,
    write_index: usize,
    
    // LFO (Low Frequency Oscillator) for modulation
    lfo_phase: f32,
    lfo_rate: f32,        // LFO frequency in Hz
    
    // Parameters
    delay_base: f32,      // Base delay time in samples
    delay_range: f32,     // Modulation range in samples
    depth: f32,           // Effect depth (0.0 - 1.0)
    feedback: f32,        // Feedback amount (0.0 - 0.99)
    mix: f32,             // Dry/wet mix (0.0 - 1.0)
    
    sample_rate: u32,
}

impl FlangerEffect {
    /// Create a new flanger effect
    /// 
    /// # Parameters
    /// - `lfo_rate`: LFO rate in Hz (typically 0.1 - 2.0)
    /// - `depth`: Effect depth (0.0 - 1.0)
    /// - `feedback`: Feedback amount (0.0 - 0.99)
    /// - `mix`: Dry/wet mix (0.0 - 1.0)
    /// - `sample_rate`: Audio sample rate
    pub fn new(lfo_rate: f32, depth: f32, feedback: f32, mix: f32, sample_rate: u32) -> Self {
        // Flanger delay range: typically 1-10ms
        let delay_base = 0.002 * sample_rate as f32; // 2ms base delay
        let delay_range = 0.008 * sample_rate as f32; // 8ms modulation range
        
        // Buffer size needs to accommodate maximum delay
        let buffer_size = ((delay_base + delay_range) as usize + 1).max(1024);
        
        Self {
            buffer: vec![0.0; buffer_size],
            write_index: 0,
            lfo_phase: 0.0,
            lfo_rate: lfo_rate.max(0.01), // Prevent division by zero
            delay_base,
            delay_range,
            depth: depth.clamp(0.0, 1.0),
            feedback: feedback.clamp(0.0, 0.99),
            mix: mix.clamp(0.0, 1.0),
            sample_rate,
        }
    }
    
    /// Linear interpolation between two values
    fn lerp(&self, a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }
    
    /// Get interpolated sample from delay buffer
    fn get_delayed_sample(&self, delay_samples: f32) -> f32 {
        let delay_int = delay_samples as usize;
        let delay_frac = delay_samples - delay_int as f32;
        
        // Calculate read positions (circular buffer)
        let read_index1 = if self.write_index >= delay_int {
            self.write_index - delay_int
        } else {
            self.buffer.len() - (delay_int - self.write_index)
        };
        
        let read_index2 = if read_index1 == 0 {
            self.buffer.len() - 1
        } else {
            read_index1 - 1
        };
        
        // Linear interpolation between adjacent samples
        let sample1 = self.buffer[read_index1 % self.buffer.len()];
        let sample2 = self.buffer[read_index2 % self.buffer.len()];
        
        self.lerp(sample1, sample2, delay_frac)
    }
    
    /// Set LFO rate in Hz
    pub fn set_lfo_rate(&mut self, rate: f32) {
        self.lfo_rate = rate.max(0.01);
    }
    
    /// Set effect depth (0.0 - 1.0)
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }
    
    /// Set feedback amount (0.0 - 0.99)
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.99);
    }
    
    /// Set dry/wet mix (0.0 - 1.0)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
}

impl AudioEffect for FlangerEffect {
    fn process_sample(&mut self, input: f32) -> f32 {
        // Generate LFO (sine wave)
        let lfo_value = (self.lfo_phase * 2.0 * PI).sin();
        
        // Calculate modulated delay time
        let delay_offset = (lfo_value * 0.5 + 0.5) * self.delay_range * self.depth;
        let total_delay = self.delay_base + delay_offset;
        
        // Get delayed sample with interpolation
        let delayed_sample = self.get_delayed_sample(total_delay);
        
        // Write input + feedback to buffer
        self.buffer[self.write_index] = input + delayed_sample * self.feedback;
        
        // Advance write index (circular)
        self.write_index = (self.write_index + 1) % self.buffer.len();
        
        // Update LFO phase
        self.lfo_phase += self.lfo_rate / self.sample_rate as f32;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }
        
        // Mix dry and wet signals
        input * (1.0 - self.mix) + delayed_sample * self.mix
    }
    
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_index = 0;
        self.lfo_phase = 0.0;
    }
    
    fn name(&self) -> &str {
        "Flanger"
    }
}