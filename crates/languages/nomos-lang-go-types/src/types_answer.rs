//! [`TypesAnswer`], what typing a set of Go sources produced.

use crate::{FileFact, Untyped};

/// What typing a set of Go sources produced: one fact per source the helper checked, and
/// everything else.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypesAnswer
{
    pub facts: Vec<FileFact>,
    pub untyped: Vec<Untyped>,
}
