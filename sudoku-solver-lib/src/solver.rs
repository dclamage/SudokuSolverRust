//! Constains the [`Solver`] struct which is the main entry point for solving a puzzle.

use std::sync::Arc;

use crate::{board::Board, constraint::Constraint};

#[derive(Clone)]
pub struct Solver {
    pub board: Board,
}

impl Solver {
    pub fn new(size: usize, regions: &[usize], constraints: &[Arc<dyn Constraint>]) -> Solver {
        Solver {
            board: Board::new(size, regions, constraints),
        }
    }
}
