mod listener;

use clap::Parser;
use colored::Colorize;

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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

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
