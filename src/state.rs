//! The module keeping track of the state of the game.

use crate::error::GameError;
use game_loop::{Renderer, Updater};

/// The state of the game.
#[derive(Debug, Default)]
pub(crate) struct GameState {
    updates: usize,
    renders: usize,
}

impl Updater for GameState {
    type Error = GameError;

    fn update(&mut self) -> Result<(), Self::Error> {
        self.updates += 1;
        Ok(())
    }
}

impl Renderer for GameState {
    type Error = GameError;

    fn render(&mut self, _remainder: f32) -> Result<(), Self::Error> {
        self.renders += 1;
        Ok(())
    }
}
