//! What a run needs from a syntax provider before it will accept an answer.
//!
//! A floor rather than a choice of provider. With two offers on the table the run says what
//! it needs and the registry says who can serve it — a composition root that named a
//! provider directly would be deciding the thing the registry exists to decide, and the two
//! would disagree the day a third provider arrived.

use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

/// What a caller needs when it needs the file actually parsed.
///
/// Syntactic resolution is enough to count declarations, and soundness is not optional: a
/// rollup over facts that might include items the files do not contain is a number about
/// nothing. This is the floor every run used before there was anything else to resolve to,
/// and it is still the default.
#[must_use]
pub const fn Parsed_Floor() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// What a caller needs when it would rather have a weak answer than none.
///
/// Below the parser on every axis it can be below, so both offers clear it. That is what
/// makes a *preference* meaningful: with two usable providers, which one answers is a
/// choice rather than a consequence.
#[must_use]
pub const fn Approximate_Floor() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Approximate,
        Assurance::Unknown,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}
