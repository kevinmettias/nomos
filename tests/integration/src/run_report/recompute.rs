//! One fact written during a pass, identified by what it is about.

/// One fact written during a pass, identified by what it is about.
///
/// Named rather than counted. "Two facts recomputed" is satisfied by recomputing the wrong
/// two, which is why every invalidation assertion in this harness is made against these
/// rather than against a length.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Recompute
{
    pub capability: String,
    pub subject: String,
}
