use minifb::{Key, KeyRepeat, Window};
use rodio::{Sink, Source};
use crate::state::State;
use crate::music_theory::note::Note;
use crate::state::utils::{get_key_mappings, handle_musical_note};
use crate::waveforms::adsr_envelope::ADSREnvelope;
use crate::waveforms::sine_wave::SineWave;
use crate::waveforms::square_wave::SquareWave;
use crate::waveforms::triangle_wave::TriangleWave;
use crate::waveforms::{Waveform, AMPLITUDE};
use crate::waveforms::sawtooth_wave::SawtoothWave;
use super::super::InputCommand;

/// Command for handling musical note keyboard input
pub struct KeyboardInputCommand {
    key: Key,
}

impl KeyboardInputCommand {
    pub fn new(key: Key) -> Self {
        Self { key }
    }
}

impl InputCommand for KeyboardInputCommand {
    fn execute(&self, state: &mut State, window: &mut Window, sink: &mut Sink) {
        // Key press is already checked by the handler, so we can directly execute
        
        // Find the note associated with this key
        let key_mappings = get_key_mappings();
        if let Some((_, note, _, _)) = key_mappings.iter().find(|(k, _, _, _)| *k == self.key) {
            handle_musical_note(state, sink, *note);
            state.pressed_key = Some((self.key, *note));
            
            // Handle recording if active - record to current track
            if state.recording_state == crate::state::RecordingState::Recording {
                // Finish previous note if there was one
                if let Some((start_time, prev_note, prev_octave)) = state.current_note_start.take() {
                    let duration = start_time.elapsed().as_secs_f32();
                    let timestamp = state.recording_start_time
                        .map(|start| start.elapsed().as_secs_f32() - duration)
                        .unwrap_or(0.0);

                    // Add to current track instead of global recorded_notes
                    state.add_note_to_current_track(crate::state::RecordedNote {
                        note: prev_note,
                        octave: prev_octave,
                        timestamp,
                        duration,
                    });
                }

                // Start recording new note using current track's octave
                let current_track_octave = state.tracks[state.current_track_id].octave;
                state.current_note_start = Some((std::time::Instant::now(), *note, current_track_octave));
            }
        }
    }
}

