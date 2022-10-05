//! Contains [`CandidateLinks`] for storing a list of candidate links.

use crate::prelude::*;
use bitvec::prelude::*;

/// A collection to store whether two candidates are linked.
///
/// Can be used by both strong and weak links, or any other kind of
/// link.
///
/// Internally, a BitVec is used to store the links. The index of the
/// BitVec is the index of the candidate. The value of the BitVec is
/// whether the candidate is linked to the candidate with the index of
/// the BitVec.
#[derive(Clone, Debug)]
pub struct CandidateLinks {
    links: BitVec,
    size: usize,
}

impl CandidateLinks {
    /// Creates a new CandidateLinks with the correct number of candidates for the given
    /// board size.
    pub fn new(size: usize) -> Self {
        let num_candidates = size * size * size;
        Self { links: bitvec!(0; num_candidates), size }
    }

    /// Returns true if the candidate is linked
    pub fn is_linked(&self, candidate: CandidateIndex) -> bool {
        self.links[candidate.index()]
    }

    /// Returns true if there are no candidate links
    pub fn is_empty(&self) -> bool {
        self.links.iter().all(|x| !x)
    }

    /// Sets the link status for the given candidate.
    ///
    /// Returns true if the link status was changed.
    pub fn set(&mut self, candidate: CandidateIndex, value: bool) -> bool {
        if self.is_linked(candidate) == value {
            return false;
        }

        self.links.set(candidate.index(), value);

        true
    }

    /// Unions the candidates
    pub fn union(&mut self, other: &Self) {
        self.links |= &other.links;
    }

    /// Intersects the candidates
    pub fn intersect(&mut self, other: &Self) {
        self.links &= &other.links;
    }

    /// Returns an iterator over all the linked candidates
    pub fn links(&self) -> impl Iterator<Item = CandidateIndex> + '_ {
        let cu = CellUtility::new(self.size);

        self.links.iter().enumerate().filter_map(move |(i, b)| if *b { Some(cu.candidate_index(i)) } else { None })
    }
}

impl std::fmt::Display for CandidateLinks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CandidateLinks {{")?;
        for candidate in self.links() {
            write!(f, " {}", candidate)?;
        }
        write!(f, " }}")
    }
}
