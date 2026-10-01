//! A v15 document as it arrives: where it came from and what it says.

/// A v15 document as it arrives: where it came from and what it says.
///
/// The two travel together because a filler block is only reportable as a path plus what
/// the markdown said at that ordinal — neither half identifies a block on its own.
pub struct Overlaid<'a>
{
    /// The path the report names.
    pub path: &'a str,
    /// What the document says.
    pub markdown: &'a str,
}
