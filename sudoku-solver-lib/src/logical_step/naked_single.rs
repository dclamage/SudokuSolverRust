use crate::prelude::*;
use macros::*;

/// A "Naked Single" is when a cell has been reduced to a single candidate.
///
/// This is the simplest logical step and is required for the solver to function.
pub struct NakedSingle;

impl LogicalStep for NakedSingle {
    fn name(&self) -> &'static str {
        "Naked Single"
    }

    fn step(&self, board: &mut Board, mut desc: Option<&mut LogicalStepDescList>) -> LogicResult {
        for cell in board.all_cells() {
            let mask = board.cell(cell);
            if mask.is_solved() {
                continue;
            }

            if mask.is_single() {
                let value = mask.value();
                if board.set_solved(cell, value) {
                    add_step!(self, desc, format!("{}={}", cell, value));
                    return LogicResult::Changed;
                } else {
                    add_step!(self, desc, format!("{} cannot be set to {}", cell, value));
                    return LogicResult::Invalid;
                }
            } else if mask.is_empty() {
                add_step!(self, desc, format!("{} has no candidates", cell));
                return LogicResult::Invalid;
            }
        }

        LogicResult::None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_naked_single() {
        let mut board = Board::default();
        let cu = board.cell_utility();
        let naked_single = NakedSingle;
        let mut desc = LogicalStepDescList::new();

        // There should be no naked singles on the initial board
        assert!(naked_single.step(&mut board, None) == LogicResult::None);

        // Clear all candidates except 9 from r1c1
        let cell = cu.cell(0, 0);
        board.clear_candidates((1..=8).map(|v| cu.candidate(cell, v)));

        // There should be a naked single in r1c1
        assert!(naked_single.step(&mut board, Some(&mut desc)) == LogicResult::Changed);
        assert_eq!(desc.to_string(), "Naked Single: r1c1=9");
    }
}
