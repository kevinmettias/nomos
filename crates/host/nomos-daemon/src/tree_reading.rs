//! What one request read of the watched tree.

/// The watched tree as one request read it.
///
/// Carried on every answer that touched the tree, so a client is told what the resident
/// actually observed rather than being left to infer it from a finding list that may be the
/// same as last time for either of two reasons. [`Self::changed`] is what separates them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeReading
{
    /// Watched files this request read.
    pub watched: usize,
    /// The watched files whose content differs from what the resident was already holding,
    /// relative to the root and spelled the way the walk spells a path.
    ///
    /// Every walked file on a first request, because a resident holding nothing has nothing
    /// to compare against, and a request that reported no change there would be claiming a
    /// currency it had not established.
    pub changed: Vec<String>,
}

impl TreeReading
{
    /// Whether anything the resident was holding moved under it.
    #[must_use]
    pub fn Moved(&self) -> bool
    {
        return !self.changed.is_empty();
    }
}
