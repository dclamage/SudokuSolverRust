//! Contains the [`SolutionReceiver`] trait for receiving solutions from a solver.

use crate::prelude::*;

/// A trait for receiving solutions from a solver.
pub trait SolutionReceiver {
    fn receive(&mut self, result: Box<Board>);
}
