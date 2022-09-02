use itertools::Itertools;

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

pub fn cell_index_name(cell: usize, size: usize) -> String {
    let (row, col) = cell_row_col(cell, size);
    format!("r{}c{}", row + 1, col + 1)
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

/// Returns a list of candidate pairs for each value within the given set of cells.
pub fn get_candidate_pairs(size: usize, cells: &[usize]) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    for val in 1..=size {
        for cell_pair in cells.iter().combinations(2) {
            let cand0 = candidate_index(*cell_pair[0], val, size);
            let cand1 = candidate_index(*cell_pair[1], val, size);
            result.push((cand0, cand1));
        }
    }
    result
}

/// Generates a compact description of a group of cells.
///
/// # Arguments
/// - `cells` - The cells to describe.
/// - `size` - The size of the grid.
///
/// # Returns
/// A string describing the cells.
/// - If they all share a row, then it returns for example `r1c123`
/// - If they all share a column, then it returns for example `r123c1`
/// - Otherwise, the cells are separated into groups like `r1c123,r2c123,r3c123`
///
/// # Example
/// ```
/// # use sudoku_solver_lib::board_utility::*;
/// // Assume a 9x9 grid.
/// let size = 9;
///
/// // Create a list of the following cells: r1c1, r1c2, r1c3
/// // (Cell indices are 0-based)
/// let cells = vec![cell_index(0, 0, size), cell_index(0, 1, size), cell_index(0, 2, size)];
///
/// // Get the compact name.
/// let compact_name = compact_name(&cells, size);
/// assert_eq!(compact_name, "r1c123");
/// ```
pub fn compact_name(cells: &[usize], size: usize) -> String {
    let cell_separator = if size <= 9 { "" } else { "," };
    let group_separator = ",";

    if cells.len() == 0 {
        return "".to_string();
    }

    if cells.len() == 1 {
        return cell_index_name(cells[0], size);
    }

    let cells: Vec<(usize, usize)> = cells
        .iter()
        .sorted()
        .map(|cell| cell_row_col(*cell, size))
        .collect();

    // If all share a row, group all by row
    let first_row = cells[0].0;
    if cells.iter().all(|cell| cell.0 == first_row) {
        return format!(
            "r{}c{}",
            cells[0].0 + 1,
            cells
                .iter()
                .map(|cell| cell.1 + 1)
                .sorted()
                .join(&cell_separator)
        );
    }

    // If all share a column, group all by column
    let first_col = cells[0].1;
    if cells.iter().all(|cell| cell.1 == first_col) {
        return format!(
            "r{}c{}",
            cells
                .iter()
                .map(|cell| cell.0 + 1)
                .sorted()
                .join(&cell_separator),
            cells[0].1 + 1
        );
    }

    // More complex case that spans rows and cols
    let grouped_by_row =
        compact_name_grouped_by_row(&cells, size, &cell_separator, &group_separator);
    let grouped_by_col =
        compact_name_grouped_by_col(&cells, size, &cell_separator, &group_separator);

    if grouped_by_row.len() < grouped_by_col.len() {
        grouped_by_row
    } else {
        grouped_by_col
    }
}

fn compact_name_grouped_by_row(
    cells: &[(usize, usize)],
    size: usize,
    cell_separator: &str,
    group_separator: &str,
) -> String {
    let mut cols_per_row: Vec<Vec<usize>> = vec![vec![]; size];
    for cell in cells {
        cols_per_row[cell.0].push(cell.1 + 1);
    }
    for i in 0..size {
        cols_per_row[i].sort();
    }

    let mut groups: Vec<String> = Vec::new();
    for i in 0..size {
        if cols_per_row[i].len() == 0 {
            continue;
        }

        let mut rows_in_group: Vec<usize> = vec![i + 1];
        for j in i + 1..size {
            if cols_per_row[j] == cols_per_row[i] {
                rows_in_group.push(j + 1);
                cols_per_row[j].clear();
            }
        }

        groups.push(format!(
            "r{}c{}",
            rows_in_group.iter().join(cell_separator),
            cols_per_row[i].iter().join(cell_separator)
        ));
    }

    groups.join(&group_separator)
}

fn compact_name_grouped_by_col(
    cells: &[(usize, usize)],
    size: usize,
    cell_separator: &str,
    group_separator: &str,
) -> String {
    let mut rows_per_col: Vec<Vec<usize>> = vec![vec![]; size];
    for cell in cells {
        rows_per_col[cell.1].push(cell.0 + 1);
    }
    for i in 0..size {
        rows_per_col[i].sort();
    }

    let mut groups: Vec<String> = Vec::new();
    for i in 0..size {
        if rows_per_col[i].len() == 0 {
            continue;
        }

        let mut cols_in_group: Vec<usize> = vec![i + 1];
        for j in i + 1..size {
            if rows_per_col[j] == rows_per_col[i] {
                cols_in_group.push(j + 1);
                rows_per_col[j].clear();
            }
        }

        groups.push(format!(
            "r{}c{}",
            rows_per_col[i].iter().join(&cell_separator),
            cols_in_group.iter().join(&cell_separator)
        ));
    }

    groups.join(&group_separator)
}

#[rustfmt::skip]
#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn test_cell_names() {
        assert_eq!(cell_index_name(0, 9), "r1c1");
        assert_eq!(cell_index_name(40, 9), "r5c5");
        assert_eq!(cell_index_name(80, 9), "r9c9");
        assert_eq!(compact_name(&[], 9), "");
        assert_eq!(compact_name(&[0], 9), "r1c1");
        assert_eq!(compact_name(&[0,1,2], 9), "r1c123");
        assert_eq!(compact_name(&[0,9,18], 9), "r123c1");
        assert_eq!(compact_name(&[0,1,2,9,18], 9), "r123c1,r1c23");
        assert_eq!(compact_name(&[0,10,20], 9), "r1c1,r2c2,r3c3");
        assert_eq!(compact_name(&[0,10,20,21,22], 9), "r1c1,r2c2,r3c345");
        assert_eq!(compact_name(&[0,10,20,29,38], 9), "r1c1,r2c2,r345c3");
    }
}
