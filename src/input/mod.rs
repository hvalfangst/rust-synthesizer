use std::sync::Arc;
use minifb::Window;
use rodio::Sink;
use crate::state::State;

pub mod commands;
pub mod handler;

/// Trait that all input commands must implement
pub trait InputCommand: Send + Sync {
    /// Execute the input command with the given state and dependencies
    fn execute(&self, state: &mut State, window: &mut Window, sink: &mut Sink);
}

/// Type alias for easier usage of command references
pub type InputCommandRef = Arc<dyn InputCommand>;