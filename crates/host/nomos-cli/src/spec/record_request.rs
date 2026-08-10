//! What `nomos spec record` was asked for.

/// Which record is being asked about.
///
/// One type for `record` and `markdown` because they address the same thing and differ in
/// what they answer about it, which is the distinction `Markdown`'s own documentation
/// draws. A second identical type would let the two drift apart in what they accept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordRequest
{
    /// The node identifier.
    pub id: String,
    /// Which revision of it, when more than one is held.
    pub revision: Option<String>,
}
