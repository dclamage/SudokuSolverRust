//! Contains [`Board`] which represents a Sudoku puzzle's size, constraints, and current solve state.

use bitvec::bitvec;
use bitvec::vec::BitVec;
use itertools::Itertools;

use crate::prelude::*;
use std::{collections::HashMap, sync::Arc};

/// Represents the state of the sudoku board.
///
/// Operations for querying and modifying the board are provided.
///
/// Meta-data about the board is stored in the [`BoardData`] struct which is
/// accessible via the `data` method.
///
/// Unless [`Board::deep_clone`] is used, the board metadata is not copied,
/// and instead is shared among boards when cloned. This makes cloning faster,
/// and is generally safe because board metadata can't be changed after initialization.
#[derive(Clone)]
pub struct Board {
    board: Vec<ValueMask>,
    solved_count: usize,
    data: Arc<BoardData>,
}

/// Contains meta-data about the board.
///
/// This data is immutable after initialization and contains information
/// about the board's size, constraints, and other information.
#[derive(Clone)]
pub struct BoardData {
    size: usize,
    num_cells: usize,
    num_candidates: usize,
    all_values_mask: ValueMask,
    houses: Vec<Arc<House>>,
    houses_by_cell: Vec<Vec<Arc<House>>>,
    powerful_cells: Vec<CellIndex>,
    weak_links: Vec<CandidateLinks>,
    total_weak_links: usize,
    exclusive_cells: Vec<BitVec>,
    constraints: Vec<Arc<dyn Constraint>>,
}

impl Board {
    pub fn new(size: usize, regions: &[usize], constraints: Vec<Arc<dyn Constraint>>) -> Board {
        let mut data = BoardData::new(size, regions, constraints);
        let elims = data.init_weak_links();

        let mut board =
            Board { board: vec![data.all_values_mask; data.num_cells], solved_count: 0, data: Arc::new(data) };

        board.clear_candidates(elims.iter());

        board
    }

    pub fn deep_clone(&self) -> Board {
        Board {
            board: self.board.clone(),
            solved_count: self.solved_count,
            data: Arc::new(BoardData::clone(&self.data)),
        }
    }

    pub fn init_constraints(&mut self) -> Result<(), String> {
        let constraint_count = self.data.constraints().len();

        loop {
            let mut changed = false;

            for idx in 0..constraint_count {
                let constraint = Arc::get_mut(&mut self.data).map(|d| d.take_constraint(idx));
                if let Some(mut constraint) = constraint {
                    let mut constraint_mut = Arc::get_mut(&mut constraint);
                    if let Some(ref mut constraint_mut) = constraint_mut {
                        let result = constraint_mut.init_board(self);
                        if let LogicalStepResult::Invalid(desc) = result {
                            if let Some(desc) = desc {
                                return Err(format!("{} has found the board is invalid: {}", constraint.name(), desc));
                            } else {
                                return Err(format!("{} has found the board is invalid.", constraint.name()));
                            }
                        } else if result.is_changed() {
                            changed = true;
                        }
                    } else {
                        return Err(format!("Failed to get mutable constraint for {}", constraint.name()));
                    }
                    if let Some(d) = Arc::get_mut(&mut self.data) {
                        d.insert_constraint(idx, constraint)
                    }
                } else {
                    return Err("Failed to get mutable board data".to_owned());
                }
            }

            if !changed {
                break;
            }
        }

        Ok(())
    }

    pub fn solved_count(&self) -> usize {
        self.solved_count
    }

    pub fn is_solved(&self) -> bool {
        self.solved_count == self.data.num_cells
    }

    pub fn data(&self) -> Arc<BoardData> {
        self.data.clone()
    }

    pub fn size(&self) -> usize {
        self.data.size
    }

    pub fn num_cells(&self) -> usize {
        self.data.num_cells
    }

    pub fn num_candidates(&self) -> usize {
        self.data.num_candidates
    }

    pub fn all_values_mask(&self) -> ValueMask {
        self.data.all_values_mask
    }

    pub fn houses(&self) -> &[Arc<House>] {
        &self.data.houses
    }

    pub fn houses_for_cell(&self, cell: CellIndex) -> &[Arc<House>] {
        &self.data.houses_by_cell[cell.index()]
    }

    pub fn total_weak_links(&self) -> usize {
        self.data.total_weak_links
    }

    pub fn weak_links(&self) -> &[CandidateLinks] {
        &self.data.weak_links
    }

    pub fn constraints(&self) -> &[Arc<dyn Constraint>] {
        &self.data.constraints
    }

    pub fn cell(&self, cell: CellIndex) -> ValueMask {
        self.board[cell.index()]
    }

    pub fn cell_utility(&self) -> CellUtility {
        CellUtility::new(self.size())
    }

    pub fn all_cells(&self) -> impl Iterator<Item = CellIndex> {
        self.cell_utility().all_cells()
    }

    pub fn all_cell_masks(&self) -> impl Iterator<Item = (CellIndex, ValueMask)> + '_ {
        self.all_cells().map(move |cell| (cell, self.cell(cell)))
    }

    pub fn has_candidate(&self, candidate: CandidateIndex) -> bool {
        let (cell, val) = candidate.cell_index_and_value();
        self.cell(cell).has(val)
    }

    pub fn clear_value(&mut self, cell: CellIndex, val: usize) -> bool {
        let cell = cell.index();
        self.board[cell] = self.board[cell].without(val);
        !self.board[cell].is_empty()
    }

    pub fn clear_candidate(&mut self, candidate: CandidateIndex) -> bool {
        let (cell, val) = candidate.cell_index_and_value();
        self.clear_value(cell, val)
    }

    pub fn clear_candidates(&mut self, candidates: impl Iterator<Item = CandidateIndex>) -> bool {
        let mut valid = true;
        for candidate in candidates {
            if !self.clear_candidate(candidate) {
                valid = false;
            }
        }
        valid
    }

    pub fn keep_mask(&mut self, cell: CellIndex, mask: ValueMask) -> bool {
        let cell = cell.index();
        self.board[cell] = self.board[cell] & mask;
        !self.board[cell].is_empty()
    }

    pub fn set_solved(&mut self, cell: CellIndex, value: usize) -> bool {
        // Is this value possible?
        if !self.cell(cell).has(value) {
            return false;
        }

        // Check if already solved
        if self.board[cell.index()].is_solved() {
            return false;
        }

        // Mark as solved
        self.board[cell.index()] = self.board[cell.index()].with_only(value).solved();
        self.solved_count += 1;

        // Clone the BoardData Arc to avoid borrowing issues
        let board_data = self.data.clone();

        // Apply all weak links
        let cu = CellUtility::new(self.size());
        let set_candidate_index = cu.candidate(cell, value);
        for candidate_index in board_data.weak_links[set_candidate_index.index()].links() {
            if !self.clear_candidate(candidate_index) {
                return false;
            }
        }

        // Enforce all constraints
        for constraint in board_data.constraints.iter() {
            if constraint.enforce(self, cell, value).is_invalid() {
                return false;
            }
        }

        true
    }

    pub fn set_mask(&mut self, cell: usize, mask: ValueMask) -> bool {
        assert!(!mask.is_solved());
        if mask.is_empty() {
            return false;
        }

        self.board[cell] = mask;
        true
    }

    pub fn is_exclusive(&self, cell1: CellIndex, cell2: CellIndex) -> bool {
        self.data.is_exclusive(cell1, cell2)
    }

    pub fn is_grouped(&self, cells: &[CellIndex]) -> bool {
        for (i0, i1) in cells.iter().tuple_combinations() {
            if !self.is_exclusive(*i0, *i1) {
                return false;
            }
        }
        true
    }

    pub fn is_grouped_for_value(&self, cells: &[CellIndex], value: usize) -> bool {
        for (cell0, cell1) in cells.iter().tuple_combinations() {
            let candidate0 = CandidateIndex::from_cv(*cell0, value);
            let candidate1 = CandidateIndex::from_cv(*cell1, value);
            if !self.data.has_weak_link(candidate0, candidate1) {
                return false;
            }
        }
        true
    }

    pub fn is_grouped_for_values(&self, cells: &[CellIndex], values: ValueMask) -> bool {
        for (cell0, cell1) in cells.iter().tuple_combinations() {
            for value in values.into_iter() {
                let candidate0 = CandidateIndex::from_cv(*cell0, value);
                let candidate1 = CandidateIndex::from_cv(*cell1, value);
                if !self.data.has_weak_link(candidate0, candidate1) {
                    return false;
                }
            }
        }
        true
    }
}

impl BoardData {
    pub fn new(size: usize, regions: &[usize], constraints: Vec<Arc<dyn Constraint>>) -> BoardData {
        let all_values_mask = ValueMask::from_all_values(size);
        let num_cells = size * size;
        let num_candidates = size * num_cells;
        let houses = Self::create_houses(size, regions, &constraints);
        let houses_by_cell = Self::create_houses_by_cell(size, &houses);
        let weak_links = vec![CandidateLinks::new(size); num_candidates];
        let exclusive_cells = vec![bitvec![0; num_cells]; num_cells];
        let powerful_cells = constraints.iter().flat_map(|c| c.powerful_cells()).unique().collect();

        BoardData {
            size,
            num_cells,
            num_candidates,
            all_values_mask,
            houses,
            houses_by_cell,
            powerful_cells,
            weak_links,
            total_weak_links: 0,
            exclusive_cells,
            constraints,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn num_cells(&self) -> usize {
        self.num_cells
    }

    pub fn num_candidates(&self) -> usize {
        self.num_candidates
    }

    pub fn all_values_mask(&self) -> ValueMask {
        self.all_values_mask
    }

    pub fn houses(&self) -> &[Arc<House>] {
        &self.houses
    }

    pub fn houses_by_cell(&self) -> &[Vec<Arc<House>>] {
        &self.houses_by_cell
    }

    pub fn weak_links(&self) -> &[CandidateLinks] {
        &self.weak_links
    }

    pub fn weak_links_for(&self, candidate: CandidateIndex) -> &CandidateLinks {
        &self.weak_links[candidate.index()]
    }

    pub fn total_weak_links(&self) -> usize {
        self.total_weak_links
    }

    pub fn powerful_cells(&self) -> &[CellIndex] {
        &self.powerful_cells
    }

    pub fn constraints(&self) -> &[Arc<dyn Constraint>] {
        &self.constraints
    }

    fn take_constraint(&mut self, idx: usize) -> Arc<dyn Constraint> {
        self.constraints.remove(idx)
    }

    fn insert_constraint(&mut self, idx: usize, constraint: Arc<dyn Constraint>) {
        self.constraints.insert(idx, constraint);
    }

    pub fn has_weak_link(&self, candidate0: CandidateIndex, candidate1: CandidateIndex) -> bool {
        self.weak_links[candidate0.index()].is_linked(candidate1)
    }

    pub fn is_exclusive(&self, cell1: CellIndex, cell2: CellIndex) -> bool {
        self.exclusive_cells[cell1.index()][cell2.index()]
    }

    fn create_houses(size: usize, regions: &[usize], constraints: &[Arc<dyn Constraint>]) -> Vec<Arc<House>> {
        let cu = CellUtility::new(size);
        let num_cells = size * size;
        let regions = if regions.len() == num_cells { regions.to_vec() } else { default_regions(size) };

        let mut houses: Vec<Arc<House>> = Vec::new();

        // Create a house for each row
        for row in 0..size {
            let name = format!("Row {}", row + 1);
            let mut house = Vec::new();
            for col in 0..size {
                let cell = cu.cell(row, col);
                house.push(cell);
            }
            houses.push(Arc::new(House::new(&name, &house)));
        }

        // Create a house for each column
        for col in 0..size {
            let name = format!("Column {}", col + 1);
            let mut house = Vec::new();
            for row in 0..size {
                let cell = cu.cell(row, col);
                house.push(cell);
            }
            houses.push(Arc::new(House::new(&name, &house)));
        }

        // Create a house for each region
        let mut house_for_region: HashMap<usize, Vec<CellIndex>> = HashMap::new();
        for cell in cu.all_cells() {
            let region = regions[cell.index()];
            let house = house_for_region.entry(region).or_default();
            house.push(cell);
        }

        // Add any regions that are not duplicates of a row/col
        for (region, house) in house_for_region.iter() {
            if house.len() == size {
                let name = format!("Region {}", region + 1);
                let house = House::new(&name, house);
                if !houses.iter().any(|h| h.cells() == house.cells()) {
                    houses.push(Arc::new(house));
                }
            }
        }

        // Add any non-duplicate regions created by constraints
        for constraint in constraints.iter() {
            let constraint_houses = constraint.get_houses(size);
            for house in constraint_houses {
                if !houses.iter().any(|h| h.cells() == house.cells()) {
                    houses.push(Arc::new(house));
                }
            }
        }

        houses
    }

    fn create_houses_by_cell(size: usize, houses: &[Arc<House>]) -> Vec<Vec<Arc<House>>> {
        let num_cells = size * size;
        let mut houses_by_cell = Vec::new();
        for _ in 0..num_cells {
            houses_by_cell.push(Vec::new());
        }
        for house in houses {
            for cell in house.cells().iter() {
                houses_by_cell[cell.index()].push(house.clone());
            }
        }
        houses_by_cell
    }

    fn add_weak_link(&mut self, candidate1: CandidateIndex, candidate2: CandidateIndex) {
        if self.weak_links[candidate1.index()].set(candidate2, true) {
            self.total_weak_links += 1;
        }

        if self.weak_links[candidate2.index()].set(candidate1, true) {
            self.total_weak_links += 1;
        }
    }

    fn init_weak_links(&mut self) -> EliminationList {
        self.init_sudoku_weak_links();
        let elminiation_list = self.init_constraint_weak_links();
        self.init_exclusive_cells();

        elminiation_list
    }

    fn init_sudoku_weak_links(&mut self) {
        let size = self.size;
        let cu = CellUtility::new(size);

        for candidate1 in cu.all_candidates() {
            let (cell1, val1) = candidate1.cell_index_and_value();

            // Add a weak link to every other candidate in the same cell
            for val2 in (val1 + 1)..=size {
                let candidate2 = cu.candidate(cell1, val2);
                self.add_weak_link(candidate1, candidate2);
            }

            // Add a weak link to every other candidate with the same value that shares a house
            for house in self.houses_by_cell[cell1.index()].clone() {
                for (cand0, cand1) in cu.candidate_pairs(house.cells()) {
                    self.add_weak_link(cand0, cand1);
                }
            }
        }
    }

    fn init_constraint_weak_links(&mut self) -> EliminationList {
        let mut elims: EliminationList = EliminationList::new();
        for constraint in self.constraints.clone() {
            let weak_links = constraint.get_weak_links(self.size);
            for (candidate0, candidate1) in weak_links {
                if candidate0 != candidate1 {
                    self.add_weak_link(candidate0, candidate1);
                } else {
                    elims.add(candidate0);
                }
            }
        }
        elims
    }

    fn init_exclusive_cells(&mut self) {
        let cu = CellUtility::new(self.size);
        for (cell1, cell2) in (0..self.num_cells).tuple_combinations() {
            let cell1 = cu.cell_index(cell1);
            let cell2 = cu.cell_index(cell2);
            let mut exclusive = true;
            for val in 1..=self.size {
                let candidate1 = cu.candidate(cell1, val);
                let candidate2 = cu.candidate(cell2, val);
                if !self.weak_links[candidate1.index()].is_linked(candidate2) {
                    exclusive = false;
                    break;
                }
            }
            self.exclusive_cells[cell1.index()].set(cell2.index(), exclusive);
            self.exclusive_cells[cell2.index()].set(cell1.index(), exclusive);
        }
    }
}

impl Default for Board {
    /// Create an empty board of size 9x9 with standard regions (boxes)
    /// and no additional constraints.
    fn default() -> Self {
        Board::new(9, &[], vec![])
    }
}

impl std::hash::Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.board.hash(state);
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
    }
}

impl Eq for Board {}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cell in self.all_cells() {
            let mask = self.cell(cell);
            if mask.is_single() {
                write!(f, "{}", mask.value())?;
            } else {
                write!(f, ".")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_board9() {
        let board = Board::new(9, &[], vec![]);
        assert_eq!(board.size(), 9);
        assert_eq!(board.num_cells(), 81);
        assert_eq!(board.num_candidates(), 729);
        assert_eq!(board.houses().len(), 27);
        assert_eq!(board.total_weak_links(), ((board.size() - 1) * 4 - 4) * board.num_candidates());
    }

    #[test]
    fn test_board16() {
        let board = Board::new(16, &[], vec![]);
        assert_eq!(board.size(), 16);
        assert_eq!(board.num_cells(), 256);
        assert_eq!(board.num_candidates(), 4096);
        assert_eq!(board.houses().len(), 48);
        assert_eq!(board.total_weak_links(), ((board.size() - 1) * 4 - 6) * board.num_candidates());
    }
}
