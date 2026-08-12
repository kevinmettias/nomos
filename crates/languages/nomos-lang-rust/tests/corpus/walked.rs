//! What a walk of the corpus found, and every figure stated over its denominator.
//!
//! The three outcomes are kept apart on purpose. A file that cannot be read off disk is not
//! a file that failed to parse: one is this provider's answer about the source, the other is
//! the machine, and folding them together would attribute an antivirus lock to a syntax
//! error.

use crate::claims::Corpus;
use nomos_lang_rust::{Read_Source, Reading, SyntaxFacts};
use std::path::{Path, PathBuf};

/// What a walk of the corpus found.
#[derive(Default)]
pub(crate) struct Walked
{
    pub(crate) read: usize,
    pub(crate) refused: Vec<(PathBuf, String)>,
    pub(crate) unreadable: Vec<PathBuf>,
    pub(crate) items: usize,
    pub(crate) unexpanded: u64,
    pub(crate) declaring_nothing: usize,
}

pub(crate) fn Walk(corpus: &Corpus) -> Walked
{
    let mut walked = Walked::default();

    for path in &corpus.files
    {
        walked.Read_One(path);
    }

    return walked;
}

impl Walked
{
    /// One file, under whichever of the three outcomes it reached.
    fn Read_One(&mut self, path: &Path)
    {
        let Ok(source) = std::fs::read_to_string(path)
        else
        {
            self.unreadable.push(path.to_path_buf());

            return;
        };

        match Read_Source(&source)
        {
            Reading::Parsed(facts) => self.Counted(&facts),
            Reading::Unparseable(failure) =>
            {
                self.refused.push((path.to_path_buf(), failure.to_string()));
            }
        }
    }

    /// One file this provider answered for, folded into the totals.
    fn Counted(&mut self, facts: &SyntaxFacts)
    {
        self.read = self.read.saturating_add(1);
        self.items = self.items.saturating_add(facts.items.len());
        self.unexpanded = self.unexpanded.saturating_add(u64::from(facts.unexpanded));

        if facts.Declares_Nothing()
        {
            self.declaring_nothing = self.declaring_nothing.saturating_add(1);
        }
    }
}

/// Every figure a run prints, each stated over the denominator it was measured against.
pub(crate) fn Report_The_Walk(corpus: &Corpus, walked: &Walked)
{
    eprintln!(
        "{}: {} Rust files, {} read, {} refused, {} unreadable, {} items, {} unexpanded \
         regions, {} files declaring nothing",
        corpus.root.display(),
        corpus.files.len(),
        walked.read,
        walked.refused.len(),
        walked.unreadable.len(),
        walked.items,
        walked.unexpanded,
        walked.declaring_nothing
    );
}
