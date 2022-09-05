use crate::prelude::*;

/// This logical step finds naked singles until none are found or the board is invalid.
///
/// Generally, this is used during brute force only, when there is no
/// need for user-facing descriptions.
#[derive(Debug)]
pub struct AllNakedSingles;

impl LogicalStep for AllNakedSingles {
    fn name(&self) -> &'static str {
        "All Naked Singles"
    }

    fn is_active_during_brute_force_solves(&self) -> bool {
        true
    }

    fn is_active_during_logical_solves(&self) -> bool {
        false
    }

    fn run(&self, board: &mut Board, generate_description: bool) -> LogicalStepResult {
        assert!(
            !generate_description,
            "AllNakedSingles should not be used during logical solves"
        );

        let mut result = LogicalStepResult::None;
        loop {
            if board.is_solved() {
                break;
            }

            let mut changed = false;
            for cell in board.all_cells() {
                let mask = board.cell(cell);
                if mask.is_solved() {
                    continue;
                }

                if mask.is_single() {
                    let value = mask.value();
                    if board.set_solved(cell, value) {
                        changed = true;
                    } else {
                        return LogicalStepResult::Invalid(None);
                    }
                } else if mask.is_empty() {
                    return LogicalStepResult::Invalid(None);
                }
            }
            if !changed {
                break;
            } else {
                result = LogicalStepResult::Changed(None);
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_all_naked_singles() {
        let mut board = Board::default();
        let cu = board.cell_utility();
        let all_naked_singles = AllNakedSingles;

        // There should be no naked singles on the initial board
        assert!(all_naked_singles.run(&mut board, false).is_none());

        // Set up the board so that the entire thing solves with just naked singles
        let board_str =
            "5.6....29.9....13..4...376.........232.5......5..186.32..64.38..1.37529....821.7.";
        board_str.chars().enumerate().for_each(|(i, c)| {
            if let Some(value) = c.to_digit(10) {
                assert!(board.set_solved(cu.cell_index(i), value as usize));
            }
        });

        // The board should fully solve with naked singles
        assert!(all_naked_singles.run(&mut board, false).is_changed());
        assert!(board.is_solved());
        assert_eq!(
            board.to_string(),
            "536187429897462135142953768681734952324596817759218643275649381418375296963821574"
        );
    }
}
