//! Contains the [`SolutionReceiver`] trait for receiving solutions from a solver
//! and a [`VecSolutionReceiver`] implementation for receiving solutions into a vector
//! and a [`CountSolutionReceiver`] implementation for counting solutions as they come in.

use crate::prelude::*;

/// A trait for receiving solutions from a solver.
pub trait SolutionReceiver {
    /// Receives a solution from a solver.
    ///
    /// Return false to end the solution count early.
    fn receive(&mut self, result: Box<Board>) -> bool;

    /// Receive a ping every once in a while with a monotonically increasing number.
    ///
    /// The number has no meaning other than to indicate that the solver is still running.
    ///
    /// This is useful on platforms like WASM which have no way to quickly retrieve
    /// the current time, but want to periodically update the UI at a relatively steady rate,
    /// rather than updating the UI based on the number of solutions found, for which progress
    /// can vary wildly depending on the puzzle.
    fn progress_ping(&mut self, progress: usize) {
        // Do nothing by default.
        let _ = progress;
    }
}

/// A [`SolutionReceiver`] that stores the solutions in a vector.
pub struct VecSolutionReceiver {
    solutions: Vec<Board>,
}

impl VecSolutionReceiver {
    /// Creates a new [`VecSolutionReceiver`].
    pub fn new() -> Self {
        Self {
            solutions: Vec::new(),
        }
    }

    /// Returns the solutions.
    pub fn solutions(&self) -> &Vec<Board> {
        &self.solutions
    }

    /// Consumes the [`VecSolutionReceiver`] and returns the solutions.
    pub fn take_solutions(self) -> Vec<Board> {
        self.solutions
    }
}

impl SolutionReceiver for VecSolutionReceiver {
    fn receive(&mut self, result: Box<Board>) -> bool {
        self.solutions.push(result.as_ref().clone());
        true
    }
}

impl Default for VecSolutionReceiver {
    fn default() -> Self {
        Self::new()
    }
}

impl From<VecSolutionReceiver> for Vec<Board> {
    fn from(receiver: VecSolutionReceiver) -> Self {
        receiver.solutions
    }
}

/// A [`SolutionReceiver`] that counts the number of solutions so far.
pub struct CountSolutionReceiver {
    count: usize,
}

impl CountSolutionReceiver {
    /// Creates a new [`CountSolutionReceiver`].
    pub fn new() -> Self {
        Self { count: 0 }
    }

    /// Returns the number of solutions so far.
    pub fn count(&self) -> usize {
        self.count
    }
}

impl SolutionReceiver for CountSolutionReceiver {
    fn receive(&mut self, _result: Box<Board>) -> bool {
        self.count += 1;
        true
    }
}

impl Default for CountSolutionReceiver {
    fn default() -> Self {
        Self::new()
    }
}
