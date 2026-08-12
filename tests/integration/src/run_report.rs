//! What one pass over a corpus did.

// What a pass did: which provider answered it, what an edit changed, and the facts it
// wrote.
mod edited;
mod recompute;
mod resolved;

pub use edited::Edited;
pub use recompute::Recompute;
pub use resolved::Resolved;

/// What one pass over a corpus did.
///
/// Every field is a count with a named denominator somewhere in this struct. The
/// prototype reported "0 findings" for a check that had walked nothing, and the defect was
/// invisible because the report had no place to put the number that would have shown it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunReport
{
    pub files_seen: usize,
    pub syntax_materialized: usize,
    pub syntax_reused: usize,
    /// Files no admitted provider could answer for, by path and reason. Named, not counted.
    ///
    /// The reason is the *chosen* provider's, because that is the refusal a caller can act
    /// on. Under a floor that admits nobody weaker this is every refusal there is.
    pub refused: Vec<(String, String)>,
    /// How many subjects each provider answered for, by provider name.
    ///
    /// The census `OD-CAPABILITY-003` requires. Without it a run that fell back for one
    /// file in six is indistinguishable from one the parser answered whole, and the point
    /// of admitting fallback was to buy coverage visibly rather than quietly.
    pub answered_by: std::collections::BTreeMap<String, usize>,
    /// Subjects the chosen provider refused and a weaker one answered, with who answered.
    ///
    /// Named rather than counted, for the same reason `refused` is: "one file was
    /// approximated" is satisfied by approximating the wrong one.
    pub fell_back: Vec<(String, String)>,
    /// Groups whose rollup read at least one fallback answer.
    ///
    /// Distinct from `degraded`, and the distinction is the whole of what fallback buys and
    /// costs. A degraded rollup could not read a member at all; an approximated one read
    /// every member and one of them came from a weaker provider than the run asked for.
    pub approximated: Vec<String>,
    pub groups_seen: usize,
    pub surface_materialized: usize,
    pub surface_reused: usize,
    /// Rollups that could not read one of their members, by group. A rollup over three of
    /// four files is a degraded answer, and reporting it as an answer is how a corpus with
    /// a hole in it reads as a corpus that is fine.
    pub degraded: Vec<String>,
    /// Subjects whose facts were written in this pass, in the order they were written.
    ///
    /// The list the invalidation assertions are made against. "Two facts recomputed" is
    /// satisfied by recomputing the wrong two; naming them is not.
    pub recomputed: Vec<Recompute>,
}

impl RunReport
{
    /// The subjects recomputed for one capability, sorted.
    #[must_use]
    pub fn Recomputed_For(&self, capability: &str) -> Vec<String>
    {
        let mut subjects: Vec<String> = self
            .recomputed
            .iter()
            .filter(|entry| return entry.capability == capability)
            .map(|entry| return entry.subject.clone())
            .collect();
        subjects.sort();

        return subjects;
    }

    /// How many subjects one provider answered for in this pass.
    #[must_use]
    pub fn Answered_By(&self, provider: &str) -> usize
    {
        return self.answered_by.get(provider).copied().unwrap_or(0);
    }

    /// Whether this pass got every answer from the provider the registry chose.
    ///
    /// The question a caller asks before treating the run as exact. A run that fell back is
    /// not a clean run — it is a run that bought coverage, and it says what that cost.
    #[must_use]
    pub fn Wholly_Chosen(&self) -> bool
    {
        return self.fell_back.is_empty();
    }

    /// Everything written in this pass, sorted, as `capability of subject`.
    #[must_use]
    pub fn Recomputed(&self) -> Vec<String>
    {
        let mut all: Vec<String> = self
            .recomputed
            .iter()
            .map(|entry| return format!("{} of {}", entry.capability, entry.subject))
            .collect();
        all.sort();

        return all;
    }
}
