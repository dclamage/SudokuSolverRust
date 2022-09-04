use std::collections::BTreeSet;

use crate::prelude::*;

use super::macros::add_step_with_elims;

/// "Simple Cell Forcing" &is when all candidates remaining in a cell all have a weak link
/// to the same candidate in another cell. This other candidate can be eliminated.
pub struct SimpleCellForcing;

impl LogicalStep for SimpleCellForcing {
    fn name(&self) -> &'static str {
        "Simple Cell Forcing"
    }

    fn step(&self, board: &mut Board, mut desc: Option<&mut LogicalStepDescList>) -> LogicResult {
        let cu = board.cell_utility();
        let bd = board.data();

        for cell in board.all_cells() {
            let mask = board.cell(cell);
            if mask.is_solved() {
                continue;
            }

            let mut elim_set: BTreeSet<CandidateIndex> = BTreeSet::new();
            let mut is_first = true;
            for value in mask {
                let candidate = cu.candidate(cell, value);
                if is_first {
                    elim_set = bd.weak_links_for(candidate).clone();
                    is_first = false;
                } else {
                    elim_set = &elim_set & bd.weak_links_for(candidate);
                }

                if elim_set.is_empty() {
                    break;
                }
            }

            elim_set.retain(|&c| board.has_candidate(c));

            if !elim_set.is_empty() {
                let elim_set: EliminationList = elim_set.into();
                if !board.clear_candidates(elim_set.iter()) {
                    add_step_with_elims!(self, desc, format!("{}", cell), &elim_set);
                    return LogicResult::Invalid;
                }
                add_step_with_elims!(self, desc, format!("{}", cell), &elim_set);
                return LogicResult::Changed;
            }
        }

        LogicResult::None
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;

    struct ExtraWeakLinksConstraint;

    impl Constraint for ExtraWeakLinksConstraint {
        fn name(&self) -> String {
            "Test Extra Weak Links".to_owned()
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
        let mut desc = LogicalStepDescList::new();

        // No cell forcing should be possible here
        let result = simple_cell_forcing.step(&mut board, Some(&mut desc));
        assert!(result == LogicResult::None);

        // Remove 9 as a candidate from r1c1
        assert!(board.clear_candidate(cu.candidate(cu.cell(0, 0), 9)));

        // Cell forcing should be possible here
        let result = simple_cell_forcing.step(&mut board, Some(&mut desc));
        assert_eq!(result, LogicResult::Changed);

        // Check that 1 has been eliminated from r1c2
        assert!(!board.cell(cu.cell(0, 1)).has(1));

        // Check that the description is correct
        assert!(desc.len() == 1);
        assert_eq!(desc.to_string(), "Simple Cell Forcing: r1c1 => -1r1c2");
    }
}
