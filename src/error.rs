//! The module keeping track of the possible game errors.

use std::error::Error;
use std::fmt;

/// All possible error states the game can end up in.
#[derive(Debug)]
pub(crate) enum GameError {
    Unknown,
}

impl Error for GameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use GameError::*;

        match self {
            Unknown => None,
        }
    }
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use GameError::*;

        match self {
            Unknown => f.write_str("unknown!"),
        }
    }
}
