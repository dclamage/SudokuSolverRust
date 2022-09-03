//! Contains the [`LogicResult`] enum for representing the result of a logic step.

/// This enum is used to represent the result of a logical step.
///
/// The result of a logical step can be one of the following:
/// * None: No change to the board.
/// * Changed: A change to the board.
/// * Invalid: A contradiction was found. Do not try to continue solving.
/// * Solved: The board is solved.
#[derive(Debug, PartialEq, Eq)]
pub enum LogicResult {
    None,
    Changed,
    Invalid,
    Solved,
}
