//! `D-129`'s round trip: a record read out of the store as markdown, and an edit written
//! back as a transaction that previews what it changes before committing.
//!
//! The assertions that matter are the two that could not be made before this existed. The
//! first is byte-identity from the *rows* rather than from the blob — the store holds enough
//! to be the substrate, or it holds a copy. The second is that the preview answers the
//! question `D-129` calls mandatory, and answers it for this repository's own records rather
//! than only for the corpus-ingested ones a statement table covers.


mod common;
mod editing;
mod preview;
mod projection;
mod refusals;
mod relations;
