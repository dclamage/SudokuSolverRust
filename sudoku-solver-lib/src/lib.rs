//! This library contains the core logic for solving a variant Sudoku puzzle.
//!
//! The [`crate::solver::Solver`] struct is the main entry point for solving a puzzle.
//! It contains a [`crate::board::Board`] which represents the current state of the puzzle to be solved.
//! The logic of the solver can be expanded using the [`crate::logical_step::LogicalStep`] trait.
//! This libary contains some basic implementations of this trait for core functionality which
//! cannot be disabled. Additional implementations can be added by the consumer of this library
//! to logically solve more complex puzzles.
//!
//! [`crate::constraint::Constraint`] is a trait that defines the logic of a variant constraint.
//! This library does not provide any implementations of this trait, and instead relies on the
//! consumer of this library to provide the constraints for the puzzle to be solved.

pub mod board;
pub mod candidate_index;
pub mod candidate_links;
pub mod cell_index;
pub mod cell_utility;
pub mod constraint;
pub mod elimination_list;
pub mod house;
pub mod logic_result;
pub mod logical_step;
pub mod logical_step_desc;
pub mod math;
pub mod prelude;
pub mod solver;
pub mod value_mask;
