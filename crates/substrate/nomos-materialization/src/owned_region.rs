//! Where a `Composed` target's owned region begins and ends.
//!
//! `OD-PACKAGE-004`: "Regeneration is scoped to the region tied to a declared source of
//! truth, and never touches the region that is not." Two halves of that sentence are
//! declared in two different places, and keeping them apart is the whole point of this type:
//!
//! - **Which class the target is** is declared by whatever places the asset, on the intent,
//!   because that record refuses self-attestation — a tag inside the bytes a generator is
//!   about to overwrite is authored by the same actor whose mistake it would need to catch.
//! - **Where the owned region sits inside those bytes** cannot be declared anywhere but in
//!   the bytes, because a mechanism that has never seen the file has no other way to find a
//!   boundary in it. So the placer declares the two markers that delimit the region, and the
//!   target carries them.
//!
//! That is not the self-attestation `OD-PACKAGE-004` refuses. A marker says "the owned
//! region starts here", never "this file is generated" — moving a marker moves a boundary
//! inside a region the placer already owns, and cannot promote the free region into the
//! owned one, because both markers are matched and neither is written.
//!
//! The markers themselves are preserved. `Materialize` replaces what lies strictly between
//! them, so the opening marker, the closing marker and everything outside the pair come
//! through byte for byte.

/// The two markers that delimit a `Composed` target's owned region.
///
/// Free text, not a syntax this crate defines. A Markdown comment pair, a pair of fenced
/// banner lines, a pair of YAML comments: the target's own format decides what a marker can
/// be, and a mechanism that knows nothing about a crate, a rule or a gate is in no position
/// to close that vocabulary.
///
/// # What a marker pair can and cannot say
///
/// A pair names **one contiguous span**, and that is the whole of what this maturity
/// expresses. `OD-PACKAGE-004` version 3 states the rule a region is judged by — "A
/// `Composed` asset's owned region is exactly what some check actually compares against a
/// declared source, in the units that check reads" — together with the corollary "State a
/// region in the comparison's units, never the artifact's."
///
/// So a placer applies that rule first and then asks whether the region it arrives at is a
/// span. Where the comparison reads at a finer granularity than a span — that record's own
/// worked instance is a table whose first two columns are compared and whose third is not,
/// so the owned region is per-cell and the free region is interleaved with it — a marker pair
/// cannot name it, and a placer that declared one anyway would be declaring the artifact
/// rather than the comparison. This crate cannot detect that: it has no idea what a table, a
/// column or a check is, by design. The guard against it is where the record puts it, at the
/// declaration, and the thing this crate promises in return is that nothing outside the
/// declared span is written for any reason.
///
/// Teaching this crate the finer granularity is not the fix. A splicer that understood a
/// table's columns would be a format-aware mechanism, and the first thing such a thing wants
/// to know is which kind of package it is serving.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedRegion
{
    /// The text the owned region opens after. Preserved; the write begins immediately
    /// after its last byte.
    pub opening_marker: String,
    /// The text the owned region closes before. Preserved; the write ends immediately
    /// before its first byte.
    pub closing_marker: String,
}

impl OwnedRegion
{
    /// A region delimited by two markers.
    #[must_use]
    pub fn New(opening_marker: impl Into<String>, closing_marker: impl Into<String>) -> Self
    {
        return Self { opening_marker: opening_marker.into(), closing_marker: closing_marker.into() };
    }
}
