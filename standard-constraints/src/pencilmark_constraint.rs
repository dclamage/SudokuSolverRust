//! Contains the [`PencilmarkConstraint`] struct for restricting a cell to specific pencilmarks.

use sudoku_solver_lib::prelude::*;

/// A [`Constraint`] implementation for restricting a cell to specific pencilmarks.
#[derive(Debug)]
pub struct PencilmarkConstraint {
    specific_name: String,
    cell: CellIndex,
    values: ValueMask,
}

impl PencilmarkConstraint {
    /// Creates a new [`PencilmarkConstraint`] with the given cell and values.
    pub fn new(cell: CellIndex, values: ValueMask) -> Self {
        Self { specific_name: format!("{}{}", values, cell), cell, values }
    }

    /// Creates a new [`PencilmarkConstraint`] that restricts the given cell to only even values.
    pub fn even(cell: CellIndex) -> Self {
        let size = cell.size();
        let mut values = ValueMask::default();
        for i in (2..=size).step_by(2) {
            values = values | ValueMask::from_value(i);
        }
        Self { specific_name: format!("Even {}", cell), cell, values }
    }

    /// Creates a new [`PencilmarkConstraint`] that restricts the given cell to only odd values.
    pub fn odd(cell: CellIndex) -> Self {
        let size = cell.size();
        let mut values = ValueMask::default();
        for i in (1..=size).step_by(2) {
            values = values | ValueMask::from_value(i);
        }
        Self { specific_name: format!("Odd {}", cell), cell, values }
    }

    /// Creates a new [`PencilmarkConstraint`] that restricts the given cell to only prime values.
    pub fn prime(cell: CellIndex) -> Self {
        let size = cell.size();
        let mut values = ValueMask::default();
        for i in 2..=size {
            if Self::is_prime(i) {
                values = values | ValueMask::from_value(i);
            }
        }
        Self { specific_name: format!("Prime {}", cell), cell, values }
    }

    fn is_prime(n: usize) -> bool {
        if n <= 3 {
            n > 1
        } else if n % 2 == 0 || n % 3 == 0 {
            false
        } else {
            let mut i = 5;
            while i * i <= n {
                if n % i == 0 || n % (i + 2) == 0 {
                    return false;
                }
                i += 6;
            }
            true
        }
    }
}

impl Constraint for PencilmarkConstraint {
    fn name(&self) -> &str {
        self.specific_name.as_str()
    }

    fn get_weak_links(&self, size: usize) -> Vec<(CandidateIndex, CandidateIndex)> {
        let mut result = Vec::new();
        let clear_mask = self.values.inverted(size);
        for value in clear_mask {
            let candidate = self.cell.candidate(value);
            result.push((candidate, candidate));
        }

        result
    }
}
