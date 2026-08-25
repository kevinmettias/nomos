//! What became of materializing one fact.

use crate::ParseFailure;
use nomos_analysis::MaterializedFact;

/// The outcome of asking this provider for a fact about one file.
///
/// Mirrors [`Reading`](crate::Reading) deliberately, for the reason `nomos-lang-rust`'s own
/// equivalent states: a `Result<MaterializedFact, ParseFailure>` would invite `.ok()`, and a
/// corpus walk that maps failures to `None` and counts the `Some`s is exactly the shape of a
/// run reporting a clean corpus it never read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Materialization
{
    Materialized(Box<MaterializedFact>),
    Unparseable(ParseFailure),
}
