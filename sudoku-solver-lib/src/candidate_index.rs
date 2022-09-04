//! Contains [`CandidateIndex`] for representing the location of a specific candidate
//! within a cell.

use crate::prelude::*;

/// Represents the location of a specific candidate within a cell.
///
/// A 9x9 grid has 81 cells with 9 candidates each, and thus
/// 729 candidates in total, so candidates are indexed 0 to 729
/// in that case.
#[derive(Clone, Copy, Debug)]
pub struct CandidateIndex {
    index: usize,
    size: usize,
}

impl CandidateIndex {
    /// Creates a new instance.
    pub fn new(index: usize, size: usize) -> Self {
        Self { index, size }
    }

    /// Creates a new instance from a cell index and value.
    pub fn from_cv(cell: CellIndex, value: usize) -> Self {
        Self {
            index: cell.index() * cell.size() + value - 1,
            size: cell.size(),
        }
    }

    /// Gets the index of the candidate.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Gets the size of the board.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Gets the cell index of the candidate.
    pub fn cell_index(&self) -> CellIndex {
        CellIndex::new(self.index / self.size, self.size)
    }

    /// Gets the value of the candidate.
    pub fn value(&self) -> usize {
        self.index % self.size + 1
    }

    /// Gets the cell index and value of the candidate.
    pub fn cell_index_and_value(&self) -> (CellIndex, usize) {
        (self.cell_index(), self.value())
    }
}

impl std::fmt::Display for CandidateIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let (cell, value) = self.cell_index_and_value();
        write!(f, "{}{}", value, cell)
    }
}

impl Eq for CandidateIndex {}

impl PartialEq for CandidateIndex {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Ord for CandidateIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl PartialOrd for CandidateIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for CandidateIndex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_index() {
        assert_eq!(CandidateIndex::from_cv(CellIndex::new(0, 9), 1).index(), 0);
        assert_eq!(CandidateIndex::from_cv(CellIndex::new(1, 9), 1).index(), 9);
        assert_eq!(
            CandidateIndex::from_cv(CellIndex::new(1, 16), 2).index(),
            17
        );
        assert_eq!(CandidateIndex::from_cv(CellIndex::new(9, 8), 2).index(), 73);
        assert_eq!(
            CandidateIndex::from_cv(CellIndex::new(40, 9), 5).index(),
            364
        );
        assert_eq!(
            CandidateIndex::from_cv(CellIndex::new(80, 9), 9).index(),
            728
        );
    }
}
