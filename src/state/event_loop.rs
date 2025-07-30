use crate::graphics::draw::draw_buffer;
use std::thread;
use std::time::{Duration, Instant};

use minifb::{Key as key, Window, WindowOptions};
use rodio::Sink;

use crate::{
    graphics::constants::*,
    graphics::sprites::*,
    state::{FRAME_DURATION, State},
    input::handler::InputHandler,
};
use crate::state::utils::{update_buffer_with_state};
use crate::state::updaters::{AudioStateUpdater, VisualStateUpdater, RecordingStateUpdater, MouseStateUpdater};

/// Starts the event loop for the synthesizer application, handling user input and rendering visuals.
///
/// # Parameters
/// - `state`: Mutable reference to `State`, which manages the current state of the synthesizer.
/// - `sink`: Mutable reference to `Sink`, the audio sink responsible for playing sound.
/// - `sprites`: Reference to `Sprites`, containing all graphical assets used for rendering visuals.
///
/// # Event Loop Logic
/// - Initializes a window with specific dimensions and title.
/// - Continuously checks for user input and updates synthesizer state accordingly.
/// - Updates the visual representation of the synthesizer based on the current state.
/// - Renders the updated visual buffer onto the window.
/// - Maintains a frame rate of approximately 60 frames per second by calculating necessary sleep time.
pub fn start_event_loop(state: &mut State, sink: &mut Sink, sprites: &Sprites) {
    // Create a window with error handling
    let mut window = Window::new(
        "Rust Synthesizer 1.0",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("{}", e); // Panic if window creation fails
    });

    let mut rack_index = 0; // Default rack sprite index
    let mut last_rack_change = Instant::now(); // Records time of last rack index change

    // Initialize command pattern handlers
    let input_handler = InputHandler::new();
    let audio_updater = AudioStateUpdater::new();
    let visual_updater = VisualStateUpdater::new();
    let recording_updater = RecordingStateUpdater::new();
    let mouse_updater = MouseStateUpdater::new();

    // Initialize window buffer to store pixel data
    let mut window_buffer = vec![0; WINDOW_WIDTH * WINDOW_HEIGHT];

    // Main event loop: runs as long as the window is open and the Escape key is not pressed
    while window.is_open() && !window.is_key_down(key::Escape) {
        let start = Instant::now(); // Record start time for frame timing

        // Handle all input using command pattern
        input_handler.handle_input(state, &mut window, sink);

        // Update state using updater pattern
        audio_updater.update(state, sink);
        visual_updater.update(state);
        recording_updater.update(state);
        mouse_updater.update(state);
        

        // Change rack index every 2 seconds by toggling between 0 and 1
        if last_rack_change.elapsed() >= Duration::from_secs(2) {
            rack_index = 1 - rack_index;
            last_rack_change = Instant::now();
        }


        // Update the pixel buffer with the current state visuals
        update_buffer_with_state(state, sprites, &mut window_buffer, rack_index);

        // Draw the current buffer onto the window
        draw_buffer(&mut window, &mut window_buffer);

        // Maintain a frame rate of approximately 60 fps
        let elapsed = start.elapsed();
        if elapsed < FRAME_DURATION {
            thread::sleep(FRAME_DURATION - elapsed);
        }
    }
}