//! Contains [`SingleSolutionResult`] for storing the result of finding a single solution.

use crate::prelude::*;

/// The result of running a solve that returns a single solution.
#[derive(Clone)]
pub enum SingleSolutionResult {
    /// No solution is possible.
    None,
    /// A solution was found.
    Solved(Box<Board>),
    /// There was an error while solving.
    Error(String),
}

impl SingleSolutionResult {
    pub fn is_none(&self) -> bool {
        matches!(self, SingleSolutionResult::None)
    }

    pub fn is_solved(&self) -> bool {
        matches!(self, SingleSolutionResult::Solved(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, SingleSolutionResult::Error(_))
    }

    pub fn board(&self) -> Option<Box<Board>> {
        match self {
            SingleSolutionResult::None | SingleSolutionResult::Error(_) => None,
            SingleSolutionResult::Solved(board) => Some(board.clone()),
        }
    }
}

impl std::fmt::Display for SingleSolutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(board) = self.board() {
            write!(f, "{}", board)
        } else if let SingleSolutionResult::Error(err) = self {
            write!(f, "Error: {}", err)
        } else {
            write!(f, "No solution")
        }
    }
}
