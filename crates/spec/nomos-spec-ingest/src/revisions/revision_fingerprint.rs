use std::collections::BTreeMap;

/// One revision, reduced to what the four sets need.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevisionFingerprint
{
    pub label: String,
    /// Path within the revision, with the archive's own top directory removed, against the
    /// normalized hash of the document. Normalized rather than verbatim, so a reflow
    /// nobody can see is not reported as a change in place.
    pub documents: BTreeMap<String, String>,
}
