use rodio::Source;
use std::time::Duration;

pub mod delay;
pub mod reverb;
pub mod flanger;

pub use delay::DelayEffect;
pub use reverb::ReverbEffect;
pub use flanger::FlangerEffect;

/// Trait that all audio effects must implement
pub trait AudioEffect: Send + Sync {
    /// Process a single audio sample
    fn process_sample(&mut self, input: f32) -> f32;
    
    /// Reset the effect's internal state
    fn reset(&mut self);
    
    /// Get the effect's name
    fn name(&self) -> &str;
}

/// Wrapper that applies an effect to any audio source
pub struct EffectWrapper<S, E> 
where
    S: Source<Item = f32>,
    E: AudioEffect,
{
    source: S,
    effect: E,
}

impl<S, E> EffectWrapper<S, E>
where
    S: Source<Item = f32>,
    E: AudioEffect,
{
    pub fn new(source: S, effect: E) -> Self {
        Self { source, effect }
    }
}

impl<S, E> Iterator for EffectWrapper<S, E>
where
    S: Source<Item = f32>,
    E: AudioEffect,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next().map(|sample| self.effect.process_sample(sample))
    }
}

impl<S, E> Source for EffectWrapper<S, E>
where
    S: Source<Item = f32>,
    E: AudioEffect,
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
        self.source.total_duration()
    }
}

/// Chain multiple effects together
pub struct EffectChain {
    effects: Vec<Box<dyn AudioEffect>>,
}

impl EffectChain {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }
    
    pub fn add_effect(&mut self, effect: Box<dyn AudioEffect>) {
        self.effects.push(effect);
    }
    
    pub fn process_sample(&mut self, mut input: f32) -> f32 {
        for effect in &mut self.effects {
            input = effect.process_sample(input);
        }
        input
    }
    
    pub fn reset(&mut self) {
        for effect in &mut self.effects {
            effect.reset();
        }
    }
}