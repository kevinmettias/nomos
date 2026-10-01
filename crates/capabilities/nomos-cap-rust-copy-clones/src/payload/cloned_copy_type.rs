//! One `.clone()` call whose receiver resolved to a `Copy` type.

/// One `.clone()` call whose receiver a real compiler frontend resolved to a type that
/// already implements `Copy`.
///
/// No resolved type name: `ra_ap_hir::Type` renders through a `DisplayTarget` the analysis
/// pass that produces this would have to construct a second time only to throw the string
/// away again on the next call, and nothing this capability promises depends on which
/// `Copy` type was cloned -- only that one was. A location is what lets a caller find the
/// call; the type it names is visible at that location already.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClonedCopyType
{
    /// Where the `.clone()` call sits, rendered `path:line:column` (one-based, the same
    /// convention a compiler diagnostic uses) against the file this analysis pass parsed
    /// it out of.
    pub location: String,
}
