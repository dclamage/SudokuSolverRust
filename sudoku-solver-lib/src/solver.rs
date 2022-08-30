use std::sync::Arc;

use crate::{board::Board, constraint::Constraint};

#[derive(Clone)]
pub struct Solver {
    pub board: Board,
}

impl Solver {
    pub fn new(size: usize, constraints: &[Arc<dyn Constraint>]) -> Solver {
        Solver {
            board: Board::new(size, constraints),
        }
    }
}
