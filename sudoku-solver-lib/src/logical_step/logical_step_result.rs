use crate::prelude::*;

/// Represents the result of a logical step and can contain a
/// description of the step and its of eliminations.
#[derive(Clone, Debug)]
pub enum LogicalStepResult {
    /// The logical step did not perform any changes.
    None,
    /// The logical step changed the board.
    Changed(Option<LogicalStepDesc>),
    /// The logical step found that the board is invalid.
    Invalid(Option<LogicalStepDesc>),
}

impl LogicalStepResult {
    pub fn is_none(&self) -> bool {
        matches!(self, LogicalStepResult::None)
    }

    pub fn is_changed(&self) -> bool {
        matches!(self, LogicalStepResult::Changed(_))
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, LogicalStepResult::Invalid(_))
    }

    pub fn description(&self) -> Option<&LogicalStepDesc> {
        match self {
            LogicalStepResult::None => None,
            LogicalStepResult::Changed(desc) => desc.as_ref(),
            LogicalStepResult::Invalid(desc) => desc.as_ref(),
        }
    }

    pub fn with_prefix(&self, prefix: &str) -> Self {
        match self {
            LogicalStepResult::None => LogicalStepResult::None,
            LogicalStepResult::Changed(desc) => {
                if let Some(desc) = desc {
                    LogicalStepResult::Changed(Some(desc.with_prefix(prefix)))
                } else {
                    LogicalStepResult::Changed(None)
                }
            }
            LogicalStepResult::Invalid(desc) => {
                if let Some(desc) = desc {
                    LogicalStepResult::Invalid(Some(desc.with_prefix(prefix)))
                } else {
                    LogicalStepResult::Invalid(None)
                }
            }
        }
    }
}

impl std::fmt::Display for LogicalStepResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = self.description();
        if let Some(desc) = desc {
            write!(f, "{desc}")
        } else {
            write!(f, "No Description")
        }
    }
}
