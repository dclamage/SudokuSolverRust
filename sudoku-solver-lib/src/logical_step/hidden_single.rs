use crate::prelude::*;
use macros::*;

/// A "Hidden Single" is when a candidate can appear in only one cell within a house.
pub struct HiddenSingle;

impl LogicalStep for HiddenSingle {
    fn name(&self) -> &'static str {
        "Hidden Single"
    }

    fn is_active_during_brute_force_solves(&self) -> bool {
        true
    }

    fn step(&self, board: &mut Board, mut desc: Option<&mut LogicalStepDescList>) -> LogicResult {
        let board_data = board.data();
        let all_values = board_data.all_values_mask();

        for house in board_data.houses() {
            let mut at_least_once = ValueMask::new();
            let mut more_than_once = ValueMask::new();
            let mut set_mask = ValueMask::new();
            for cell in house.cells() {
                let mask = board.cell(*cell);
                if mask.is_solved() {
                    set_mask = set_mask | mask;
                } else {
                    more_than_once = more_than_once | (at_least_once & mask);
                    at_least_once = at_least_once | mask;
                }
            }
            set_mask = set_mask.unsolved();

            let all_values_seen = at_least_once | set_mask;
            if all_values_seen != all_values {
                let missing_mask: ValueMask = all_values & !all_values_seen;
                add_step!(
                    self,
                    desc,
                    format!("{} has nowhere to place {}", house, missing_mask)
                );
                return LogicResult::Invalid;
            }

            let exactly_once = at_least_once & !more_than_once;
            if exactly_once.is_empty() {
                continue;
            }

            let value = exactly_once.min();
            for &cell in house.cells() {
                let cell_mask = board.cell(cell);
                if cell_mask.has(value) {
                    if board.set_solved(cell, value) {
                        add_step!(self, desc, format!("In {}: {}={}", house, cell, value));
                        return LogicResult::Changed;
                    } else {
                        add_step!(
                            self,
                            desc,
                            format!("In {}: {} cannot be set to {}", house, cell, value)
                        );
                        return LogicResult::Invalid;
                    }
                }
            }
        }

        LogicResult::None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hidden_single() {
        let mut board = Board::default();
        let cu = board.cell_utility();
        let hidden_single = HiddenSingle;
        let mut desc = LogicalStepDescList::new();

        // There should be no hidden singles on the initial board
        assert!(hidden_single.step(&mut board, None) == LogicResult::None);

        // Clear 9 from all cells in row 1 except r1c1
        board.clear_candidates((1..=8).map(|col| cu.candidate(cu.cell(0, col), 9)));

        // There should be a hidden single 9 in r1c1
        assert!(hidden_single.step(&mut board, Some(&mut desc)) == LogicResult::Changed);
        assert_eq!(desc.to_string(), "Hidden Single: In Row 1: r1c1=9");
    }
}
