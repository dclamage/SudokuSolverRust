//! Contains [`SingleSolutionResult`] for storing the result of running multiple logical steps.

use crate::prelude::*;

/// The result of running a solve that returns a single solution.
#[derive(Clone)]
pub enum SingleSolutionResult {
    /// No solution is possible.
    None,
    /// A solution was found.
    Solved(Box<Board>),
}

impl SingleSolutionResult {
    pub fn is_none(&self) -> bool {
        matches!(self, SingleSolutionResult::None)
    }

    pub fn is_solved(&self) -> bool {
        matches!(self, SingleSolutionResult::Solved(_))
    }

    pub fn board(&self) -> Option<Box<Board>> {
        match self {
            SingleSolutionResult::None => None,
            SingleSolutionResult::Solved(board) => Some(board.clone()),
        }
    }
}

impl std::fmt::Display for SingleSolutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(board) = self.board() {
            write!(f, "{}", board)
        } else {
            write!(f, "No solution.")
        }
    }
}
