//! Contains the [SolutionCountResult] enum.

/// The result of running a solve that returns the number of solutions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SolutionCountResult {
    None,
    ExactCount(usize),
    AtLeastCount(usize),
    Error(String),
}

impl SolutionCountResult {
    pub fn is_none(&self) -> bool {
        matches!(self, SolutionCountResult::None)
    }

    pub fn is_exact_count(&self) -> bool {
        matches!(self, SolutionCountResult::ExactCount(_))
    }

    pub fn is_at_least_count(&self) -> bool {
        matches!(self, SolutionCountResult::AtLeastCount(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, SolutionCountResult::Error(_))
    }

    pub fn has_count(&self) -> bool {
        self.is_exact_count() || self.is_at_least_count()
    }

    pub fn count(&self) -> Option<usize> {
        match self {
            SolutionCountResult::None => None,
            SolutionCountResult::ExactCount(count) => Some(*count),
            SolutionCountResult::AtLeastCount(count) => Some(*count),
            SolutionCountResult::Error(_) => None,
        }
    }

    pub fn error(&self) -> Option<String> {
        match self {
            SolutionCountResult::None => None,
            SolutionCountResult::ExactCount(_) => None,
            SolutionCountResult::AtLeastCount(_) => None,
            SolutionCountResult::Error(err) => Some(err.clone()),
        }
    }
}
