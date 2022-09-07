//! Constains the [`Solver`] struct which is the main entry point for solving a puzzle.

pub mod logical_solve_result;
pub mod prelude;
pub mod single_solution_result;
pub mod solution_count_result;
pub mod solution_receiver;
pub mod solver_builder;
pub mod true_candidates_count_result;

use itertools::Itertools;

use crate::prelude::*;
use std::{collections::HashSet, sync::Arc};

/// The main entry point for solving a puzzle.
///
/// Use the [`SolverBuilder`] struct to create a [`Solver`].
/// Do not create a [`Solver`] directly.
///
/// The [`Solver`] struct contains a [`Board`] which represents the current state of the puzzle to be solved.
///
/// The logic of the solver can be expanded using the [`LogicalStep`] trait.
/// This libary contains some basic implementations of this trait for core functionality.
/// Additional implementations can be added by the consumer of this library
/// to logically solve more complex puzzles.
///
/// Additionally, the [`Solver`] struct contains a list of [`Constraint`]s which define the rules of the puzzle.
/// This library does not provide any implementations of this trait, and instead relies on the
/// consumer of this library to provide the constraints for the puzzle to be solved.
#[derive(Clone)]
pub struct Solver {
    board: Board,
    logical_solve_steps: Vec<Arc<dyn LogicalStep>>,
    brute_force_steps: Vec<Arc<dyn LogicalStep>>,
}

impl Solver {
    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn size(&self) -> usize {
        self.board.size()
    }

    pub fn cell_utility(&self) -> CellUtility {
        self.board.cell_utility()
    }

    /// Find a single logical step that can be applied to the puzzle.
    pub fn run_single_logical_step(&mut self) -> LogicalStepResult {
        for step in self.logical_solve_steps.iter() {
            let step_result = step.run(&mut self.board, true);
            if !step_result.is_none() {
                if step.has_own_prefix() {
                    return step_result;
                } else {
                    return step_result.with_prefix(format!("{}: ", step.name()).as_str());
                }
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

        while !board_stack.is_empty() {
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

    fn find_best_brute_force_cell(board: &Board) -> Option<CellIndex> {
        let mut best_cell = None;
        let mut best_cell_candidate_count = usize::MAX;
        let board_data = board.data();

        for &cell in board_data.powerful_cells() {
            let mask = board.cell(cell);
            if mask.is_solved() {
                continue;
            }

            let cell_count = mask.count();
            if cell_count <= 2 {
                return Some(cell);
            }
            if cell_count < best_cell_candidate_count {
                best_cell = Some(cell);
                best_cell_candidate_count = cell_count;
            }
        }

        if best_cell.is_some() {
            return best_cell;
        }

        for cell in board.all_cells() {
            let mask = board.cell(cell);
            if mask.is_solved() {
                continue;
            }

            let cell_count = mask.count();
            if cell_count == 1 {
                continue;
            }

            if cell_count == 2 {
                return Some(cell);
            }

            if cell_count < best_cell_candidate_count {
                best_cell = Some(cell);
                best_cell_candidate_count = cell_count;
            }
        }

        best_cell
    }

    fn find_random_solution_for_board(&self, board: &Board) -> SingleSolutionResult {
        let mut board_stack = Vec::new();
        board_stack.push(Box::new(board.clone()));

        while !board_stack.is_empty() {
            let mut board = board_stack.pop().unwrap();
            if !self.run_brute_force_logic(&mut board) {
                continue;
            }

            if board.is_solved() {
                return SingleSolutionResult::Solved(board);
            }

            let cell = Self::find_best_brute_force_cell(&board);
            if let Some(cell) = cell {
                let mask = board.cell(cell);
                let value = mask.random();

                // Push a copy of the board onto the stack with the value unset.
                let mut board_copy = board.clone();
                if board_copy.clear_value(cell, value) {
                    board_stack.push(board_copy);
                }

                // Push a the board onto the stack with the value solved.
                if board.set_solved(cell, value) {
                    board_stack.push(board);
                }
            } else {
                return SingleSolutionResult::Error(
                    "Internal error finding a cell to check.".to_owned(),
                );
            }
        }

        SingleSolutionResult::None
    }
    /// Use brute-force methods to find a random solution to the puzzle.
    /// This can be faster than [`Solver::find_first_solution`] because it
    /// is not forced to find the lexicographically first solution.
    ///
    /// The solution is not guaranteed to be the only solution.
    pub fn find_random_solution(&self) -> SingleSolutionResult {
        self.find_random_solution_for_board(&self.board)
    }

    /// Using brute force methods, return a board with only candidates which lead to a valid solution to the puzzle.
    /// These candidates are guaranteed to lead to at least one solution if given.
    pub fn find_true_candidates(&self) -> SingleSolutionResult {
        let mut board = Box::new(self.board.clone());

        // Run the brute force logic to remove trivially invalid candidates.
        if !self.run_brute_force_logic(&mut board) {
            return SingleSolutionResult::None;
        }

        if board.is_solved() {
            return SingleSolutionResult::Solved(board);
        }

        let mut true_cell_values = board
            .all_cells()
            .map(|cell| {
                let mask = board.cell(cell);
                if mask.is_solved() {
                    mask
                } else {
                    ValueMask::new()
                }
            })
            .collect_vec();

        for (cell, mask) in board.all_cell_masks() {
            if mask.is_solved() {
                continue;
            }

            let mask = mask & !true_cell_values[cell.index()];
            for value in mask {
                let mut new_board = board.clone();
                if !new_board.set_solved(cell, value) {
                    continue;
                }

                let solution_result = self.find_random_solution_for_board(&new_board);
                if let SingleSolutionResult::Solved(solution) = solution_result {
                    for (cell, mask) in solution.all_cell_masks() {
                        true_cell_values[cell.index()] =
                            true_cell_values[cell.index()] | mask.unsolved();
                    }
                }
            }
        }

        for cell in board.all_cells() {
            if !board.keep_mask(cell, true_cell_values[cell.index()]) {
                return SingleSolutionResult::None;
            }
        }

        if AllNakedSingles.run(&mut board, false).is_invalid() {
            return SingleSolutionResult::None;
        }

        SingleSolutionResult::Solved(board)
    }

    /// Using brute force methods, return a board with only candidates which lead to a valid solution to the puzzle.
    /// These candidates are guaranteed to lead to at least one solution if given.
    pub fn find_true_candidates_with_count(
        &self,
        maximum_count: usize,
    ) -> TrueCandidatesCountResult {
        let mut board = Box::new(self.board.clone());
        let size = board.size();
        let num_candidates = size * size * size;

        // Run the brute force logic to remove trivially invalid candidates.
        if !self.run_brute_force_logic(&mut board) {
            return TrueCandidatesCountResult::None;
        }

        if board.is_solved() {
            return TrueCandidatesCountResult::Solved(board);
        }

        struct TrueCandidatesCountReceiver {
            true_cell_values: Vec<ValueMask>,
            num_solutions_per_candidate: Vec<usize>,
            solutions_seen: HashSet<Box<Board>>,
            maximum_count: usize,
            candidate: CandidateIndex,
        }

        impl SolutionReceiver for TrueCandidatesCountReceiver {
            fn receive(&mut self, board: Box<Board>) -> bool {
                if self.solutions_seen.contains(board.as_ref()) {
                    return true;
                }

                for (cell, mask) in board.all_cell_masks() {
                    self.true_cell_values[cell.index()] =
                        self.true_cell_values[cell.index()] | mask.unsolved();
                    let candidate_index = cell.candidate(mask.value());
                    self.num_solutions_per_candidate[candidate_index.index()] += 1;
                }
                self.solutions_seen.insert(board);

                self.num_solutions_per_candidate[self.candidate.index()] < self.maximum_count
            }
        }

        let true_cell_values = board
            .all_cells()
            .map(|cell| {
                let mask = board.cell(cell);
                if mask.is_solved() {
                    mask
                } else {
                    ValueMask::new()
                }
            })
            .collect_vec();

        let mut solution_receiver = TrueCandidatesCountReceiver {
            true_cell_values,
            num_solutions_per_candidate: vec![0; num_candidates],
            solutions_seen: HashSet::new(),
            maximum_count,
            candidate: CandidateIndex::new(0, size),
        };

        for (cell, mask) in board.all_cell_masks() {
            if mask.is_solved() {
                continue;
            }

            let mask = mask;
            for value in mask {
                let cur_candidate = cell.candidate(value);
                let cur_candidate_count =
                    solution_receiver.num_solutions_per_candidate[cur_candidate.index()];
                if cur_candidate_count >= maximum_count {
                    continue;
                }
                let count_needed = maximum_count - cur_candidate_count;

                let mut new_board = board.clone();
                if !new_board.set_solved(cell, value) {
                    continue;
                }

                solution_receiver.candidate = cur_candidate;
                self.find_solution_count_for_board(
                    &new_board,
                    count_needed,
                    Some(&mut solution_receiver),
                );
            }
        }

        let true_cell_values = solution_receiver.true_cell_values;
        for cell in board.all_cells() {
            if !board.keep_mask(cell, true_cell_values[cell.index()]) {
                return TrueCandidatesCountResult::None;
            }
        }

        if AllNakedSingles.run(&mut board, false).is_invalid() {
            return TrueCandidatesCountResult::None;
        }

        if board.is_solved() {
            TrueCandidatesCountResult::Solved(board)
        } else {
            TrueCandidatesCountResult::Candidates(
                board,
                solution_receiver.num_solutions_per_candidate,
            )
        }
    }

    fn find_solution_count_for_board(
        &self,
        board: &Board,
        maximum_count: usize,
        mut solution_receiver: Option<&mut dyn SolutionReceiver>,
    ) -> SolutionCountResult {
        let mut board_stack = Vec::new();
        board_stack.push(Box::new(board.clone()));

        let mut solution_count = 0;

        while !board_stack.is_empty() {
            let mut board = board_stack.pop().unwrap();
            if !self.run_brute_force_logic(&mut board) {
                continue;
            }

            if board.is_solved() {
                solution_count += 1;

                if let Some(ref mut solution_receiver) = solution_receiver {
                    if !solution_receiver.receive(board) {
                        return SolutionCountResult::AtLeastCount(solution_count);
                    }
                }

                if solution_count >= maximum_count {
                    return SolutionCountResult::AtLeastCount(solution_count);
                }
                continue;
            }

            let cell = Self::find_best_brute_force_cell(&board);
            if let Some(cell) = cell {
                let mask = board.cell(cell);
                for value in mask {
                    // Push a copy of the board onto the stack with each value set.
                    let mut board_copy = board.clone();
                    if board_copy.set_solved(cell, value) {
                        board_stack.push(board_copy);
                    }
                }
            } else {
                return SolutionCountResult::Error(
                    "Internal error finding a cell to check.".to_owned(),
                );
            }
        }

        if solution_count == 0 {
            SolutionCountResult::None
        } else {
            SolutionCountResult::ExactCount(solution_count)
        }
    }

    // Find the solution count of the puzzle via brute force with an optional receiver for each solution.
    pub fn find_solution_count(
        &self,
        maximum_count: usize,
        solution_receiver: Option<&mut dyn SolutionReceiver>,
    ) -> SolutionCountResult {
        self.find_solution_count_for_board(&self.board, maximum_count, solution_receiver)
    }
}

impl Default for Solver {
    fn default() -> Self {
        SolverBuilder::new(9).build().unwrap()
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
    }

    #[test]
    fn test_random_solution() {
        let solver = Solver::default();

        let result = solver.find_random_solution();
        assert!(result.is_solved());

        let board = result.board().unwrap();
        assert!(board.is_solved());

        let solution = board.to_string();
        assert!(solution.len() == 81);
        assert!(!solution.chars().any(|c| c < '1' || c > '9'));
    }

    #[test]
    fn test_true_candidates() {
        let solver = Solver::default();

        let result = solver.find_true_candidates();
        assert!(result.is_solved());
        assert!(result
            .board()
            .unwrap()
            .all_cell_masks()
            .all(|(_, mask)| mask.count() == 9));

        // Test phistomefel ring
        let solver = SolverBuilder::default()
            .with_givens_string(
                "....................23456....4...2....5...3....6...4....74365....................",
            )
            .build()
            .unwrap();
        let result = solver.find_true_candidates();
        assert!(result.is_solved());
        let board = result.board().unwrap();
        assert!(!board.is_solved());

        let cu = board.cell_utility();
        assert!(board.cell(cu.cell(0, 0)) == ValueMask::from_values(&[3, 4, 5, 6, 7]));
        assert!(board.cell(cu.cell(0, 1)) == ValueMask::from_values(&[3, 4, 5, 6, 7]));
        assert!(board.cell(cu.cell(1, 0)) == ValueMask::from_values(&[3, 4, 5, 6, 7]));
        assert!(board.cell(cu.cell(1, 1)) == ValueMask::from_values(&[3, 4, 5, 6, 7]));
        assert!(board.cell(cu.cell(7, 0)) == ValueMask::from_values(&[2, 3, 4, 5, 6]));
        assert!(board.cell(cu.cell(7, 1)) == ValueMask::from_values(&[2, 3, 4, 5, 6]));
        assert!(board.cell(cu.cell(8, 0)) == ValueMask::from_values(&[2, 3, 4, 5, 6]));
        assert!(board.cell(cu.cell(8, 1)) == ValueMask::from_values(&[2, 3, 4, 5, 6]));
        assert!(board.cell(cu.cell(7, 7)) == ValueMask::from_values(&[2, 3, 4, 6, 7]));
        assert!(board.cell(cu.cell(7, 8)) == ValueMask::from_values(&[2, 3, 4, 6, 7]));
        assert!(board.cell(cu.cell(8, 7)) == ValueMask::from_values(&[2, 3, 4, 6, 7]));
        assert!(board.cell(cu.cell(7, 8)) == ValueMask::from_values(&[2, 3, 4, 6, 7]));
    }

    #[test]
    fn test_true_candidates_with_count() {
        let solver = SolverBuilder::default()
            .with_givens_string(
                "1...2..4...7...3...6..1..5..7......4.4.5.9..6.....8.3.4..2.........5.....8...6.7.",
            )
            .build()
            .unwrap();
        let result = solver.find_true_candidates_with_count(8);
        assert!(result.is_candidates());
        let board = result.board().unwrap();
        let cu = board.cell_utility();
        assert_eq!(
            board.cell(cu.cell(0, 1)),
            ValueMask::from_values(&[3, 5, 9])
        );
        assert_eq!(board.cell(cu.cell(1, 5)), ValueMask::from_values(&[4, 5]));

        let candidates = result.candidate_counts().unwrap();
        assert_eq!(candidates.len(), 9 * 9 * 9);

        let candidate3r1c2 = cu.cell(0, 1).candidate(3);
        let candidate5r1c2 = cu.cell(0, 1).candidate(5);
        let candidate3r1c6 = cu.cell(0, 5).candidate(3);
        let candidate5r2c6 = cu.cell(1, 5).candidate(5);
        let candidate4r8c6 = cu.cell(7, 5).candidate(4);
        assert!(candidates[candidate3r1c2.index()] >= 8);
        assert_eq!(candidates[candidate5r1c2.index()], 2);
        assert_eq!(candidates[candidate3r1c6.index()], 2);
        assert_eq!(candidates[candidate5r2c6.index()], 2);
        assert_eq!(candidates[candidate4r8c6.index()], 2);
    }

    #[test]
    fn test_solution_count() {
        let solver = SolverBuilder::default().build().unwrap();
        let result = solver.find_solution_count(100, None);
        assert!(result.is_at_least_count());
        assert!(result.count().unwrap() >= 100);

        let solver = SolverBuilder::default()
            .with_givens_string(
                "........1....23.4.....452....1.3.....3...4...6..7....8..6.....9.5....62.7.9...1..",
            )
            .build()
            .unwrap();
        let result = solver.find_solution_count(100, None);
        assert!(result.is_exact_count());
        assert_eq!(result.count().unwrap(), 1);

        let solver = SolverBuilder::default()
            .with_givens_string(
                ".............23.4.....452....1.3.....3...4...6..7....8..6.....9.5....62.7.9...1..",
            )
            .build()
            .unwrap();
        let result = solver.find_solution_count(10000, None);
        assert!(result.is_exact_count());
        assert_eq!(result.count().unwrap(), 2357);

        let solver = SolverBuilder::default()
            .with_givens_string(
                "1...................23456....4...2....5...3....6...4....74365....................",
            )
            .build()
            .unwrap();
        let result = solver.find_solution_count(2, None);
        assert!(result.is_none());

        let mut receiver = VecSolutionReceiver::new();
        let solver = SolverBuilder::default()
            .with_givens_string(
                "8...62..1.5.....7..197...5........9.....28..3.....36.54...1..6...74...3.5.2......",
            )
            .build()
            .unwrap();
        let result = solver.find_solution_count(100, Some(&mut receiver));
        assert!(result.is_exact_count());
        assert_eq!(result.count().unwrap(), 2);

        let solutions = receiver.take_solutions();
        assert_eq!(solutions.len(), 2);
        assert!(solutions.iter().any(|b| b.to_string() == "873562941654891372219734856326157498945628713781943625438219567167485239592376184"));
        assert!(solutions.iter().any(|b| b.to_string() == "873562941254891376619734852326157498945628713781943625438219567167485239592376184"));
    }

    #[test]
    fn test_single_logical_step() {
        let mut solver = SolverBuilder::default()
            .with_givens_string(
                "8...62..125.....7..197...5........9.....28..3.....36.54...1..6...74...3.5.2......",
            )
            .build()
            .unwrap();
        let result = solver.run_single_logical_step();
        assert!(result.is_changed());
        let desc = result.description().unwrap();
        assert!(desc.to_string().contains("Single"));
    }

    #[test]
    fn test_logical_solve() {
        let mut solver = SolverBuilder::default()
            .with_givens_string(
                "8...62..125.....7..197...5........9.....28..3.....36.54...1..6...74...3.5.2......",
            )
            .build()
            .unwrap();
        let result = solver.run_logical_solve();
        assert!(result.is_solved());
        let desc = result.description().unwrap();
        assert_eq!(desc.len(), 56);

        let board = solver.board();
        assert!(board.is_solved());
        assert_eq!(
            board.to_string(),
            "873562941254891376619734852326157498945628713781943625438219567167485239592376184"
        );
    }
}
