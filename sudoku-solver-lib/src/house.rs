//! Contains [`House`] for representing the cells in a house along with its name.

use crate::cell_index::CellIndex;

/// A *house* is a group of N cells where N is the size of the board where
/// digits cannot repeat within that group.
///
/// Conclusions for a house:
///  - Every possible digit from 1-N appears within the house exactly once
///  - No digit repeats within a house.
///
/// Examples of houses:
///  - A row
///  - A column
///  - A region (i.e. 3x3 square)
///  - An "extra region"
///  - A Killer Cage of size N
///  - A Renban of size N
#[derive(Debug, Clone)]
pub struct House {
    name: String,
    cells: Vec<CellIndex>,
}

impl House {
    /// Create a new house with the given name and cells.
    pub fn new(name: &str, cells: &[CellIndex]) -> House {
        let mut cells = cells.to_vec();
        cells.sort();

        House { name: name.to_string(), cells }
    }

    /// Get the name of the house.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the cells that make up the house.
    pub fn cells(&self) -> &Vec<CellIndex> {
        &self.cells
    }
}

impl std::fmt::Display for House {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
