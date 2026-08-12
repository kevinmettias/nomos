//! How a built projection stands against the store it came from.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Freshness
{
    pub absent: bool,
    pub stale: Option<(String, String)>,
    pub edited: Option<(String, String)>,
    /// The file and its stamp agree with each other and not with the store.
    ///
    /// `stale` reads the stamp's inputs against the store and `edited` reads the stamp's
    /// digest against the file. Both are satisfied by a body and a `content_digest`
    /// rewritten together: the pair is internally consistent, so neither comparison has
    /// anything to say, and neither of them ever looks at what the store renders. This is
    /// the residue — what is left over once the store has been ruled out as the cause and
    /// the stamp has been ruled out as out of date.
    pub diverged: Option<(String, String)>,
}

impl Freshness
{
    #[must_use]
    pub const fn Is_Fresh(&self) -> bool
    {
        return !self.absent
            && self.stale.is_none()
            && self.edited.is_none()
            && self.diverged.is_none();
    }

    #[must_use]
    pub fn Report(&self, output: &str) -> String
    {
        if self.absent
        {
            return format!("{output} has never been built");
        }

        let said = self.Verdicts();
        if said.is_empty()
        {
            return format!("{output} is current");
        }

        return format!("{output} {}", said.join("; "));
    }

    /// Every verdict that applies, each naming the two values that disagree.
    ///
    /// More than one can hold at once and all of them are reported. A stale output that was
    /// also hand-edited is two problems, and printing only the first sends the reader to
    /// rebuild and find the file still wrong.
    fn Verdicts(&self) -> Vec<String>
    {
        // The name, how the recorded value is introduced, and how the found one is. Written
        // out as three `if let` blocks these were three chances for one verdict to stop
        // naming both of the values it is about.
        let phrasings = [
            ("stale", "built over inputs", "the store now holds", &self.stale),
            ("edited", "the stamp declares", "the file hashes to", &self.edited),
            ("diverged", "the file and its stamp agree on", "the store renders", &self.diverged),
        ];
        let mut said = Vec::new();

        for (name, recorded, found, difference) in phrasings
        {
            if let Some((left, right)) = difference
            {
                said.push(format!("{name}: {recorded} {left} and {found} {right}"));
            }
        }

        return said;
    }
}
