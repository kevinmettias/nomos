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

/// The identity of the subject a path denotes, for the purpose of holding it.
///
/// Deliberately not [`nomos_model::Subject_Of_Path`], which is what a *fact* is about.
/// The two agree on every path but one shape: this one folds a record filename onto the
/// identifier it carries, so an item that reserved `docs/records/OD-LEDGER-006` excludes
/// the holder of the file it became. See [`Normalize_Path`] for that rule and
/// `OD-MODEL-001` for why the difference is kept rather than settled either way.
#[must_use]
pub fn Subject_Of(path: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(Normalize_Path(path).as_bytes()));
}

/// Reduces an authored path to the text its identity is computed from.
///
/// Separators are unified, `./` prefixes and repeated or trailing separators are
/// dropped, the result is lowercased, and a record filename is reduced to the record
/// identifier it carries.
///
/// # Why the spelling half is not written here
///
/// Everything up to the record fold is [`nomos_model::Normalize_Path`], and it is called
/// rather than repeated. It used to be repeated — three times, here and in two fact
/// producers — and `OD-MODEL-001` records the decision to converge them. What a path
/// spells is the same question whether the answer addresses a claim or a fact, so the
/// reasoning for unifying separators and folding case now lives with the rule, in the
/// kernel.
///
/// What survives here is the half that is genuinely the ledger's, and the composition runs
/// in this direction — ledger over kernel, never a flag passed down — because the kernel
/// must not learn what a decision record is. A fact *about* a record file has to stay a
/// fact about that file.
///
/// # Why a record filename folds onto its identifier
///
/// `OD-LEDGER-001`'s authoring rule tells an item to reserve the record it will write *by
/// identifier*: `docs/records/OD-<AREA>-<NNN>`, not the directory records live in and not
/// a pattern. The reason it says identifier is good and has not changed — the slug on the
/// end of the filename is the writing, and the writing is not knowable when the item is
/// authored. So the identifier is the only name two items can both write down in advance,
/// which makes it the coordination token whether or not it is a path.
///
/// It is not a path. Nothing is ever at `docs/records/OD-LEDGER-006`; the file is
/// `docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md`.
/// Compared as paths the two are *siblings* — neither is a prefix of the other at a
/// separator — so before this rule an item reserving the identifier excluded nobody from
/// the file, and an item that named the filename outright excluded nobody who had
/// reserved the identifier. Two items could hold one record between them and both be told
/// the ledger was disjoint.
///
/// That is the same failure as `src/Main.rs` against `src/main.rs`, arriving by a
/// different route, and it has the same answer: **two spellings of one thing are one
/// subject.** A record has two spellings — the identifier it was allocated and the file it
/// became — and folding the second onto the first is what makes them one. Containment is
/// deliberately *not* where this lives. The identifier does not contain the record; it is
/// the record, and saying so here leaves [`Contains_Or_Equals`] the pure path relation it
/// has always been, and fixes [`Subject_Of`] — and so [`Territory::As_Subject_Set`] — at
/// the same time. A rule written into containment would have left the identity form still
/// answering that the two are unrelated.
///
/// # Why it is scoped to `docs/records`
///
/// Because the general version of it destroys the ledger. "A name contains the names that
/// extend it with a hyphen" would make `crates/nomos-spec` contain
/// `crates/nomos-spec-model`, and the whole repository would serialize — the exact case
/// [`Contains_Or_Equals`] appends a separator to avoid, and
/// `Test_A_Sibling_With_A_Shared_Prefix_Should_Not_Be_Contained` is the guard on it.
///
/// So the rule is confined to the one directory where the identifier-to-filename relation
/// is a stated fact rather than an inference from the shape of a name: every canonical
/// record's registration under `crates/spec/nomos-spec-store/records/<ID>.record` writes
/// that mapping down as a `path:` line, and `OD-SPEC-006` makes `docs/records` the
/// authoring substrate and nothing else. Outside it, `tests/fixtures/case-001` and
/// `tests/fixtures/case-001-expected.md` stay two subjects, because there the resemblance
/// really is a coincidence.
///
/// Inside it the residual risk is over-folding, not under-folding: a file in
/// `docs/records` whose name happens to fit the grammar without being a record would fold
/// onto a name it does not mean, and two items would serialize over a record neither is
/// writing. That is the same trade case folding already makes and it falls the same way —
/// over-folding costs throughput, under-folding costs an edit.
#[must_use]
pub fn Normalize_Path(path: &str) -> String
{
    let folded = nomos_model::Normalize_Path(path);

    // Applied last, and to the folded text rather than to the authored spelling, so that
    // `Docs\Records\OD-LEDGER-006-x.md` reaches the same identifier as
    // `docs/records/od-ledger-006-x.md`. A rule that read the identifier off what the
    // author typed would have reintroduced, one level up, the hole the folding closes.
    if let Some(identifier) = Record_Identifier_Form(&folded)
    {
        return identifier;
    }

    return folded;
}

/// The directory this repository authors its decision records in, normalized.
///
/// Named here rather than passed in, because the rule below is about this directory and no
/// other — see [`Normalize_Path`] for why it has to be.
const RECORD_DIRECTORY: &str = "docs/records";

/// A record filename reduced to the identifier it carries, or [`None`] if it carries none.
///
/// Takes an already-folded path, because the identifier is read off the text and reading
/// it off two spellings would produce two identifiers.
///
/// # The grammar
///
/// Every record in this repository is named `<IDENTIFIER>-<slug>.md`, and every identifier
/// is one or more hyphen-separated words followed by an ordinal: `d-129`, `od-ledger-006`,
/// `arc-specdb-001`. The ordinal is what *ends* an identifier, so the first all-digit
/// component is its last component and everything after it is prose. Reading the ordinal
/// as the first digit group rather than the last is what keeps
/// `d-130-no-xvpe-dependency-before-phase-5.md` from being read as ending at the `5`.
///
/// Two things are deliberately not accepted. A name that does not *begin* with a word is
/// not an identifier at all — `2026-08-09-notes.md` reaches its first all-digit component
/// at `08` and would otherwise reduce to `docs/records/2026-08`, and a date is not an
/// allocation. And a component of digits must match a reservation's ordinal exactly rather
/// than prefix it, which falls out of comparing whole components: nothing makes
/// `od-ledger-001` the identifier of `od-ledger-0011-something.md`, because `0011` is one
/// component and it is not `001`.
fn Record_Identifier_Form(folded: &str) -> Option<String>
{
    let name = folded.strip_prefix(RECORD_DIRECTORY)?.strip_prefix('/')?;

    // Records are flat. Anything deeper is some other thing that happens to live under the
    // directory, and guessing at its shape is how a rule scoped to one convention escapes
    // the scope it was given. `docs/records-archive/...` is refused by the same two steps
    // above: stripping the directory leaves `-archive/...`, which has no leading separator.
    if name.contains('/')
    {
        return None;
    }

    let stem = name.strip_suffix(".md")?;
    let mut identifier = String::new();

    for (position, component) in stem.split('-').enumerate()
    {
        // An identifier begins with a word. `2026-08-09-notes.md` reaches its first
        // all-digit component at `08` and would otherwise reduce to `docs/records/2026-08`,
        // which two unrelated notes from the same month would then share. Requiring a
        // letter first refuses the whole name rather than the first component of it.
        if position == 0 && !component.bytes().any(|byte| return byte.is_ascii_alphabetic())
        {
            return None;
        }

        if position > 0
        {
            identifier.push('-');
        }
        identifier.push_str(component);

        let is_ordinal = position > 0
            && !component.is_empty()
            && component.bytes().all(|byte| return byte.is_ascii_digit());

        if is_ordinal
        {
            return Some(format!("{RECORD_DIRECTORY}/{identifier}"));
        }
    }

    return None;
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

    /// The defect `OD-LEDGER-016` closes, with both spellings written out.
    ///
    /// `docs/records/OD-LEDGER-006` is the identifier an item reserves and
    /// `docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md`
    /// is the file that identifier was allocated for. They are one record, so they are one
    /// subject; before the folding they were siblings and compared `Disjoint`.
    #[test]
    fn Test_A_Record_Identifier_And_Its_File_Should_Be_One_Subject()
    {
        let identifier = "docs/records/OD-LEDGER-006";
        let file = "docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-\
                    survive-it.md";

        assert_eq!(
            Subject_Of(file),
            Subject_Of(identifier),
            "`{file}` is the file `{identifier}` names, so the two must be one subject"
        );
    }

    /// The same thing through the type the ledger actually asks, in both directions.
    ///
    /// An item amending an existing record and an item that reserved that record's
    /// identifier must exclude each other, and which of them wrote which spelling must not
    /// decide it.
    #[test]
    fn Test_Reserving_A_Record_Should_Exclude_The_Writer_Of_Its_File()
    {
        let by_identifier = Territory::Of_Files(["docs/records/OD-LEDGER-006"]);
        let by_file = Territory::Of_Files([
            "docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md",
        ]);

        assert!(
            !by_identifier.Intersect(&by_file).Permits_Concurrency(),
            "reserving a record's identifier must exclude the item that writes its file"
        );
        assert!(
            !by_file.Intersect(&by_identifier).Permits_Concurrency(),
            "and from the other side, because exclusion is symmetric"
        );

        // Case and separator folding has to survive the reduction, or the hole closes for
        // one spelling of the filename and stays open for the next.
        let shouted = Territory::Of_Files([
            r"docs\Records\OD-LEDGER-006-A-Reason-Attached-To-A-Transition-Does-Not-Survive-It.MD",
        ]);
        assert!(!by_identifier.Intersect(&shouted).Permits_Concurrency());
    }

    /// Two records are still two records. The reduction must fold a filename onto *its*
    /// identifier and not onto a neighbouring one.
    #[test]
    fn Test_Two_Records_Should_Still_Be_Two_Subjects()
    {
        let one = Territory::Of_Files(["docs/records/OD-LEDGER-006"]);
        let another = Territory::Of_Files([
            "docs/records/OD-LEDGER-009-a-documents-validity-must-not-depend-on-when-it-is-read.md",
        ]);

        assert_eq!(one.Intersect(&another), Intersection::Disjoint);
    }

    /// The nearest miss, and the reason the ordinal is compared as a whole component.
    ///
    /// `od-ledger-001` is a prefix of the *text* `od-ledger-0011-...`, and a rule written
    /// with `starts_with` would fold the hundred-and-first record onto the first — the same
    /// shape as `crates/a` swallowing `crates/abc`, which is why containment appends a
    /// separator rather than matching bare text.
    #[test]
    fn Test_A_Longer_Ordinal_Should_Be_A_Different_Record()
    {
        let first = Territory::Of_Files(["docs/records/OD-LEDGER-001"]);
        let hundred_and_first =
            Territory::Of_Files(["docs/records/OD-LEDGER-0011-a-much-later-record.md"]);

        assert_eq!(first.Intersect(&hundred_and_first), Intersection::Disjoint);
        assert_ne!(
            Subject_Of("docs/records/OD-LEDGER-0011-a-much-later-record.md"),
            Subject_Of("docs/records/OD-LEDGER-001")
        );
    }

    /// The blast-radius control. The reduction is scoped to one directory, and everywhere
    /// else a name that merely looks like an identifier is a coincidence.
    ///
    /// Both misses are constructed rather than found: a fixture pair outside the record
    /// directory, and a directory whose *name* extends the record directory's. The second
    /// is the one a prefix test gets wrong — `docs/records-archive` starts with
    /// `docs/records` as text and is not inside it.
    #[test]
    fn Test_The_Reduction_Should_Not_Escape_The_Record_Directory()
    {
        let case = Territory::Of_Files(["tests/fixtures/case-001"]);
        let expectation = Territory::Of_Files(["tests/fixtures/case-001-expected.md"]);

        assert_eq!(
            case.Intersect(&expectation),
            Intersection::Disjoint,
            "outside `docs/records` an ordinal in a filename means nothing"
        );

        let record = Territory::Of_Files(["docs/records/OD-LEDGER-001"]);
        let neighbour =
            Territory::Of_Files(["docs/records-archive/OD-LEDGER-001-an-old-copy.md"]);

        assert_eq!(
            record.Intersect(&neighbour),
            Intersection::Disjoint,
            "a directory whose name extends `docs/records` is not `docs/records`"
        );
    }

    /// A leading digit group is a date, not an allocation, and folding it would invent a
    /// record called `docs/records/2026` that two unrelated notes would then share.
    #[test]
    fn Test_A_Leading_Digit_Group_Should_Not_Be_An_Ordinal()
    {
        assert_eq!(
            Normalize_Path("docs/records/2026-08-09-notes.md"),
            "docs/records/2026-08-09-notes.md"
        );
        assert_eq!(
            Territory::Of_Files(["docs/records/2026-08-09-notes.md"])
                .Intersect(&Territory::Of_Files(["docs/records/2026-08-10-notes.md"])),
            Intersection::Disjoint
        );
    }

    /// What is left alone. A file under `docs/records` that carries no ordinal is a file,
    /// and a registration file elsewhere keeps its extension — that directory is named by
    /// identifier too, and `records/OD-LEDGER-016.record` is a real path that resolves.
    #[test]
    fn Test_A_Path_Without_A_Record_Identifier_Should_Be_Untouched()
    {
        for path in [
            "docs/records/readme.md",
            "docs/records/OD-LEDGER-006",
            "crates/spec/nomos-spec-store/records/OD-LEDGER-016.record",
            "docs/records",
        ]
        {
            assert_eq!(
                Normalize_Path(path),
                path.to_lowercase(),
                "`{path}` carries no record filename to reduce"
            );
        }
    }

    #[test]
    fn Test_An_Empty_Territory_Should_Be_Empty()
    {
        assert!(Territory::Empty().Is_Empty());
        assert!(!Territory::Of_Files(["a.rs"]).Is_Empty());
        assert!(!Territory::Empty().With_Pattern("a/**").Is_Empty());
    }
}
