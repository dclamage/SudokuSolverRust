pub mod message;
pub mod responses;
mod utils;

use std::time::Instant;

use crate::message::*;
use crate::responses::*;
use itertools::Itertools;
use standard_constraints::prelude::*;
use sudoku_solver_lib::prelude::*;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn solve(message: &str, receive_result: &js_sys::Function) {
    let message = match Message::from_json(message) {
        Ok(message) => message,
        Err(error) => {
            send_result(
                InvalidResponse::new(0, &error.to_string())
                    .to_json()
                    .as_str(),
                receive_result,
            );
            return;
        }
    };
    let nonce = message.nonce();

    cancel_current_solver();

    if message.command() == "cancel" {
        send_result(
            CanceledResponse::new(nonce).to_json().as_str(),
            receive_result,
        );
        return;
    }

    if message.data_type() != "fpuzzles" {
        send_result(
            InvalidResponse::new(nonce, "Invalid data type. Expected 'fpuzzles'.")
                .to_json()
                .as_str(),
            receive_result,
        );
        return;
    }

    let only_givens = matches!(
        message.command(),
        "solve" | "truecandidates" | "check" | "count"
    );

    let board = match FPuzzlesBoard::from_lzstring_json(message.data()) {
        Ok(board) => board,
        Err(error) => {
            send_result(
                InvalidResponse::new(nonce, &error).to_json().as_str(),
                receive_result,
            );
            return;
        }
    };

    let parser = FPuzzlesParser::new();
    let solver = match parser.parse_board(&board, only_givens) {
        Ok(puzzle) => puzzle,
        Err(error) => {
            send_result(
                InvalidResponse::new(nonce, &error).to_json().as_str(),
                receive_result,
            );
            return;
        }
    };

    let result = match message.command() {
        "truecandidates" => true_candidates(nonce, solver),
        "solve" => find_solution(nonce, solver),
        "check" => count(nonce, solver, 2, receive_result),
        "count" => count(nonce, solver, 0, receive_result),
        "solvepath" => solve_path(nonce, solver),
        "step" => step(nonce, solver),
        _ => InvalidResponse::new(
            message.nonce(),
            format!("Unknown command: {}", message.command()).as_str(),
        )
        .to_json(),
    };

    send_result(result.as_str(), receive_result);
}

fn send_result(result: &str, receive_result: &js_sys::Function) {
    let this = JsValue::NULL;
    let args = js_sys::Array::of1(&JsValue::from_str(result));
    let _ = receive_result.call1(&this, &args);
}

fn cancel_current_solver() {
    // TODO
}

fn get_bool_option(solver: &Solver, option: &str) -> bool {
    match solver.get_custom_info(option) {
        Some(value) => value == "true",
        None => false,
    }
}

fn true_candidates(nonce: i32, solver: Solver) -> String {
    let colored = get_bool_option(&solver, "truecandidatescolored");
    let logical = get_bool_option(&solver, "truecandidateslogical");

    let mut logical_solver: Option<Solver> = if logical { Some(solver.clone()) } else { None };
    if let Some(solver) = logical_solver.as_mut() {
        let logical_result = solver.run_logical_solve();
        if logical_result.is_invalid() {
            return InvalidResponse::new(nonce, "No solutions found.").to_json();
        }
    }

    let size = solver.size();
    let total_candidates = size * size * size;
    let mut solutions_per_candidate: Vec<i32> = vec![0; total_candidates];
    let real_cells: Vec<ValueMask>;
    let mut candidate_counts: Option<Vec<usize>> = None;
    if colored {
        let result = solver.find_true_candidates_with_count(8);
        match result {
            TrueCandidatesCountResult::None => {
                return InvalidResponse::new(nonce, "No solutions found.").to_json();
            }
            TrueCandidatesCountResult::Error(error) => {
                return InvalidResponse::new(nonce, &error).to_json();
            }
            TrueCandidatesCountResult::Solved(board) => {
                real_cells = board.all_cell_masks().map(|(_, mask)| mask).collect();
            }
            TrueCandidatesCountResult::Candidates(board, counts) => {
                real_cells = board.all_cell_masks().map(|(_, mask)| mask).collect();
                candidate_counts = Some(counts);
            }
        }
    } else {
        let result = solver.find_true_candidates();
        match result {
            SingleSolutionResult::None => {
                return InvalidResponse::new(nonce, "No solutions found.").to_json();
            }
            SingleSolutionResult::Error(error) => {
                return InvalidResponse::new(nonce, &error).to_json();
            }
            SingleSolutionResult::Solved(board) => {
                real_cells = board.all_cell_masks().map(|(_, mask)| mask).collect();
            }
        }
    }

    let logical_cells: Vec<ValueMask> = if let Some(logical_solver) = logical_solver.as_ref() {
        logical_solver
            .board()
            .all_cell_masks()
            .map(|(_, mask)| mask)
            .collect()
    } else {
        real_cells.clone()
    };

    for cell_index in 0..size {
        let real_mask = real_cells[cell_index];
        let logical_mask = logical_cells[cell_index];
        for value in 1..=size {
            let solution_index = (cell_index * size + value - 1) as usize;
            let have_value_real = real_mask.has(value);
            let have_value_logical = logical_mask.has(value);
            if !have_value_real && have_value_logical {
                solutions_per_candidate[solution_index] = -1;
            } else if have_value_real {
                if let Some(candidate_counts) = candidate_counts.as_ref() {
                    solutions_per_candidate[solution_index] =
                        candidate_counts[solution_index] as i32;
                } else {
                    solutions_per_candidate[solution_index] = 1;
                }
            }
        }
    }

    TrueCandidatesResponse::new(nonce, &solutions_per_candidate).to_json()
}

fn find_solution(nonce: i32, solver: Solver) -> String {
    let result = solver.find_random_solution();
    match result {
        SingleSolutionResult::None => InvalidResponse::new(nonce, "No solutions found.").to_json(),
        SingleSolutionResult::Error(error) => InvalidResponse::new(nonce, &error).to_json(),
        SingleSolutionResult::Solved(board) => {
            let board: Vec<i32> = board
                .all_cell_masks()
                .map(|(_, mask)| mask.value() as i32)
                .collect();
            SolvedResponse::new(nonce, &board).to_json()
        }
    }
}

struct ReportCountSolutionReceiver<'a> {
    count: usize,
    nonce: i32,
    receive_result: &'a js_sys::Function,
    last_report_time: Instant,
}

impl<'a> ReportCountSolutionReceiver<'a> {
    pub fn new(nonce: i32, receive_result: &'a js_sys::Function) -> Self {
        Self {
            count: 0,
            nonce,
            receive_result,
            last_report_time: Instant::now(),
        }
    }
}

impl SolutionReceiver for ReportCountSolutionReceiver<'_> {
    fn receive(&mut self, _result: Box<Board>) -> bool {
        self.count += 1;

        let now = Instant::now();
        if now.duration_since(self.last_report_time).as_millis() >= 1000 {
            self.last_report_time = now;

            let in_progress_response =
                CountResponse::new(self.nonce, self.count as u64, true).to_json();
            send_result(in_progress_response.as_str(), self.receive_result);
        }

        true
    }
}

fn count(
    nonce: i32,
    solver: Solver,
    max_solutions: i32,
    receive_result: &js_sys::Function,
) -> String {
    let result = if max_solutions <= 2 {
        solver.find_solution_count(max_solutions as usize, None)
    } else {
        let mut receiver = ReportCountSolutionReceiver::new(nonce, receive_result);
        solver.find_solution_count(0, Some(&mut receiver))
    };
    match result {
        SolutionCountResult::None => InvalidResponse::new(nonce, "No solutions found.").to_json(),
        SolutionCountResult::Error(error) => InvalidResponse::new(nonce, &error).to_json(),
        SolutionCountResult::ExactCount(count) | SolutionCountResult::AtLeastCount(count) => {
            CountResponse::new(nonce, count as u64, false).to_json()
        }
    }
}

fn logical_cells(solver: &Solver) -> Vec<LogicalCell> {
    solver
        .board()
        .all_cell_masks()
        .map(|(_, mask)| {
            if mask.is_solved() {
                LogicalCell {
                    value: mask.value() as i32,
                    candidates: Vec::new(),
                }
            } else {
                LogicalCell {
                    value: 0,
                    candidates: mask.into_iter().map(|v| v as i32).collect(),
                }
            }
        })
        .collect()
}

fn solve_path(nonce: i32, mut solver: Solver) -> String {
    let result = solver.run_logical_solve();
    let cells: Vec<LogicalCell> = logical_cells(&solver);

    match result {
        LogicalSolveResult::None => {
            LogicalResponse::new(nonce, &cells, "No logical steps found.", true).to_json()
        }
        LogicalSolveResult::Changed(desc) | LogicalSolveResult::Solved(desc) => {
            LogicalResponse::new(nonce, &cells, desc.to_string().as_str(), true).to_json()
        }
        LogicalSolveResult::Invalid(mut desc) => {
            desc.push("Board is invalid!".into());
            LogicalResponse::new(nonce, &cells, desc.to_string().as_str(), false).to_json()
        }
    }
}

fn step(nonce: i32, mut solver: Solver) -> String {
    let cells: Vec<LogicalCell> = logical_cells(&solver);

    if solver.board().is_solved() {
        return LogicalResponse::new(nonce, &cells, "Solved!", true).to_json();
    }

    if let Some(original_center_marks) = solver.get_custom_info("OriginalCenterMarks") {
        let new_center_marks = solver
            .board()
            .all_cell_masks()
            .map(|(_, mask)| mask.into_iter().join(","))
            .join(";");
        if original_center_marks != new_center_marks {
            return LogicalResponse::new(nonce, &cells, "Initial candidates.", false).to_json();
        }
    }

    let result = solver.run_single_logical_step();
    match result {
        LogicalStepResult::None => {
            LogicalResponse::new(nonce, &cells, "No logical steps found.", true).to_json()
        }
        LogicalStepResult::Changed(desc) => {
            let desc = desc.unwrap_or_else(|| "ERROR: No logical step description!".into());
            LogicalResponse::new(nonce, &cells, desc.to_string().as_str(), true).to_json()
        }
        LogicalStepResult::Invalid(desc) => {
            let mut desc_list = LogicalStepDescList::new();
            desc_list.push(desc.unwrap_or_else(|| "ERROR: No logical step description!".into()));
            desc_list.push("Board is invalid!".into());
            LogicalResponse::new(nonce, &cells, desc_list.to_string().as_str(), false).to_json()
        }
    }
}
