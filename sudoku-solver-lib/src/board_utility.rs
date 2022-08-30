use bit_iter::BitIter;

pub const VALUE_SET_MASK: u32 = 1u32 << 31;
pub const CANDIDATES_MASK: u32 = !VALUE_SET_MASK;

pub fn value_count(mask: u32) -> usize {
    (mask & CANDIDATES_MASK).count_ones() as usize
}

// Assumes only one value remains
pub fn get_value(mask: u32) -> usize {
    ((mask & CANDIDATES_MASK).trailing_zeros() + 1) as usize
}

pub fn min_value(mask: u32) -> usize {
    ((mask & CANDIDATES_MASK).trailing_zeros() + 1) as usize
}

pub fn max_value(mask: u32) -> usize {
    32 - (mask & CANDIDATES_MASK).leading_zeros() as usize
}

pub fn is_value_set(mask: u32) -> bool {
    (mask & VALUE_SET_MASK) != 0
}

pub fn value_mask(val: usize) -> u32 {
    1u32 << (val - 1)
}

pub fn values_mask(vals: &[usize]) -> u32 {
    vals.iter().fold(0u32, |acc, &val| acc | value_mask(val))
}

pub fn all_values_mask(size: usize) -> u32 {
    (1u32 << size) - 1
}

pub fn has_value(mask: u32, val: usize) -> bool {
    (mask & value_mask(val)) != 0
}

pub fn mask_strictly_lower(val: usize) -> u32 {
    (1u32 << (val - 1)) - 1
}

pub fn mask_val_and_lower(val: usize) -> u32 {
    (1u32 << val) - 1
}

pub fn mask_strictly_higher(val: usize, all_values_mask: u32) -> u32 {
    all_values_mask & !mask_val_and_lower(val)
}

pub fn mask_val_and_higher(val: usize, all_values_mask: u32) -> u32 {
    all_values_mask & !mask_strictly_lower(val)
}

pub fn mask_between_inclusive(low: usize, high: usize, all_values_mask: u32) -> u32 {
    all_values_mask & !(mask_strictly_lower(low) | mask_strictly_higher(high, all_values_mask))
}

pub fn mask_between_exclusive(low: usize, high: usize, all_values_mask: u32) -> u32 {
    all_values_mask & !(mask_val_and_lower(low) | mask_val_and_higher(high, all_values_mask))
}

pub fn cell_index(row: usize, col: usize, size: usize) -> usize {
    row * size + col
}

pub fn cell_row_col(cell: usize, size: usize) -> (usize, usize) {
    (cell / size, cell % size)
}

pub fn candidate_index(cell: usize, val: usize, size: usize) -> usize {
    (cell * size) + val - 1
}

pub fn candidate_index_to_cell_and_value(candidate_index: usize, size: usize) -> (usize, usize) {
    (candidate_index / size, candidate_index % size + 1)
}

pub fn values_from_mask(mask: u32) -> impl Iterator<Item = usize> {
    BitIter::from(mask & CANDIDATES_MASK).map(|n| n + 1)
}

pub fn mask_to_string(mask: u32) -> String {
    let mut s = String::new();
    if mask != 0 {
        for val in values_from_mask(mask) {
            if s.len() > 0 {
                s.push(',');
            }
            s.push_str(&val.to_string());
        }
    }
    s
}

pub fn taxicab_distance(i0: usize, j0: usize, i1: usize, j1: usize) -> usize {
    let i0 = i0 as isize;
    let j0 = j0 as isize;
    let i1 = i1 as isize;
    let j1 = j1 as isize;
    ((i1 - i0).abs() + (j1 - j0).abs()) as usize
}

pub fn is_adjacent(i0: usize, j0: usize, i1: usize, j1: usize) -> bool {
    taxicab_distance(i0, j0, i1, j1) == 1
}

pub fn is_diagonal(i0: usize, j0: usize, i1: usize, j1: usize) -> bool {
    let i0 = i0 as isize;
    let j0 = j0 as isize;
    let i1 = i1 as isize;
    let j1 = j1 as isize;
    (i0 == i1 - 1 || i0 == i1 + 1) && (j0 == j1 - 1 || j0 == j1 + 1)
}

pub fn cell_name(cell: (usize, usize)) -> String {
    format!("r{}c{}", cell.0 + 1, cell.1 + 1)
}

pub fn cell_names(cells: &[(usize, usize)]) -> String {
    cells
        .iter()
        .map(|&cell| cell_name(cell))
        .collect::<Vec<_>>()
        .join(", ")
}

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

fn add_range(list: &mut Vec<usize>, start: usize, end: usize) -> Result<(), ()> {
    if start == 0 {
        return Err(());
    }

    if end == 0 {
        list.push(start);
    } else {
        let start = usize::min(start, end);
        let end = usize::max(start, end);
        for i in start..=end {
            list.push(i);
        }
    }

    Ok(())
}

fn add_cells(list: &mut Vec<usize>, rows: &[usize], cols: &[usize], size: usize) -> Result<(), ()> {
    for &r in rows {
        for &c in cols {
            if r == 0 || c == 0 || r > size || c > size {
                return Err(());
            }

            list.push(cell_index(r - 1, c - 1, size));
        }
    }

    Ok(())
}

pub fn adjacent_cells(cell: usize, size: usize) -> Vec<usize> {
    let (row, col) = cell_row_col(cell, size);
    let mut cells = Vec::new();
    if row > 0 {
        cells.push(cell_index(row - 1, col, size));
    }
    if row < size - 1 {
        cells.push(cell_index(row + 1, col, size));
    }
    if col > 0 {
        cells.push(cell_index(row, col - 1, size));
    }
    if col < size - 1 {
        cells.push(cell_index(row, col + 1, size));
    }
    cells.sort();
    cells
}

pub fn diagonal_cells(cell: usize, size: usize) -> Vec<usize> {
    let (row, col) = cell_row_col(cell, size);
    let mut cells = Vec::new();
    if row > 0 && col > 0 {
        cells.push(cell_index(row - 1, col - 1, size));
    }
    if row > 0 && col < size - 1 {
        cells.push(cell_index(row - 1, col + 1, size));
    }
    if row < size - 1 && col > 0 {
        cells.push(cell_index(row + 1, col - 1, size));
    }
    if row < size - 1 && col < size - 1 {
        cells.push(cell_index(row + 1, col + 1, size));
    }
    cells.sort();
    cells
}

pub fn parse_cells(cell_string: &str, size: usize) -> Result<Vec<Vec<usize>>, String> {
    let mut result = Vec::new();

    for cell_group in cell_string
        .split(";")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if !cell_group.is_ascii() {
            return Err(format!(
                "Invalid cell group (invalid characters): {}",
                cell_group
            ));
        }

        let err_msg = format!("Invalid cell group: {}", cell_group);
        let cell_group = cell_group.to_ascii_lowercase();
        let cell_group = cell_group.as_bytes();

        if cell_group.len() < 4 {
            return Err(err_msg);
        }

        let first_char = cell_group[0];
        if first_char != b'r' {
            return Err(err_msg);
        }

        let mut cells = Vec::new();

        let mut rows: Vec<usize> = Vec::new();
        let mut cols: Vec<usize> = Vec::new();
        let mut adding_rows = true;
        let mut value_start = true;
        let mut last_added_directions = false;
        let mut cur_val_start = 0;
        let mut cur_val_end = 0;
        let mut i = 1;
        while i < cell_group.len() {
            last_added_directions = false;

            let cur_char = cell_group[i];
            if cur_char == b'r' {
                if adding_rows
                    || add_range(&mut cols, cur_val_start, cur_val_end).is_err()
                    || add_cells(&mut cells, &rows, &cols, size).is_err()
                {
                    return Err(err_msg);
                }
                rows.clear();
                cols.clear();
                adding_rows = true;
                value_start = true;
                cur_val_start = 0;
                cur_val_end = 0;
            } else if cur_char == b'c' {
                if !adding_rows
                    || add_range(&mut rows, cur_val_start, cur_val_end).is_err()
                    || add_cells(&mut cells, &rows, &cols, size).is_err()
                {
                    return Err(err_msg);
                }
                adding_rows = false;
                value_start = true;
                cur_val_start = 0;
                cur_val_end = 0;
            } else if cur_char == b'd' {
                if adding_rows
                    || add_range(&mut cols, cur_val_start, cur_val_end).is_err()
                    || add_cells(&mut cells, &rows, &cols, size).is_err()
                {
                    return Err(err_msg);
                }
                rows.clear();
                cols.clear();
                adding_rows = true;
                value_start = true;
                cur_val_start = 0;
                cur_val_end = 0;

                if cells.len() == 0 {
                    return Err(err_msg);
                }

                i += 1;
                let mut complete = false;
                while i < cell_group.len() && !complete {
                    let cell = cells[cells.len() - 1];
                    let (r, c) = cell_row_col(cell, size);
                    let (r, c) = (r as isize, c as isize);
                    let dir_char = cell_group[i];
                    let mut to_add = (r, c);
                    match dir_char {
                        b'1' => to_add = (r + 1, c - 1),
                        b'2' => to_add = (r + 1, c),
                        b'3' => to_add = (r + 1, c + 1),
                        b'4' => to_add = (r, c - 1),
                        b'5' => to_add = (r, c),
                        b'6' => to_add = (r, c + 1),
                        b'7' => to_add = (r - 1, c - 1),
                        b'8' => to_add = (r - 1, c),
                        b'9' => to_add = (r - 1, c + 1),
                        b'r' => complete = true,
                        _ => {
                            return Err(err_msg);
                        }
                    }
                    if to_add.0 < 0
                        || to_add.0 > size as isize
                        || to_add.1 < 0
                        || to_add.1 > size as isize
                    {
                        return Err(err_msg);
                    }
                    cells.push(cell_index(to_add.0 as usize, to_add.1 as usize, size));
                    last_added_directions = true;
                    i += 1;
                }
                i -= 1;
            } else if cur_char >= b'0' && cur_char <= b'9' {
                if value_start {
                    cur_val_start = cur_val_start * 10 + (cur_char - b'0') as usize;
                } else {
                    cur_val_end = cur_val_end * 10 + (cur_char - b'0') as usize;
                }
            } else if cur_char == b'-' {
                if !value_start {
                    return Err(err_msg);
                }
                value_start = false;
            } else if cur_char == b',' {
                if add_range(
                    if adding_rows { &mut rows } else { &mut cols },
                    cur_val_start,
                    cur_val_end,
                )
                .is_err()
                {
                    return Err(err_msg);
                }
                value_start = true;
                cur_val_start = 0;
                cur_val_end = 0;
            } else {
                return Err(err_msg);
            }

            i += 1;
        }

        if !last_added_directions {
            if adding_rows
                || add_range(&mut cols, cur_val_start, cur_val_end).is_err()
                || add_cells(&mut cells, &rows, &cols, size).is_err()
                || cells.len() == 0
            {
                return Err(err_msg);
            }
        }

        result.push(cells);
    }

    result.sort();

    Result::Ok(result)
}

#[rustfmt::skip]
#[cfg(test)]
mod test {
    use itertools::assert_equal;

    use super::*;

	#[test]
	fn test_mask_to_string() {
		assert_eq!(mask_to_string(0b0000_0000_0000_0000_0000_0000_0000_0001), "1");
		assert_eq!(mask_to_string(0b0000_0000_0000_0000_0000_0000_0000_0011), "1,2");
		assert_eq!(mask_to_string(0b0000_0000_0000_0000_0000_0000_0001_0001), "1,5");
		assert_eq!(mask_to_string(0b1000_0000_0000_0000_0000_0000_0001_0001), "1,5");
		assert_eq!(mask_to_string(0b1000_0000_0000_0000_0000_0001_1111_1111), "1,2,3,4,5,6,7,8,9");
	}

	#[test]
	fn test_mask_ranges() {
		assert_eq!(all_values_mask(9), 0b0000_0000_0000_0000_0000_0001_1111_1111);
		assert_eq!(all_values_mask(16), 0b0000_0000_0000_0000_1111_1111_1111_1111);

		let avm = all_values_mask(9);
		assert_eq!(mask_strictly_lower(2), 0b0000_0000_0000_0000_0000_0000_0000_0001);
		assert_eq!(mask_strictly_lower(4), 0b0000_0000_0000_0000_0000_0000_0000_0111);
		assert_eq!(mask_val_and_lower(2), 0b0000_0000_0000_0000_0000_0000_0000_0011);
		assert_eq!(mask_val_and_lower(4), 0b0000_0000_0000_0000_0000_0000_0000_1111);
		assert_eq!(mask_strictly_higher(2, avm), 0b0000_0000_0000_0000_0000_0001_1111_1100);
		assert_eq!(mask_strictly_higher(4, avm), 0b0000_0000_0000_0000_0000_0001_1111_0000);
		assert_eq!(mask_val_and_higher(2, avm), 0b0000_0000_0000_0000_0000_0001_1111_1110);
		assert_eq!(mask_val_and_higher(4, avm), 0b0000_0000_0000_0000_0000_0001_1111_1000);
		assert_eq!(mask_between_exclusive(1, 5, avm), 0b0000_0000_0000_0000_0000_0000_0000_1110);
		assert_eq!(mask_between_inclusive(1, 5, avm), 0b0000_0000_0000_0000_0000_0000_0001_1111);
	}

	#[test]
	fn test_mask_values() {
		assert_eq!(get_value(value_mask(1)), 1);
		assert_eq!(get_value(value_mask(2)), 2);
		assert_eq!(get_value(value_mask(3)), 3);
		assert_eq!(get_value(value_mask(4)), 4);
		assert_eq!(get_value(value_mask(5)), 5);
		assert_eq!(get_value(value_mask(6)), 6);
		assert_eq!(get_value(value_mask(7)), 7);
		assert_eq!(get_value(value_mask(8)), 8);
		assert_eq!(get_value(value_mask(9)), 9);
		assert_eq!(get_value(0b0000_0000_0000_0000_0000_0000_0000_0001), 1);
		assert_eq!(get_value(0b0000_0000_0000_0000_0000_0000_0000_0010), 2);
		assert_eq!(get_value(0b0000_0000_0000_0000_0000_0001_0000_0000), 9);
		assert_eq!(get_value(0b1000_0000_0000_0000_0000_0001_0000_0000), 9);
		assert_eq!(min_value(0b0000_0000_0000_0000_0000_0001_1100_1000), 4);
		assert_eq!(max_value(0b0000_0000_0000_0000_0000_0001_1100_1000), 9);
		assert_eq!(max_value(0b1000_0000_0000_0000_0000_0001_1100_1000), 9);
		assert_eq!(min_value(values_mask(&[3,5,8])), 3);
		assert_eq!(max_value(values_mask(&[3,5,8])), 8);
		assert!(has_value(value_mask(3), 3));
		assert!(!has_value(values_mask(&[1,2,3,5,6,7,8,9]), 4));
	}

    #[test]
    fn test_mask_iterator() {
        assert_equal(values_from_mask(0), vec![]);
        assert_equal(values_from_mask(0b0000_0000_0000_0000_0000_0000_0000_0001), vec![1]);
        assert_equal(values_from_mask(0b1000_0000_0000_0000_0000_0000_0000_0001), vec![1]);
        assert_equal(values_from_mask(0b0000_0000_0000_0000_0000_0000_0000_0010), vec![2]);
        assert_equal(values_from_mask(0b0000_0000_0000_0000_0000_0000_0001_0010), vec![2, 5]);
        assert_equal(values_from_mask(0b0000_0000_0000_0000_0000_0001_1111_1111), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_equal(values_from_mask(0b1000_0000_0000_0000_0000_0001_1111_1111), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_equal(values_from_mask(values_mask(&[1, 4, 8])), vec![1, 4, 8]);
    }

	#[test]
	fn test_cell_index() {
		assert_eq!(cell_index(0, 0, 9), 0);
		assert_eq!(cell_index(1, 0, 9), 9);
		assert_eq!(cell_index(1, 0, 16), 16);
		assert_eq!(cell_index(1, 1, 8), 9);
		assert_eq!(cell_index(1, 1, 16), 17);
		assert_eq!(cell_index(1, 2, 16), 18);
		assert_eq!(cell_index(8, 8, 9), 80);
		assert_eq!(cell_index(4, 4, 9), 40);
	}

	#[test]
	fn test_cell_row_col() {
		assert_eq!(cell_row_col(0, 9), (0, 0));
		assert_eq!(cell_row_col(1, 9), (0, 1));
		assert_eq!(cell_row_col(16, 16), (1, 0));
		assert_eq!(cell_row_col(9, 8), (1, 1));
		assert_eq!(cell_row_col(17, 16), (1, 1));
		assert_eq!(cell_row_col(18, 16), (1, 2));
		assert_eq!(cell_row_col(80, 9), (8, 8));
		assert_eq!(cell_row_col(40, 9), (4, 4));
	}

	#[test]
	fn test_candidate_index() {
		assert_eq!(candidate_index(0, 1, 9), 0);
		assert_eq!(candidate_index(1, 1, 9), 9);
		assert_eq!(candidate_index(1, 2, 16), 17);
		assert_eq!(candidate_index(9, 2, 8), 73);
		assert_eq!(candidate_index(40, 5, 9), 364);
		assert_eq!(candidate_index(80, 9, 9), 728);
	}

    #[test]
    fn test_parse_cell_group() {
        assert_eq!(parse_cells("", 9), Result::Ok(vec![]));
        assert_eq!(parse_cells("r1c1", 9), Result::Ok(vec![vec![0]]));
        assert_eq!(parse_cells("R1C1", 9), Result::Ok(vec![vec![0]]));
        assert_eq!(parse_cells("r2c1", 9), Result::Ok(vec![vec![9]]));
        assert_eq!(parse_cells("r2c1", 4), Result::Ok(vec![vec![4]]));
        assert_eq!(parse_cells("r2c2", 9), Result::Ok(vec![vec![10]]));
        assert_eq!(parse_cells("r10c10", 10), Result::Ok(vec![vec![99]]));
        assert_eq!(parse_cells("r1-3c1-2", 9), Result::Ok(vec![vec![0, 1, 9, 10, 18, 19]]));
        assert_eq!(parse_cells("r1c1r2c2", 9), Result::Ok(vec![vec![0, 10]]));
        assert_eq!(parse_cells("r1c1d222", 9),Result::Ok(vec![vec![0, 9, 18, 27]]));
        assert_eq!(parse_cells("r1,3c1-2", 9),Result::Ok(vec![vec![0, 1, 18, 19]]));
        assert_eq!(parse_cells("r1c1;r2c2", 9),Result::Ok(vec![vec![0], vec![10]]));
        assert!(parse_cells("x", 9).is_err());
        assert!(parse_cells("x1c1", 9).is_err());
        assert!(parse_cells("r0c1", 9).is_err());
        assert!(parse_cells("r2c1d88", 9).is_err());
        assert!(parse_cells("r1-10c1", 9).is_err());
    }

    #[test]
    fn test_adjacent_cells() {
        assert_eq!(adjacent_cells(cell_index(0, 0, 9), 9), vec![1, 9]);
        assert_eq!(adjacent_cells(cell_index(0, 1, 9), 9), vec![0, 2, 10]);
        assert_eq!(adjacent_cells(cell_index(0, 2, 9), 9), vec![1, 3, 11]);
        assert_eq!(adjacent_cells(cell_index(0, 3, 9), 9), vec![2, 4, 12]);
        assert_eq!(adjacent_cells(cell_index(0, 4, 9), 9), vec![3, 5, 13]);
        assert_eq!(adjacent_cells(cell_index(0, 5, 9), 9), vec![4, 6, 14]);
        assert_eq!(adjacent_cells(cell_index(0, 6, 9), 9), vec![5, 7, 15]);
        assert_eq!(adjacent_cells(cell_index(0, 7, 9), 9), vec![6, 8, 16]);
        assert_eq!(adjacent_cells(cell_index(0, 8, 9), 9), vec![7, 17]);
        assert_eq!(adjacent_cells(cell_index(1, 0, 9), 9), vec![0, 10, 18]);
        assert_eq!(adjacent_cells(cell_index(4, 4, 9), 9), vec![cell_index(3, 4, 9), cell_index(4, 3, 9), cell_index(4, 5, 9), cell_index(5, 4, 9)]);
        assert_eq!(adjacent_cells(cell_index(8, 8, 9), 9), vec![cell_index(7, 8, 9), cell_index(8, 7, 9)]);
        assert_eq!(adjacent_cells(80, 9), vec![71, 79]);
    }

	#[test]
    fn test_diagonal_cells() {
        assert_eq!(diagonal_cells(cell_index(0, 0, 9), 9), vec![10]);
        assert_eq!(diagonal_cells(cell_index(0, 1, 9), 9), vec![9, 11]);
        assert_eq!(diagonal_cells(cell_index(0, 2, 9), 9), vec![10, 12]);
        assert_eq!(diagonal_cells(cell_index(0, 3, 9), 9), vec![11, 13]);
        assert_eq!(diagonal_cells(cell_index(0, 4, 9), 9), vec![12, 14]);
        assert_eq!(diagonal_cells(cell_index(0, 5, 9), 9), vec![13, 15]);
        assert_eq!(diagonal_cells(cell_index(0, 6, 9), 9), vec![14, 16]);
        assert_eq!(diagonal_cells(cell_index(0, 7, 9), 9), vec![15, 17]);
        assert_eq!(diagonal_cells(cell_index(0, 8, 9), 9), vec![16]);
        assert_eq!(diagonal_cells(cell_index(1, 0, 9), 9), vec![1, 19]);
        assert_eq!(diagonal_cells(cell_index(4, 4, 9), 9), vec![cell_index(3, 3, 9), cell_index(3, 5, 9), cell_index(5, 3, 9), cell_index(5, 5, 9)]);
        assert_eq!(diagonal_cells(cell_index(8, 8, 9), 9), vec![cell_index(7, 7, 9)]);
        assert_eq!(diagonal_cells(80, 9), vec![70]);
    }
}
