use crate::prelude::*;

/// A "Naked Single" is when a cell has been reduced to a single candidate.
///
/// This is the simplest logical step and is required for the solver to function.
#[derive(Debug)]
pub struct NakedSingle;

impl LogicalStep for NakedSingle {
    fn name(&self) -> &'static str {
        "Naked Single"
    }

    fn run(&self, board: &mut Board, generate_description: bool) -> LogicalStepResult {
        for cell in board.all_cells() {
            let mask = board.cell(cell);
            if mask.is_solved() {
                continue;
            }

            if mask.is_single() {
                let value = mask.value();
                if board.set_solved(cell, value) {
                    let desc = if generate_description {
                        Some(format!("{}={}", cell, value).into())
                    } else {
                        None
                    };
                    return LogicalStepResult::Changed(desc);
                } else {
                    let desc = if generate_description {
                        Some(format!("{} cannot be set to {}", cell, value).into())
                    } else {
                        None
                    };
                    return LogicalStepResult::Invalid(desc);
                }
            } else if mask.is_empty() {
                let desc = if generate_description {
                    Some(format!("{} has no candidates", cell).into())
                } else {
                    None
                };
                return LogicalStepResult::Invalid(desc);
            }
        }

        LogicalStepResult::None
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

        // There should be no naked singles on the initial board
        assert!(naked_single.run(&mut board, true).is_none());

        // Clear all candidates except 9 from r1c1
        let cell = cu.cell(0, 0);
        board.clear_candidates((1..=8).map(|v| cu.candidate(cell, v)));

        // There should be a naked single in r1c1
        let result = naked_single.run(&mut board, true);
        assert!(result.is_changed());
        assert_eq!(result.to_string(), "r1c1=9");
    }
}
