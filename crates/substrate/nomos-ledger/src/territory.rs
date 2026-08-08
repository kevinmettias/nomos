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

use nomos_contracts::SubjectId;
use nomos_model::{Content_Digest, Intersection, SetResolution, SubjectSet, UnknownReason};
use serde::{Deserialize, Serialize};

/// What a piece of work touches, as authored.
///
/// Paths are stored as written so the file stays reviewable, and compared after
/// normalization so two spellings of one file are one subject.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// [`nomos_model::Intersection::Unknown`]. An item may honestly say "this touches
    /// everything under `crates/spec/`" before anyone can enumerate that; what it may
    /// not do is have that claim silently compare as touching nothing.
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

        let mut shared = Vec::new();
        for mine in &self.paths
        {
            for theirs in &other.paths
            {
                if Contains_Or_Equals(mine, theirs)
                {
                    // The more specific of the two names the conflict most usefully: a
                    // report saying `crates/a` overlaps is less actionable than one
                    // saying `crates/a/src/lib.rs` does.
                    let narrower = if Normalize_Path(mine).len() >= Normalize_Path(theirs).len()
                    {
                        mine
                    }
                    else
                    {
                        theirs
                    };
                    let subject = Subject_Of(narrower);
                    if !shared.contains(&subject)
                    {
                        shared.push(subject);
                    }
                }
            }
        }

        return if shared.is_empty()
        {
            Intersection::Disjoint
        }
        else
        {
            Intersection::Overlaps(shared)
        };
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

/// Whether one path is the other, or contains it.
///
/// Purely textual, on normalized segments. `a/b` contains `a/b/c`; it does not contain
/// `a/bc`, which is why the comparison appends a separator rather than using a bare
/// `starts_with`.
fn Contains_Or_Equals(left: &str, right: &str) -> bool
{
    let left = Normalize_Path(left);
    let right = Normalize_Path(right);

    if left == right
    {
        return true;
    }

    // An empty path is the repository root, which contains everything. It arises from a
    // territory entry of "." or "/", and treating it as a normal name would make the
    // root disjoint from every file in the repository.
    if left.is_empty() || right.is_empty()
    {
        return true;
    }

    return right.starts_with(&format!("{left}/")) || left.starts_with(&format!("{right}/"));
}

/// The identity of the subject a path denotes.
#[must_use]
pub fn Subject_Of(path: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(Normalize_Path(path).as_bytes()));
}

/// Reduces an authored path to the text its identity is computed from.
///
/// Separators are unified, `./` prefixes and repeated or trailing separators are
/// dropped, and the result is lowercased.
///
/// # Why case is folded
///
/// On Windows and macOS, `src/Main.rs` and `src/main.rs` are one file. Not folding means
/// two agents claim the same file, both are told the territory is disjoint, and the
/// second one's edit silently replaces the first — the exact failure this ledger exists
/// to prevent.
///
/// Folding has a cost, and it is the honest one to pay: on Linux those really are two
/// files, so two agents who could have worked in parallel are serialized instead. That
/// costs throughput. The alternative costs an edit, and an edit does not come back.
#[must_use]
pub fn Normalize_Path(path: &str) -> String
{
    let unified = path.trim().replace('\\', "/");

    // `.` segments are dropped along with empty ones, so `.`, `./`, `a/./b` and `a//b`
    // all reduce the same way. This is also what makes a lone `.` normalize to the empty
    // string, which is how the repository root is represented — and the root has to be
    // empty rather than a name, or it would compare as a sibling of everything it
    // actually contains.
    let joined = unified
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<&str>>()
        .join("/");

    return joined.to_lowercase();
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_model::Intersection;

    /// The property the normalization exists for. Two spellings of one file must be one
    /// subject, or the ledger hands out overlapping territory believing it is disjoint.
    #[test]
    fn Test_Spellings_Of_One_Path_Should_Be_One_Subject()
    {
        let canonical = Subject_Of("crates/kernel/nomos-model/src/digest.rs");

        for spelling in [
            "./crates/kernel/nomos-model/src/digest.rs",
            "crates\\kernel\\nomos-model\\src\\digest.rs",
            "crates//kernel/nomos-model/src/digest.rs",
            "  crates/kernel/nomos-model/src/digest.rs  ",
            "crates/kernel/nomos-model/src/Digest.rs",
        ]
        {
            assert_eq!(
                Subject_Of(spelling),
                canonical,
                "`{spelling}` must denote the same subject"
            );
        }
    }

    /// The negative control. If normalization collapsed everything, the test above would
    /// pass while the ledger refused every concurrent claim in the repository.
    #[test]
    fn Test_Different_Paths_Should_Be_Different_Subjects()
    {
        assert_ne!(Subject_Of("src/a.rs"), Subject_Of("src/b.rs"));
        assert_ne!(Subject_Of("src/a.rs"), Subject_Of("tests/a.rs"));
        assert_ne!(Subject_Of("a/b.rs"), Subject_Of("a-b.rs"));
    }

    #[test]
    fn Test_Distinct_Territories_Should_Be_Disjoint()
    {
        let left = Territory::Of_Files(["crates/a/src/lib.rs"]);
        let right = Territory::Of_Files(["crates/b/src/lib.rs"]);

        assert_eq!(left.Intersect(&right), Intersection::Disjoint);
    }

    /// The same file written two ways in two items must still collide. This is the
    /// end-to-end version of the normalization test, through the type the ledger uses.
    #[test]
    fn Test_One_File_Written_Two_Ways_Should_Overlap()
    {
        let left = Territory::Of_Files(["crates/a/src/lib.rs"]);
        let right = Territory::Of_Files(["./crates\\a\\src\\LIB.rs"]);

        assert!(!left.Intersect(&right).Permits_Concurrency());
    }

    /// The case this comparison exists for. A directory and a file inside it are the
    /// same work; comparing their identities says they are unrelated, because as
    /// identities they are.
    #[test]
    fn Test_A_Directory_Should_Contain_Its_Files()
    {
        let crate_root = Territory::Of_Files(["crates/spec/nomos-spec-model"]);
        let one_file = Territory::Of_Files(["crates/spec/nomos-spec-model/src/normalizer.rs"]);

        assert!(
            !crate_root.Intersect(&one_file).Permits_Concurrency(),
            "a directory claim must exclude a file inside it"
        );
        assert!(
            !one_file.Intersect(&crate_root).Permits_Concurrency(),
            "containment must be caught from either side"
        );

        // And the identity comparison genuinely cannot see it, which is why the
        // containment check is not redundant.
        assert_eq!(
            crate_root
                .As_Subject_Set()
                .Intersect(&one_file.As_Subject_Set()),
            Intersection::Disjoint
        );
    }

    /// The negative control for containment. Prefix matching on raw strings would make
    /// `crates/a` contain `crates/abc`, and the whole repository would serialize.
    #[test]
    fn Test_A_Sibling_With_A_Shared_Prefix_Should_Not_Be_Contained()
    {
        let short = Territory::Of_Files(["crates/nomos-spec"]);
        let similar = Territory::Of_Files(["crates/nomos-spec-model/src/lib.rs"]);

        assert_eq!(short.Intersect(&similar), Intersection::Disjoint);
    }

    /// The repository root contains everything, so an item reserving it excludes all
    /// others rather than colliding with none of them.
    #[test]
    fn Test_The_Repository_Root_Should_Contain_Everything()
    {
        let everything = Territory::Of_Files(["."]);
        let something = Territory::Of_Files(["crates/a/src/lib.rs"]);

        assert!(!everything.Intersect(&something).Permits_Concurrency());
    }

    #[test]
    fn Test_A_Pattern_Should_Make_Comparison_Unknown()
    {
        let vague = Territory::Empty().With_Pattern("crates/spec/**");
        let concrete = Territory::Of_Files(["crates/host/nomos-cli/src/work.rs"]);

        let answer = vague.Intersect(&concrete);

        assert!(matches!(answer, Intersection::Unknown(_)));
        assert!(!answer.Permits_Concurrency());
    }

    /// Territory stated at two resolutions cannot be compared by matching paths, and
    /// answering "disjoint" would be a confident wrong answer.
    #[test]
    fn Test_Mismatched_Resolutions_Should_Be_Unknown()
    {
        let by_file = Territory::Of_Files(["src/a.rs"]);
        let mut by_symbol = Territory::Of_Files(["src/a.rs"]);
        by_symbol.resolution = SetResolution::Symbol;

        let answer = by_file.Intersect(&by_symbol);

        assert!(matches!(answer, Intersection::Unknown(_)));
        assert!(!answer.Permits_Concurrency());
    }

    /// Two entries denoting one subject is a mistake by whoever wrote the item, and the
    /// ledger should say so rather than quietly treating the pair as one.
    #[test]
    fn Test_Ambiguous_Paths_Should_Be_Reported()
    {
        let confused = Territory::Of_Files(["src/Main.rs", "src/main.rs", "src/other.rs"]);

        let ambiguous = confused.Ambiguous_Paths();

        assert_eq!(ambiguous.len(), 1);
        assert!(
            Territory::Of_Files(["src/a.rs", "src/b.rs"])
                .Ambiguous_Paths()
                .is_empty(),
            "distinct paths are not ambiguous"
        );
    }

    /// A territory serialized for review must show paths, not digests. This is the
    /// property that justifies the ledger being a file.
    #[test]
    fn Test_The_Serialized_Form_Should_Show_Paths()
    {
        let territory = Territory::Of_Files(["crates/kernel/nomos-model/src/digest.rs"]);

        let rendered = serde_json::to_string(&territory).unwrap();

        assert!(
            rendered.contains("crates/kernel/nomos-model/src/digest.rs"),
            "a reviewer must be able to read what an item reserves: {rendered}"
        );
    }

    #[test]
    fn Test_An_Empty_Territory_Should_Be_Empty()
    {
        assert!(Territory::Empty().Is_Empty());
        assert!(!Territory::Of_Files(["a.rs"]).Is_Empty());
        assert!(!Territory::Empty().With_Pattern("a/**").Is_Empty());
    }
}
