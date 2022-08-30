use crate::{board_utility::*, constraint::Constraint, logic_result::LogicResult};
use std::{collections::BTreeSet, sync::Arc};

pub struct Board {
    board: Vec<u32>,
    is_in_set_value: bool,
    data: Arc<BoardData>,
}

#[derive(Clone)]
pub struct BoardData {
    size: usize,
    num_cells: usize,
    num_candidates: usize,
    all_values_mask: u32,
    weak_links: Vec<BTreeSet<usize>>,
    total_weak_links: usize,
    constraints: Vec<Arc<dyn Constraint>>,
}

impl BoardData {
    pub fn new(size: usize, constraints: &[Arc<dyn Constraint>]) -> BoardData {
        let all_values_mask = all_values_mask(size);
        let num_cells = size * size;
        let num_candidates = size * num_cells;
        BoardData {
            size,
            num_cells,
            num_candidates,
            all_values_mask,
            weak_links: vec![BTreeSet::new(); num_cells],
            total_weak_links: 0,
            constraints: constraints.to_vec(),
        }
    }
}

impl Clone for Board {
    fn clone(&self) -> Board {
        Board {
            board: self.board.clone(),
            is_in_set_value: false,
            data: self.data.clone(),
        }
    }
}

impl Board {
    pub fn new(size: usize, constraints: &[Arc<dyn Constraint>]) -> Board {
        let immutable = Arc::new(BoardData::new(size, constraints));
        Board {
            board: vec![immutable.all_values_mask; immutable.num_cells],
            is_in_set_value: false,
            data: immutable,
        }
    }

    pub fn deep_clone(&self) -> Board {
        Board {
            board: self.board.clone(),
            is_in_set_value: false,
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

    pub fn get_cell_mask(&self, cell: usize) -> u32 {
        self.board[cell]
    }

    pub fn cell_has_value(&self, cell: usize, val: usize) -> bool {
        self.get_cell_mask(cell) & value_mask(val) != 0
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
        if self.board[cell] & val_mask != 0 {
            return false;
        }

        if self.is_in_set_value {
            self.board[cell] = val_mask;
            return true;
        }

        self.is_in_set_value = true;

        self.board[cell] = val_mask | VALUE_SET_MASK;

        // Apply all weak links
        let set_candidate_index = candidate_index(cell, val, self.size());
        let immutable = self.data.clone();
        for &elim_candidate_index in immutable.weak_links[set_candidate_index].iter() {
            let (elim_candidate_cell, elim_candidate_val) =
                candidate_index_to_cell_and_value(elim_candidate_index, self.size());
            if !self.clear_value(elim_candidate_cell, elim_candidate_val) {
                return false;
            }
        }

        // Enforce all constraints
        for constraint in immutable.constraints.iter() {
            if constraint.enforce(self, cell, val) == LogicResult::Invalid {
                return false;
            }
        }

        self.is_in_set_value = false;
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
}
