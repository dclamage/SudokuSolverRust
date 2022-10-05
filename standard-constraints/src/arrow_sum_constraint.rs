//! Contains the [`ArrowSumConstraint`] struct for representing an arrow sum constraint.

use sudoku_solver_lib::prelude::*;

/// A [`Constraint`] implementation for representing an arrow sum constraint.
#[derive(Debug)]
pub struct SudokuSolverConstraint {
    specific_name: String,
    circle_cells: Vec<CellIndex>,
    arrow_cells: Vec<CellIndex>,
    all_cells: Vec<CellIndex>,
    is_arrow_group: bool,
    is_circle_group: bool,
    is_all_grouped: bool,
}

impl SudokuSolverConstraint {
    pub fn new(circle_cells: Vec<CellIndex>, arrow_cells: Vec<CellIndex>) -> Self {
        let all_cells = circle_cells.iter().chain(arrow_cells.iter()).cloned().collect();

        let specific_name = format!("Arrow at {}", circle_cells[0]);
        Self {
            specific_name,
            circle_cells,
            arrow_cells,
            all_cells,
            is_arrow_group: false,
            is_circle_group: false,
            is_all_grouped: false,
        }
    }
}

impl Constraint for SudokuSolverConstraint {
    fn name(&self) -> &str {
        &self.specific_name
    }

    fn init_board(&mut self, board: &mut Board) -> LogicalStepResult {
        // let mut changed = false;

        if self.arrow_cells.len() > 1 {
            self.is_all_grouped = board.is_grouped(&self.all_cells);
            if self.is_all_grouped {
                self.is_arrow_group = true;
                self.is_circle_group = true;
            } else {
                self.is_arrow_group = board.is_grouped(&self.arrow_cells);
                self.is_circle_group = board.is_grouped(&self.circle_cells);
            }
        }

        LogicalStepResult::None
    }
}
