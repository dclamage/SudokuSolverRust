#[derive(Clone)]
pub struct LogicalStepDesc {
    pub desc: String,
    pub highlight_cells: Vec<usize>,
    pub source_candidates: Vec<usize>,
    pub elim_candidates: Vec<usize>,
    pub strong_links: Vec<(usize, usize)>,
    pub weak_links: Vec<(usize, usize)>,
    pub sub_steps: Vec<LogicalStepDesc>,
    pub is_single_step: bool,
}

pub struct LogicalStepDescList {
    pub steps: Vec<LogicalStepDesc>,
}

impl LogicalStepDescList {
    pub fn new() -> LogicalStepDescList {
        LogicalStepDescList { steps: Vec::new() }
    }
}
