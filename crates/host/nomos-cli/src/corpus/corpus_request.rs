//! Which corpus files an assembly was asked to read.

use std::path::PathBuf;
/// Where a corpus root was named, so an absence can say how to supply one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorpusRequest
{
    /// The environment variable that names the corpus root.
    ///
    /// Passed in rather than read here. The composition root owns the environment; this
    /// module is handed a value and a name for where it came from, which is what lets
    /// every test below run without one.
    pub variable: String,
    /// The root, if anything named one.
    pub root: Option<PathBuf>,
    /// The revision label to ingest the corpus under.
    pub revision: String,
}
