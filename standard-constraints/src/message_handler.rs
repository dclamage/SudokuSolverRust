pub mod message;
pub mod responses;

use std::time::Instant;

use crate::prelude::*;
use itertools::Itertools;
use sudoku_solver_lib::prelude::*;

use self::message::*;
use self::responses::*;

pub trait SendResult {
    fn send_result(&mut self, result: &str);
}

pub struct MessageHandler {
    send_result: Box<dyn SendResult>,
    cancellation: Cancellation,
}

impl MessageHandler {
    pub fn new(send_result: Box<dyn SendResult>) -> Self {
        Self {
            send_result,
            cancellation: Cancellation::default(),
        }
    }

    fn send_result(&mut self, result: &str) {
        self.send_result.send_result(result);
    }

    pub fn handle_message(&mut self, message: &str, cancellation: Cancellation) {
        self.cancellation = cancellation;

        if self.cancellation.check() {
            return;
        }

        let message = match Message::from_json(message) {
            Ok(message) => message,
            Err(error) => {
                self.send_result(
                    InvalidResponse::new(0, &error.to_string())
                        .to_json()
                        .as_str(),
                );
                return;
            }
        };
        let nonce = message.nonce();

        if message.command() == "cancel" {
            self.send_result(CanceledResponse::new(nonce).to_json().as_str());
            return;
        }

        if message.data_type() != "fpuzzles" {
            self.send_result(
                InvalidResponse::new(nonce, "Invalid data type. Expected 'fpuzzles'.")
                    .to_json()
                    .as_str(),
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
                self.send_result(InvalidResponse::new(nonce, &error).to_json().as_str());
                return;
            }
        };

        let parser = FPuzzlesParser::new();
        let solver = match parser.parse_board(&board, !only_givens) {
            Ok(puzzle) => puzzle,
            Err(error) => {
                self.send_result(InvalidResponse::new(nonce, &error).to_json().as_str());
                return;
            }
        };

        let result = match message.command() {
            "truecandidates" => self.true_candidates(nonce, solver),
            "solve" => self.find_solution(nonce, solver),
            "check" => self.count(nonce, solver, 2),
            "count" => self.count(nonce, solver, 0),
            "solvepath" => self.solve_path(nonce, solver),
            "step" => self.step(nonce, solver),
            _ => InvalidResponse::new(
                message.nonce(),
                format!("Unknown command: {}", message.command()).as_str(),
            )
            .to_json(),
        };

        self.send_result(result.as_str());
    }

    #[allow(dead_code)]
    fn debug_log(&mut self, message: &str) {
        let response = DebugLogResponse::new(message).to_json();
        self.send_result(response.as_str());
    }

    fn get_bool_option(solver: &Solver, option: &str) -> bool {
        match solver.get_custom_info(option) {
            Some(value) => value == "true",
            None => false,
        }
    }

    fn true_candidates(&mut self, nonce: i32, solver: Solver) -> String {
        let colored = Self::get_bool_option(&solver, "truecandidatescolored");
        let logical = Self::get_bool_option(&solver, "truecandidateslogical");

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
            let result = solver.find_true_candidates_with_count(8, self.cancellation.clone());
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

        for cell_index in 0..real_cells.len() {
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

    fn find_solution(&mut self, nonce: i32, solver: Solver) -> String {
        let result = solver.find_random_solution();
        match result {
            SingleSolutionResult::None => {
                InvalidResponse::new(nonce, "No solutions found.").to_json()
            }
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

    fn count(&mut self, nonce: i32, solver: Solver, max_solutions: i32) -> String {
        let cancellation = self.cancellation.clone();
        let result = if max_solutions > 0 && max_solutions <= 2 {
            solver.find_solution_count(max_solutions as usize, None, cancellation)
        } else {
            let mut receiver = ReportCountSolutionReceiver::new(nonce, self);
            solver.find_solution_count(0, Some(&mut receiver), cancellation)
        };
        match result {
            SolutionCountResult::None => {
                InvalidResponse::new(nonce, "No solutions found.").to_json()
            }
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

    fn solve_path(&mut self, nonce: i32, mut solver: Solver) -> String {
        let result = solver.run_logical_solve();
        let cells: Vec<LogicalCell> = Self::logical_cells(&solver);

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

    fn step(&mut self, nonce: i32, mut solver: Solver) -> String {
        let cells: Vec<LogicalCell> = Self::logical_cells(&solver);

        if solver.board().is_solved() {
            return LogicalResponse::new(nonce, &cells, "Solved!", true).to_json();
        }

        if let Some(original_center_marks) = solver.get_custom_info("OriginalCenterMarks") {
            let new_center_marks = solver
                .board()
                .all_cell_masks()
                .map(|(_, mask)| {
                    if mask.is_solved() {
                        String::new()
                    } else {
                        mask.into_iter().join(",")
                    }
                })
                .join(";");
            if original_center_marks != new_center_marks {
                return LogicalResponse::new(nonce, &cells, "Initial candidates.", false).to_json();
            }
        }

        let result = solver.run_single_logical_step();
        let cells: Vec<LogicalCell> = Self::logical_cells(&solver);
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
                desc_list
                    .push(desc.unwrap_or_else(|| "ERROR: No logical step description!".into()));
                desc_list.push("Board is invalid!".into());
                LogicalResponse::new(nonce, &cells, desc_list.to_string().as_str(), false).to_json()
            }
        }
    }
}

struct ReportCountSolutionReceiver<'a> {
    count: usize,
    nonce: i32,
    message_handler: &'a mut MessageHandler,
    last_report_time: Instant,
}

impl<'a> ReportCountSolutionReceiver<'a> {
    pub fn new(nonce: i32, message_handler: &'a mut MessageHandler) -> Self {
        Self {
            count: 0,
            nonce,
            message_handler,
            last_report_time: Instant::now(),
        }
    }
}

impl<'a> SolutionReceiver for ReportCountSolutionReceiver<'a> {
    fn receive(&mut self, _result: Box<Board>) -> bool {
        self.count += 1;

        let now = Instant::now();
        if now.duration_since(self.last_report_time).as_millis() >= 1000 {
            self.last_report_time = now;

            let in_progress_response =
                CountResponse::new(self.nonce, self.count as u64, true).to_json();
            self.message_handler
                .send_result(in_progress_response.as_str());
        }

        true
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::fpuzzles_parser::fpuzzles_test_data::FPUZZLES_CLASSICS_DATA;

    struct TestSendResult {
        results: Arc<Mutex<Vec<String>>>,
    }

    impl TestSendResult {
        fn new(results: Arc<Mutex<Vec<String>>>) -> Self {
            Self { results }
        }
    }

    impl SendResult for TestSendResult {
        fn send_result(&mut self, result: &str) {
            self.results.lock().unwrap().push(result.to_string());
        }
    }

    fn create_test_handler() -> (MessageHandler, Arc<Mutex<Vec<String>>>) {
        let results = Arc::new(Mutex::new(Vec::new()));
        let test_handler = Box::new(TestSendResult::new(results.clone()));
        (MessageHandler::new(test_handler), results)
    }

    #[test]
    fn test_solve_classic() {
        let (mut handler, results) = create_test_handler();
        for (lzstr, expected_solution) in FPUZZLES_CLASSICS_DATA.iter() {
            results.lock().unwrap().clear();

            let message = Message::new(123, "solve", "fpuzzles", lzstr).to_json();
            handler.handle_message(&message, Cancellation::default());
            let result = results.lock().unwrap();
            assert_eq!(result.len(), 1);

            let response = SolvedResponse::from_json(result[0].as_str()).unwrap();
            assert_eq!(
                response.nonce, 123,
                "Nonce should be 123 for solve message, but was {}",
                response.nonce
            );

            let expected_solution: Vec<i32> = expected_solution
                .chars()
                .map(|c| c as i32 - '0' as i32)
                .collect();
            assert_eq!(
                response.solution, *expected_solution,
                "Solution should be {:?} for solve message, but was {:?}",
                expected_solution, response.solution
            );
        }
    }

    #[test]
    fn test_antikropki_count() {
        // Empty grid with negative constraint for kropki.
        let lzstr = r#"N4IgzglgXgpiBcBOANCA5gJwgEwQbT2AF9ljSSzKLryBdZQmq8l54+x1p7rjtn/nQaCR3PgIm9hk0UM6zR4rssX0QAOwD26gMbawMHQFcALhABuceCYxGYqdTDQBDM5fwgMriJpBqvZr7weLREQA"#;

        let message = Message::new(123, "count", "fpuzzles", lzstr).to_json();

        let (mut handler, results) = create_test_handler();
        handler.handle_message(&message, Cancellation::default());

        let result = results.lock().unwrap();
        assert!(result.len() > 0);

        let response = CountResponse::from_json(result.last().unwrap().as_str()).unwrap();
        assert_eq!(
            response.nonce, 123,
            "Nonce should be 123 for solve message, but was {}",
            response.nonce
        );
        assert_eq!(
            response.in_progress, false,
            "Count should be finished, but was in progress"
        );
        assert_eq!(
            response.count, 8448,
            "Count should be 8448 for solve message, but was {}",
            response.count
        );
    }

    #[test]
    fn test_xv_true_candidates() {
        // Empty grid other than an X between r1c12 and a V between r2c12.
        let lzstr = r#"N4IgzglgXgpiBcBOANCA5gJwgEwQbT2AF9ljSSzKLryBdZQmq8l54+x1p7rjtn/nQaCR3PgIm9hk0UM6zR4rssX0QADwBu+UAGMYAGwNh8IAEoBGAMIAmEKktWLINZoCGBgK5x4IABogFCD6RibweOY2TvaRti6o7l4+IABqgbREQA=="#;

        let message = Message::new(123, "truecandidates", "fpuzzles", lzstr).to_json();

        let (mut handler, results) = create_test_handler();
        handler.handle_message(&message, Cancellation::default());

        let result = results.lock().unwrap();
        assert!(result.len() == 1);

        let response = TrueCandidatesResponse::from_json(result.last().unwrap().as_str()).unwrap();
        assert_eq!(
            response.nonce, 123,
            "Nonce should be 123 for solve message, but was {}",
            response.nonce
        );
        assert_eq!(response.solutions_per_candidate.len(), 9 * 9 * 9);

        let cu = CellUtility::new(9);

        // Test the X clue
        {
            let cell_r1c1 = cu.cell(0, 0);
            let cell_r1c2 = cu.cell(0, 1);
            assert_eq!(
                response.solutions_per_candidate[cell_r1c1.candidate(1).index()],
                1,
                "There should be 1 solution for candidate 1 in cell R1C1"
            );
            assert_eq!(
                response.solutions_per_candidate[cell_r1c1.candidate(5).index()],
                0,
                "There should be 0 solutions for candidate 5 in cell R1C1"
            );
            assert_eq!(
                response.solutions_per_candidate[cell_r1c1.candidate(9).index()],
                1,
                "There should be 1 solution for candidate 9 in cell R1C1"
            );
            assert_eq!(
                response.solutions_per_candidate[cell_r1c2.candidate(1).index()],
                1,
                "There should be 1 solution for candidate 1 in cell R1C1"
            );
            assert_eq!(
                response.solutions_per_candidate[cell_r1c2.candidate(5).index()],
                0,
                "There should be 0 solutions for candidate 5 in cell R1C1"
            );
            assert_eq!(
                response.solutions_per_candidate[cell_r1c2.candidate(9).index()],
                1,
                "There should be 1 solution for candidate 9 in cell R1C1"
            );
        }

        // Test the V clue
        {
            let cell_r2c1 = cu.cell(1, 0);
            let cell_r2c2 = cu.cell(1, 1);
            assert_eq!(
                response.solutions_per_candidate[cell_r2c1.candidate(1).index()],
                1,
                "There should be 1 solution for candidate 1 in cell R2C1"
            );
            assert_eq!(
                response.solutions_per_candidate[cell_r2c1.candidate(5).index()],
                0,
                "There should be 0 solutions for candidate 5 in cell R2C1"
            );
            assert_eq!(
                response.solutions_per_candidate[cell_r2c1.candidate(9).index()],
                0,
                "There should be 0 solutions for candidate 9 in cell R2C1"
            );
            assert_eq!(
                response.solutions_per_candidate[cell_r2c2.candidate(1).index()],
                1,
                "There should be 1 solution for candidate 1 in cell R2C1"
            );
            assert_eq!(
                response.solutions_per_candidate[cell_r2c2.candidate(5).index()],
                0,
                "There should be 0 solutions for candidate 5 in cell R2C1"
            );
            assert_eq!(
                response.solutions_per_candidate[cell_r2c2.candidate(9).index()],
                0,
                "There should be 0 solutions for candidate 9 in cell R2C1"
            );
        }
    }
}
