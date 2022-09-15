//! Contains the [`OrthogonalPairsConstraint`] struct for representing constraints where adjacent cells
//! must have certain number combinations.

use std::collections::{HashMap, HashSet};

use crate::prelude::*;
use itertools::Itertools;
use sudoku_solver_lib::prelude::*;

/// A [`Constraint`] implementation for representing constraints where adjacent cells must have
/// certain number combinations.
///
/// This constraint is useful for representing constraints like consecutive, anti-ratio, XV along
/// with their optional negative constraints.
#[derive(Debug)]
pub struct OrthogonalPairsConstraint {
    specific_name: String,
    markers: Vec<OrthogonalPairsMarker>,
    negative_constraints: Vec<String>,
    candidate_pairs: HashMap<String, Vec<ValueMask>>,
}

impl OrthogonalPairsConstraint {
    /// Creates a new [`OrthogonalPairsConstraint`] with the given parameters.
    pub fn new_with_candidate_pairs(
        specific_name: &str,
        markers: Vec<OrthogonalPairsMarker>,
        negative_constraints: &[&str],
        candidate_pairs: HashMap<String, Vec<ValueMask>>,
    ) -> Self {
        Self {
            specific_name: specific_name.to_owned(),
            markers,
            negative_constraints: negative_constraints.iter().map(|&s| s.to_owned()).collect(),
            candidate_pairs,
        }
    }

    /// Creates a new [`OrthogonalPairsConstraint`] with the given parameters
    /// and using a function to generate the candidate pairs.
    pub fn from_generic_markers_with_func(
        size: usize,
        specific_name: &str,
        markers: Vec<OrthogonalPairsMarker>,
        negative_constraints: &[&str],
        pair_allowed_func: impl Fn(&str, usize, usize) -> bool,
    ) -> Self {
        let mut candidate_pairs = HashMap::new();
        for marker_type in markers.iter().map(|m| m.marker_type.as_str()).unique() {
            let mut cur_candidate_pairs: Vec<ValueMask> = Vec::new();
            for i in 1..=size {
                let mut mask = ValueMask::new();
                for j in 1..=size {
                    if i != j && pair_allowed_func(marker_type, i, j) {
                        mask = mask.with(j);
                    }
                }
                cur_candidate_pairs.push(mask);
            }
            candidate_pairs.insert(marker_type.to_owned(), cur_candidate_pairs);
        }

        Self::new_with_candidate_pairs(
            specific_name,
            markers,
            negative_constraints,
            candidate_pairs,
        )
    }

    /// Creates a new [`OrthogonalPairsConstraint`] with the given parameters
    /// and standard marker types.
    pub fn from_standard_markers(
        size: usize,
        specific_name: &str,
        standard_markers: &[StandardOrthogonalPairsMarker],
        negative_constraints: &[StandardPairType],
    ) -> Self {
        let mut markers = Vec::new();
        let mut candidate_pairs = HashMap::new();

        for &marker in standard_markers {
            let type_name = marker.marker_type.name();
            if !candidate_pairs.contains_key(&type_name) {
                candidate_pairs.insert(type_name.clone(), marker.marker_type.candidate_pairs(size));
            }
            markers.push(marker.into());
        }
        for pair_type in negative_constraints {
            let type_name = pair_type.name();
            if !candidate_pairs.contains_key(&type_name) {
                candidate_pairs.insert(type_name.clone(), pair_type.candidate_pairs(size));
            }
        }

        let negative_constraints: Vec<String> =
            negative_constraints.iter().map(|&s| s.name()).collect();
        let negative_constraints: Vec<&str> =
            negative_constraints.iter().map(|s| s.as_str()).collect();

        Self::new_with_candidate_pairs(
            specific_name,
            markers,
            &negative_constraints,
            candidate_pairs,
        )
    }
}

impl Constraint for OrthogonalPairsConstraint {
    fn name(&self) -> &str {
        &self.specific_name
    }

    fn get_weak_links(&self, size: usize) -> Vec<(CandidateIndex, CandidateIndex)> {
        let cu = CellUtility::new(size);

        let mut result = Vec::new();

        let mut cell_pairs_seen = HashSet::new();
        for marker in &self.markers {
            if !self.negative_constraints.is_empty() {
                if marker.cell0 < marker.cell1 {
                    cell_pairs_seen.insert((marker.cell0, marker.cell1));
                } else {
                    cell_pairs_seen.insert((marker.cell1, marker.cell0));
                }
            }

            let candidate_pairs = self.candidate_pairs.get(marker.marker_type.as_str());
            if let Some(candidate_pairs) = candidate_pairs {
                for value in 1..=size {
                    let mask = candidate_pairs[value - 1].without(value);
                    if mask.is_empty() {
                        // This value isn't allowed on the marker at all. Eliminate it from both cells.
                        result.push((marker.cell0.candidate(value), marker.cell0.candidate(value)));
                        result.push((marker.cell1.candidate(value), marker.cell1.candidate(value)));
                    }

                    let inv_mask = !mask & ValueMask::from_all_values(size);
                    for other_value in inv_mask {
                        // This other value isn't allowed on the marker next to this value.
                        // Add a weak link between the two cells.
                        result.push((
                            marker.cell0.candidate(value),
                            marker.cell1.candidate(other_value),
                        ));
                        result.push((
                            marker.cell1.candidate(value),
                            marker.cell0.candidate(other_value),
                        ));
                    }
                }
            }
        }

        if !self.negative_constraints.is_empty() {
            let mut combined_candidate_pairs = vec![ValueMask::new(); size];
            for name in self.negative_constraints.iter() {
                let candidate_pairs = self.candidate_pairs.get(name.as_str());
                if let Some(candidate_pairs) = candidate_pairs {
                    for value in 1..=size {
                        combined_candidate_pairs[value - 1] =
                            combined_candidate_pairs[value - 1] | candidate_pairs[value - 1];
                    }
                }
            }

            for cell0 in cu.all_cells() {
                for cell1 in cell0.orthogonally_adjacent_cells() {
                    if cell0 > cell1 || cell_pairs_seen.contains(&(cell0, cell1)) {
                        continue;
                    }

                    for value in 1..=size {
                        let mask = combined_candidate_pairs[value - 1].without(value);
                        let inv_mask = !mask & ValueMask::from_all_values(size);
                        if inv_mask.is_empty() {
                            // This value isn't allowed off a marker at all. Eliminate it from both cells.
                            result.push((cell0.candidate(value), cell0.candidate(value)));
                            result.push((cell1.candidate(value), cell1.candidate(value)));
                        }

                        for other_value in mask {
                            // This other value isn't allowed off a marker next to this value.
                            // Add a weak link between the two cells.
                            result.push((cell0.candidate(value), cell1.candidate(other_value)));
                            result.push((cell1.candidate(value), cell0.candidate(other_value)));
                        }
                    }
                }
            }
        }

        result
    }
}

/// Represents a pair of cells that are adjacent to each other and have a marker between them.
#[derive(Debug, Clone)]
pub struct OrthogonalPairsMarker {
    marker_type: String,
    cell0: CellIndex,
    cell1: CellIndex,
}

impl OrthogonalPairsMarker {
    /// Creates a new [`OrthogonalPairsMarker`] with the given parameters.
    pub fn new(marker_type: &str, cell0: CellIndex, cell1: CellIndex) -> Self {
        Self {
            marker_type: marker_type.to_owned(),
            cell0,
            cell1,
        }
    }
}

/// Represents a pair of cells that are adjacent to each other and have a marker between them.
/// This only supports the "standard" marker types but can be used more conveniently.
#[derive(Debug, Clone, Copy)]
pub struct StandardOrthogonalPairsMarker {
    marker_type: StandardPairType,
    cell0: CellIndex,
    cell1: CellIndex,
}

impl StandardOrthogonalPairsMarker {
    /// Creates a new [`StandardOrthogonalPairsMarker`] with the given parameters.
    pub fn new(marker_type: StandardPairType, cell0: CellIndex, cell1: CellIndex) -> Self {
        Self {
            marker_type,
            cell0,
            cell1,
        }
    }

    /// Create a new [`StandardOrthogonalPairsMarker`] with a the given sum.
    pub fn sum(sum: usize, cell0: CellIndex, cell1: CellIndex) -> Self {
        Self::new(StandardPairType::Sum(sum), cell0, cell1)
    }

    /// Create a new [`StandardOrthogonalPairsMarker`] with a the given difference.
    pub fn difference(difference: usize, cell0: CellIndex, cell1: CellIndex) -> Self {
        Self::new(StandardPairType::Diff(difference), cell0, cell1)
    }

    /// Create a new [`StandardOrthogonalPairsMarker`] with a the given ratio.
    pub fn ratio(ratio: usize, cell0: CellIndex, cell1: CellIndex) -> Self {
        Self::new(StandardPairType::Ratio(ratio), cell0, cell1)
    }

    pub fn marker_type(&self) -> StandardPairType {
        self.marker_type
    }

    pub fn cell0(&self) -> CellIndex {
        self.cell0
    }

    pub fn cell1(&self) -> CellIndex {
        self.cell1
    }
}

impl From<StandardOrthogonalPairsMarker> for OrthogonalPairsMarker {
    fn from(marker: StandardOrthogonalPairsMarker) -> Self {
        match marker.marker_type {
            StandardPairType::Sum(n) => Self::new(&format!("s{}", n), marker.cell0, marker.cell1),
            StandardPairType::Diff(n) => Self::new(&format!("d{}", n), marker.cell0, marker.cell1),
            StandardPairType::Ratio(n) => Self::new(&format!("r{}", n), marker.cell0, marker.cell1),
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_antikropki_count() {
        let kropki_constraint = Arc::new(OrthogonalPairsConstraint::from_standard_markers(
            9,
            "Kropki",
            &[],
            &[StandardPairType::Diff(1), StandardPairType::Ratio(2)],
        ));
        let solver = SolverBuilder::default()
            .with_constraint(kropki_constraint)
            .build()
            .unwrap();

        let solution_count = solver.find_solution_count(10000, None, None);
        assert!(solution_count.is_exact_count());
        assert_eq!(solution_count.count().unwrap(), 8448);
    }

    #[test]
    fn test_single_dot_kropki_count() {
        let size = 9;
        let cu = CellUtility::new(size);
        let cell0 = cu.cell(5, 7);
        let cell1 = cu.cell(6, 7);
        let marker = StandardOrthogonalPairsMarker::ratio(2, cell0, cell1);
        let kropki_constraint = Arc::new(OrthogonalPairsConstraint::from_standard_markers(
            size,
            "Kropki",
            &[marker],
            &[StandardPairType::Diff(1), StandardPairType::Ratio(2)],
        ));
        let solver = SolverBuilder::default()
            .with_constraint(kropki_constraint)
            .build()
            .unwrap();

        let solution_count = solver.find_solution_count(2, None, None);
        assert!(solution_count.is_exact_count());
        assert_eq!(solution_count.count().unwrap(), 1);
    }

    #[test]
    fn test_sum() {
        let size = 9;
        let cu = CellUtility::new(size);
        let cell0 = cu.cell(0, 0);
        let cell1 = cu.cell(0, 1);
        let marker = StandardOrthogonalPairsMarker::sum(10, cell0, cell1);
        let xv_constraint = Arc::new(OrthogonalPairsConstraint::from_standard_markers(
            size,
            "XV",
            &[marker],
            &[],
        ));
        let solver = SolverBuilder::default()
            .with_constraint(xv_constraint.clone())
            .build()
            .unwrap();
        assert_eq!(solver.board().cell(cell0).count(), size - 1);
        assert!(!solver.board().cell(cell0).has(5));
        assert_eq!(solver.board().cell(cell1).count(), size - 1);
        assert!(!solver.board().cell(cell1).has(5));

        let solver = SolverBuilder::default()
            .with_constraint(xv_constraint)
            .with_given(cell0, 2)
            .build()
            .unwrap();
        assert_eq!(solver.board().cell(cell1).count(), 1);
        assert_eq!(solver.board().cell(cell1).value(), 8);

        let marker = StandardOrthogonalPairsMarker::sum(5, cell0, cell1);
        let xv_constraint = Arc::new(OrthogonalPairsConstraint::from_standard_markers(
            size,
            "XV",
            &[marker],
            &[],
        ));
        let solver = SolverBuilder::default()
            .with_constraint(xv_constraint)
            .build()
            .unwrap();
        assert_eq!(solver.board().cell(cell0).count(), 4);
        assert_eq!(solver.board().cell(cell0), ValueMask::from_lower_equal(4));
        assert_eq!(solver.board().cell(cell1).count(), 4);
        assert_eq!(solver.board().cell(cell1), ValueMask::from_lower_equal(4));
    }
}
