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

pub(crate) mod spelling;
mod overlap;
#[cfg(test)]
mod tests;

// The one place this crate names band 0, and check-dependency-placement reports the edge
// for it. A SubjectId is the shared identity a territory is a territory *of*; it is
// published for peers outside this repository and is not this crate's to redefine, so
// naming it once here is the edge working rather than a file in the wrong crate.
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
    /// already satisfied by an ordinary path: [`Is_Overlapping`] decides containment from
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
        use spelling::Subject_Of;

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
        use overlap::Shared_Subjects;

        if let Some(reason) = self.Incomparable(other)
        {
            return Intersection::Unknown(reason);
        }

        let shared = Shared_Subjects(&self.paths, &other.paths);

        return if shared.is_empty() { Intersection::Disjoint } else { Intersection::Overlaps(shared) };
    }

    /// Why `self` and `other` cannot be compared for overlap at all, if there is a reason —
    /// an unexpanded pattern on either side, or paths resolved under different rules.
    fn Incomparable(&self, other: &Self) -> Option<UnknownReason>
    {
        if let Some(pattern) = self.patterns.first().or_else(|| other.patterns.first())
        {
            return Some(UnknownReason::UnexpandedPattern {
                pattern: pattern.clone(),
            });
        }

        if self.resolution != other.resolution
        {
            return Some(UnknownReason::IncomparableResolution {
                left: self.resolution,
                right: other.resolution,
            });
        }

        return None;
    }

    /// Authored paths that denote the same subject as another entry.
    ///
    /// An authoring error worth naming rather than silently deduplicating: an item
    /// listing `src/Main.rs` and `src/main.rs` was probably written by someone who
    /// believed they were reserving two things.
    #[must_use]
    pub fn Ambiguous_Paths(&self) -> Vec<(String, String)>
    {
        use spelling::Normalize_Path;

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

    /// Authored paths that are not in the tree at `root`, in the order they were authored.
    ///
    /// # Why this exists, and why it answers "absent" and not "decayed"
    ///
    /// Territory is authored once and the tree moves under it. A peer's refactor renames or
    /// splits a reserved file, and the reservation is then false while continuing to look
    /// exactly like a correct one. Nothing reports that. This names the paths a reservation
    /// points at that no longer exist in the tree the reservation is being checked against.
    ///
    /// The finding is authored as *absent* rather than as *decayed* because the two are
    /// indistinguishable by existence alone: an item that reserves a file it is about to
    /// create is exactly as absent as one whose file a peer moved. A report that guessed
    /// would be wrong on a board that already contains the first kind, and an author who
    /// learned the report cries wolf would stop reading it. So this states the fact — the
    /// path is not in the tree — and leaves which case it is to the caller who holds the
    /// context that decides it.
    ///
    /// Patterns are deliberately not checked: an unexpanded pattern has no single file to
    /// point at, so "absent" has no meaning for it, and reporting it would say a reservation
    /// is stale when the reservation never named a file in the first place.
    #[must_use]
    pub fn Absent_Paths(&self, root: &std::path::Path, filesystem: &impl nomos_platform::FileSystem) -> Vec<String>
    {
        let mut absent = Vec::new();

        for path in &self.paths
        {
            if !filesystem.Exists(&root.join(path))
            {
                absent.push(path.clone());
            }
        }

        return absent;
    }
}

// `mod tests` above is a SEPARATE file (`territory/tests.rs`), loaded via a bare `mod
// tests;` with no `#[path]` override. check-test-coverage's Rust front end keys a test's
// companion unit off the literal file it is textually written in, so a test living in that
// separate file can never address a function declared here, however it is named — see
// `check-test-coverage: allow-untested` note history for this crate. This second, LITERAL
// inline module gives each public method here the one-file address the check reads, without
// disturbing `territory/tests.rs`'s own broader behavioural suite.
#[cfg(test)]
mod local_tests
{
    use super::*;

    #[test]
    fn Test_Of_Files_Should_Set_File_Resolution_And_Store_The_Given_Paths()
    {
        let territory = Territory::Of_Files(["a/b.rs", "c/d.rs"]);

        assert_eq!(territory.resolution, SetResolution::File);
        assert_eq!(territory.paths, vec!["a/b.rs".to_owned(), "c/d.rs".to_owned()]);
        assert!(territory.patterns.is_empty());
    }

    #[test]
    fn Test_Empty_Should_Have_No_Paths_And_No_Patterns()
    {
        let territory = Territory::Empty();

        assert!(territory.paths.is_empty());
        assert!(territory.patterns.is_empty());
    }

    #[test]
    fn Test_With_Pattern_Should_Record_An_Unexpanded_Pattern()
    {
        let territory = Territory::Empty().With_Pattern("crates/spec/**");

        assert_eq!(territory.patterns, vec!["crates/spec/**".to_owned()]);
    }

    #[test]
    fn Test_Is_Empty_Should_Turn_False_Once_A_Path_Is_Present()
    {
        let empty = Territory::Empty();
        let with_path = Territory::Of_Files(["a.rs"]);

        assert!(empty.Is_Empty());
        assert!(!with_path.Is_Empty());
    }

    #[test]
    fn Test_As_Subject_Set_Should_Compare_Equal_For_The_Same_Paths()
    {
        let one = Territory::Of_Files(["a/b.rs", "c/d.rs"]);
        let other = Territory::Of_Files(["a/b.rs", "c/d.rs"]);
        let different = Territory::Of_Files(["a/b.rs"]);

        assert_eq!(one.As_Subject_Set(), other.As_Subject_Set());
        assert_ne!(one.As_Subject_Set(), different.As_Subject_Set());
    }

    #[test]
    fn Test_Intersect_Should_Report_Overlaps_For_A_Shared_Path()
    {
        let left = Territory::Of_Files(["a/b.rs"]);
        let right = Territory::Of_Files(["a/b.rs", "c/d.rs"]);

        assert!(matches!(left.Intersect(&right), Intersection::Overlaps(_)));
        assert_eq!(right.Intersect(&Territory::Of_Files(["e/f.rs"])), Intersection::Disjoint);
    }

    #[test]
    fn Test_Ambiguous_Paths_Should_Pair_Two_Spellings_Of_One_File()
    {
        let territory = Territory::Of_Files(["src/Main.rs", "src/main.rs", "src/other.rs"]);

        let pairs = territory.Ambiguous_Paths();

        assert_eq!(pairs, vec![("src/Main.rs".to_owned(), "src/main.rs".to_owned())]);
    }

    #[test]
    fn Test_Absent_Paths_Should_Name_Only_The_Paths_Not_In_The_Tree()
    {
        let territory = Territory::Of_Files(["present.rs", "absent.rs", "created-later.rs"]);
        let filesystem = Filesystem_With(&["present.rs"]);
        let root = std::path::Path::new("repository-root");

        let absent = territory.Absent_Paths(root, &filesystem);

        assert_eq!(absent, vec!["absent.rs".to_owned(), "created-later.rs".to_owned()]);
    }

    #[test]
    fn Test_Absent_Paths_Should_Ignore_Patterns()
    {
        let territory = Territory::Empty().With_Pattern("crates/**");
        let filesystem = Filesystem_With(&[]);

        assert!(territory.Absent_Paths(std::path::Path::new("root"), &filesystem).is_empty());
    }

    /// A [`nomos_platform::FileSystem`] that reports only the given file names as present,
    /// so an absent-path assertion can be driven against a tree this test constructs rather
    /// than the real one.
    fn Filesystem_With<'a>(present: &'a [&'a str]) -> impl nomos_platform::FileSystem + 'a
    {
        use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
        use nomos_platform::{FileSystem, FileSystemError};
        use std::path::Path;

        struct Fake<'a>
        {
            present: &'a [&'a str],
        }

        impl Strategy for Fake<'_>
        {
            const STRENGTH: DeterminismStrength = DeterminismStrength::State;
            const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
            const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
        }

        impl FileSystem for Fake<'_>
        {
            fn Read_To_String(&self, _path: &Path) -> Result<String, FileSystemError>
            {
                unimplemented!("Absent_Paths only calls Exists")
            }

            fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
            {
                unimplemented!("Absent_Paths only calls Exists")
            }

            fn Exists(&self, path: &Path) -> bool
            {
                return self.present.iter().any(|name| path.ends_with(name));
            }
        }

        return Fake { present };
    }
}
