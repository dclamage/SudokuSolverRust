//! Provides some commonly needed math functions.

use crate::prelude::*;
use itertools::Itertools;

/// Returns the binoomial coefficient of `n` choose `k`.
///
/// Useful for computing the number of combinations of `k` items
/// from a set of `n` items.
///
/// # Example
/// ```
/// # use sudoku_solver_lib::math::binomial_coefficient;
/// assert_eq!(binomial_coefficient(5, 1), 5);
/// assert_eq!(binomial_coefficient(5, 2), 10);
/// assert_eq!(binomial_coefficient(5, 3), 10);
/// assert_eq!(binomial_coefficient(5, 4), 5);
/// assert_eq!(binomial_coefficient(5, 5), 1);
///
/// assert_eq!(binomial_coefficient(10, 1), 10);
/// assert_eq!(binomial_coefficient(10, 2), 45);
/// assert_eq!(binomial_coefficient(10, 3), 120);
/// assert_eq!(binomial_coefficient(10, 4), 210);
/// assert_eq!(binomial_coefficient(10, 5), 252);
/// assert_eq!(binomial_coefficient(10, 6), 210);
/// assert_eq!(binomial_coefficient(10, 7), 120);
/// assert_eq!(binomial_coefficient(10, 8), 45);
/// assert_eq!(binomial_coefficient(10, 9), 10);
/// assert_eq!(binomial_coefficient(10, 10), 1);
/// ```
pub fn binomial_coefficient(n: usize, k: usize) -> usize {
    if k > n {
        0
    } else if k == 0 || k == n {
        1
    } else if k == 1 || k == n - 1 {
        n
    } else if k + k < n {
        (binomial_coefficient(n - 1, k - 1) * n) / k
    } else {
        (binomial_coefficient(n - 1, k) * n) / (n - k)
    }
}

/// Returns the default regions assignments for a board of the given size.
///
/// This is a flat list of which region index each cell belongs to.
///
/// # Example
/// ```
/// # use sudoku_solver_lib::math::default_regions;
/// let regions = default_regions(9);
/// assert_eq!(regions.len(), 81);
/// assert_eq!(regions, vec![
///     0, 0, 0, 1, 1, 1, 2, 2, 2,
///     0, 0, 0, 1, 1, 1, 2, 2, 2,
///     0, 0, 0, 1, 1, 1, 2, 2, 2,
///     3, 3, 3, 4, 4, 4, 5, 5, 5,
///     3, 3, 3, 4, 4, 4, 5, 5, 5,
///     3, 3, 3, 4, 4, 4, 5, 5, 5,
///     6, 6, 6, 7, 7, 7, 8, 8, 8,
///     6, 6, 6, 7, 7, 7, 8, 8, 8,
///     6, 6, 6, 7, 7, 7, 8, 8, 8,
/// ]);
///
/// let regions = default_regions(6);
/// assert_eq!(regions.len(), 36);
/// assert_eq!(regions, vec![
///     0, 0, 0, 1, 1, 1,
///     0, 0, 0, 1, 1, 1,
///     2, 2, 2, 3, 3, 3,
///     2, 2, 2, 3, 3, 3,
///     4, 4, 4, 5, 5, 5,
///     4, 4, 4, 5, 5, 5,
/// ]);
/// ```
pub fn default_regions(size: usize) -> Vec<usize> {
    if size == 0 {
        return Vec::new();
    }

    let mut regions = Vec::new();
    regions.reserve(size * size);

    let mut region_height = (size as f64).sqrt().floor() as usize;
    while size % region_height != 0 {
        region_height -= 1;
    }

    let region_width = size / region_height;
    for i in 0..size {
        for j in 0..size {
            regions.push((i / region_height) * region_height + (j / region_width));
        }
    }

    regions
}

/// Utility function to generate the weak links for a group of cells where the same digit
/// cannot repeat in the group.
pub fn get_weak_links_for_nonrepeat(
    group: impl Iterator<Item = CellIndex> + Clone,
) -> Vec<(CandidateIndex, CandidateIndex)> {
    group
        .tuple_combinations()
        .flat_map(move |(cell1, cell2)| {
            (1..=cell1.size()).map(move |value| (cell1.candidate(value), cell2.candidate(value)))
        })
        .collect()
}

/// Utility function to generate the weak links for a group of cells where the value
/// must be the same for every cell the group.
pub fn get_weak_links_for_clone(
    group: impl Iterator<Item = CellIndex> + Clone,
) -> Vec<(CandidateIndex, CandidateIndex)> {
    group
        .tuple_combinations()
        .flat_map(move |(cell1, cell2)| {
            (1..=cell1.size()).flat_map(move |value1| {
                (1..=cell1.size())
                    .filter(move |value2| value1 != *value2)
                    .map(move |value2| (cell1.candidate(value1), cell2.candidate(value2)))
            })
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_weak_links_for_nonrepeat() {
        let cu = CellUtility::new(6);
        let cell0 = cu.cell(0, 0);
        let cell1 = cu.cell(0, 1);
        let cell2 = cu.cell(0, 2);
        let links = get_weak_links_for_nonrepeat(vec![cell0, cell1, cell2].into_iter());
        assert_eq!(links.len(), 18);
        for val in 1..=6 {
            assert!(links.contains(&(cell0.candidate(val), cell1.candidate(val))));
            assert!(links.contains(&(cell0.candidate(val), cell2.candidate(val))));
            assert!(links.contains(&(cell1.candidate(val), cell2.candidate(val))));
        }
    }

    #[test]
    fn test_weak_links_for_clone() {
        let cu = CellUtility::new(6);
        let cell0 = cu.cell(0, 0);
        let cell1 = cu.cell(0, 1);
        let cell2 = cu.cell(0, 2);
        let links = get_weak_links_for_clone(vec![cell0, cell1, cell2].into_iter());
        assert_eq!(links.len(), 6 * 3 * 5);
        for val in 1..=6 {
            assert!(!links.contains(&(cell0.candidate(val), cell1.candidate(val))));
            assert!(!links.contains(&(cell0.candidate(val), cell2.candidate(val))));
            assert!(!links.contains(&(cell1.candidate(val), cell2.candidate(val))));
        }
    }
}
