use rodio::{Source, Sink};
use std::time::Duration;
use crate::state::{State, Track, MasterTrack, RecordedNote};
use crate::waveforms::{Waveform, AMPLITUDE};
use crate::waveforms::adsr_envelope::ADSREnvelope;
use crate::waveforms::sine_wave::SineWave;
use crate::waveforms::square_wave::SquareWave;
use crate::waveforms::triangle_wave::TriangleWave;
use crate::waveforms::sawtooth_wave::SawtoothWave;
use crate::effects::AudioEffect;
use crate::music_theory::note::Note;

/// Multi-track audio mixer that handles playback of all tracks
pub struct MultiTrackMixer {
    sample_rate: u32,
}

impl MultiTrackMixer {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }
    
    /// Play a note on a specific track
    pub fn play_note_on_track(
        &self,
        track: &Track,
        note: Note,
        sink: &mut Sink,
    ) {
        let base_frequency = note.frequency(track.octave);
        
        // Create waveform based on track settings
        let synth = match track.waveform {
            Waveform::SINE => {
                let sine_wave = SineWave::new(base_frequency);
                let adsr_envelope = ADSREnvelope::new(
                    sine_wave,
                    track.attack as f32 / 99.0 * 2.0,
                    track.decay as f32 / 99.0 * 2.0,
                    track.sustain as f32 / 99.0,
                    track.release as f32 / 99.0 * 2.0
                );
                Box::new(adsr_envelope) as Box<dyn Source<Item=f32> + 'static + Send>
            },
            Waveform::SQUARE => {
                let square_wave = SquareWave::new(base_frequency);
                let adsr_envelope = ADSREnvelope::new(
                    square_wave,
                    track.attack as f32 / 99.0 * 2.0,
                    track.decay as f32 / 99.0 * 2.0,
                    track.sustain as f32 / 99.0,
                    track.release as f32 / 99.0 * 2.0
                );
                Box::new(adsr_envelope) as Box<dyn Source<Item=f32> + 'static + Send>
            },
            Waveform::TRIANGLE => {
                let triangle_wave = TriangleWave::new(base_frequency);
                let adsr_envelope = ADSREnvelope::new(
                    triangle_wave,
                    track.attack as f32 / 99.0 * 2.0,
                    track.decay as f32 / 99.0 * 2.0,
                    track.sustain as f32 / 99.0,
                    track.release as f32 / 99.0 * 2.0
                );
                Box::new(adsr_envelope) as Box<dyn Source<Item=f32> + 'static + Send>
            },
            Waveform::SAWTOOTH => {
                let sawtooth_wave = SawtoothWave::new(base_frequency);
                let adsr_envelope = ADSREnvelope::new(
                    sawtooth_wave,
                    track.attack as f32 / 99.0 * 2.0,
                    track.decay as f32 / 99.0 * 2.0,
                    track.sustain as f32 / 99.0,
                    track.release as f32 / 99.0 * 2.0
                );
                Box::new(adsr_envelope) as Box<dyn Source<Item=f32> + 'static + Send>
            },
        };
        
        // Apply track volume and pan
        let source_with_volume = synth.amplify(AMPLITUDE * track.volume);
        
        // Apply track-specific effects
        let source_with_effects = self.apply_track_effects(source_with_volume, track);
        
        // Add to sink
        sink.append(source_with_effects);
    }
    
    /// Apply effects to a track's audio source
    fn apply_track_effects<S>(&self, source: S, track: &Track) -> Box<dyn Source<Item=f32> + Send>
    where
        S: Source<Item=f32> + Send + 'static,
    {
        // For now, just return the source as-is since we need to implement effects processing
        // TODO: Implement proper track-specific effects processing
        if track.delay_enabled || track.reverb_enabled || track.flanger_enabled {
            // Effects are enabled but we need to implement the processor
            Box::new(source)
        } else {
            Box::new(source)
        }
    }
    
    /// Play back recorded notes from multiple tracks simultaneously
    pub fn play_multi_track_sequence(
        &self,
        state: &State,
        sink: &mut Sink,
        playback_time: f32,
    ) {
        let playing_tracks = state.playing_tracks();
        
        for track_id in playing_tracks {
            let track = &state.tracks[track_id];
            self.play_track_at_time(track, sink, playback_time);
        }
    }
    
    /// Play a specific track's notes at a given time
    fn play_track_at_time(&self, track: &Track, sink: &mut Sink, playback_time: f32) {
        let frame_time_threshold = 0.05; // 50ms threshold
        
        for recorded_note in &track.recorded_notes {
            let note_start = recorded_note.timestamp;
            
            // Check if this note should start playing now
            if playback_time >= note_start && playback_time < note_start + frame_time_threshold {
                self.play_note_on_track(track, recorded_note.note, sink);
            }
        }
    }
    
    /// Calculate final master mix with master effects
    pub fn apply_master_effects(&self, _master_track: &MasterTrack, sample: f32) -> f32 {
        // Apply master volume
        sample * _master_track.volume
        // TODO: Apply master effects (delay, reverb, flanger)
    }
}

/// Panning utility function
pub fn apply_pan(sample: f32, pan: f32) -> (f32, f32) {
    // Pan from -1.0 (left) to 1.0 (right)
    let left_gain = ((1.0 - pan) / 2.0).sqrt();
    let right_gain = ((1.0 + pan) / 2.0).sqrt();
    
    (sample * left_gain, sample * right_gain)
}