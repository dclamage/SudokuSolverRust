//! Contains the [`SolverBuilder`] struct for building a [`Solver`].

use itertools::Itertools;

use crate::prelude::*;

use std::{any::TypeId, collections::HashMap, sync::Arc};

/// Builds a [`Solver`].
#[derive(Clone, Debug)]
pub struct SolverBuilder {
    size: usize,
    regions: Vec<usize>,
    logical_steps: Vec<Arc<dyn LogicalStep>>,
    constraints: Vec<Arc<dyn Constraint>>,
    givens: Vec<(CellIndex, usize)>,
    errors: Vec<String>,
    custom_info: HashMap<String, String>,
}

impl SolverBuilder {
    /// Creates a new solver builder.
    pub fn new(size: usize) -> Self {
        Self {
            size,
            regions: Vec::new(),
            logical_steps: Vec::new(),
            constraints: Vec::new(),
            givens: Vec::new(),
            errors: Vec::new(),
            custom_info: HashMap::new(),
        }
    }

    /// Set the regions of the board.
    ///
    /// The vector is expected to be of length `size * size`.
    ///
    /// Each element of the vector is the region index of the corresponding cell.
    /// The region index must be in the range `[0, size)`.
    ///
    /// The following cases are treated the same as [`Self::with_no_regions`]:
    /// * Empty region vector
    /// * A region vector of the correct length, but with all region indexes being the same value.
    #[must_use]
    pub fn with_regions(mut self, regions: Vec<usize>) -> Self {
        let size = self.size;

        // Special case an empty vector or a vector of the correct length
        // but with all elements the same value to mean "use no regions".
        if regions.is_empty()
            || regions.len() == size * size && regions.iter().all(|&r| r == regions[0])
        {
            return self.with_no_regions();
        }

        if regions.len() != size * size {
            self.errors.push(format!(
                "Region vector is of length {}, expected {}",
                regions.len(),
                self.size * self.size
            ));
            return self;
        }
        for value in 0..size {
            if regions.iter().filter(|&&x| x == value).count() != size {
                self.errors.push(format!(
                    "Region vector contains {} instances of region index {}, expected {}",
                    regions.iter().filter(|&&x| x == value).count(),
                    value,
                    size
                ));
                return self;
            }
        }
        self.regions = regions;
        self
    }

    /// Set the board to use no regions.
    #[must_use]
    pub fn with_no_regions(mut self) -> Self {
        // The solver interprets an all 0 region vector as no regions.
        let num_cells = self.size * self.size;
        self.regions = vec![0; num_cells];
        self
    }

    /// Set the full list of logical steps to use.
    /// This will replace any existing logical steps.
    #[must_use]
    pub fn with_logical_steps(mut self, logical_steps: Vec<Arc<dyn LogicalStep>>) -> Self {
        self.logical_steps = logical_steps;
        self
    }

    /// Add a logical step to the list of logical steps to use.
    /// This will not replace any existing logical steps and will append to the end.
    #[must_use]
    pub fn with_logical_step(mut self, logical_step: Arc<dyn LogicalStep>) -> Self {
        self.logical_steps.push(logical_step);
        self
    }

    /// Set the full list of constraints to use.
    /// This will replace any existing constraints.
    #[must_use]
    pub fn with_constraints(mut self, constraints: Vec<Arc<dyn Constraint>>) -> Self {
        self.constraints = constraints;
        self
    }

    /// Add a constraint to the list of constraints to use.
    /// This will not replace any existing constraints and will append to the end.
    #[must_use]
    pub fn with_constraint(mut self, constraint: Arc<dyn Constraint>) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Set a single given to use.
    /// This will append to the list of givens.
    #[must_use]
    pub fn with_given(mut self, cell_index: CellIndex, value: usize) -> Self {
        self.givens.push((cell_index, value));
        self
    }

    /// Set multiple givens to use.
    /// This will append to the list of givens.
    #[must_use]
    pub fn with_givens(mut self, givens: &[(CellIndex, usize)]) -> Self {
        self.givens.extend(givens);
        self
    }

    /// Set the givens from a given string, appending those to any existing givens.
    /// The string should be a sequence of numbers, with 0 or any non-digit representing an empty cell.
    /// The string should be in row-major order.
    /// For grid sizes larger than 9, the each number takes the same number of characters, so use 01 for 1, for example.
    #[must_use]
    pub fn with_givens_string(mut self, givens: &str) -> Self {
        let cu = CellUtility::new(self.size);
        if cu.size() <= 9 {
            if givens.len() != self.size * self.size {
                self.errors.push("Invalid givens string length".to_owned());
                return self;
            }

            self.givens
                .extend(givens.chars().enumerate().filter_map(|(i, c)| {
                    let value = c.to_digit(10)?;
                    if value == 0 {
                        None
                    } else {
                        Some((cu.cell_index(i), value as usize))
                    }
                }));
        } else {
            let num_digits = cu.size().to_string().len();
            if givens.len() != self.size * self.size * num_digits {
                self.errors.push("Invalid givens string length".to_owned());
                return self;
            }

            let givens_chunks_itr = givens.chars().chunks(num_digits);
            self.givens.extend(
                givens_chunks_itr
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
                    }),
            );
        }
        self
    }

    pub fn with_custom_info(mut self, key: &str, value: &str) -> Self {
        self.custom_info.insert(key.to_owned(), value.to_owned());
        self
    }

    fn standard_logic() -> Vec<Arc<dyn LogicalStep>> {
        vec![
            Arc::new(AllNakedSingles),
            Arc::new(HiddenSingle),
            Arc::new(NakedSingle),
            Arc::new(StepConstraints),
            Arc::new(SimpleCellForcing),
        ]
    }

    pub fn build(mut self) -> Result<Solver, String> {
        if !self.errors.is_empty() {
            return Err(self.errors.join(", "));
        }

        let mut board = Board::new(self.size, &self.regions, &self.constraints);
        let board_data = board.data();

        // Apply the givens.
        for (cell, value) in self.givens {
            if !board.cell(cell).is_solved() && !board.set_solved(cell, value) {
                return Err(format!("Failed to set given {}{}", value, cell));
            }
        }

        // Initialize the constraints
        let mut changed = true;
        while changed {
            changed = false;

            for constraint in board_data.constraints() {
                let result = constraint.init_board(&mut board);
                if let LogicalStepResult::Invalid(desc) = result {
                    if let Some(desc) = desc {
                        return Err(format!(
                            "{} has found the board is invalid: {}",
                            constraint.name(),
                            desc
                        ));
                    } else {
                        return Err(format!(
                            "{} has found the board is invalid.",
                            constraint.name()
                        ));
                    }
                } else if result.is_changed() {
                    changed = true;
                }
            }
        }

        // Construct the logical step lists.
        if self.logical_steps.is_empty() {
            self.logical_steps = Self::standard_logic();
        } else {
            // There are two required logical steps which must always be present:
            // 1. AllNakedSingles is used by the brute force solver.
            // 2. StepConstraints is used to apply constraint logic.

            if !self
                .logical_steps
                .iter()
                .any(|step| step.type_id() == TypeId::of::<AllNakedSingles>())
            {
                // The AllNakedSingles step is required by the brute force solver.
                // Put it first in the list.
                self.logical_steps.insert(0, Arc::new(AllNakedSingles));
            }

            if !self
                .logical_steps
                .iter()
                .any(|step| step.type_id() == TypeId::of::<StepConstraints>())
            {
                // The StepConstraints step is required to apply constraint logic.
                // Put it in the list after any singles steps.
                let naked_single_index = self
                    .logical_steps
                    .iter()
                    .position(|step| step.type_id() == TypeId::of::<NakedSingle>());
                let hidden_single_index = self
                    .logical_steps
                    .iter()
                    .position(|step| step.type_id() == TypeId::of::<HiddenSingle>());

                let index = match (naked_single_index, hidden_single_index) {
                    (Some(naked_single_index), Some(hidden_single_index)) => {
                        naked_single_index.max(hidden_single_index) + 1
                    }
                    (Some(naked_single_index), None) => naked_single_index + 1,
                    (None, Some(hidden_single_index)) => hidden_single_index + 1,
                    (None, None) => 0,
                };
                self.logical_steps.insert(index, Arc::new(StepConstraints));
            }
        }

        let logical_solve_steps = self
            .logical_steps
            .iter()
            .cloned()
            .filter(|step| step.is_active_during_logical_solves())
            .collect();

        let brute_force_steps = self
            .logical_steps
            .iter()
            .cloned()
            .filter(|step| step.is_active_during_brute_force_solves())
            .collect();

        let solver = Solver {
            board,
            logical_solve_steps,
            brute_force_steps,
            custom_info: self.custom_info,
        };

        Ok(solver)
    }
}

impl Default for SolverBuilder {
    fn default() -> Self {
        Self::new(9)
    }
}

#[cfg(test)]
mod test {
    use itertools::assert_equal;

    use super::*;

    #[test]
    fn test_solver_default() {
        let solver = SolverBuilder::default().build().unwrap();
        let board = solver.board();

        // Check that the created board has all the expected defaults.
        assert_eq!(board.size(), 9);
        assert_eq!(board.solved_count(), 0);
        assert!(!board.is_solved());
        assert_eq!(board.houses().len(), 27);
        assert_eq!(board.constraints().len(), 0);
    }

    #[test]
    fn test_solver_no_regions() {
        let solver = SolverBuilder::default().with_no_regions().build().unwrap();
        let board = solver.board();

        // Check that the created board has all the expected info.
        assert_eq!(board.size(), 9);
        assert_eq!(board.solved_count(), 0);
        assert!(!board.is_solved());
        assert_eq!(board.houses().len(), 18);
        assert_eq!(board.constraints().len(), 0);
    }

    #[test]
    fn test_required_logic() {
        let solver = SolverBuilder::new(9)
            .with_logical_step(Arc::new(HiddenSingle))
            .build()
            .unwrap();
        assert_equal(
            solver
                .brute_force_steps
                .iter()
                .map(|s| s.name())
                .collect::<Vec<_>>(),
            ["All Naked Singles", "Hidden Single", "Step Constraints"],
        );

        assert_equal(
            solver
                .logical_solve_steps
                .iter()
                .map(|s| s.name())
                .collect::<Vec<_>>(),
            ["Hidden Single", "Step Constraints"],
        );
    }
}
