//! Constains the [`Solver`] struct which is the main entry point for solving a puzzle.

use crate::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct Solver {
    board: Board,
    logical_steps: Vec<Arc<dyn LogicalStep>>,
}

impl Solver {
    /// Create a new solver.
    ///
    /// # Arguments
    /// * `size` - The size of the board (use 9 for a 9x9 board).
    /// * `regions` - The regions of the board. Pass an empty slice to use default regions.
    /// * `logical_steps` - The logical steps that should be used to solve the puzzle.
    /// Pass an empty slice to use default logical steps.
    /// * `constraints` - The additional constraints that should be used to solve the puzzle, if any.
    pub fn new(
        size: usize,
        regions: &[usize],
        logical_steps: impl Iterator<Item = Arc<dyn LogicalStep>>,
        constraints: impl Iterator<Item = Arc<dyn Constraint>>,
    ) -> Solver {
        let constraints: Vec<_> = constraints.collect();
        let board = Board::new(size, regions, &constraints);
        let mut logical_steps: Vec<_> = logical_steps.collect();
        if logical_steps.is_empty() {
            logical_steps = Self::standard_logic();
        } else {
            // Ensure all the required logical steps are present in the list.
            // Required steps generally happen first, so they're added to the
            // front.
            for required_logic in Self::required_logic() {
                if !logical_steps
                    .iter()
                    .any(|l| l.name() == required_logic.name())
                {
                    let required_logic_slice = [required_logic];
                    logical_steps.splice(..0, required_logic_slice.iter().cloned());
                }
            }
        }

        Solver {
            board,
            logical_steps,
        }
    }

    fn required_logic() -> Vec<Arc<dyn LogicalStep>> {
        vec![Arc::new(AllNakedSingles), Arc::new(NakedSingle)]
    }

    pub fn standard_logic() -> Vec<Arc<dyn LogicalStep>> {
        vec![
            Arc::new(AllNakedSingles),
            Arc::new(HiddenSingle),
            Arc::new(NakedSingle),
            Arc::new(SimpleCellForcing),
        ]
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    pub fn logical_steps(&self) -> &[Arc<dyn LogicalStep>] {
        &self.logical_steps
    }

    pub fn set_givens(&mut self, givens: impl Iterator<Item = (CellIndex, usize)>) -> LogicResult {
        let mut changed = false;

        for (cell, value) in givens {
            if !self.board.cell(cell).is_solved() {
                if !self.board.set_solved(cell, value) {
                    return LogicResult::Invalid;
                }
                changed = true;
            }
        }

        if changed {
            LogicResult::Changed
        } else {
            LogicResult::None
        }
    }
}

impl Default for Solver {
    fn default() -> Self {
        Solver::new(9, &[], std::iter::empty(), std::iter::empty())
    }
}
