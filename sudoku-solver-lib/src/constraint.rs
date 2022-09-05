//! Contains the [`Constraint`] trait for defining the behavior of additional constraints.

use crate::prelude::*;
use std::vec::Vec;

/// Constraints are used by variant sudokus to define extra rules
/// beyond the standard Sudoku rules.
///
/// Most of the methods in this trait are optional, but aside from very
/// simple constraints, most will override most or all of them.
///
/// Some methods provide an optional implementation for convenience:
/// - [`Constraint::cells_must_contain`] can call [`Constraint::cells_must_contain_by_running_logic`]
/// to automatically determine the answer based on running the [`Constraint::step_logic`] method.
///
/// - [`Constraint::get_weak_links`] can call [`get_weak_links_for_nonrepeat`]
/// to automatically generate weak links based on the constraint having cells
/// which cannot repeat a value.
pub trait Constraint: std::any::Any {
    /// A generic name for the constaint which is independent of how it was intialized.
    fn name(&self) -> String;

    /// Override if there is a more specific name for this constraint instance,
    /// such as "Killer Cage at r1c1".
    fn specific_name(&self) -> String {
        self.name()
    }

    /// Called when the board is initially created to give the constraint the opportunity
    /// to do obvious modifications to the board based on the constraint which the end-user
    /// would not care to be reported about.
    ///
    /// For example, a killer cage could remove obvious candidates that would make the overall sum
    /// too large.
    ///
    /// Avoid doing any logic that the end-user may not understand why it happened.
    ///
    /// This method may be called multiple times, but only during board creation.
    /// It is called on all constraints until all of them return [`LogicalStepResult::None`].
    /// This allows them react to each other and how they may have changed the board.
    ///
    /// Return the following based on the situation (Description is ignored and can be None):
    /// - [`LogicalStepResult::None`] if the board is unchanged.
    /// - [`LogicalStepResult::Changed`] if the board is changed.
    /// - [`LogicalStepResult::Invalid`] if this constraint has made the solve impossible.
    /// - All other values are treated as [`LogicalStepResult::None`].
    fn init_board(&self, board: &mut Board) -> LogicalStepResult {
        let _ = board;
        LogicalStepResult::None
    }

    /// Called when a value has just been set on the board.
    /// The job of this function is to determine if setting this value is a violation of the constraint.
    ///   
    /// **Avoid complex logic in this function.** Just enforcement of the direct, actual rule is advised.
    /// For example, a Killer Cage would do nothing if all values in the cage are not yet set, and otherwise
    /// would check the sum of the values against the desired cage sum.
    ///
    /// The board is immutable in this function. Any changes to the board should be enforced via the
    /// [`Constraint::get_weak_links`] method and/or the [`Constraint::step_logic`] method.
    ///
    /// All weak links will be applied before this function is called.
    ///
    /// Return the following based on the situation:
    /// - [`LogicalStepResult::None`] if the constraint is not violated.
    /// - [`LogicalStepResult::Invalid`] if the constraint is violated.
    fn enforce(&self, board: &Board, cell: CellIndex, val: usize) -> LogicalStepResult {
        let (_, _, _) = (board, cell, val);
        LogicalStepResult::None
    }

    /// Called during logical solving.
    /// Go through the board and perform a single step of logic related to this constraint.
    /// For example, a Killer Cage constraint may check which candidates are still possible
    /// based on the desired sum and remove any which are not.
    ///
    /// Use your judgement and testing to determine if any of the logic should occur during brute force
    /// solving. The brute force solving boolean is set to true when this logic is not going to be
    /// visible to the end-user and so anything done during brute forcing is only advised if it's faster
    /// than guessing.
    ///
    /// Do not attempt to do any logic which isn't relevant to this constraint.
    ///
    /// Any eliminations should be tracked and added to the returned [`LogicalStepResult`],
    /// along with a human readable description of why those eliminations occurred.
    ///
    /// Eliminations do not need to be tracked if the brute forcing boolean is set to true.
    ///
    /// Return the following based on the situation. You must track this yourself and return an accurate [`LogicalStepResult`]:
    /// - [`LogicalStepResult::None`] if the board is unchanged.
    /// - [`LogicalStepResult::Changed`] if the board is changed.
    /// - [`LogicalStepResult::Invalid`] if this constraint can no longer be satisfied.
    fn step_logic(&self, board: &mut Board, is_brute_forcing: bool) -> LogicalStepResult {
        let (_, _) = (board, is_brute_forcing);
        LogicalStepResult::None
    }

    /// Return a vector of cells which must contain the given value.
    ///
    /// For example, a Killer Cage may determine that there must be a 9 in one of the cells
    /// in order to fulfill the sum. This would return a [`Vec`] of all the cells in the cage
    /// which can still be 9.
    fn cells_must_contain(&self, board: &Board, val: usize) -> Vec<CellIndex> {
        let (_, _) = (board, val);
        Vec::new()
    }

    /// Can be used by [`Constraint::cells_must_contain`] to automatically determine the
    /// answer based on running the [`Constraint::step_logic`] method.
    ///
    /// This is determined by cloning the board, and then removing the given value from all
    /// cells in the constraint and then running the [`Constraint::step_logic`] method to see
    /// if it returns [`LogicalStepResult::Invalid`].
    fn cells_must_contain_by_running_logic(
        &self,
        board: &mut Board,
        cells: &[CellIndex],
        value: usize,
    ) -> Vec<CellIndex> {
        let mut result = Vec::new();

        for &cell in cells {
            let mask = board.cell(cell);
            if mask.is_single() || !mask.has(value) {
                continue;
            }

            result.push(cell);
        }

        if !result.is_empty() {
            let mut board_clone = board.clone();
            for &cell in &result {
                board_clone.clear_value(cell, value);
            }

            let mut logic_result = LogicalStepResult::Changed(None);
            while logic_result.is_changed() {
                logic_result = self.step_logic(&mut board_clone, true);
            }

            if !logic_result.is_invalid() {
                result.clear();
            }
        }

        result
    }

    /// A weak link is a relationship between candidates A and B which may be in different
    /// cells which is equivalent to the logic `A → !B`.
    ///
    /// Essentially, if A is true, then B must be false and so is eliminated.
    ///
    /// Return a [`Vec`] of candidate pairs which form a weak links. Weak links eliminations
    /// are assumed to be symmetrical, so if `A → !B` then `B → !A`, so only `(A, B)` or `(B, A)`
    /// is necessary to include, not both. It is not harmful to include both, however.
    ///
    /// For example, a nonconsecutive constraint would return that the candidate 1 in r1c1 has
    /// a weak link to the candidate 2 in r1c2 (among others).
    ///
    /// Including a weak link of a candidate to itself `(A, A)` tells the solver that this
    /// candidate is never possible and it is immediately eliminated.
    ///
    /// The solver can quickly figure out both cell and region forcing eliminations using these
    /// weak links. A cell forcing elimination is when all candidates remaining in a cell all
    /// have a weak link to the same candidate, so that candidate can be eliminated. A region forcing
    /// elimination is when all instances of a value remaining in a region all have a weak link
    /// to the same candidate, so that candidate can be eliminated.
    ///
    /// As a result, proper generation of weak links means that some logic can be omitted from the
    /// [`Constraint::step_logic`] method. For example, a nonconsecutive constraint does not need
    /// to check if a cell has only `1,2` left, which elimiates `1,2` from adjacent cells. The solver
    /// will figure this out itself via cell forcing.
    ///
    /// [`crate::cell_utility::CellUtility::candidate_pairs`] is especially useful for this method, which generates all candidates
    /// pairs for all values within a group of cells. Passing in a group of cells which cannot
    /// repeat will generate the needed weak link pairs for that group.
    fn get_weak_links(&self, size: usize) -> Vec<(CandidateIndex, CandidateIndex)> {
        let _ = size;
        Vec::new()
    }

    /// Some contraints essentially create new houses. For example, an extra region
    /// constraint, or a Killer Cage sized such that it must contain every digit.
    /// Even constraints like a Renban can be considered to create new houses if
    /// they are of the correct size.
    ///
    /// This method returns a [`Vec`] of [`House`] which are created by the constraint.
    ///
    /// The size of the board is passed in so that the constraint can know the size of the house.
    fn get_houses(&self, size: usize) -> Vec<House> {
        let _ = size;
        Vec::new()
    }
}
