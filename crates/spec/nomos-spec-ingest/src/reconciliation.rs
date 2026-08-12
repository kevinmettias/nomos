//! Matching what the archive held against what the store now holds, and reporting the
//! difference.
//!
//! This is the half of the ingest that can disagree with itself: a fate per document and
//! per member, an overlay of one revision on another, the lineage a thing was relocated
//! along, and the censuses that say how much of it reconciled.

pub(crate) mod collision;
pub(crate) mod disposition;
pub(crate) mod fate;
pub(crate) mod hollow;
pub(crate) mod identifier_outcome;
pub(crate) mod lineage;
pub(crate) mod overlay;
pub(crate) mod report;
pub(crate) mod restored;
pub(crate) mod scope;
