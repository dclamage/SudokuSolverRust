//! Contains the [`FPuzzlesParser`] struct for parsing the f-puzzles format.

pub mod fpuzzles_json;
pub mod fpuzzles_test_data;
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
        Self { parse_cell_regex: Regex::new(r"^[rR](\d+)[cC](\d+)$").unwrap() }
    }

    /// Parses the given [`FPuzzlesBoard`] into a [`Solver`].
    /// Treating the center pencilmarks as given is optional.
    /// Generally, brute force solves use `false` and logical solves use `true`.
    pub fn parse_board(&self, board: &FPuzzlesBoard, treat_pencilmarks_as_given: bool) -> Result<Solver, String> {
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
                if (treat_pencilmarks_as_given || entry.given) && entry.value > 0 && entry.value <= size as i32 {
                    givens.push((cell, entry.value as usize));
                }

                if !entry.given_pencil_marks.is_empty()
                    || treat_pencilmarks_as_given && !entry.center_pencil_marks.is_empty()
                {
                    let mut pencilmarks = if !entry.given_pencil_marks.is_empty() {
                        let given_pencil_marks: Vec<usize> =
                            entry.given_pencil_marks.iter().map(|x| *x as usize).collect();
                        ValueMask::from_values(&given_pencil_marks)
                    } else {
                        all_values_mask
                    };
                    if treat_pencilmarks_as_given && !entry.center_pencil_marks.is_empty() {
                        let center_pencil_marks: Vec<usize> =
                            entry.center_pencil_marks.iter().map(|x| *x as usize).collect();
                        pencilmarks = pencilmarks & ValueMask::from_values(&center_pencil_marks);
                    }
                    if pencilmarks != all_values_mask {
                        solver = solver.with_constraint(Arc::new(PencilmarkConstraint::new(cell, pencilmarks)));
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
        solver = solver.with_regions(regions.clone());

        // Add solving options
        for option in board.truecandidatesoptions.iter() {
            if option == "colored" {
                solver = solver.with_custom_info("truecandidatescolored", "true");
            } else if option == "logical" {
                solver = solver.with_custom_info("truecandidateslogical", "true");
            }
        }

        // Store the original center marks if they are treated as given
        if treat_pencilmarks_as_given {
            let mut center_marks = Vec::new();
            for i in 0..size {
                for j in 0..size {
                    let entry = &board.grid[i][j];
                    let center_pencil_marks: String = entry.center_pencil_marks.iter().map(|x| *x as usize).join(",");
                    center_marks.push(center_pencil_marks);
                }
            }
            solver = solver.with_custom_info("OriginalCenterMarks", center_marks.iter().join(";").as_str());
        }

        // Add global constraints
        if board.diagonal_p {
            solver = solver.with_constraint(Arc::new(NonRepeatConstraint::from_diagonalp(size)));
        }
        if board.diagonal_n {
            solver = solver.with_constraint(Arc::new(NonRepeatConstraint::from_diagonaln(size)));
        }
        if board.antiknight {
            solver = solver.with_constraint(Arc::new(ChessConstraint::anti_knight()));
        }
        if board.antiking {
            solver = solver.with_constraint(Arc::new(ChessConstraint::anti_king()));
        }
        if board.disjointgroups {
            for region_id in regions.iter().copied().unique() {
                let cells: Vec<CellIndex> = regions
                    .iter()
                    .copied()
                    .enumerate()
                    .filter(move |(_, id)| *id == region_id)
                    .map(move |(index, _)| cu.cell_index(index))
                    .collect();
                if cells.len() == size {
                    let name = format!("DisjointGroup{}", region_id + 1);
                    solver = solver.with_constraint(Arc::new(NonRepeatConstraint::new(&name, cells)));
                }
            }
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
            for (id, extra_region) in board.extraregion.iter().enumerate() {
                let cells = self.parse_cells(extra_region, size);
                if cells.len() == size {
                    let name = format!("ExtraRegion{}", id + 1);
                    solver = solver.with_constraint(Arc::new(NonRepeatConstraint::new(&name, cells)));
                }
            }
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
        if negative_consecutive || negative_ratio || !board.difference.is_empty() || !board.ratio.is_empty() {
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
                        markers.push(StandardOrthogonalPairsMarker::difference(value, cell1, cell2));
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

            solver = solver.with_constraint(Arc::new(OrthogonalPairsConstraint::from_standard_markers(
                size, "Kropki", &markers, &negatives,
            )));
        }

        let negative_xv = board.negative.iter().any(|x| x == "xv");
        if negative_xv || !board.xv.is_empty() {
            let mut markers: Vec<StandardOrthogonalPairsMarker> = Vec::new();
            for cells in board.xv.iter() {
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

            solver = solver.with_constraint(Arc::new(OrthogonalPairsConstraint::from_standard_markers(
                size, "XV", &markers, &negatives,
            )));
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
            // TODO: Entropic line constraint
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

    fn parse_cells(&self, cells: &FPuzzlesCells, size: usize) -> Vec<CellIndex> {
        cells.cells.iter().filter_map(|fpuzzles_cell| self.parse_cell(fpuzzles_cell, size)).collect()
    }
}

impl Default for FPuzzlesParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::fpuzzles_test_data::FPUZZLES_CLASSICS_DATA;
    use super::*;

    fn test_unqiue_solution_from_lzstring(parser: &FPuzzlesParser, lzstring: &str, expected_solution: &str) {
        struct Receiver {
            solution: Option<String>,
        }

        impl SolutionReceiver for Receiver {
            fn receive(&mut self, result: Box<Board>) -> bool {
                self.solution = Some(result.to_string());
                true
            }
        }
        let mut receiver = Receiver { solution: None };

        let board = FPuzzlesBoard::from_lzstring_json(lzstring).unwrap();
        let solver = parser.parse_board(&board, false).unwrap();
        let count = solver.find_solution_count(10000, Some(&mut receiver), None);
        assert!(count.is_exact_count());
        assert_eq!(count.count().unwrap(), 1);
        assert_eq!(receiver.solution.unwrap(), expected_solution);

        let solution = solver.find_first_solution();
        assert!(solution.is_solved());
        let solution_board = solution.board().unwrap();
        assert!(solution_board.is_solved());
        assert_eq!(solution_board.to_string(), expected_solution);

        let solution = solver.find_random_solution();
        assert!(solution.is_solved());
        let solution_board = solution.board().unwrap();
        assert!(solution_board.is_solved());
        assert_eq!(solution_board.to_string(), expected_solution);

        let true_candidates = solver.find_true_candidates();
        assert!(true_candidates.is_solved());
        let solution_board = solution.board().unwrap();
        assert!(solution_board.is_solved());
        assert_eq!(solution_board.to_string(), expected_solution);
    }

    #[test]
    fn test_classics() {
        let parser = FPuzzlesParser::new();
        for classic in FPUZZLES_CLASSICS_DATA {
            test_unqiue_solution_from_lzstring(&parser, &classic.0, &classic.1);
        }
    }

    #[test]
    fn test_miracle() {
        let parser = FPuzzlesParser::new();
        let lzstring = r#"N4IgzglgXgpiBcBOANCA5gJwgEwQbT2AF9ljSSzKLryBdZQmq8l54+x1p7rjtn/nQZsQANwCGAGwCuceAEZUaCKJgA7BABcMsgXr56upMVNkIATEpXqtOmNwNHB/R88Pth7r7XohxazQgAazUINAALTVtZVH9AoIg1NGiYVDUAezUAY0ywGCzpQNUUoiA==="#;
        let expected_solution = r#"483726159726159483159483726837261594261594837594837261372615948615948372948372615"#;
        test_unqiue_solution_from_lzstring(&parser, lzstring, expected_solution)
    }

    #[test]
    fn test_kropki() {
        let parser = FPuzzlesParser::new();
        let lzstring = r#"N4IgzglgXgpiBcBOANCA5gJwgEwQbT2AF9ljSSzKLryBdZQmq8l54+x1p7rjtn/nQaCR3PgIm9hk0UM6zR4rssX0QAOwD26gMbawMHQFcALhABuceCYxGYqbBABmTmBhi6rhEDpgAbPzB8EAAlRABhRBBUEIAOSJBaCh9/QOCQgHZwgFZo0PjcpNIUgKD4PFCIgBY8uPCaotBfUvT4gEZa+IAmROTmtPLQgDZwgGZarPHGkoGKkJHxmOyx3uL+srmRmpiF1abUjeH62pHCvoP0rNydnL2Zw5CqhKWE6fX00efQrtfzlsGQj9YrU2uFgW8LgCfkNaj8zmtIXNPttvscIf8kbcYnC7u8AZ8YTFPvD9hjQp9FqEnlM/rMqSsYk8GrSHk9CfSMrjEaFlsCXpz0XSQst2Y9wjDBQ8sqKRhKWa1wpyYvE5QiyWEwbUIgL5QCsnz8prJZdFSdTcaASMDfNzUQ1OoYGgAIZmSzBDAuiCaRKoD1mb3lUlCoG1T7g3WY61PcNqoXdLXhHoWubxmJZJMRnlYrPM2MPZbXLOqoNSzU3GMlj6J2pPDN5hWFuq5yt601po2ZjWUjXN+4KlGZNGd2W1EVc9WfJXkjv1/HVonhDrJ0KgjoxUF1ltzUHdn40pJAA="#;
        let expected_solution = r#"482617953951832647637945128268793415593124786174568239815279364329486571746351892"#;
        test_unqiue_solution_from_lzstring(&parser, lzstring, expected_solution)
    }

    #[test]
    fn test_kropki_pairs() {
        let parser = FPuzzlesParser::new();
        let lzstring = r#"N4IgzglgXgpiBcBOANCA5gJwgEwQbT1ADcBDAGwFc54AWAX2WPKoQA4GnLqAmDkUrggBsfAS3gBGUc2oB2aYKQLxAVmXUAzHQC6yQvxkI1jA4sTqEUk2Ortrh+FvuLez8SLfV6nhPN36bYQtHYPkfJXDvTlVgq2ieYPZ/eIRXFPgPdLt0sPTzcKd0qNNxOJLqNWTyhHz043TC6slgtKbiwPhcpuymkSqOso6ujvaHet7EluDGjvN+hx6B4PGO1qHg2qaZh0y2nT0GqfDNkcnw3Y6Vh2GHKXnFG8VRxQuHbcUrxUGFjantXRA2AgADNgTAMDAAHYAY2o+lhZDIYHwIAASogAMKIECoVGsLEgbR8BFIlGo2QYlQ4tH4qlEkwk5HwPBozE0al4jHs+mgRlk/ESDn47iE4kwRFMlmooQYjQcilynkgPnMtEyuW4lSy0UM8Wk1XSrkc9U63l6yVqo24mV0sUSskUqnWymm5Xmsk0AmaglKlVSjRetHcH12/VS4OsDkSDGR33ug3BoQc4O23X2g0B9m44Pc0MW1EBp1Bl1x9P+jFJ3GF11+tEBjVoz2KvMe7W4z25tNhxsVjme2Q1+NSrWR70D0vd1Fays9pMT/MUmeGuctg34ge4/Errv5zGj1kY8erqUU/ec2PHtEUjeWo87skys8yu9mstorUi9sYkVEgEYEgAC4QAA9vgr6ThGHIBhe94ZjGfbwfO/LfhymI/penKfleKFIQaWpFlORq4cOLretu4ELvBzowRRZIBlhqKeuhsFSrSQpERh17yohGGYg26IcSxNJWthna0QaMpLtOg5vgWh5QTxQlyQxAaCsRaLRoKuLRsx4lStG/HBs2RJAA=="#;
        let expected_solution = r#"482617953591832647637945128268793415953124786174568239815279364329486571746351892"#;
        test_unqiue_solution_from_lzstring(&parser, lzstring, expected_solution)
    }

    #[test]
    fn test_irregular() {
        let parser = FPuzzlesParser::new();
        let lzstring = r#"N4IgzglgXgpiBcBOANCA5gJwgEwQbT1ADcBDAGwFc54BGVNCImAOwQBcMqBfZUDGBgHtWtHnwERhCGmNm8xIfkJEzeiiVNFqlklVwC6yQiFKVqAJnqMW7TjAU7Nq8coTmHGke+2e3H1/DewHLBhsamVAgAzFZMIhzcPgFB6sn+utHpmlFZIjlJGfD5LoXFqaUGRiWaAAyoEdQALLE28An28p3VeSG5CI198AMFmgNh3Qh1JuSR8ACsLfF2g8MTQ70jInMboVXltfUz1ABsi7aJa9ubCFeXg7f7W4PHz6/jj5OHZggA7Gdtyy6r2u8BeILBawhH3gP0GsPejhEUwaCAAHP92nCsUCutDYSD8btjIjJoNUWSKSDyVSdgZ9FwgA"#;
        let expected_solution = r#"198765432219876543321987654432198765543219876654321987765432198876543219987654321"#;
        test_unqiue_solution_from_lzstring(&parser, lzstring, expected_solution)
    }

    #[test]
    fn test_windowku() {
        let parser = FPuzzlesParser::new();
        let lzstring = r#"N4IgzglgXgpiBcBOANCA5gJwgEwQbT2AF9ljSQA3AQwBsBXOeAdlTQgpgDsEAXDBkmSGDBlWgwQAOVuy69+MEQF1khEaXVCx9RgEYZHbvD4CNZ4irXnN2ifACsBucYWbR1HQgDMToycUalsLWIcGgHnYAbL7yAkHu4owALDEupmE2EYwATKn+bvHkWQiO6LJ+rua2enmVYdXetenKquGJCCxlhrEBGaEN8CldzvlF7fDRwxXNhX1zLVZaxfA+Uz1uoQme8LlraYpKKiAwAB58VBgwbAD2RoQgAMYwNDRg+CAAStkAwtkgqF9vl5/p8fkkQR8vL8IVDgQCoeCAUloUigRDkeClKIni83vA8J9IijCWiAUTEZ8mMSPlS4ZTvhSPpJqcy6UyGSAseQca93h8iZEIUSmELvpIIVTBQCqSLpWKIcypZ9mbLlfKuaAeXiCYClYDVYDxfDvnqoQaoUbPsi9ciDcjxVisUA="#;
        let expected_solution = r#"497263185835491627126857934271938546348576291659142378782315469564729813913684752"#;
        test_unqiue_solution_from_lzstring(&parser, lzstring, expected_solution)
    }

    #[test]
    fn test_sudokux() {
        let parser = FPuzzlesParser::new();
        let lzstring = r#"N4IgzglgXgpiBcBOANCA5gJwgEwQbT1ADcBDAGwFc54A2VNCImAOwQBcMqBfZYHv3vxClK1ACz1GLdpxj95g3sPJUEADklNW8DtwC6yQguIrqAVk3SdsoSNXwA7Je265igSdEIJ6KS5vuXAZGgaEetqYITr5aMtyCwR7KXkjOcW7hSnbUGjFWrhEpAExp1vGZfInG1VmRtKUFtSkAjA0B4VVhyfatef7lhfbRDLFlGYPibQOdnvYWfek1s9TDfoth8jMTCCgLY0vd1L0j+e2H6lPjCYbLCADMl0vb8PMn/VcVQQYg2BAkaAB7ZjkADU6VQv3+QPIAFpFkA=="#;
        let expected_solution = r#"637945218925718463418623579591482637743596182862137945154879326279361854386254791"#;
        test_unqiue_solution_from_lzstring(&parser, lzstring, expected_solution)
    }
}
