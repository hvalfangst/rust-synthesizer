use rodio::{OutputStream, Sink};

use crate::{
    state::{event_loop::start_event_loop, State},
    graphics::sprites::Sprites
};

mod waveforms;
mod state;
mod music_theory;
mod graphics;
mod input;

fn main() {

    // Initialize the audio output stream and sink
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sink = Sink::try_new(&stream_handle).unwrap();

    // Instantiate the Sprites struct, which in turn will load sprites from sprite maps into 3d Vectors
    let sprites = Sprites::new();

    // Instantiate the state struct with default values for octave and waveform
    let mut state = State::new();

    // Execute the main event loop, which handles user input and associated sound generation
    start_event_loop(&mut state, &mut sink, &sprites);
}