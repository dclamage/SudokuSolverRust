//! Contains the [`TrueCandidatesCountResult`] enum.

use crate::prelude::*;

/// The result of running the true candidates with count solve.
#[derive(Clone)]
pub enum TrueCandidatesCountResult {
    None,
    Solved(Box<Board>),
    Candidates(Box<Board>, Vec<usize>),
    Error(String),
}

impl TrueCandidatesCountResult {
    pub fn is_none(&self) -> bool {
        matches!(self, TrueCandidatesCountResult::None)
    }

    pub fn is_candidates(&self) -> bool {
        matches!(self, TrueCandidatesCountResult::Candidates(_, _))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, TrueCandidatesCountResult::Error(_))
    }

    pub fn board(&self) -> Option<Box<Board>> {
        match self {
            TrueCandidatesCountResult::None | TrueCandidatesCountResult::Error(_) => None,
            TrueCandidatesCountResult::Solved(board) => Some(board.clone()),
            TrueCandidatesCountResult::Candidates(board, _) => Some(board.clone()),
        }
    }

    pub fn candidate_counts(&self) -> Option<Vec<usize>> {
        match self {
            TrueCandidatesCountResult::None
            | TrueCandidatesCountResult::Error(_)
            | TrueCandidatesCountResult::Solved(_) => None,
            TrueCandidatesCountResult::Candidates(_, candidate_counts) => Some(candidate_counts.clone()),
        }
    }
}
