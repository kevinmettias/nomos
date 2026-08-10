//! Everything one file was scanned into.

use crate::scan::ScannedItem;
/// What one file looks like to a line-reader.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScannedFile
{
    pub items: Vec<ScannedItem>,
    /// Lines read. The denominator: a scan that reported three items out of four lines and
    /// one that reported three out of four thousand are different answers, and a count with
    /// no denominator cannot tell them apart.
    pub lines: u32,
}
