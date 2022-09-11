//! Contains the [`NonRepeatConstraint`] struct for representing a constraint where cells cannot repeat values.

use sudoku_solver_lib::prelude::*;

/// A [`Constraint`] implementation for representing a group of cells which cannot repeat digits.
/// The number of cells passed cannot exceed the size of the grid, as that would be impossible.
/// If the number of cells is equal to the size of the grid, this constraint is also considered
/// to be a "house" for logical steps which use houses, like hidden singles and tuples.
#[derive(Debug)]
pub struct NonRepeatConstraint {
    specific_name: String,
    cells: Vec<CellIndex>,
}

impl NonRepeatConstraint {
    pub fn new(specific_name: &str, cells: Vec<CellIndex>) -> Self {
        Self {
            specific_name: specific_name.to_owned(),
            cells,
        }
    }

    pub fn from_diagonalp(size: usize) -> Self {
        let cu = CellUtility::new(size);
        let mut cells = Vec::new();
        for i in 0..size {
            cells.push(cu.cell(size - i - 1, i));
        }
        Self::new("Diagonal+", cells)
    }

    pub fn from_diagonaln(size: usize) -> Self {
        let cu = CellUtility::new(size);
        let mut cells = Vec::new();
        for i in 0..size {
            cells.push(cu.cell(i, i));
        }
        Self::new("Diagonal-", cells)
    }
}

impl Constraint for NonRepeatConstraint {
    fn name(&self) -> &str {
        self.specific_name.as_str()
    }

    fn get_weak_links(&self, size: usize) -> Vec<(CandidateIndex, CandidateIndex)> {
        if self.cells.len() > 1 && self.cells.len() <= size {
            get_weak_links_for_nonrepeat(self.cells.iter().copied())
        } else {
            Vec::new()
        }
    }

    fn get_houses(&self, size: usize) -> Vec<House> {
        if self.cells.len() == size {
            vec![House::new(self.specific_name.as_str(), &self.cells)]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_sudokux() {
        let size = 9;
        let solver = SolverBuilder::new(size)
            .with_constraint(Arc::new(NonRepeatConstraint::from_diagonalp(size)))
            .with_constraint(Arc::new(NonRepeatConstraint::from_diagonaln(size)))
            .with_givens_string(
                "......78............9.........................1.5.........4.....3....5.1....98...",
            )
            .build()
            .unwrap();
        assert_eq!(solver.board().houses().len(), 29);
        let solution_count = solver.find_solution_count(10000, None);
        assert!(solution_count.is_exact_count());
        assert_eq!(solution_count.count().unwrap(), 2);
    }
}
