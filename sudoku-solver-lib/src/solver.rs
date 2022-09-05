//! Constains the [`Solver`] struct which is the main entry point for solving a puzzle.

pub mod logical_solve_result;
pub mod prelude;
pub mod single_solution_result;

use itertools::Itertools;

use crate::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct Solver {
    board: Board,
    logical_solve_steps: Vec<Arc<dyn LogicalStep>>,
    brute_force_steps: Vec<Arc<dyn LogicalStep>>,
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

        let logical_solve_steps = logical_steps
            .iter()
            .cloned()
            .filter(|step| step.is_active_during_logical_solves())
            .collect();

        let brute_force_steps = logical_steps
            .iter()
            .cloned()
            .filter(|step| step.is_active_during_brute_force_solves())
            .collect();

        Solver {
            board,
            logical_solve_steps,
            brute_force_steps,
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

    pub fn cell_utility(&self) -> CellUtility {
        self.board.cell_utility()
    }

    pub fn logical_solve_steps(&self) -> &[Arc<dyn LogicalStep>] {
        &self.logical_solve_steps
    }

    pub fn brute_force_steps(&self) -> &[Arc<dyn LogicalStep>] {
        &self.brute_force_steps
    }

    /// Set the givesn on the board.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::prelude::*;
    /// let mut solver = Solver::default();
    /// let cu = solver.cell_utility();
    /// let cells = [(cu.cell(0, 0), 1), (cu.cell(0, 1), 2), (cu.cell(0, 2), 3)];
    /// assert!(solver.set_givens(cells.into_iter()));
    /// assert!(solver.board().cell(cu.cell(0, 0)).is_solved());
    /// assert!(solver.board().cell(cu.cell(0, 1)).is_solved());
    /// assert!(solver.board().cell(cu.cell(0, 2)).is_solved());
    /// assert!(!solver.board().cell(cu.cell(0, 3)).is_solved());
    /// assert_eq!(solver.board().cell(cu.cell(0, 0)).value(), 1);
    /// assert_eq!(solver.board().cell(cu.cell(0, 1)).value(), 2);
    /// assert_eq!(solver.board().cell(cu.cell(0, 2)).value(), 3);
    /// assert_eq!(solver.board().cell(cu.cell(0, 3)).min(), 4);
    /// ```
    pub fn set_givens(&mut self, givens: impl Iterator<Item = (CellIndex, usize)>) -> bool {
        for (cell, value) in givens {
            if !self.board.cell(cell).is_solved() && !self.board.set_solved(cell, value) {
                return false;
            }
        }

        true
    }

    /// Set the givens from a given string.
    /// The string should be a sequence of numbers, with 0 or any non-digit representing an empty cell.
    /// The string should be in row-major order.
    /// For grid sizes larger than 9, the each number takes the same number of characters, so use 01 for 1, for example.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::prelude::*;
    /// let mut solver = Solver::default();
    /// assert!(solver.set_givens_from_string("123000000000000000000000000000000000000000000000000000000000000000000000000000000"));
    ///
    /// let cu = solver.cell_utility();
    /// assert!(solver.board().cell(cu.cell(0, 0)).is_solved());
    /// assert!(solver.board().cell(cu.cell(0, 1)).is_solved());
    /// assert!(solver.board().cell(cu.cell(0, 2)).is_solved());
    /// assert!(!solver.board().cell(cu.cell(0, 3)).is_solved());
    /// assert_eq!(solver.board().cell(cu.cell(0, 0)).value(), 1);
    /// assert_eq!(solver.board().cell(cu.cell(0, 1)).value(), 2);
    /// assert_eq!(solver.board().cell(cu.cell(0, 2)).value(), 3);
    /// assert_eq!(solver.board().cell(cu.cell(0, 3)).min(), 4);
    ///
    /// let mut solver16 = Solver::new(16, &[], std::iter::empty(), std::iter::empty());
    /// ```
    pub fn set_givens_from_string(&mut self, givens: &str) -> bool {
        let cu = self.board.cell_utility();
        if cu.size() <= 9 {
            if givens.len() != cu.size() * cu.size() {
                return false;
            }

            let givens_itr = givens.chars().enumerate().filter_map(|(i, c)| {
                let value = c.to_digit(10)?;
                if value == 0 {
                    None
                } else {
                    Some((cu.cell_index(i), value as usize))
                }
            });
            self.set_givens(givens_itr)
        } else {
            let num_digits = cu.size().to_string().len();
            if givens.len() != cu.size() * cu.size() * num_digits {
                return false;
            }

            let givens_chunks_itr = givens.chars().chunks(num_digits);
            let givens_itr = givens_chunks_itr
                .into_iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    // Convert the chunk into a string.
                    let val_str = c.collect::<String>();

                    // Convert the string into a number.
                    let value = val_str.parse::<usize>().ok()?;

                    // If the value is 0, ignore it.
                    if value == 0 {
                        None
                    } else {
                        Some((cu.cell_index(i), value))
                    }
                });
            self.set_givens(givens_itr)
        }
    }

    fn run_single_logical_step(&mut self) -> LogicalStepResult {
        for step in self.logical_solve_steps.iter() {
            let step_result = step.run(&mut self.board, true);
            if !step_result.is_none() {
                return step_result.with_prefix(format!("{}: ", step.name()).as_str());
            }
        }

        LogicalStepResult::None
    }

    /// Run a full logical solve. This mutates the solver's board.
    pub fn run_logical_solve(&mut self) -> LogicalSolveResult {
        let mut desc_list = LogicalStepDescList::new();
        let mut changed = false;
        loop {
            if self.board.is_solved() {
                desc_list.push("Solved!".into());
                return LogicalSolveResult::Solved(desc_list);
            }

            let step_result = self.run_single_logical_step();
            if step_result.is_none() {
                break;
            }

            changed = true;

            if let Some(desc) = step_result.description() {
                desc_list.push(desc.clone());
            }

            if step_result.is_invalid() {
                return LogicalSolveResult::Invalid(desc_list);
            }
        }

        if changed {
            LogicalSolveResult::Changed(desc_list)
        } else {
            LogicalSolveResult::None
        }
    }

    fn run_single_brute_force_step(&self, board: &mut Board) -> LogicalStepResult {
        for step in self.brute_force_steps.iter() {
            let step_result = step.run(board, false);
            if !step_result.is_none() {
                return step_result;
            }
        }

        LogicalStepResult::None
    }

    fn run_brute_force_logic(&self, board: &mut Board) -> bool {
        loop {
            let step_result = self.run_single_brute_force_step(board);
            if step_result.is_none() {
                break;
            }
            if step_result.is_invalid() {
                return false;
            }
        }

        true
    }

    /// Use brute-force methods to find the first solution to the puzzle.
    ///
    /// The solution is the lexicographically first solution and is not
    /// guaranteed to be the only solution.
    pub fn find_first_solution(&self) -> SingleSolutionResult {
        let cu = self.cell_utility();
        let mut board_stack = Vec::new();
        board_stack.push((Box::new(self.board.clone()), cu.cell(0, 0)));

        loop {
            if board_stack.is_empty() {
                break;
            }

            let (mut board, mut cell) = board_stack.pop().unwrap();
            if !self.run_brute_force_logic(&mut board) {
                continue;
            }

            if board.is_solved() {
                return SingleSolutionResult::Solved(board);
            }

            loop {
                if board.cell(cell).is_solved() {
                    if let Some(next_cell) = cell.next_cell() {
                        cell = next_cell;
                    } else {
                        break;
                    }
                } else {
                    let mask = board.cell(cell);
                    let value = mask.min();

                    // Push a copy of the board onto the stack with the value unset.
                    let mut board_copy = board.clone();
                    if board_copy.clear_value(cell, value) {
                        board_stack.push((board_copy, cell));
                    }

                    // Push a the board onto the stack with the value solved.
                    if board.set_solved(cell, value) {
                        board_stack.push((board, cell));
                    }

                    break;
                }
            }
        }

        SingleSolutionResult::None
    }
}

impl Default for Solver {
    fn default() -> Self {
        Solver::new(9, &[], std::iter::empty(), std::iter::empty())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_first_solution() {
        let solver = Solver::default();
        let result = solver.find_first_solution();
        assert!(result.is_solved());

        let board = result.board().unwrap();
        assert!(board.is_solved());

        let solution = board.to_string();
        assert_eq!(
            solution,
            "123456789456789123789123456214365897365897214897214365531642978642978531978531642"
        );
        println!("Solved: {}", board);
    }
}
