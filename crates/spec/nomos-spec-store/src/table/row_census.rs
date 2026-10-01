//! The counts a census produced, each measured separately.

/// The counts, each measured separately.
///
/// `lines` and `non_separator` are not derived from the other fields. They are their own
/// aggregates, so a caller reporting "282 pipe lines, 258 non-separator" is quoting two
/// measurements rather than one measurement and one subtraction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowCensus
{
    /// Every pipe line, whatever it turned out to be.
    pub lines: u32,
    pub header: u32,
    pub content: u32,
    pub separator: u32,
    /// Authored lines: header and content together.
    pub non_separator: u32,
}
