//! Cancelling various solver operations requires a [`Cancellation`].

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A Cancellation embodies a check for whether or not to abort a solve process
///
/// If you do not want to provide a cancellation, then most solver methods
/// take an `impl Into<Cancellation>` which you can give `None` to.
///
/// This object is an Arc internally and so very cheap to clone
#[derive(Clone)]
pub struct Cancellation {
    token: Arc<AtomicBool>,
}

impl Cancellation {
    /// Create a new [`Cancellation`]
    ///
    /// ```
    /// # use sudoku_solver_lib::solver::cancellation::Cancellation;
    ///
    /// let cancellation = Cancellation::default();
    /// assert_eq!(cancellation.check(), false);
    /// cancellation.cancel();
    /// assert_eq!(cancellation.check(), true);
    /// ```
    pub fn new() -> Self {
        Self { token: Arc::new(AtomicBool::from(false)) }
    }

    /// Check if the cancellation has been sent
    pub fn check(&self) -> bool {
        self.token.load(Ordering::SeqCst)
    }

    /// Cancel the operation
    pub fn cancel(&self) {
        self.token.store(true, Ordering::SeqCst);
    }

    /// Reset the cancellation
    pub fn reset(&self) {
        self.token.store(false, Ordering::SeqCst);
    }
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Option<Cancellation>> for Cancellation {
    fn from(c: Option<Cancellation>) -> Self {
        match c {
            Some(v) => v,
            None => Self::new(),
        }
    }
}
