//! What an item touches, written the way a person writes it.
//!
//! # Why this is not just a [`SubjectSet`]
//!
//! [`SubjectSet`] compares *identities*, and a subject identity is a digest. That is
//! correct for comparison — it is what makes a subject the same subject across two
//! spellings of its path — and it is unreadable on disk:
//!
//! ```text
//! "territory": { "members": ["4f9a1c3e8b2d5a70f1c4e9b6d3a80527", …] }
//! ```
//!
//! The ledger is a committed file, and the whole justification for it being a file
//! rather than a database is that a person can see what the agents did to the roadmap in
//! a `git diff`. A diff of digests is a diff nobody reads, which would leave the ledger
//! paying a file's costs for none of a file's benefit.
//!
//! So the authored form names paths and the computed form names identities: *paths are
//! navigation, identities are identity.* This type is the one place the first becomes
//! the second, which is also the one place the normalization rules can be stated once.

mod spelling;
mod overlap;
#[cfg(test)]
mod tests;

pub use spelling::{Normalize_Path, Subject_Of};
use overlap::Shared_Subjects;

use nomos_contracts::SubjectId;
use nomos_model::{Content_Digest, Intersection, SetResolution, SubjectSet, UnknownReason};
use serde::{Deserialize, Serialize};

/// What a piece of work touches, as authored.
///
/// Paths are stored as written so the file stays reviewable, and compared after
/// normalization so two spellings of one file are one subject.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// Every field here is defaulted, which made this the easiest place in the document to lose a
// key: a misspelled `paths` deserialized as an empty territory that excludes nobody. See
// `item.rs`'s module documentation for why the whole document refuses what it cannot account
// for, and `OD-LEDGER-008` for the decision.
#[serde(deny_unknown_fields)]
pub struct Territory
{
    /// The granularity these paths are stated at.
    #[serde(default = "File_Resolution")]
    pub resolution: SetResolution,
    /// Repository-relative paths, as authored.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Patterns that have not been expanded.
    ///
    /// Recorded rather than rejected, and every comparison involving one answers
    /// [`nomos_model::Intersection::Unknown`]. A claim that cannot be shown independent must
    /// not compare as touching nothing, so this stays exactly as it is.
    ///
    /// # Nothing authors one, and `OD-LEDGER-013` is why
    ///
    /// `work add --territory-pattern` used to put a value here and is now a usage error. The
    /// justification this field originally carried — an item may honestly say "this touches
    /// everything under `crates/spec/`" before anyone can enumerate that — turned out to be
    /// already satisfied by an ordinary path: [`Contains_Or_Equals`] decides containment from
    /// the text, so `crates/spec` *does* reserve everything beneath it, with no filesystem
    /// access and no `Unknown`. What the flag added was not expressiveness. It was the only
    /// documented way to reach a state in which an item is unclaimable by everyone including
    /// its own author, every other claim on the board is refused against it, and the refusal
    /// is the non-retryable one that tells an agent to stop and fetch a person.
    ///
    /// The field survives the flag for two reasons. `#[serde(deny_unknown_fields)]` above
    /// means removing the key would refuse every ledger ever written, including the hundred
    /// items carrying `"patterns": []` today. And a document that arrives with one anyway —
    /// hand-edited, or written by some future authoring surface — must still fail closed,
    /// which is what the `Unknown` in [`Territory::Intersect`] does. Withdrawing the flag
    /// removes the way in; it deliberately does not remove the guard.
    #[serde(default)]
    pub patterns: Vec<String>,
}

const fn File_Resolution() -> SetResolution
{
    return SetResolution::File;
}

impl Territory
{
    /// A territory over the given paths at file resolution.
    #[must_use]
    pub fn Of_Files(paths: impl IntoIterator<Item = impl Into<String>>) -> Self
    {
        return Self {
            resolution: SetResolution::File,
            paths: paths.into_iter().map(Into::into).collect(),
            patterns: Vec::new(),
        };
    }

    /// An empty territory at file resolution.
    #[must_use]
    pub fn Empty() -> Self
    {
        return Self::Of_Files(Vec::<String>::new());
    }

    /// Records a pattern whose membership is not yet known.
    ///
    /// No authoring surface calls this — `OD-LEDGER-013` withdrew the flag that did, and
    /// [`Territory::patterns`] says why. It remains because the state is still *reachable* in
    /// a hand-edited document, and a guard against a state nothing can construct is a guard
    /// nothing can test: this is how the fail-closed behaviour is exercised.
    #[must_use]
    pub fn With_Pattern(mut self, pattern: impl Into<String>) -> Self
    {
        self.patterns.push(pattern.into());
        return self;
    }

    /// Whether this territory names nothing at all.
    #[must_use]
    pub fn Is_Empty(&self) -> bool
    {
        return self.paths.is_empty() && self.patterns.is_empty();
    }

    /// The resolved identity form.
    ///
    /// Loses containment — a directory and a file inside it become two unrelated
    /// digests — so this is for callers that genuinely want a set of subject
    /// identities. Exclusion decisions use [`Territory::Intersect`].
    #[must_use]
    pub fn As_Subject_Set(&self) -> SubjectSet
    {
        let mut set =
            SubjectSet::Of(self.resolution, self.paths.iter().map(|path| Subject_Of(path)));

        for pattern in &self.patterns
        {
            set = set.With_Unexpanded_Pattern(pattern.clone());
        }

        return set;
    }

    /// Whether two territories overlap.
    ///
    /// # Why this is not just [`SubjectSet::Intersect`]
    ///
    /// A territory entry may name a directory, and a directory contains files.
    /// `crates/spec/nomos-spec-model` and `crates/spec/nomos-spec-model/src/lib.rs` are
    /// the same work, and comparing their identities says they are unrelated — because
    /// they *are* unrelated as identities. Two different paths hash to two different
    /// digests no matter how they nest, so no amount of care inside [`SubjectSet`]
    /// recovers containment: the information was destroyed by the hash.
    ///
    /// Containment is a property of paths, so it is decided here, where the paths still
    /// exist. This is not a second exclusion rule — both answers are the same
    /// [`Intersection`], and [`Intersection::Permits_Concurrency`] remains the only
    /// place that decides what an answer permits.
    ///
    /// Containment is decided from the text alone, with no filesystem access. That is
    /// what makes it an answer rather than a guess: `a/b` contains `a/b/c` whether or
    /// not either exists.
    #[must_use]
    pub fn Intersect(&self, other: &Self) -> Intersection
    {
        if let Some(pattern) = self.patterns.first().or_else(|| other.patterns.first())
        {
            return Intersection::Unknown(UnknownReason::UnexpandedPattern {
                pattern: pattern.clone(),
            });
        }

        if self.resolution != other.resolution
        {
            return Intersection::Unknown(UnknownReason::IncomparableResolution {
                left: self.resolution,
                right: other.resolution,
            });
        }

        let shared = Shared_Subjects(&self.paths, &other.paths);
        if shared.is_empty()
        {
            return Intersection::Disjoint;
        }

        return Intersection::Overlaps(shared);
    }

    /// Authored paths that denote the same subject as another entry.
    ///
    /// An authoring error worth naming rather than silently deduplicating: an item
    /// listing `src/Main.rs` and `src/main.rs` was probably written by someone who
    /// believed they were reserving two things.
    #[must_use]
    pub fn Ambiguous_Paths(&self) -> Vec<(String, String)>
    {
        let mut duplicates = Vec::new();

        for (index, path) in self.paths.iter().enumerate()
        {
            for other in self.paths.iter().skip(index.saturating_add(1))
            {
                if path != other && Normalize_Path(path) == Normalize_Path(other)
                {
                    duplicates.push((path.clone(), other.clone()));
                }
            }
        }

        return duplicates;
    }
}
