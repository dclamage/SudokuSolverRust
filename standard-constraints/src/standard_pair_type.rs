//! Contains the [`StandardPairType`] enum for representing the different types of standard pair constraints.

use sudoku_solver_lib::prelude::*;

/// Represents the different types of standard pair constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardPairType {
    /// The sum of the two cells must equal the given value.
    Sum(usize),
    /// The difference of the two cells must equal the given value.
    Diff(usize),
    /// The ratio of the two cells must equal the given value.
    Ratio(usize),
}

impl StandardPairType {
    /// Returns the name of the constraint type.
    pub fn name(&self) -> String {
        match self {
            Self::Sum(n) => format!("s{n}"),
            Self::Diff(n) => format!("d{n}"),
            Self::Ratio(n) => format!("r{n}"),
        }
    }

    /// Returns the candidate pairs for thie constraint type
    pub fn candidate_pairs(&self, size: usize) -> Vec<ValueMask> {
        match self {
            Self::Sum(n) => {
                let n = *n;
                let mut pairs = Vec::new();
                for i in 1..=size {
                    let mut mask = ValueMask::new();
                    for j in 1..=size {
                        if i + j == n {
                            mask = mask.with(j);
                        }
                    }
                    pairs.push(mask);
                }
                pairs
            }
            Self::Diff(n) => {
                let n = *n;
                let mut pairs = Vec::new();
                for i in 1..=size {
                    let mut mask = ValueMask::new();
                    for j in 1..=size {
                        if i + n == j || j + n == i {
                            mask = mask.with(j);
                        }
                    }
                    pairs.push(mask);
                }
                pairs
            }
            Self::Ratio(n) => {
                let n = *n;
                let mut pairs = Vec::new();
                for i in 1..=size {
                    let mut mask = ValueMask::new();
                    for j in 1..=size {
                        if i * n == j || j * n == i {
                            mask = mask.with(j);
                        }
                    }
                    pairs.push(mask);
                }
                pairs
            }
        }
    }
}
