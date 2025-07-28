use std::fmt;

pub mod sine_wave;
pub mod square_wave;
pub mod triangle_wave;
pub mod sawtooth_wave;
pub mod adsr_envelope;

pub const MONO: u16 = 1;
pub const SAMPLE_RATE: f32 = 48000.0;
pub const AMPLITUDE: f32 = 0.20;
pub const DURATION: f32 = 0.19;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    SINE,
    SQUARE,
    TRIANGLE,
    SAWTOOTH
}

/// Implements the [Display] trait for [WaveForm]
impl fmt::Display for Waveform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Waveform::SINE => write!(f, "Sine"),
            Waveform::SQUARE => write!(f, "Square"),
            Waveform::TRIANGLE => write!(f, "Triangle"),
            Waveform::SAWTOOTH => write!(f, "Sawtooth")
        }
    }
}