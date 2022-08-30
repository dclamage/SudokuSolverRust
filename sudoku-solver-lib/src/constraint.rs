use crate::board::Board;
use crate::board_utility::*;
use crate::logic_result::LogicResult;
use crate::logical_steps::LogicalSteps;
use std::vec::Vec;

pub trait Constraint {
    fn name(&self) -> String;

    fn specific_name(&self) -> String {
        self.name().to_string()
    }

    fn enforce(&self, _board: &mut Board, _cell: usize, _val: usize) -> LogicResult {
        LogicResult::None
    }

    fn step_logic(
        &self,
        _board: &mut Board,
        _logical_steps: Option<&mut LogicalSteps>,
        _is_brute_forcing: bool,
    ) -> LogicResult {
        LogicResult::None
    }

    fn cells_must_contain(&self, _board: &Board, _val: usize) -> Vec<usize> {
        Vec::new()
    }

    fn cells_must_contain_by_running_logic(
        &self,
        board: &mut Board,
        cells: &[usize],
        value: usize,
    ) -> Vec<usize> {
        let mut result = Vec::new();

        for &cell in cells {
            let mask = board.get_cell_mask(cell);
            if value_count(mask) <= 1 || !has_value(mask, value) {
                continue;
            }

            result.push(cell);
        }

        if result.len() > 0 {
            let mut board_clone = board.clone();
            for &cell in &result {
                board_clone.clear_value(cell, value);
            }

            let mut logic_result = LogicResult::Changed;
            while logic_result == LogicResult::Changed {
                logic_result = self.step_logic(&mut board_clone, Option::None, false);
            }

            if logic_result != LogicResult::Invalid {
                result.clear();
            }
        }

        result
    }

    fn get_weak_links(&self, _board: &Board) -> Vec<(usize, usize)> {
        Vec::new()
    }

    fn get_weak_links_by_running_logic(
        &self,
        board: &Board,
        cells: &[usize],
    ) -> Vec<(usize, usize)> {
        let size = board.size();
        let mut result = Vec::new();

        for &cell in cells {
            let orig_mask = board.get_cell_mask(cell);
            if value_count(orig_mask) <= 1 {
                continue;
            }

            for val in values_from_mask(orig_mask) {
                let cand0 = candidate_index(cell, val, size);

                let mut board_clone = board.clone();
                if !board_clone.set_value(cell, val) {
                    // A weak link to self indicates that the candidate is generally invalid
                    result.push((cand0, cand0));
                    continue;
                }

                let mut logic_result = LogicResult::Changed;
                while logic_result == LogicResult::Changed {
                    logic_result = self.step_logic(&mut board_clone, Option::None, false);
                }

                if logic_result == LogicResult::Invalid {
                    // A weak link to self indicates that the candidate is generally invalid
                    result.push((cand0, cand0));
                    continue;
                }

                for &cell1 in cells.iter() {
                    if cell == cell1 {
                        continue;
                    }

                    let orig_mask1 = board.get_cell_mask(cell1) & CANDIDATES_MASK;
                    let new_mask1 = board_clone.get_cell_mask(cell1) & CANDIDATES_MASK;
                    if orig_mask1 != new_mask1 {
                        let diff_mask = orig_mask1 & !new_mask1;
                        for val1 in values_from_mask(diff_mask) {
                            let cand1 = candidate_index(cell1, val1, size);
                            result.push((cand0, cand1));
                        }
                    }
                }
            }
        }

        result
    }
}
