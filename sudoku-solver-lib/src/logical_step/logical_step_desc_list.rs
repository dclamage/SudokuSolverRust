//! Contains the [`LogicalStepDescList`] for repesenting a list of [`LogicalStepDesc`]s.

use crate::prelude::*;

/// Container for a list of [`LogicalStepDesc`] instances and utility methods
/// for displaying them.
#[derive(Debug, Clone)]
pub struct LogicalStepDescList {
    steps: Vec<LogicalStepDesc>,
}

impl LogicalStepDescList {
    /// Create a new logical step description list.
    pub fn new() -> LogicalStepDescList {
        LogicalStepDescList { steps: Vec::new() }
    }

    /// Gets the number of logical steps in the list.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Checks if there are no logical steps in the list.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get the steps as a slice.
    pub fn steps(&self) -> &[LogicalStepDesc] {
        &self.steps
    }

    /// Clones and appends all elements in another LogicalStepDescList.
    pub fn extend_from_other(&mut self, other: &LogicalStepDescList) {
        self.steps.extend_from_slice(&other.steps);
    }

    /// Clones and appends all elements in a slice.
    pub fn extend_from_slice(&mut self, other: &[LogicalStepDesc]) {
        self.steps.extend_from_slice(other);
    }

    /// Add a logical step to the list.
    ///
    /// # Arguments
    /// - `step` - The logical step to add.
    ///
    /// # Examples
    /// ```
    /// # use sudoku_solver_lib::prelude::*;
    /// let mut list = LogicalStepDescList::new();
    /// list.push("step 1".into());
    ///
    /// assert_eq!(list.len(), 1);
    /// assert_eq!(list.to_string(), "step 1");
    /// ```
    ///
    /// ```
    /// # use sudoku_solver_lib::prelude::*;
    ///
    /// // Create a new logical step description list.
    /// let mut list = LogicalStepDescList::new();
    ///
    /// // Add a step with no sub-steps.
    /// list.push("step 1".into());
    ///
    /// // Add a step with sub-steps.
    /// let mut sub_list = LogicalStepDescList::new();
    /// sub_list.push("sub step a".into());
    /// sub_list.push("sub step b".into());
    ///
    /// list.push(LogicalStepDesc::new("step 2", &sub_list));
    ///
    ///
    /// // Print the list
    /// println!("{}", list);
    ///
    /// // Output:
    /// // step 1
    /// // step 2
    /// //   | sub step a
    /// //   | sub step b
    ///
    /// # assert_eq!(list.to_string(), "step 1\nstep 2\n  | sub step a\n  | sub step b");
    ///
    /// // Create an outer list to contain this list.
    /// let mut outer_list = LogicalStepDescList::new();
    /// outer_list.push("outer step I".into());
    /// outer_list.push(LogicalStepDesc::new("outer step II", &list));
    ///
    /// // Print the outer list.
    /// println!("{}", outer_list);
    ///
    /// // Output:
    /// // outer step I
    /// // outer step II
    /// //   | step 1
    /// //   | step 2
    /// //       | sub step a
    /// //       | sub step b
    ///
    /// # assert_eq!(outer_list.to_string(), "outer step I\nouter step II\n  | step 1\n  | step 2\n      | sub step a\n      | sub step b");
    /// ```
    ///
    /// ```
    /// # use sudoku_solver_lib::prelude::*;
    /// // Assume 9x9 board.
    /// let cu = CellUtility::new(9);
    ///
    /// // Create an elimination list which elimiates 1r1c1 and 1r1c2.
    /// let mut elims = EliminationList::new();
    /// elims.add(cu.cell(0, 0).candidate(1));
    /// elims.add(cu.cell(0, 1).candidate(1));
    ///
    /// // Create a logical step description list.
    /// let mut list = LogicalStepDescList::new();
    ///
    /// // Add a step which describes our cool logic
    /// list.push(LogicalStepDesc::from_elims("Found some cool logic", &elims));
    ///
    /// // The list can now be converted to string
    /// assert_eq!(list.to_string(), "Found some cool logic => -1r1c12");
    ///```
    pub fn push(&mut self, step: LogicalStepDesc) {
        self.steps.push(step);
    }

    pub(crate) fn with_depth(&self, depth: usize) -> LogicalStepDescList {
        let mut steps = Vec::new();
        for step in self.steps.iter() {
            steps.push(step.with_depth(depth));
        }
        LogicalStepDescList { steps }
    }
}

impl Default for LogicalStepDescList {
    fn default() -> Self {
        LogicalStepDescList::new()
    }
}

impl IntoIterator for LogicalStepDescList {
    type Item = LogicalStepDesc;
    type IntoIter = std::vec::IntoIter<LogicalStepDesc>;

    fn into_iter(self) -> Self::IntoIter {
        self.steps.into_iter()
    }
}

impl std::ops::Deref for LogicalStepDescList {
    type Target = Vec<LogicalStepDesc>;

    fn deref(&self) -> &Self::Target {
        &self.steps
    }
}

impl Extend<LogicalStepDesc> for LogicalStepDescList {
    fn extend<T: IntoIterator<Item = LogicalStepDesc>>(&mut self, iter: T) {
        self.steps.extend(iter);
    }
}

impl std::fmt::Display for LogicalStepDescList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for step in self.steps.iter().take(self.steps.len() - 1) {
            writeln!(f, "{}", step)?;
        }
        if let Some(last_step) = self.steps.last() {
            write!(f, "{}", last_step)?;
        }
        Ok(())
    }
}
