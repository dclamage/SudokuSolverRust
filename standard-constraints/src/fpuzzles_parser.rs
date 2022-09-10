//! Contains the [`FpuzzlesParser`] struct for parsing the f-puzzles format.

pub mod fpuzzles_json;
mod fpuzzles_test_data;
pub mod prelude;

use itertools::Itertools;
use regex::Regex;
use std::sync::Arc;

use super::fpuzzles_parser::prelude::*;
use crate::prelude::*;
use sudoku_solver_lib::prelude::*;

/// A utility struct for parsing the f-puzzles format.
#[derive(Clone, Debug)]
pub struct FPuzzlesParser {
    parse_cell_regex: Regex,
}

impl FPuzzlesParser {
    /// Creates a new [`FPuzzlesParser`].
    pub fn new() -> Self {
        Self {
            parse_cell_regex: Regex::new(r"^[rR](\d+)[cC](\d+)$").unwrap(),
        }
    }

    /// Parses the given [`FPuzzlesBoard`] into a [`Solver`].
    /// Treating the center pencilmarks as given is optional.
    /// Generally, brute force solves use `false` and logical solves use `true`.
    pub fn parse_board(
        &self,
        board: &FPuzzlesBoard,
        treat_pencilmarks_as_given: bool,
    ) -> Result<Solver, String> {
        let size = board.size as usize;
        let cu = CellUtility::new(size);
        let all_values_mask = ValueMask::from_all_values(size);
        let mut solver = SolverBuilder::new(size);

        // Set the givens
        let mut givens = Vec::new();
        for i in 0..size {
            for j in 0..size {
                let entry = &board.grid[i][j];
                let cell = cu.cell(i, j);
                if treat_pencilmarks_as_given || entry.given {
                    givens.push((cell, entry.value as usize));
                }

                if !entry.given_pencil_marks.is_empty()
                    || treat_pencilmarks_as_given && !entry.center_pencil_marks.is_empty()
                {
                    let mut pencilmarks = if !entry.given_pencil_marks.is_empty() {
                        let given_pencil_marks: Vec<usize> = entry
                            .given_pencil_marks
                            .iter()
                            .map(|x| *x as usize)
                            .collect();
                        ValueMask::from_values(&given_pencil_marks)
                    } else {
                        all_values_mask
                    };
                    if treat_pencilmarks_as_given && !entry.center_pencil_marks.is_empty() {
                        let center_pencil_marks: Vec<usize> = entry
                            .center_pencil_marks
                            .iter()
                            .map(|x| *x as usize)
                            .collect();
                        pencilmarks = pencilmarks & ValueMask::from_values(&center_pencil_marks);
                    }
                    if pencilmarks != all_values_mask {
                        solver = solver.with_constraint(Arc::new(PencilmarkConstraint::new(
                            cell,
                            pencilmarks,
                        )));
                    }
                }
            }
        }
        solver = solver.with_givens(&givens);

        // Start with default regions
        let mut regions = default_regions(size);

        // Override the regions
        for i in 0..size {
            for j in 0..size {
                let cell = cu.cell(i, j);
                if board.grid[i][j].region >= 0 {
                    regions[cell.index()] = board.grid[i][j].region as usize;
                }
            }
        }
        solver = solver.with_regions(regions);

        // Add global constraints
        if board.diagonal_p {
            // TODO: Add diagonal constraint
        }
        if board.diagonal_n {
            // TODO: Add diagonal constraint
        }
        if board.antiknight {
            solver = solver.with_constraint(Arc::new(ChessConstraint::anti_knight()));
        }
        if board.antiking {
            solver = solver.with_constraint(Arc::new(ChessConstraint::anti_king()));
        }
        if board.disjointgroups {
            // TODO: Add disjoint groups constraint
        }

        if !board.arrow.is_empty() {
            // TODO: Arrow
        }

        if !board.killercage.is_empty() {
            // TODO: Killer cages
        }

        if !board.littlekillersum.is_empty() {
            // TODO: Little Killer
        }

        if !board.odd.is_empty() {
            for fpuzzles_cell in board.odd.iter() {
                let cell = self.parse_cell(&fpuzzles_cell.cell, size);
                if let Some(cell) = cell {
                    solver = solver.with_constraint(Arc::new(PencilmarkConstraint::odd(cell)));
                }
            }
        }

        if !board.even.is_empty() {
            for fpuzzles_cell in board.even.iter() {
                let cell = self.parse_cell(&fpuzzles_cell.cell, size);
                if let Some(cell) = cell {
                    solver = solver.with_constraint(Arc::new(PencilmarkConstraint::even(cell)));
                }
            }
        }

        if !board.minimum.is_empty() {
            // TODO: Minimum constraint
        }

        if !board.maximum.is_empty() {
            // TODO: Maximum constraint
        }

        if !board.rowindexer.is_empty() {
            // TODO: Row indexer
        }

        if !board.columnindexer.is_empty() {
            // TODO: Column indexer
        }

        if !board.boxindexer.is_empty() {
            // TODO Box indexer
        }

        if !board.extraregion.is_empty() {
            // TODO Extra region
        }

        if !board.thermometer.is_empty() {
            // TODO Thermometer
        }

        if !board.palindrome.is_empty() {
            // TODO Palindrome
        }

        if !board.renban.is_empty() {
            // TODO Renban
        }

        if !board.whispers.is_empty() {
            // TODO Whispers
        }

        if !board.regionsumline.is_empty() {
            // TODO Region sum line
        }

        if !board.betweenline.is_empty() {
            // TODO Between line
        }

        let negative_consecutive = board.nonconsecutive;
        let negative_ratio = board.negative.iter().any(|x| x == "ratio");
        if negative_consecutive
            || negative_ratio
            || !board.difference.is_empty()
            || !board.ratio.is_empty()
        {
            let mut markers: Vec<StandardOrthogonalPairsMarker> = Vec::new();
            for cells in board.difference.iter() {
                if cells.cells.len() != 2 {
                    continue;
                }

                let cell1 = self.parse_cell(&cells.cells[0], size);
                let cell2 = self.parse_cell(&cells.cells[1], size);
                let value = cells.value.parse::<usize>().unwrap_or(1);
                if let Some(cell1) = cell1 {
                    if let Some(cell2) = cell2 {
                        markers.push(StandardOrthogonalPairsMarker::difference(
                            value, cell1, cell2,
                        ));
                    }
                }
            }
            for cells in board.ratio.iter() {
                if cells.cells.len() != 2 {
                    continue;
                }

                let cell1 = self.parse_cell(&cells.cells[0], size);
                let cell2 = self.parse_cell(&cells.cells[1], size);
                let value = cells.value.parse::<usize>().unwrap_or(2);
                if let Some(cell1) = cell1 {
                    if let Some(cell2) = cell2 {
                        markers.push(StandardOrthogonalPairsMarker::ratio(value, cell1, cell2));
                    }
                }
            }

            let mut negatives = Vec::new();
            if negative_consecutive {
                negatives.push(StandardPairType::Diff(1));
            }
            if negative_ratio {
                if markers.is_empty() {
                    negatives.push(StandardPairType::Ratio(2));
                } else {
                    negatives.extend(markers.iter().map(|x| x.marker_type()).unique());
                }
            }

            solver =
                solver.with_constraint(Arc::new(OrthogonalPairsConstraint::from_standard_markers(
                    size, "Kropki", &markers, &negatives,
                )));
        }

        let negative_xv = board.negative.iter().any(|x| x == "xv");
        if negative_xv || !board.xv.is_empty() {
            let mut markers: Vec<StandardOrthogonalPairsMarker> = Vec::new();
            for cells in board.difference.iter() {
                if cells.cells.len() != 2 {
                    continue;
                }

                let cell1 = self.parse_cell(&cells.cells[0], size);
                let cell2 = self.parse_cell(&cells.cells[1], size);
                let value = match cells.value.as_str() {
                    "V" | "v" => 5,
                    "X" | "x" => 10,
                    _ => 0,
                };
                if value != 0 {
                    if let Some(cell1) = cell1 {
                        if let Some(cell2) = cell2 {
                            markers.push(StandardOrthogonalPairsMarker::sum(value, cell1, cell2));
                        }
                    }
                }
            }

            let mut negatives = Vec::new();
            if negative_xv {
                negatives.push(StandardPairType::Sum(5));
                negatives.push(StandardPairType::Sum(10));
            }

            solver = solver.with_constraint(Arc::new(
                OrthogonalPairsConstraint::from_standard_markers(size, "XV", &markers, &negatives),
            ));
        }

        if !board.clone.is_empty() {
            // TODO: Clone constraint
        }

        if !board.quadruple.is_empty() {
            // TODO: Quadruple constraint
        }

        if !board.sandwichsum.is_empty() {
            // TODO: Sandwich sum constraint
        }

        if !board.xsum.is_empty() {
            // TODO: X-Sum constraint
        }

        if !board.skyscraper.is_empty() {
            // TODO: Skyscraper constraint
        }

        if !board.entropicline.is_empty() {
            // TODO: Entrpic line constraint
        }

        solver.build()
    }

    fn parse_cell(&self, cell_str: &str, size: usize) -> Option<CellIndex> {
        let captures = self.parse_cell_regex.captures(cell_str);
        captures.as_ref()?;

        let captures = captures.unwrap();
        if captures.len() != 3 {
            return None;
        }

        let capture1 = captures.get(1);
        let capture2 = captures.get(2);
        if capture1.is_none() || capture2.is_none() {
            return None;
        }
        let capture1 = capture1.unwrap();
        let capture2 = capture2.unwrap();

        let row = capture1.as_str().parse::<usize>();
        let col = capture2.as_str().parse::<usize>();
        if row.is_err() || col.is_err() {
            return None;
        }
        let row = row.unwrap();
        let col = col.unwrap();

        if row == 0 || col == 0 {
            return None;
        }
        Some(CellIndex::from_rc(row - 1, col - 1, size))
    }
}

impl Default for FPuzzlesParser {
    fn default() -> Self {
        Self::new()
    }
}
