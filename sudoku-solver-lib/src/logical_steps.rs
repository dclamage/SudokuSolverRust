#[derive(Clone)]
pub struct LogicalStep {
	pub desc: String,
	pub highlight_cells: Vec<usize>,
	pub source_candidates: Vec<usize>,
	pub elim_candidates: Vec<usize>,
	pub strong_links: Vec<(usize, usize)>,
	pub weak_links: Vec<(usize, usize)>,
	pub sub_steps: Vec<LogicalStep>,
	pub is_single_step: bool,
}

pub struct LogicalSteps {
	pub steps: Vec<LogicalStep>,
}

impl LogicalSteps {
	pub fn new() -> LogicalSteps {
		LogicalSteps {
			steps: Vec::new(),
		}
	}
}