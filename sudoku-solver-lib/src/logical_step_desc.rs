use std::fmt::Display;

use crate::elimination_list::EliminationList;

#[derive(Debug, Clone)]
struct LogicalStepDesc {
    step: String,
    sub_steps: LogicalStepDescList,
    depth: usize,
}

impl LogicalStepDesc {
    pub fn new(step: &str, sub_steps: &LogicalStepDescList) -> LogicalStepDesc {
        LogicalStepDesc {
            step: step.to_owned(),
            sub_steps: sub_steps.with_depth(1),
            depth: 0,
        }
    }

    fn with_depth(&self, depth: usize) -> LogicalStepDesc {
        LogicalStepDesc {
            step: self.step.clone(),
            sub_steps: self.sub_steps.with_depth(depth + 1),
            depth,
        }
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

impl Display for LogicalStepDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let indent = self.indent_str();
        if self.sub_steps.len() == 0 {
            write!(f, "{}{}", indent, self.step)
        } else {
            writeln!(f, "{}{}", indent, self.step)?;
            write!(f, "{}", self.sub_steps)
        }
    }
}

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

    /// Add a logical step to the list with no sub-steps.
    ///
    /// # Arguments
    /// - `step` - The logical step to add.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::logical_step_desc::LogicalStepDescList;
    /// let mut list = LogicalStepDescList::new();
    /// list.add("step 1");
    ///
    /// assert_eq!(list.len(), 1);
    /// assert_eq!(list.to_string(), "step 1");
    /// ```
    pub fn add_step(&mut self, step: &str) {
        self.steps
            .push(LogicalStepDesc::new(step, &LogicalStepDescList::new()));
    }

    /// Add a step to this list of logical step descriptions with sub-steps.
    ///
    /// # Arguments
    /// * `step` - The step to add.
    /// * `sub_steps` - The sub-steps to add.
    ///
    /// # Examples
    /// ```
    /// # use sudoku_solver_lib::logical_step_desc::LogicalStepDescList;
    ///
    /// // Create a new logical step description list.
    /// let mut list = LogicalStepDescList::new();
    ///
    /// // Add a step with no sub-steps.
    /// list.add_step("step 1");
    ///
    /// // Add a step with sub-steps.
    /// let mut sub_list = LogicalStepDescList::new();
    /// sub_list.add_step("sub step a");
    /// sub_list.add_step("sub step b");
    ///
    /// list.add_step_with_substeps("step 2", &sub_list);
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
    /// outer_list.add_step("outer step I");
    /// outer_list.add_step_with_substeps("outer step II", &list);
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
    /// ```
    pub fn add_step_with_substeps(&mut self, step: &str, sub_steps: &LogicalStepDescList) {
        self.steps.push(LogicalStepDesc::new(step, sub_steps));
    }

    /// Add a step based on a simple description plus a list of candidate eliminations
    /// to this list of logical step descriptions.
    ///
    /// # Arguments
    /// * `desc` - The description of the step.
    /// * `eliminations` - The list of candidate eliminations.
    ///
    /// # Examples
    /// ```
    /// # use sudoku_solver_lib::logical_step_desc::LogicalStepDescList;
    /// # use sudoku_solver_lib::elimination_list::EliminationList;
    /// // Assume 9x9 board.
    /// let mut size = 9;
    ///
    /// // Create an elimination list which elimiates 1r1c1 and 1r1c2.
    /// let mut elims = EliminationList::new(size);
    /// elims.add_row_col_value(0, 0, 1);
    /// elims.add_row_col_value(0, 1, 1);
    ///
    /// // Create a logical step description list.
    /// let mut list = LogicalStepDescList::new();
    ///
    /// // Add a step which describes our cool logic
    /// list.add_step_with_elims("Found some cool logic", &elims);
    ///
    /// // The list can now be converted to string
    /// assert_eq!(list.to_string(), "Found some cool logic => -1r1c12");
    ///```
    pub fn add_step_with_elims(&mut self, desc: &str, elimination_list: &EliminationList) {
        let step = format!("{} => {}", desc, elimination_list);
        self.steps
            .push(LogicalStepDesc::new(&step, &LogicalStepDescList::new()));
    }

    fn with_depth(&self, depth: usize) -> LogicalStepDescList {
        let mut steps = Vec::new();
        for step in self.steps.iter() {
            steps.push(step.with_depth(depth));
        }
        LogicalStepDescList { steps }
    }
}

impl Display for LogicalStepDescList {
    /// Display a logical step description list.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut full_desc = String::new();
        for step in &self.steps {
            full_desc.push_str(&format!("{}\n", step));
        }
        write!(f, "{}", full_desc.trim_end())
    }
}
