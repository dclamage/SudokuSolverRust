//! A mask of possible values for a cell.
//!
//! This is a N-bit mask, where each bit represents a possible value for a cell and N
//! is the NxN size of the grid.
//!
//! The mask is represented as a u32, where the least significant bit represents the
//! value 1, the next bit represents the value 2, and so on.
//!
//! For example, a mask of 0b11 represents the values 1
//! and 2.
//!
//! The top bit of the mask represents whether the cell has been "solved".
//! A "solved" cell has one single value and all the consequences of that value have
//! been applied to the board.  For example, if a cell has a value of 1, then all
//! other cells in the same row, column, and box have had 1 removed as a possible
//! value.
//!
//! VALUE_SET_MASK is the mask for the top bit.
//! CANDIDATES_MASK is the mask for all the candidate bits (!VALUE_SET_MASK).

use std::{fmt, ops::*};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ValueMask {
    mask: u32,
}

impl BitAnd for ValueMask {
    type Output = ValueMask;

    fn bitand(self, rhs: ValueMask) -> Self {
        ValueMask {
            mask: self.mask & rhs.mask,
        }
    }
}

impl BitOr for ValueMask {
    type Output = ValueMask;

    fn bitor(self, rhs: ValueMask) -> Self {
        ValueMask {
            mask: self.mask | rhs.mask,
        }
    }
}

impl BitXor for ValueMask {
    type Output = ValueMask;

    fn bitxor(self, rhs: ValueMask) -> Self {
        ValueMask {
            mask: self.mask ^ rhs.mask,
        }
    }
}

impl Not for ValueMask {
    type Output = ValueMask;

    fn not(self) -> Self {
        ValueMask { mask: !self.mask }
    }
}

impl ValueMask {
    /// The top bit of a cell mask is set if the cell has been solved
    /// to a single value, and all the consequences of solving the
    /// value have been executed.
    pub const VALUE_SOLVED_MASK: u32 = 1u32 << 31;

    /// A mask that will get just the value bits from a cell mask,
    /// ignoring the solved bit.
    pub const CANDIDATES_MASK: u32 = !Self::VALUE_SOLVED_MASK;

    /// Create a new ValueMask with no values set.
    ///
    /// # Examples
    /// ```
    /// use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::new();
    /// assert!(mask.is_empty());
    /// ```
    ///
    /// ```
    /// use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::new().with(5);
    /// assert!(!mask.is_empty());
    /// assert!(mask.has(5));
    /// ```
    pub fn new() -> Self {
        ValueMask { mask: 0 }
    }

    /// Creates a new ValueMask with all possible values set for a
    /// specific grid size.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_all_values(9);
    /// assert!(mask.has(1));
    /// assert!(mask.has(2));
    /// assert!(mask.has(3));
    /// assert!(mask.has(4));
    /// assert!(mask.has(5));
    /// assert!(mask.has(6));
    /// assert!(mask.has(7));
    /// assert!(mask.has(8));
    /// assert!(mask.has(9));
    /// ```
    pub fn from_all_values(size: usize) -> Self {
        ValueMask {
            mask: (1 << size) - 1,
        }
    }

    /// Creates a new ValueMask with a single value set.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_value(5);
    /// assert!(!mask.has(1));
    /// assert!(!mask.has(2));
    /// assert!(!mask.has(3));
    /// assert!(!mask.has(4));
    /// assert!(mask.has(5));
    /// assert!(!mask.has(6));
    /// assert!(!mask.has(7));
    /// assert!(!mask.has(8));
    /// assert!(!mask.has(9));
    /// ```
    pub fn from_value(value: usize) -> Self {
        ValueMask {
            mask: 1 << (value - 1),
        }
    }

    /// Create a new ValueMask with multiple values set.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_values(&[1, 3, 5, 7, 9]);
    /// assert!(mask.has(1));
    /// assert!(!mask.has(2));
    /// assert!(mask.has(3));
    /// assert!(!mask.has(4));
    /// assert!(mask.has(5));
    /// assert!(!mask.has(6));
    /// assert!(mask.has(7));
    /// assert!(!mask.has(8));
    /// assert!(mask.has(9));
    /// ```
    pub fn from_values(values: &[usize]) -> Self {
        let mut mask = 0u32;
        for value in values {
            mask |= 1 << (value - 1);
        }
        ValueMask { mask }
    }

    /// Creates a mask with all values strictly lower than the given value.
    ///
    /// # See also
    /// [`ValueMask::from_lower_equal`]
    /// [`ValueMask::from_higher`]
    /// [`ValueMask::from_higher_equal`]
    /// [`ValueMask::from_between_inclusive`]
    /// [`ValueMask::from_between_exclusive`]
    /// [`ValueMask::from_all_values`]
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Create a mask with all values strictly lower than 3.
    /// let mask = ValueMask::from_lower(3);
    ///
    /// // The mask should have only the values 1 and 2 set.
    /// assert!(mask.has(1));
    /// assert!(mask.has(2));
    /// assert!(!mask.has(3));
    /// assert!(!mask.has(4));
    /// assert!(!mask.has(5));
    /// assert!(!mask.has(6));
    /// assert!(!mask.has(7));
    /// assert!(!mask.has(8));
    /// assert!(!mask.has(9));
    /// ```
    pub fn from_lower(val: usize) -> Self {
        ValueMask {
            mask: (1u32 << (val - 1)) - 1,
        }
    }

    /// Creates a mask with all values lower than or equal to the given value.
    ///
    /// # See also
    /// [`ValueMask::from_lower`]
    /// [`ValueMask::from_higher`]
    /// [`ValueMask::from_higher_equal`]
    /// [`ValueMask::from_between_inclusive`]
    /// [`ValueMask::from_between_exclusive`]
    /// [`ValueMask::from_all_values`]
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Create a mask with all values lower than or equal to 3.
    /// let mask = ValueMask::from_lower_equal(3);
    ///
    /// // The mask should have only the values 1, 2 and 3 set.
    /// assert!(mask.has(1));
    /// assert!(mask.has(2));
    /// assert!(mask.has(3));
    /// assert!(!mask.has(4));
    /// assert!(!mask.has(5));
    /// assert!(!mask.has(6));
    /// assert!(!mask.has(7));
    /// assert!(!mask.has(8));
    /// assert!(!mask.has(9));
    /// ```
    pub fn from_lower_equal(val: usize) -> Self {
        ValueMask {
            mask: (1u32 << val) - 1,
        }
    }

    /// Creates a mask with all values strictly higher than the given value.
    ///
    /// # See also
    /// [`ValueMask::from_lower`]
    /// [`ValueMask::from_lower_equal`]
    /// [`ValueMask::from_higher_equal`]
    /// [`ValueMask::from_between_inclusive`]
    /// [`ValueMask::from_between_exclusive`]
    /// [`ValueMask::from_all_values`]
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Create a mask with all values strictly higher than 3.
    /// let mask = ValueMask::from_higher(3, 9);
    ///
    /// // The mask should have only the values 4, 5, 6, 7, 8 and 9 set.
    /// assert!(!mask.has(1));
    /// assert!(!mask.has(2));
    /// assert!(!mask.has(3));
    /// assert!(mask.has(4));
    /// assert!(mask.has(5));
    /// assert!(mask.has(6));
    /// assert!(mask.has(7));
    /// assert!(mask.has(8));
    /// assert!(mask.has(9));
    /// ```
    pub fn from_higher(val: usize, size: usize) -> Self {
        Self::from_all_values(size) & !Self::from_lower_equal(val)
    }

    /// Creates a mask with all values higher than or equal to the given value.
    ///
    /// # See also
    /// [`ValueMask::from_lower`]
    /// [`ValueMask::from_lower_equal`]
    /// [`ValueMask::from_higher`]
    /// [`ValueMask::from_between_inclusive`]
    /// [`ValueMask::from_between_exclusive`]
    /// [`ValueMask::from_all_values`]
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Create a mask with all values higher than or equal to 3.
    /// let mask = ValueMask::from_higher_equal(3, 9);
    ///
    /// // The mask should have only the values 3, 4, 5, 6, 7, 8 and 9 set.
    /// assert!(!mask.has(1));
    /// assert!(!mask.has(2));
    /// assert!(mask.has(3));
    /// assert!(mask.has(4));
    /// assert!(mask.has(5));
    /// assert!(mask.has(6));
    /// assert!(mask.has(7));
    /// assert!(mask.has(8));
    /// assert!(mask.has(9));
    /// ```
    pub fn from_higher_equal(val: usize, size: usize) -> Self {
        Self::from_all_values(size) & !Self::from_lower(val)
    }

    /// Creates a mask with all values between the given values (inclusive).
    ///
    /// # See also
    /// [`ValueMask::from_lower`]
    /// [`ValueMask::from_lower_equal`]
    /// [`ValueMask::from_higher`]
    /// [`ValueMask::from_higher_equal`]
    /// [`ValueMask::from_between_exclusive`]
    /// [`ValueMask::from_all_values`]
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Create a mask with all values between 3 and 5 (inclusive).
    /// let mask = ValueMask::from_between_inclusive(3, 5, 9);
    ///
    /// // The mask should have only the values 3, 4, 5 set.
    /// assert!(!mask.has(1));
    /// assert!(!mask.has(2));
    /// assert!(mask.has(3));
    /// assert!(mask.has(4));
    /// assert!(mask.has(5));
    /// assert!(!mask.has(6));
    /// assert!(!mask.has(7));
    /// assert!(!mask.has(8));
    /// assert!(!mask.has(9));
    /// ```
    pub fn from_between_inclusive(low: usize, high: usize, size: usize) -> Self {
        Self::from_all_values(size) & !(Self::from_lower(low) | Self::from_higher(high, size))
    }

    /// Creates a mask with all values between the given values (exclusive).
    ///
    /// # See also
    /// [`ValueMask::from_lower`]
    /// [`ValueMask::from_lower_equal`]
    /// [`ValueMask::from_higher`]
    /// [`ValueMask::from_higher_equal`]
    /// [`ValueMask::from_between_inclusive`]
    /// [`ValueMask::from_all_values`]
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Create a mask with all values between 3 and 5 (exclusive).
    /// let mask = ValueMask::from_between_exclusive(3, 5, 9);
    ///
    /// // The mask should have only the value 4 set.
    /// assert!(!mask.has(1));
    /// assert!(!mask.has(2));
    /// assert!(!mask.has(3));
    /// assert!(mask.has(4));
    /// assert!(!mask.has(5));
    /// assert!(!mask.has(6));
    /// assert!(!mask.has(7));
    /// assert!(!mask.has(8));
    /// assert!(!mask.has(9));
    /// ```
    pub fn from_between_exclusive(low: usize, high: usize, size: usize) -> Self {
        Self::from_all_values(size)
            & !(Self::from_lower_equal(low) | Self::from_higher_equal(high, size))
    }

    /// Returns just the mask of value bits without the value set bit.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_value(5);
    /// assert_eq!(mask.value_bits(), 0b10000);
    ///
    /// let mask = mask.solved();
    /// assert_eq!(mask.value_bits(), 0b10000);
    /// ```
    pub fn value_bits(self) -> u32 {
        self.mask & ValueMask::CANDIDATES_MASK
    }

    /// Returns true if the value is set.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_value(5);
    /// assert!(!mask.is_solved());
    /// assert!(mask.has(5));
    ///
    /// let mask = mask.solved();
    /// assert!(mask.is_solved());
    /// assert!(mask.has(5));
    /// ```
    #[must_use]
    pub fn is_solved(self) -> bool {
        self.mask & ValueMask::VALUE_SOLVED_MASK != 0
    }

    /// Returns a mask marked as solved.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_value(5).solved();
    /// assert!(mask.is_solved());
    /// assert!(mask.has(5));
    /// ```
    #[must_use]
    pub fn solved(self) -> Self {
        ValueMask {
            mask: self.mask | ValueMask::VALUE_SOLVED_MASK,
        }
    }

    /// Returns a mask marked as unsolved.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_value(5).solved();
    /// assert!(mask.is_solved());
    /// assert!(mask.has(5));
    ///
    /// let mask = mask.unsolved();
    /// assert!(!mask.is_solved());
    /// assert!(mask.has(5));
    /// ```
    #[must_use]
    pub fn unsolved(self) -> Self {
        ValueMask {
            mask: self.mask & !ValueMask::VALUE_SOLVED_MASK,
        }
    }

    /// Returns true if no values are possible.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_value(5);
    /// assert!(!mask.is_empty());
    ///
    /// let mask = mask.without(5);
    /// assert!(mask.is_empty());
    /// ```
    pub fn is_empty(self) -> bool {
        self.value_bits() == 0
    }

    /// Returns true if values are still possible.
    /// This is the opposite of is_empty.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_value(5);
    /// assert!(mask.is_valid());
    ///
    /// let mask = mask.without(5);
    /// assert!(!mask.is_valid());
    /// ```
    pub fn is_valid(self) -> bool {
        self.value_bits() != 0
    }

    /// Returns true if only one value is possible.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_value(5);
    /// assert!(mask.is_single());
    ///
    /// let mask = ValueMask::from_all_values(9);
    /// assert!(!mask.is_single());
    /// ```
    pub fn is_single(self) -> bool {
        let value_bits = self.value_bits();
        value_bits != 0 && (value_bits & (value_bits - 1)) == 0
    }

    /// Returns true if the value is possible.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mask = ValueMask::from_value(5);
    /// assert!(mask.has(5));
    /// assert!(!mask.has(6));
    /// ```
    pub fn has(self, value: usize) -> bool {
        (self.mask & (1 << (value - 1))) != 0
    }

    /// Returns a mask with the value added.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mut mask = ValueMask::from_value(5);
    /// assert!(mask.has(5));
    /// assert!(!mask.has(6));
    ///
    /// let mask = mask.with(6);
    /// assert!(mask.has(5));
    /// assert!(mask.has(6));
    /// ```
    #[must_use]
    pub fn with(self, value: usize) -> Self {
        ValueMask {
            mask: self.mask | (1 << (value - 1)),
        }
    }

    /// Returns a mask with the value removed.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mut mask = ValueMask::from_all_values(9);
    /// assert!(mask.has(5));
    ///
    /// let mask = mask.without(5);
    /// assert!(!mask.has(5));
    /// ```
    #[must_use]
    pub fn without(self, value: usize) -> Self {
        ValueMask {
            mask: self.mask & !(1 << (value - 1)),
        }
    }

    /// Returns a mask with all values from the mask removed except the given value.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// let mut mask = ValueMask::from_all_values(9);
    /// assert!(mask.has(5));
    /// assert!(mask.has(6));
    ///
    /// let mask = mask.with_only(5);
    /// assert!(mask.has(5));
    /// assert!(!mask.has(6));
    /// ```
    #[must_use]
    pub fn with_only(self, value: usize) -> Self {
        ValueMask {
            mask: self.mask & (Self::VALUE_SOLVED_MASK | (1 << (value - 1))),
        }
    }

    /// Counts the number of set values.
    ///
    /// # Return value
    /// The count of values in the cell mask.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Choose a size of 9
    /// let size = 9;
    ///
    /// // Create a mask with all values set.
    /// let mask = ValueMask::from_all_values(size);
    ///
    /// // The mask should start with a count of 9.
    /// assert_eq!(mask.count(), size);
    ///
    /// // Clear candidate 2
    /// let mask = mask.without(2);
    ///
    /// // The mask should now have one fewer value.
    /// assert_eq!(mask.count(), size - 1);
    ///
    /// // Clear all but the value 3
    /// let mask = mask.with_only(3);
    ///
    /// // The mask should now have one value.
    /// assert_eq!(mask.count(), 1);
    /// ```
    pub fn count(self) -> usize {
        self.value_bits().count_ones() as usize
    }

    /// Get the value of a cell mask.
    ///
    /// Assumes that only one value is possible.
    /// If more than one value is possible, then this
    /// behaves the same way as [`ValueMask::min`].
    ///
    /// # Return value
    /// The value of the cell mask.
    ///
    /// # See also
    /// - [`ValueMask::min`] - Get the minimum value set in a cell mask.
    /// - [`ValueMask::max`] - Get the maximum value set in a cell mask.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Choose a size of 9
    /// let size = 9;
    ///
    /// // Create a mask with all values set.
    /// let mut mask = ValueMask::from_value(3);
    ///
    /// // The mask should now have the value of 3.
    /// assert_eq!(mask.value(), 3);
    /// ```
    pub fn value(self) -> usize {
        self.value_bits().trailing_zeros() as usize + 1
    }

    /// Get the minimum value possible.
    ///
    /// **Assumes the cell mask is non-zero.**
    /// - If the cell mask is zero, then the result is undefined.
    ///
    /// # Return value
    /// The minimum value of the cell mask.
    ///
    /// # See also
    /// - [`ValueMask::value`] - Get the value of a cell when only one value is set.
    /// - [`ValueMask::max`] - Get the maximum value set in a cell mask.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Choose a size of 9
    /// let size = 9;
    ///
    /// // Create a mask with all values set.
    /// let mut mask = ValueMask::from_all_values(size);
    ///
    /// // The minimum value of r1c1 should be 1.
    /// assert_eq!(mask.min(), 1);
    ///
    /// // Remove 1,2,5 from r1c1
    /// let mask = mask.without(1).without(2).without(5);
    ///
    /// // The minimum value of r1c1 should be 3.
    /// assert_eq!(mask.min(), 3);
    /// ```
    pub fn min(self) -> usize {
        self.value_bits().trailing_zeros() as usize + 1
    }

    /// Get the maximum value possible.
    ///
    /// **Assumes the cell is not empty.**
    /// - If the cell mask is empty, then the result is undefined.
    ///
    /// # Return value
    /// The maximum value of the cell mask.
    ///
    /// # See also
    /// - [`ValueMask::value`] - Get the value of a cell when only one value is set.
    /// - [`ValueMask::min`] - Get the minimum value set in a cell mask.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Choose a size of 9
    /// let size = 9;
    ///
    /// // Create a mask with all values set.
    /// let mut mask = ValueMask::from_all_values(size);
    ///
    /// // The maximum value of r1c1 should be 1.
    /// assert_eq!(mask.max(), 9);
    ///
    /// // Remove 1,2,5 from r1c1
    /// let mask = mask.without(5).without(8).without(9);
    ///
    /// // The maximum value of r1c1 should be 7.
    /// assert_eq!(mask.max(), 7);
    /// ```
    pub fn max(self) -> usize {
        32 - self.value_bits().leading_zeros() as usize
    }

    /// Get a vector of all values in the mask.
    ///
    /// # Return value
    /// A vector of all values in the mask.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Choose a size of 9
    /// let size = 9;
    ///
    /// // Create a mask with all values set.
    /// let mask = ValueMask::from_all_values(size);
    ///
    /// // The mask should have all values.
    /// assert_eq!(mask.to_vec(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    pub fn to_vec(self) -> Vec<usize> {
        self.into_iter().collect()
    }
}

impl fmt::Display for ValueMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for value in *self {
            if first {
                first = false;
            } else {
                write!(f, ",")?;
            }
            write!(f, "{}", value)?;
        }
        Ok(())
    }
}

impl fmt::Debug for ValueMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ValueMask({})", self)
    }
}

impl From<u32> for ValueMask {
    fn from(mask: u32) -> Self {
        Self { mask }
    }
}

impl From<ValueMask> for u32 {
    fn from(mask: ValueMask) -> Self {
        mask.mask
    }
}

impl FromIterator<usize> for ValueMask {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        let mut mask = ValueMask::new();
        for value in iter {
            mask = mask.with(value);
        }
        mask
    }
}

impl IntoIterator for ValueMask {
    type Item = usize;
    type IntoIter = ValueMaskIter;

    /// Get an iterator over all values in the mask.
    ///
    /// # Return value
    /// An iterator over all values in the mask.
    ///
    /// # Example
    /// ```
    /// # use sudoku_solver_lib::value_mask::ValueMask;
    /// // Choose a size of 9
    /// let size = 9;
    ///
    /// // Create a mask with all values set.
    /// let mask = ValueMask::from_all_values(size);
    ///
    /// // The mask should have all values.
    /// assert_eq!(mask.into_iter().collect::<Vec<usize>>(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        ValueMaskIter {
            mask: self.value_bits(),
        }
    }
}

pub struct ValueMaskIter {
    mask: u32,
}

impl Iterator for ValueMaskIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mask == 0 {
            None
        } else {
            let index = self.mask.trailing_zeros() as usize;
            self.mask &= !(1 << index);
            Some(index + 1)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use itertools::assert_equal;

    #[test]
    fn test_mask_to_string() {
        assert_eq!(ValueMask::from_values(&[1]).to_string(), "1");
        assert_eq!(ValueMask::from_values(&[1, 2]).to_string(), "1,2");
        assert_eq!(ValueMask::from_values(&[1, 5]).to_string(), "1,5");
        assert_eq!(ValueMask::from_values(&[1, 5]).solved().to_string(), "1,5");
        assert_eq!(
            ValueMask::from_values(&[1, 2, 3, 4, 5, 6, 7, 8, 9]).to_string(),
            "1,2,3,4,5,6,7,8,9"
        );
    }

    #[test]
    fn test_mask_ranges() {
        assert_eq!(
            ValueMask::from_all_values(9).value_bits(),
            0b0000_0000_0000_0000_0000_0001_1111_1111
        );
        assert_eq!(
            ValueMask::from_all_values(16).value_bits(),
            0b0000_0000_0000_0000_1111_1111_1111_1111
        );

        let size = 9;
        assert_eq!(
            ValueMask::from_lower(2).value_bits(),
            0b0000_0000_0000_0000_0000_0000_0000_0001
        );
        assert_eq!(
            ValueMask::from_lower(4).value_bits(),
            0b0000_0000_0000_0000_0000_0000_0000_0111
        );
        assert_eq!(
            ValueMask::from_lower_equal(2).value_bits(),
            0b0000_0000_0000_0000_0000_0000_0000_0011
        );
        assert_eq!(
            ValueMask::from_lower_equal(4).value_bits(),
            0b0000_0000_0000_0000_0000_0000_0000_1111
        );
        assert_eq!(
            ValueMask::from_higher(2, size).value_bits(),
            0b0000_0000_0000_0000_0000_0001_1111_1100
        );
        assert_eq!(
            ValueMask::from_higher(4, size).value_bits(),
            0b0000_0000_0000_0000_0000_0001_1111_0000
        );
        assert_eq!(
            ValueMask::from_higher_equal(2, size).value_bits(),
            0b0000_0000_0000_0000_0000_0001_1111_1110
        );
        assert_eq!(
            ValueMask::from_higher_equal(4, size).value_bits(),
            0b0000_0000_0000_0000_0000_0001_1111_1000
        );
        assert_eq!(
            ValueMask::from_between_exclusive(1, 5, size).value_bits(),
            0b0000_0000_0000_0000_0000_0000_0000_1110
        );
        assert_eq!(
            ValueMask::from_between_inclusive(1, 5, size).value_bits(),
            0b0000_0000_0000_0000_0000_0000_0001_1111
        );
    }

    #[test]
    fn test_mask_values() {
        assert_eq!(ValueMask::from_value(1).value(), 1);
        assert_eq!(ValueMask::from_value(2).value(), 2);
        assert_eq!(ValueMask::from_value(3).value(), 3);
        assert_eq!(ValueMask::from_value(4).value(), 4);
        assert_eq!(ValueMask::from_value(5).value(), 5);
        assert_eq!(ValueMask::from_value(6).value(), 6);
        assert_eq!(ValueMask::from_value(7).value(), 7);
        assert_eq!(ValueMask::from_value(8).value(), 8);
        assert_eq!(ValueMask::from_value(9).value(), 9);
        assert_eq!(
            ValueMask::from(0b0000_0000_0000_0000_0000_0000_0000_0001).value(),
            1
        );
        assert_eq!(
            ValueMask::from(0b0000_0000_0000_0000_0000_0000_0000_0010).value(),
            2
        );
        assert_eq!(
            ValueMask::from(0b0000_0000_0000_0000_0000_0001_0000_0000).value(),
            9
        );
        assert_eq!(
            ValueMask::from(0b1000_0000_0000_0000_0000_0001_0000_0000).value(),
            9
        );
        assert_eq!(
            ValueMask::from(0b0000_0000_0000_0000_0000_0001_1100_1000).min(),
            4
        );
        assert_eq!(
            ValueMask::from(0b0000_0000_0000_0000_0000_0001_1100_1000).max(),
            9
        );
        assert_eq!(
            ValueMask::from(0b1000_0000_0000_0000_0000_0001_1100_1000).max(),
            9
        );
        assert_eq!(ValueMask::from_values(&[3, 5, 8]).min(), 3);
        assert_eq!(ValueMask::from_values(&[3, 5, 8]).max(), 8);
        assert!(ValueMask::from_value(3).has(3));
        assert!(!ValueMask::from_values(&[1, 2, 3, 5, 6, 7, 8, 9]).has(4));
    }

    #[test]
    fn test_mask_iterator() {
        assert_equal(ValueMask::from(0), vec![]);
        assert_equal(
            ValueMask::from(0b0000_0000_0000_0000_0000_0000_0000_0001),
            vec![1],
        );
        assert_equal(
            ValueMask::from(0b1000_0000_0000_0000_0000_0000_0000_0001),
            vec![1],
        );
        assert_equal(
            ValueMask::from(0b0000_0000_0000_0000_0000_0000_0000_0010),
            vec![2],
        );
        assert_equal(
            ValueMask::from(0b0000_0000_0000_0000_0000_0000_0001_0010),
            vec![2, 5],
        );
        assert_equal(
            ValueMask::from(0b0000_0000_0000_0000_0000_0001_1111_1111),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
        );
        assert_equal(
            ValueMask::from(0b1000_0000_0000_0000_0000_0001_1111_1111),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
        );
        assert_equal(ValueMask::from_values(&[1, 4, 8]), vec![1, 4, 8]);
    }
}
