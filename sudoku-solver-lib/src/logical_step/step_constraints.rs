use crate::prelude::*;

/// Applies constraint-specific logic.
#[derive(Debug)]
pub struct StepConstraints;

impl LogicalStep for StepConstraints {
    fn name(&self) -> &'static str {
        "Step Constraints"
    }

    fn has_own_prefix(&self) -> bool {
        true
    }

    fn is_active_during_brute_force_solves(&self) -> bool {
        true
    }

    fn run(&self, board: &mut Board, generate_description: bool) -> LogicalStepResult {
        let board_data = board.data();
        for constraint in board_data.constraints() {
            let result = constraint.step_logic(board, !generate_description);
            if !result.is_none() {
                return result.with_prefix(format!("{}: ", constraint.name()).as_str());
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
    struct RemoveCandidateConstraint {
        specific_name: String,
        candidate: CandidateIndex,
    }

    impl RemoveCandidateConstraint {
        fn new(candidate: CandidateIndex) -> Self {
            Self { specific_name: format!("Remove {}", candidate), candidate }
        }
    }

    impl Constraint for RemoveCandidateConstraint {
        fn name(&self) -> &str {
            &self.specific_name
        }

        fn step_logic(&self, board: &mut Board, _generate_description: bool) -> LogicalStepResult {
            if board.has_candidate(self.candidate) {
                if !board.clear_candidate(self.candidate) {
                    return LogicalStepResult::Invalid(Some(
                        format!("{} remover failed to remove it.", self.candidate).into(),
                    ));
                }
                LogicalStepResult::Changed(Some(format!("{} removed.", self.candidate).into()))
            } else {
                LogicalStepResult::None
            }
        }
    }

    #[test]
    fn test_step_constraints() {
        let size = 9;
        let cu = CellUtility::new(size);
        let candidate1 = cu.cell(0, 0).candidate(1);
        let candidate2 = cu.cell(0, 1).candidate(1);
        let mut board = Board::new(
            size,
            &[],
            vec![
                Arc::new(RemoveCandidateConstraint::new(candidate1)),
                Arc::new(RemoveCandidateConstraint::new(candidate2)),
            ],
        );
        let step_constraints = StepConstraints;

        // Both candidates should be present
        assert!(board.has_candidate(candidate1));
        assert!(board.has_candidate(candidate2));

        // Stepping the logic should remove just the first candidate
        let result = step_constraints.run(&mut board, true);
        assert!(result.is_changed());
        assert_eq!(result.description().unwrap().to_string(), "Remove 1r1c1: 1r1c1 removed.");
        assert!(!board.has_candidate(candidate1));
        assert!(board.has_candidate(candidate2));

        // Stepping the logic should remove just the second candidate
        let result = step_constraints.run(&mut board, true);
        assert!(result.is_changed());
        assert_eq!(result.description().unwrap().to_string(), "Remove 1r1c2: 1r1c2 removed.");
        assert!(!board.has_candidate(candidate1));
        assert!(!board.has_candidate(candidate2));

        // Stepping the logic should now do nothing
        let result = step_constraints.run(&mut board, true);
        assert!(result.is_none());

        // Create a new board with the same constraints
        let mut board = Board::new(
            size,
            &[],
            vec![
                Arc::new(RemoveCandidateConstraint::new(candidate1)),
                Arc::new(RemoveCandidateConstraint::new(candidate2)),
            ],
        );

        // Clear out all but 1 from r1c1
        let cell = cu.cell(0, 0);
        assert!(board.clear_candidates((2..=size).map(|value| cu.candidate(cell, value))));

        // Stepping the logic should try to remove 1r1c1 and discover this makes the board invalid
        let result = step_constraints.run(&mut board, true);
        assert!(result.is_invalid());
        assert_eq!(result.description().unwrap().to_string(), "Remove 1r1c1: 1r1c1 remover failed to remove it.");
    }
}
