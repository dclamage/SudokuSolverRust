//! Contains [`LogicalSolveResult`] for storing the result of running multiple logical steps.

use crate::prelude::*;

/// The result of running multiple logical steps.
#[derive(Debug, Clone)]
pub enum LogicalSolveResult {
    /// The logical steps did not perform any changes.
    None,
    /// The logical steps changed the board, but the board is unsolved.
    Changed(LogicalStepDescList),
    /// The logical steps solved the board.
    Solved(LogicalStepDescList),
    /// The logical steps found that the board is invalid.
    Invalid(LogicalStepDescList),
}

impl LogicalSolveResult {
    pub fn is_none(&self) -> bool {
        matches!(self, LogicalSolveResult::None)
    }

    pub fn is_changed(&self) -> bool {
        matches!(self, LogicalSolveResult::Changed(_))
    }

    pub fn is_solved(&self) -> bool {
        matches!(self, LogicalSolveResult::Solved(_))
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, LogicalSolveResult::Invalid(_))
    }

    pub fn description(&self) -> Option<&LogicalStepDescList> {
        match self {
            LogicalSolveResult::None => None,
            LogicalSolveResult::Changed(desc) => Some(desc),
            LogicalSolveResult::Solved(desc) => Some(desc),
            LogicalSolveResult::Invalid(desc) => Some(desc),
        }
    }
}

impl std::fmt::Display for LogicalSolveResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = self.description();
        if let Some(desc) = desc {
            write!(f, "{}", desc)
        } else {
            write!(f, "No Description")
        }
    }
}
