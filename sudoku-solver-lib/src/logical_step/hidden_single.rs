use crate::prelude::*;

/// A "Hidden Single" is when a candidate can appear in only one cell within a house.
#[derive(Debug)]
pub struct HiddenSingle;

impl LogicalStep for HiddenSingle {
    fn name(&self) -> &'static str {
        "Hidden Single"
    }

    fn is_active_during_brute_force_solves(&self) -> bool {
        true
    }

    fn run(&self, board: &mut Board, generate_description: bool) -> LogicalStepResult {
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
                let desc: Option<LogicalStepDesc> = if generate_description {
                    Some(format!("{house} has nowhere to place {missing_mask}").into())
                } else {
                    None
                };
                return LogicalStepResult::Invalid(desc);
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
                        let desc: Option<LogicalStepDesc> = if generate_description {
                            Some(format!("In {house}: {cell}={value}").into())
                        } else {
                            None
                        };
                        return LogicalStepResult::Changed(desc);
                    } else {
                        let desc: Option<LogicalStepDesc> = if generate_description {
                            Some(format!("In {house}: {cell} cannot be set to {value}").into())
                        } else {
                            None
                        };
                        return LogicalStepResult::Invalid(desc);
                    }
                }
            }
        }

        LogicalStepResult::None
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

        // There should be no hidden singles on the initial board
        assert!(hidden_single.run(&mut board, true).is_none());

        // Clear 9 from all cells in row 1 except r1c1
        board.clear_candidates((1..=8).map(|col| cu.candidate(cu.cell(0, col), 9)));

        // There should be a hidden single 9 in r1c1
        let result = hidden_single.run(&mut board, true);
        assert!(result.is_changed());
        assert!(result.description().is_some());
        assert_eq!(result.to_string(), "In Row 1: r1c1=9");
    }
}
