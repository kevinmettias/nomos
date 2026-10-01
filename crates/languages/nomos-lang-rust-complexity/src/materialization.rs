//! [`Materialization`], what became of materializing one fact.

use crate::ParseFailure;
use nomos_analysis::MaterializedFact;

/// The outcome of asking this provider for one file's fact.
///
/// Not a `Result`, for the reason `nomos_lang_rust::Materialization` gives: a `Result` invites
/// `.ok()`, and a walk that maps a refusal to `None` and counts the rest reports a clean corpus
/// it never read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Materialization
{
    /// The file parsed, and this is its fact.
    Materialized(Box<MaterializedFact>),
    /// The file did not parse, so no fact was produced.
    Unparseable(ParseFailure),
}
