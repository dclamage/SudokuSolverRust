//! Contains the [`ChessConstraint`] struct for representing a chess constraint.

use sudoku_solver_lib::prelude::*;

/// A [`Constraint`] implementation for representing a chess constraint.
#[derive(Debug)]
pub struct ChessConstraint {
    specific_name: String,
    offsets: Vec<(isize, isize)>,
}

impl ChessConstraint {
    /// Creates a new [`ChessConstraint`] with any arbitrary offsets.
    pub fn new(specific_name: &str, offsets: Vec<(isize, isize)>) -> Self {
        Self {
            specific_name: specific_name.to_owned(),
            offsets,
        }
    }

    /// Creates a new [`ChessConstraint`] with the symmetric offsets.
    pub fn from_symmetric_offset(specific_name: &str, offset: (isize, isize)) -> Self {
        let mut offsets = Vec::new();
        offsets.push(offset);
        offsets.push((offset.1, offset.0));
        if offset.0 != 0 {
            offsets.push((-offset.0, offset.1));
            offsets.push((offset.1, -offset.0));
        }
        if offset.1 != 0 {
            offsets.push((offset.0, -offset.1));
            offsets.push((-offset.1, offset.0));
        }
        if offset.0 != 0 && offset.1 != 0 {
            offsets.push((-offset.0, -offset.1));
            offsets.push((-offset.1, -offset.0));
        }
        Self::new(specific_name, offsets)
    }

    /// Creates the standard "anti-king" constraint.
    pub fn anti_king() -> Self {
        Self::from_symmetric_offset("Anti-King", (1, 1))
    }

    /// Creates the standard "anti-knight" constraint.
    pub fn anti_knight() -> Self {
        Self::from_symmetric_offset("Anti-Knight", (1, 2))
    }

    /// Creates the standard "anti-camel" constraint.
    pub fn anti_camel() -> Self {
        Self::from_symmetric_offset("Anti-Camel", (1, 3))
    }

    /// Creates an anti-taxicab constraint.
    pub fn anti_taxicab(dist: usize) -> Self {
        let mut offset = Vec::new();
        for i in -(dist as isize)..=(dist as isize) {
            if i == 0 {
                continue;
            }
            for j in -(dist as isize)..=(dist as isize) {
                if j == 0 {
                    continue;
                }

                if i.abs() + j.abs() == dist as isize {
                    offset.push((i, j));
                }
            }
        }

        Self::new(&format!("Anti-Taxicab {}", dist), offset)
    }
}

impl Constraint for ChessConstraint {
    fn name(&self) -> &str {
        &self.specific_name
    }

    fn get_weak_links(&self, size: usize) -> Vec<(CandidateIndex, CandidateIndex)> {
        let mut result = Vec::new();
        let cu = CellUtility::new(size);
        for cell in cu.all_cells() {
            for (offset_row, offset_col) in &self.offsets {
                let other_cell = cell.offset(*offset_row, *offset_col);
                if let Some(other_cell) = other_cell {
                    for value in 1..=size {
                        result.push((cell.candidate(value), other_cell.candidate(value)));
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_anti_king_anti_knight_count() {
        let solver = SolverBuilder::default()
            .with_constraint(Arc::new(ChessConstraint::anti_king()))
            .with_constraint(Arc::new(ChessConstraint::anti_knight()))
            .with_givens_string(
                "123456789000000000000000000000000000000000000000000000000000000000000000000000000",
            )
            .build()
            .unwrap();

        let solution_count = solver.find_solution_count(10000, None);
        assert!(solution_count.is_exact_count());
        assert_eq!(solution_count.count().unwrap(), 4);
    }
}
