//! A bound on how many judgments may be live in a test process at once.
//!
//! libtest runs a crate's tests as threads of one process, one thread per core by default, and
//! nothing in it knows that some of those tests each hold a whole compiler database. A suite that
//! admits them through this is the one place that does: a judgment that loads one waits here for
//! a permit, and every other test runs exactly as libtest would have run it.

use std::sync::{Condvar, Mutex, PoisonError};

/// A counting semaphore over judgments, with the judgment it admits run inside it.
///
/// Built from a [`Mutex`] and a [`Condvar`] rather than taken from a crate, because the standard
/// library has no semaphore and this workspace's test code takes no dependency it does not
/// already have. The count a permit holds is released when the judgment returns *or* unwinds, so
/// a judgment that panics fails its own test and does not starve every test queued behind it.
pub struct JudgmentPermits
{
    limit: usize,
    live: Mutex<usize>,
    released: Condvar,
}

/// One permit, handed back when it goes out of scope -- including while unwinding.
struct Held<'permits>
{
    permits: &'permits JudgmentPermits,
}

impl JudgmentPermits
{
    /// At most `limit` judgments admitted at once.
    #[must_use]
    pub const fn New(limit: usize) -> Self
    {
        return Self { limit, live: Mutex::new(0), released: Condvar::new() };
    }

    /// Runs `judge` once a permit is free, and frees it again afterwards.
    ///
    /// A poisoned lock is read through rather than propagated: the count is only ever changed by
    /// the two lines below that hold it, neither of which can panic between reading and writing
    /// it, so a poisoning is another thread's panic and not a corrupt count.
    pub fn Holding<Answer>(&self, judge: impl FnOnce() -> Answer) -> Answer
    {
        let _held = self.Acquired();

        return judge();
    }

    fn Acquired(&self) -> Held<'_>
    {
        let mut live = self.live.lock().unwrap_or_else(PoisonError::into_inner);
        while *live >= self.limit
        {
            live = self.released.wait(live).unwrap_or_else(PoisonError::into_inner);
        }
        *live = live.saturating_add(1);

        return Held { permits: self };
    }
}

impl Drop for Held<'_>
{
    fn drop(&mut self)
    {
        let mut live = self.permits.live.lock().unwrap_or_else(PoisonError::into_inner);
        *live = live.saturating_sub(1);
        drop(live);
        self.permits.released.notify_one();
    }
}

#[cfg(test)]
mod tests;
