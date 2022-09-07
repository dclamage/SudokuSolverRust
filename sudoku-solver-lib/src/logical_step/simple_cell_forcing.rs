use crate::prelude::*;

/// "Simple Cell Forcing" &is when all candidates remaining in a cell all have a weak link
/// to the same candidate in another cell. This other candidate can be eliminated.
#[derive(Debug)]
pub struct SimpleCellForcing;

impl LogicalStep for SimpleCellForcing {
    fn name(&self) -> &'static str {
        "Simple Cell Forcing"
    }

    fn run(&self, board: &mut Board, generate_description: bool) -> LogicalStepResult {
        let size = board.size();
        let cu = board.cell_utility();
        let bd = board.data();

        for cell in board.all_cells() {
            let mask = board.cell(cell);
            if mask.is_solved() {
                continue;
            }

            let mut elim_set = CandidateLinks::new(size);
            let mut is_first = true;
            for value in mask {
                let candidate = cu.candidate(cell, value);
                if is_first {
                    elim_set.union(bd.weak_links_for(candidate));
                    is_first = false;
                } else {
                    elim_set.intersect(bd.weak_links_for(candidate));
                }
            }

            if elim_set.is_empty() {
                continue;
            }

            let mut elims = EliminationList::new();
            for candidate in elim_set.links() {
                if board.has_candidate(candidate) {
                    elims.add(candidate);
                }
            }

            if !elims.is_empty() {
                let desc = if generate_description {
                    let desc = LogicalStepDesc::from_elims(&cell.to_string(), &elims);
                    Some(desc)
                } else {
                    None
                };

                if !board.clear_candidates(elims.iter()) {
                    return LogicalStepResult::Invalid(desc);
                }
                return LogicalStepResult::Changed(desc);
            }
        }

        LogicalStepResult::None
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;

    #[derive(Debug)]
    struct ExtraWeakLinksConstraint;

    impl Constraint for ExtraWeakLinksConstraint {
        fn name(&self) -> &str {
            "Test Extra Weak Links"
        }

        fn get_weak_links(&self, size: usize) -> Vec<(CandidateIndex, CandidateIndex)> {
            let cu = CellUtility::new(size);
            let cell_r1c1 = cu.cell(0, 0);
            let cell_r1c2 = cu.cell(0, 1);
            let candidate_1r1c2 = cu.candidate(cell_r1c2, 1);
            (2..=8)
                .map(|v| (cu.candidate(cell_r1c1, v), candidate_1r1c2))
                .collect()
        }
    }

    #[test]
    fn test_cell_forcing() {
        let mut board = Board::new(9, &[], &[Arc::new(ExtraWeakLinksConstraint)]);
        let cu = board.cell_utility();
        let simple_cell_forcing = SimpleCellForcing;

        // No cell forcing should be possible here
        let result = simple_cell_forcing.run(&mut board, true);
        assert!(result.is_none());

        // Remove 9 as a candidate from r1c1
        assert!(board.clear_candidate(cu.candidate(cu.cell(0, 0), 9)));

        // Cell forcing should be possible here
        let result = simple_cell_forcing.run(&mut board, true);
        assert!(result.is_changed());

        // Check that 1 has been eliminated from r1c2
        assert!(!board.cell(cu.cell(0, 1)).has(1));

        // Check that the description is correct
        let desc = result.to_string();
        assert_eq!(desc.to_string(), "r1c1 => -1r1c2");
    }
}
