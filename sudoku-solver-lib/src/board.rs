use crate::{board_utility::*, constraint::Constraint, house::House, logic_result::LogicResult};
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

#[derive(Clone)]
pub struct Board {
    board: Vec<u32>,
    data: Arc<BoardData>,
}

#[derive(Clone)]
pub struct BoardData {
    size: usize,
    num_cells: usize,
    num_candidates: usize,
    all_values_mask: u32,
    houses: Vec<Arc<House>>,
    houses_by_cell: Vec<Vec<Arc<House>>>,
    weak_links: Vec<BTreeSet<usize>>,
    total_weak_links: usize,
    constraints: Vec<Arc<dyn Constraint>>,
}

impl Board {
    pub fn new(size: usize, regions: &[usize], constraints: &[Arc<dyn Constraint>]) -> Board {
        let mut data = BoardData::new(size, regions, constraints);
        let elims = data.init_weak_links();

        let mut board = Board {
            board: vec![data.all_values_mask; data.num_cells],
            data: Arc::new(data),
        };

        board.clear_candidates(&elims);

        board
    }

    pub fn deep_clone(&self) -> Board {
        Board {
            board: self.board.clone(),
            data: Arc::new(BoardData::clone(&self.data)),
        }
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

    pub fn all_values_mask(&self) -> u32 {
        self.data.all_values_mask
    }

    pub fn houses(&self) -> &[Arc<House>] {
        &self.data.houses
    }

    pub fn houses_for_cell(&self, cell: usize) -> &[Arc<House>] {
        &self.data.houses_by_cell[cell]
    }

    pub fn total_weak_links(&self) -> usize {
        self.data.total_weak_links
    }

    pub fn weak_links(&self) -> &[BTreeSet<usize>] {
        &self.data.weak_links
    }

    pub fn constraints(&self) -> &[Arc<dyn Constraint>] {
        &self.data.constraints
    }

    pub fn get_cell_mask(&self, cell: usize) -> u32 {
        self.board[cell]
    }

    pub fn cell_has_value(&self, cell: usize, val: usize) -> bool {
        self.get_cell_mask(cell) & value_mask(val) != 0
    }

    pub fn has_candidate(&self, candidate: usize) -> bool {
        let (cell, val) = candidate_index_to_cell_and_value(candidate, self.size());
        self.cell_has_value(cell, val)
    }

    pub fn clear_value(&mut self, cell: usize, val: usize) -> bool {
        self.board[cell] &= !value_mask(val);
        (self.board[cell] & CANDIDATES_MASK) != 0
    }

    pub fn set_value(&mut self, cell: usize, val: usize) -> bool {
        let val_mask = value_mask(val);
        if !self.cell_has_value(cell, val) {
            return false;
        }

        // Check if already set
        if self.board[cell] & VALUE_SET_MASK != 0 {
            return false;
        }

        self.board[cell] = val_mask | VALUE_SET_MASK;

        // Clone the BoardData Arc to avoid borrowing issues
        let board_data = self.data.clone();

        // Apply all weak links
        let set_candidate_index = candidate_index(cell, val, self.size());
        for &elim_candidate_index in board_data.weak_links[set_candidate_index].iter() {
            let (elim_candidate_cell, elim_candidate_val) =
                candidate_index_to_cell_and_value(elim_candidate_index, self.size());
            if !self.clear_value(elim_candidate_cell, elim_candidate_val) {
                return false;
            }
        }

        // Enforce all constraints
        for constraint in board_data.constraints.iter() {
            if constraint.enforce(self, cell, val) == LogicResult::Invalid {
                return false;
            }
        }

        true
    }

    pub fn set_mask(&mut self, cell: usize, mask: u32) -> bool {
        if mask & CANDIDATES_MASK == 0 {
            return false;
        }

        self.board[cell] = mask;
        true
    }

    pub fn set_mask_from_values(&mut self, cell: usize, values: &[usize]) -> bool {
        let mask = values_mask(values);
        self.set_mask(cell, mask)
    }

    pub fn clear_candidate(&mut self, candidate: usize) -> bool {
        let (cell, val) = candidate_index_to_cell_and_value(candidate, self.size());
        self.clear_value(cell, val)
    }

    pub fn clear_candidates(&mut self, candidates: &[usize]) -> bool {
        let mut valid = true;
        for candidate in candidates {
            if !self.clear_candidate(*candidate) {
                valid = false;
            }
        }
        valid
    }
}

impl BoardData {
    pub fn new(size: usize, regions: &[usize], constraints: &[Arc<dyn Constraint>]) -> BoardData {
        let all_values_mask = all_values_mask(size);
        let num_cells = size * size;
        let num_candidates = size * num_cells;
        let houses = Self::create_houses(size, regions, constraints);
        let houses_by_cell = Self::create_houses_by_cell(size, &houses);

        BoardData {
            size,
            num_cells,
            num_candidates,
            all_values_mask,
            houses,
            houses_by_cell,
            weak_links: vec![BTreeSet::new(); num_candidates],
            total_weak_links: 0,
            constraints: constraints.to_vec(),
        }
    }

    fn create_houses(
        size: usize,
        regions: &[usize],
        constraints: &[Arc<dyn Constraint>],
    ) -> Vec<Arc<House>> {
        let num_cells = size * size;
        let regions = if regions.len() == num_cells {
            regions.to_vec()
        } else {
            default_regions(size)
        };

        let mut houses: Vec<Arc<House>> = Vec::new();

        // Create a house for each row
        for row in 0..size {
            let name = format!("Row {}", row + 1);
            let mut house = Vec::new();
            for col in 0..size {
                let cell = cell_index(row, col, size);
                house.push(cell);
            }
            houses.push(Arc::new(House::new(&name, &house)));
        }

        // Create a house for each column
        for col in 0..size {
            let name = format!("Column {}", col + 1);
            let mut house = Vec::new();
            for row in 0..size {
                let cell = cell_index(row, col, size);
                house.push(cell);
            }
            houses.push(Arc::new(House::new(&name, &house)));
        }

        // Create a house for each region
        let mut house_for_region: HashMap<usize, Vec<usize>> = HashMap::new();
        for cell in 0..num_cells {
            let region = regions[cell];
            let house = house_for_region.entry(region).or_insert(Vec::new());
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
                houses_by_cell[*cell].push(house.clone());
            }
        }
        houses_by_cell
    }

    fn add_weak_link(&mut self, candidate0: usize, candidate1: usize) {
        if self.weak_links[candidate0].insert(candidate1) {
            self.total_weak_links += 1;
        }
        if self.weak_links[candidate1].insert(candidate0) {
            self.total_weak_links += 1;
        }
    }

    fn init_weak_links(&mut self) -> Vec<usize> {
        self.init_sudoku_weak_links();
        self.init_constraint_weak_links()
    }

    fn init_sudoku_weak_links(&mut self) {
        let size = self.size;

        for cell in 0..self.num_cells {
            for val in 1..=size {
                let candidate_index1 = candidate_index(cell, val, size);

                // Add a weak link to every other candidate in the same cell
                for val2 in (val + 1)..=size {
                    let candidate_index2 = candidate_index(cell, val2, size);
                    self.add_weak_link(candidate_index1, candidate_index2);
                }

                // Add a weak link to every other candidate with the same value that shares a house
                for house in self.houses_by_cell[cell].clone() {
                    for (cand0, cand1) in get_candidate_pairs(size, house.cells()) {
                        self.add_weak_link(cand0, cand1);
                    }
                }
            }
        }
    }

    fn init_constraint_weak_links(&mut self) -> Vec<usize> {
        let mut elims: Vec<usize> = Vec::new();
        for constraint in self.constraints.clone() {
            let weak_links = constraint.get_weak_links(self.size);
            for (candidate0, candidate1) in weak_links {
                if candidate0 != candidate1 {
                    self.add_weak_link(candidate0, candidate1);
                } else {
                    elims.push(candidate0);
                }
            }
        }
        elims
    }
}

impl Default for Board {
    /// Create an empty board of size 9x9 with standard regions (boxes)
    /// and no additional constraints.
    fn default() -> Self {
        Board::new(9, &[], &[])
    }
}
