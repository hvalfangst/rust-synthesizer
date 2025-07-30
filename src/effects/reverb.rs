use super::AudioEffect;

/// Simple reverb effect using multiple delay lines (Schroeder reverb)
#[derive(Debug, Clone)]
pub struct ReverbEffect {
    // Comb filters (feedback delay lines)
    comb_delays: Vec<Vec<f32>>,
    comb_indices: Vec<usize>,
    comb_feedback: Vec<f32>,
    
    // All-pass filters
    allpass_delays: Vec<Vec<f32>>,
    allpass_indices: Vec<usize>,
    allpass_feedback: f32,
    
    // Parameters
    room_size: f32,   // 0.0 - 1.0
    damping: f32,     // 0.0 - 1.0
    mix: f32,         // 0.0 - 1.0
    
    // Low-pass filter for damping
    damping_filter: f32,
}

impl ReverbEffect {
    /// Create a new reverb effect
    /// 
    /// # Parameters
    /// - `room_size`: Size of the room (0.0 - 1.0)
    /// - `damping`: High frequency damping (0.0 - 1.0)
    /// - `mix`: Dry/wet mix (0.0 - 1.0)
    /// - `sample_rate`: Audio sample rate
    pub fn new(room_size: f32, damping: f32, mix: f32, sample_rate: u32) -> Self {
        // Enhanced comb filter delay times - carefully tuned for musical intervals
        let comb_delays_ms = [29.7, 37.1, 41.1, 43.7, 47.0, 50.3, 53.5, 56.3];
        let base_feedback = 0.6 + room_size * 0.35; // Increased feedback for fuller sound
        
        let mut comb_delays = Vec::new();
        let mut comb_feedback = Vec::new();
        
        for (i, &delay_ms) in comb_delays_ms.iter().enumerate() {
            let delay_samples = ((delay_ms / 1000.0) * sample_rate as f32) as usize;
            comb_delays.push(vec![0.0; delay_samples.max(1)]);
            // Vary feedback slightly for each comb filter
            let feedback_variation = 1.0 + (i as f32 * 0.02 - 0.07);
            comb_feedback.push(base_feedback * feedback_variation);
        }
        
        // Enhanced all-pass filter delays for better diffusion
        let allpass_delays_ms = [5.0, 1.7, 12.9, 9.3, 15.1, 8.2];
        let mut allpass_delays = Vec::new();
        
        for &delay_ms in &allpass_delays_ms {
            let delay_samples = ((delay_ms / 1000.0) * sample_rate as f32) as usize;
            allpass_delays.push(vec![0.0; delay_samples.max(1)]);
        }
        
        Self {
            comb_delays,
            comb_indices: vec![0; comb_delays_ms.len()],
            comb_feedback,
            allpass_delays,
            allpass_indices: vec![0; allpass_delays_ms.len()],
            allpass_feedback: 0.618, // Golden ratio for more natural sound
            room_size: room_size.clamp(0.0, 1.0),
            damping: damping.clamp(0.0, 1.0),
            mix: mix.clamp(0.0, 1.0),
            damping_filter: 0.0,
        }
    }
    
    /// Process sample through comb filters with improved damping
    fn process_comb_filters(&mut self, input: f32) -> f32 {
        let mut output = 0.0;
        
        for i in 0..self.comb_delays.len() {
            let delay_line = &mut self.comb_delays[i];
            let index = &mut self.comb_indices[i];
            let feedback = self.comb_feedback[i];
            
            // Read delayed sample
            let delayed = delay_line[*index];
            
            // Apply consistent damping across all comb filters
            self.damping_filter = delayed * (1.0 - self.damping) + self.damping_filter * self.damping;
            
            // Write new sample with feedback
            delay_line[*index] = input + self.damping_filter * feedback;
            
            // Advance index (circular)
            *index = (*index + 1) % delay_line.len();
            
            // Accumulate output
            output += delayed;
        }
        
        output / self.comb_delays.len() as f32
    }
    
    /// Process sample through all-pass filters
    fn process_allpass_filters(&mut self, mut input: f32) -> f32 {
        for i in 0..self.allpass_delays.len() {
            let delay_line = &mut self.allpass_delays[i];
            let index = &mut self.allpass_indices[i];
            
            // Read delayed sample
            let delayed = delay_line[*index];
            
            // All-pass filter calculation
            let output = -input + delayed;
            delay_line[*index] = input + delayed * self.allpass_feedback;
            
            // Advance index (circular)
            *index = (*index + 1) % delay_line.len();
            
            input = output;
        }
        
        input
    }
    
    /// Set room size (0.0 - 1.0)
    pub fn set_room_size(&mut self, room_size: f32) {
        self.room_size = room_size.clamp(0.0, 1.0);
        let base_feedback = 0.5 + self.room_size * 0.28;
        
        for feedback in &mut self.comb_feedback {
            *feedback = base_feedback;
        }
    }
    
    /// Set damping (0.0 - 1.0)
    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping.clamp(0.0, 1.0);
    }
    
    /// Set dry/wet mix (0.0 - 1.0)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
}

impl AudioEffect for ReverbEffect {
    fn process_sample(&mut self, input: f32) -> f32 {
        // Process through comb filters
        let comb_output = self.process_comb_filters(input);
        
        // Process through all-pass filters
        let reverb_output = self.process_allpass_filters(comb_output);
        
        // Mix dry and wet signals
        input * (1.0 - self.mix) + reverb_output * self.mix
    }
    
    fn reset(&mut self) {
        // Clear all delay lines
        for delay_line in &mut self.comb_delays {
            delay_line.fill(0.0);
        }
        for delay_line in &mut self.allpass_delays {
            delay_line.fill(0.0);
        }
        
        // Reset indices
        self.comb_indices.fill(0);
        self.allpass_indices.fill(0);
        
        // Reset damping filter
        self.damping_filter = 0.0;
    }
    
    fn name(&self) -> &str {
        "Reverb"
    }
}