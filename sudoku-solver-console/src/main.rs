mod listener;

use std::io::{self, BufRead, Write};

use clap::Parser;
use colored::Colorize;

use sudoku_solver_lib::prelude::*;

#[derive(Debug, Parser)]
#[clap(name = "Sudoku Solver")]
#[clap(about = r#"Sudoku variant solver utility.

GitHub: https://github.com/dclamage/SudokuSolverRust
Patreon: https://www.patreon.com/rangsk
YouTube: https://www.youtube.com/rangsk"#)]
#[clap(author, version, long_about = None)]
struct Args {
    /// Listen for websocket connections
    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    listen: bool,
    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    benchmark: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.benchmark {
        // Declare a counts array
        let mut counts = [0; 2];

        // Read from stdin until EOF
        let stdin = io::stdin();
        let mut solver = SolverBuilder::default()
            .build()
            .unwrap();
        for line in stdin.lock().lines() {
            let line = line.expect("Failed to read line");
            let puzzle = line.trim_end();
            assert!(puzzle.len() == 81);
            
            solver.reset();

            let mut givens_invalid = false;
            for (i, c) in puzzle.chars().enumerate() {
                if ('1'..='9').contains(&c) {
                    let value = (c as u8) - b'0';
                    if !solver.set_given(CellIndex::new(i, 9), value as usize) {
                        givens_invalid = true;
                        break;
                    }
                }
            }
            if givens_invalid {
                counts[0] += 1;
                continue;
            }

            let result = solver.run_singles_only();
            counts[result as usize] += 1;
        }
        let valid = counts[1];
        let invalid = counts[0];
        println!("Valid: {valid}");
        println!("Invalid: {invalid}");
        std::io::stdout().flush().unwrap();
        return;
    }

    const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    println!("{} {}", "Sudoku Solver".to_owned().green(), VERSION);
    println!("{AUTHOR}");
    println!("Sudoku variant solver utility.");
    println!();
    println!("GitHub: https://github.com/dclamage/SudokuSolverRust");
    println!("Patreon: https://www.patreon.com/rangsk");
    println!("YouTube: https://www.youtube.com/rangsk");
    println!();

    if args.listen {
        listener::listen().await;
    } else {
        println!("No arguments provided. Use --help for more information.");
    }
}
