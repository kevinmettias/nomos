//! [`LintAnswer`], what linting a set of Go sources produced.

use crate::{ModuleFact, Unlinted};

/// What linting a set of Go sources produced: one fact per module that holds any of them and that
/// `go vet` answered for, and everything else.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LintAnswer
{
    pub facts: Vec<ModuleFact>,
    pub unlinted: Vec<Unlinted>,
}
