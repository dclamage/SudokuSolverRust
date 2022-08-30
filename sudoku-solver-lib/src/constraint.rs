use crate::board::Board;
use crate::board_utility::*;
use crate::logic_result::LogicResult;
use crate::logical_steps::LogicalSteps;
use std::vec::Vec;

/// [`Constraint`] is a trait that defines the logic of a constraint.
/// Constraints are used by variant Sudokus to define extra rules
/// beyond the standard Sudoku rules.
///
/// Most of the methods in this trait are optional, but aside from very
/// simple constraints, most will override most or all of them.
///
/// Some methods provide an optional implementation for convenience:
/// - [`Constraint::cells_must_contain`] can call [`Constraint::cells_must_contain_by_running_logic`]
/// to automatically determine the answer based on running the [`Constraint::step_logic`] method.
///
/// - [`Constraint::get_weak_links`] can call [`Constraint::get_weak_links_by_running_logic`]
/// to automatically generate weak links based on running the [`Constraint::enforce`]
/// and [`Constraint::step_logic`] methods.
pub trait Constraint {
    /// A generic name for the constaint which is independent of how it was intialized.
    fn name(&self) -> String;

    /// Override if there is a more specific name for this constraint instance,
    /// such as "Killer Cage at r1c1".
    fn specific_name(&self) -> String {
        self.name().to_string()
    }

    /// Called once passing in the [`Board`] so the constaint can initialize itself based
    /// on the board properties and all other constraints on the board.
    ///
    /// This method may be called multiple times, but only during board creation.
    /// It is called on all constraints until all of them return [`LogicResult::None`].
    /// This allows them react to each other and how they may have changed the board.
    ///
    /// Return the following based on the situation:
    /// - [`LogicResult::None`] if the board is unchanged.
    /// - [`LogicResult::Changed`] if the board is changed.
    /// - [`LogicResult::Invalid`] if this constraint has made the solve impossible.
    /// - All other values are treated as [`LogicResult::None`].
    fn init(&mut self, _board: &mut Board) -> LogicResult {
        LogicResult::None
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
    /// - [`LogicResult::None`] if the constraint is not violated.
    /// - [`LogicResult::Invalid`] if the constraint is violated.
    /// - All other values are treated as [`LogicResult::None`].
    fn enforce(&self, _board: &Board, _cell: usize, _val: usize) -> LogicResult {
        LogicResult::None
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
    /// Any eliminations should be tracked and added to the [`LogicalSteps`] object if provided,
    /// along with a human readable description of why those eliminations occurred.
    ///
    /// Eliminations do not need to be tracked if the [`LogicalSteps`] object is not provided.
    ///
    /// Return the following based on the situation. You must track this yourself and return an accurate [`LogicResult`]:
    /// - [`LogicResult::None`] if the board is unchanged.
    /// - [`LogicResult::Changed`] if the board is changed.
    /// - [`LogicResult::Invalid`] if this constraint can no longer be satisfied.
    /// - All other values are treated as [`LogicResult::None`].
    fn step_logic(
        &self,
        _board: &mut Board,
        _logical_steps: Option<&mut LogicalSteps>,
        _is_brute_forcing: bool,
    ) -> LogicResult {
        LogicResult::None
    }

    /// Return a [`Vec`] of cell indices which must contain the given value.
    ///
    /// For example, a Killer Cage may determine that there must be a 9 in one of the cells
    /// in order to fulfill the sum. This would return a [`Vec`] of all the cells in the cage
    /// which can still be 9.
    fn cells_must_contain(&self, _board: &Board, _val: usize) -> Vec<usize> {
        Vec::new()
    }

    /// **Do not override or call directly.**
    ///
    /// Can be used by [`Constraint::cells_must_contain`] to automatically determine the
    /// answer based on running the [`Constraint::step_logic`] method.
    ///
    /// This is determined by cloning the board, and then removing the given value from all
    /// cells in the constraint and then running the [`Constraint::step_logic`] method to see
    /// if it returns [`LogicResult::Invalid`].
    fn cells_must_contain_by_running_logic(
        &self,
        board: &mut Board,
        cells: &[usize],
        value: usize,
    ) -> Vec<usize> {
        let mut result = Vec::new();

        for &cell in cells {
            let mask = board.get_cell_mask(cell);
            if value_count(mask) <= 1 || !has_value(mask, value) {
                continue;
            }

            result.push(cell);
        }

        if result.len() > 0 {
            let mut board_clone = board.clone();
            for &cell in &result {
                board_clone.clear_value(cell, value);
            }

            let mut logic_result = LogicResult::Changed;
            while logic_result == LogicResult::Changed {
                logic_result = self.step_logic(&mut board_clone, Option::None, false);
            }

            if logic_result != LogicResult::Invalid {
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
    /// Use [`candidate_index`] to calculate the candidate index of a value within a cell.
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
    fn get_weak_links(&self) -> Vec<(usize, usize)> {
        Vec::new()
    }

    /// **Do not override or call directly.**
    ///
    /// Can be used by [`Constraint::get_weak_links`] to automatically determine the
    /// answer based on running the [`Constraint::step_logic`] method.
    ///
    /// This is determined by setting each candidate in each cell one at a time to a cloned board,
    /// then running the [`Constraint::step_logic`] method to see if it returns [`LogicResult::Invalid`].
    fn get_weak_links_by_running_logic(
        &self,
        board: &Board,
        cells: &[usize],
    ) -> Vec<(usize, usize)> {
        let size = board.size();
        let mut result = Vec::new();

        for &cell in cells {
            let orig_mask = board.get_cell_mask(cell);
            if value_count(orig_mask) <= 1 {
                continue;
            }

            for val in values_from_mask(orig_mask) {
                let cand0 = candidate_index(cell, val, size);

                let mut board_clone = board.clone();
                if !board_clone.set_value(cell, val) {
                    // A weak link to self indicates that the candidate is generally invalid
                    result.push((cand0, cand0));
                    continue;
                }

                let mut logic_result = LogicResult::Changed;
                while logic_result == LogicResult::Changed {
                    logic_result = self.step_logic(&mut board_clone, Option::None, false);
                }

                if logic_result == LogicResult::Invalid {
                    // A weak link to self indicates that the candidate is generally invalid
                    result.push((cand0, cand0));
                    continue;
                }

                for &cell1 in cells.iter() {
                    if cell == cell1 {
                        continue;
                    }

                    let orig_mask1 = board.get_cell_mask(cell1) & CANDIDATES_MASK;
                    let new_mask1 = board_clone.get_cell_mask(cell1) & CANDIDATES_MASK;
                    if orig_mask1 != new_mask1 {
                        let diff_mask = orig_mask1 & !new_mask1;
                        for val1 in values_from_mask(diff_mask) {
                            let cand1 = candidate_index(cell1, val1, size);
                            result.push((cand0, cand1));
                        }
                    }
                }
            }
        }

        result
    }
}
