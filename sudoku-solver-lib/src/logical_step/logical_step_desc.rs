//! Contains the [`LogicalStepDesc`] struct for repesenting the human-readable
//! descriptions of a logical step performed and their results.

use crate::prelude::*;

/// Represents the human-readable description of a single logical step performed and
/// its results.
///
/// This is used to generate the human-readable output of the solver.
///
/// Logical steps can have sub-steps. For example, if a complicated contradiction
/// is found by testing a value and then performing additional logic to determine
/// that the value is incorrect, the sub-steps will contain the additional logic
/// performed. However, most logical steps do no have sub-steps.
#[derive(Debug, Clone)]
pub struct LogicalStepDesc {
    step: String,
    sub_steps: LogicalStepDescList,
    depth: usize,
}

impl LogicalStepDesc {
    /// Creates a new instance.
    pub fn new(step: &str, sub_steps: &LogicalStepDescList) -> Self {
        Self { step: step.to_owned(), sub_steps: sub_steps.with_depth(1), depth: 0 }
    }

    /// Creates a new instance from a description string an no sub-steps.
    pub fn from_desc(desc: &str) -> Self {
        Self { step: desc.to_owned(), sub_steps: LogicalStepDescList::new(), depth: 0 }
    }

    /// Creates a new instance from a description and a list of eliminations.
    pub fn from_elims(desc: &str, elimination_list: &EliminationList) -> Self {
        let step = format!("{} => {}", desc, elimination_list);
        Self::from_desc(&step)
    }

    /// Creates a new instance where the description is prefixed with the provided
    /// string.
    pub fn with_prefix(&self, prefix: &str) -> Self {
        let step = format!("{}{}", prefix, self.step);
        Self { step, sub_steps: self.sub_steps.clone(), depth: self.depth }
    }

    pub(crate) fn with_depth(&self, depth: usize) -> LogicalStepDesc {
        LogicalStepDesc { step: self.step.clone(), sub_steps: self.sub_steps.with_depth(depth + 1), depth }
    }

    fn indent_str(&self) -> String {
        let mut indent = String::new();
        if self.depth > 0 {
            indent.reserve(self.depth * 4);
            for _ in 0..self.depth - 1 {
                indent.push_str("    ");
            }
            indent.push_str("  | ");
        }
        indent
    }
}

impl From<&str> for LogicalStepDesc {
    fn from(step: &str) -> Self {
        Self { step: step.to_owned(), sub_steps: LogicalStepDescList::new(), depth: 0 }
    }
}

impl From<String> for LogicalStepDesc {
    fn from(step: String) -> Self {
        Self { step, sub_steps: LogicalStepDescList::new(), depth: 0 }
    }
}

impl std::fmt::Display for LogicalStepDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let indent = self.indent_str();
        if self.sub_steps.is_empty() {
            write!(f, "{}{}", indent, self.step)
        } else {
            writeln!(f, "{}{}", indent, self.step)?;
            write!(f, "{}", self.sub_steps)
        }
    }
}
