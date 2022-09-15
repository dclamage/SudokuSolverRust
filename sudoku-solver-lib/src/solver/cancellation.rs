//! Cancelling various solver operations requires a [`Cancellation`].

/// A Cancellation embodies a check for whether or not to abort a solve process
///
/// If you do not want to provide a cancellation, then most solver methods
/// take an `Option<Cancellation>` anyway.

pub struct Cancellation {
    func: Box<dyn Fn() -> bool>,
}

impl Cancellation {
    /// Create a new Cancellation from a checking function
    ///
    /// ```
    /// # use sudoku_solver_lib::solver::cancellation::Cancellation;
    /// # use std::sync::Arc;
    /// # use std::sync::atomic::AtomicBool;
    /// # use std::sync::atomic::Ordering;
    ///
    /// let cancel_token = Arc::new(AtomicBool::from(false));
    /// let cancellation = Cancellation::new({
    ///     let cancel_token = Arc::clone(&cancel_token);
    ///     move || cancel_token.load(Ordering::SeqCst)
    /// });
    ///
    /// assert_eq!(cancellation.check(), false);
    /// cancel_token.store(true, Ordering::SeqCst);
    /// assert_eq!(cancellation.check(), true);
    /// ```
    pub fn new<F>(func: F) -> Self
    where
        F: (Fn() -> bool) + 'static,
    {
        Self {
            func: Box::new(func),
        }
    }

    /// Check if the cancellation has been sent
    pub fn check(&self) -> bool {
        (self.func)()
    }
}

impl<F> From<F> for Cancellation
where
    F: (Fn() -> bool) + 'static,
{
    fn from(func: F) -> Self {
        Self {
            func: Box::new(func),
        }
    }
}

impl From<Option<Cancellation>> for Cancellation {
    fn from(c: Option<Cancellation>) -> Self {
        c.unwrap_or_else(|| Cancellation::new(|| false))
    }
}
