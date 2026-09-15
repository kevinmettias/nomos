//! One blocking Phantom finding from `Check_Completeness_Mirrors`, turned into a real,
//! judgment-free [`CorrectionCandidate`].
//!
//! Moved here from `nomos-cli`'s own `correct/candidate.rs`, unchanged in substance, as
//! part of giving correction planning and lifecycle a seam both hosts call rather than a
//! CLI module the second host cannot reach. Everything this module's own doc originally
//! said about why this one finding shape, and only this one, is safe to correct without
//! judgment still applies unchanged.
//!
//! # Why this rule, and why only this one finding shape of it
//!
//! `OD-CORRECTIONS-001` checked every rule shipped at the time and found none produced a
//! violation with a safe, judgment-free fix — `Check_Naming_Convention`'s own violation
//! would need every call site renamed to stay correct, "a real refactor requiring semantic
//! understanding this workspace has no infrastructure for." That reasoning is still right
//! for a rename. It does not reach this case, and the amendment this item makes to that
//! record says why: a **Phantom** finding from `Check_Completeness_Mirrors` is a doc
//! comment naming a check that provably does not exist — `crates/rules/nomos-rules/src/
//! mirror/reach.rs`'s own `Resolution` only raises `Phantom` when the claimed name does
//! not resolve against the real check index. The one safe, judgment-free action is
//! striking that already-false claim. It requires no semantic understanding of what the
//! *right* check would be, because it does not assert one; it only stops the doc comment
//! from asserting a wrong one. It cannot break compilation, because a doc comment is not
//! code and nothing in this workspace resolves an identifier against one. And it cannot
//! remove a *true* claim, because the rule only reaches `Phantom` when the claim is
//! already false — the same asymmetry `mirror.rs`'s own module doc states: "A false claim
//! of coverage is worse than an admitted gap."
//!
//! An **admitted-gap** finding (`Verdict::Admitted_Gap`, no claim at all) is deliberately
//! out of scope here: inventing a mirror name for one is a judgment this crate refuses to
//! make on a rule's behalf, the same declared-not-computed boundary `CorrectionClass`
//! itself draws.
//!
//! # Why the claimed name is read off `Finding::summary`, not off a new field
//!
//! `nomos_contracts::Finding` is a widely shared, load-bearing shape, and widening it for
//! one rule's one violation kind would be the second, wider change this crate's own
//! module doc warns against building ahead of a second real case. `EnforcementBreach::
//! Phantom::Describe` is already a tested, stable contract
//! (`crates/contracts/nomos-contracts/src/enforcement/breach.rs`'s own
//! `Test_Every_Breach_Should_Describe_Itself_Usefully`), and `Unresolved_Claim`
//! (`crates/rules/nomos-rules/src/mirror/verdict.rs`) only ever puts that `Describe()`
//! text — verbatim, at most with a shortfall clause appended — into a Blocking finding's
//! summary. [`Phantom_Claim`] parses exactly that stable prefix and refuses, returning
//! `None`, on anything that does not match it byte for byte, rather than guess.

use nomos_contracts::{Finding, GateCategory};
use nomos_corrections::{CandidateLabel, ChangeSet, CorrectionCandidate, CorrectionClass, Edit};
use nomos_model::EvidenceRef;
use nomos_platform::FileSystem;
use std::path::Path;

/// The stable text `EnforcementBreach::Phantom::Describe` puts after the claimed name,
/// which `Unresolved_Claim` never alters for a Blocking finding — only ever appends to.
const PHANTOM_TAIL: &str = "resolves to no check, so this rule is declared enforced and never runs";

/// The marker a universe declares its claimed mirror with, in the real source text —
/// `crates/rules/nomos-rules/src/universe.rs`'s own `MIRROR_MARKER`, restated here because
/// that constant is private to a crate this one only consumes through its public rule
/// functions.
const MIRROR_MARKER: &str = "Mirrored by `";

/// A phantom mirror claim this module can safely correct: the exact line it lives on, and
/// what a corrected file looks like with that line struck.
pub(crate) struct PhantomClaim<'a>
{
    /// The finding this claim was read from — carried through so a caller can report it
    /// alongside the candidate it produced.
    pub finding: &'a Finding,
    /// The file the claim is declared in, exactly as `finding.locations` names it.
    pub path: &'a str,
    /// The exact backtick-quoted name the doc comment claims.
    pub claimed: String,
}

/// The claimed name a blocking Phantom finding names, or `None` if `finding` is not one --
/// not this rule's finding, not a Blocking gate, or a summary that does not match the one
/// stable shape `Unresolved_Claim` ever produces for a phantom. Refusing on a mismatch
/// rather than approximating is the same discipline `naming.rs`'s own floor states:
/// "nothing weaker than a sound parse can promise that."
#[must_use]
pub(crate) fn Phantom_Claim(finding: &Finding) -> Option<PhantomClaim<'_>>
{
    if !Is_A_Blocking_Phantom(finding)
    {
        return None;
    }

    let claimed = Claimed_Name(&finding.summary)?;
    let path = Sole_Location(finding)?;

    return Some(PhantomClaim {
        finding,
        path,
        claimed: claimed.to_owned(),
    });
}

/// Whether `finding` is even a candidate to parse: this rule's own identifier, at the
/// Blocking gate a phantom is always raised at.
fn Is_A_Blocking_Phantom(finding: &Finding) -> bool
{
    return finding.rule.As_Str() == nomos_rules::COMPLETENESS_MIRROR && finding.gate == GateCategory::Blocking;
}

/// The claimed name `summary` names, or `None` if the text does not match the one stable
/// shape `Unresolved_Claim` ever produces for a phantom.
fn Claimed_Name(summary: &str) -> Option<&str>
{
    let after_open_quote = summary.strip_prefix('`')?;
    let (claimed, rest) = after_open_quote.split_once('`')?;
    let rest = rest.strip_prefix(' ')?;
    if !rest.starts_with(PHANTOM_TAIL)
    {
        return None;
    }
    if claimed.trim().is_empty()
    {
        return None;
    }

    return Some(claimed);
}

/// The one location `finding` names, or `None` if it names zero locations or more than one
/// -- this module refuses to guess which one a correction would apply to.
fn Sole_Location(finding: &Finding) -> Option<&str>
{
    let [path] = finding.locations.as_slice()
    else
    {
        return None;
    };

    return Some(path);
}

/// Why [`Candidate_For`] could not build a candidate for an otherwise-real phantom claim.
#[derive(Debug)]
pub(crate) enum ClaimError
{
    /// The file `claim.path` names could not be read from `root`.
    Unreadable(nomos_platform::FileSystemError),
    /// The exact "Mirrored by `{name}`" marker this claim names appears zero, or more
    /// than one, time in the file. Either way this module refuses to guess which line is
    /// the real declaration.
    Ambiguous
    {
        occurrences: usize,
    },
}

/// Builds the one real [`CorrectionCandidate`] for `claim`: the file with its false claim's
/// whole line struck, nothing else touched. Returns the candidate alongside the exact
/// `before`/`after` text it read and computed, so a caller stages against the same read
/// this function made rather than a second, independent one that could disagree with it.
///
/// # Errors
///
/// See [`ClaimError`].
pub(crate) fn Candidate_For<Fs: FileSystem>(root: &Path, claim: &PhantomClaim<'_>, filesystem: &Fs) -> Result<(CorrectionCandidate, String, String), ClaimError>
{
    let before = filesystem.Read_To_String(&root.join(claim.path)).map_err(ClaimError::Unreadable)?;
    let after = Strike_Claim_Line(ClaimStrike { before: &before, claimed: &claim.claimed })?;

    let edit = Edit::New(claim.path, Some(before.clone()), Some(after.clone()));
    let description = format!(
        "`{}` names no real check ({}); striking the false claim so `{}` reads as an admitted gap instead of a phantom cleared build",
        claim.claimed, claim.finding.subject_name, claim.path
    );
    let labels = vec![CandidateLabel::MechanicallySafe, CandidateLabel::BehaviorPreserving];

    let candidate = CorrectionCandidate::New(description, ChangeSet::Empty().With(edit), CorrectionClass::Mechanical, labels);

    return Ok((candidate, before, after));
}

/// The text to strike a line from, and the name the line to strike must claim -- paired so
/// a caller cannot transpose which is which, since both are `&str` and the compiler cannot
/// catch a swap between them on its own.
struct ClaimStrike<'a>
{
    /// The file's own text, before the strike.
    before: &'a str,
    /// The exact claimed name the one line to strike must name.
    claimed: &'a str,
}

/// `strike.before` with the one line naming `strike.claimed` removed, or a
/// [`ClaimError::Ambiguous`] if that line does not appear exactly once.
fn Strike_Claim_Line(strike: ClaimStrike<'_>) -> Result<String, ClaimError>
{
    let ClaimStrike { before, claimed } = strike;
    let marker = format!("{MIRROR_MARKER}{claimed}`");

    let occurrences = before.split_inclusive('\n').filter(|line| return line.contains(&marker)).count();
    if occurrences != 1
    {
        return Err(ClaimError::Ambiguous { occurrences });
    }

    let mut after = String::with_capacity(before.len());
    for line in before.split_inclusive('\n')
    {
        if !line.contains(&marker)
        {
            after.push_str(line);
        }
    }

    return Ok(after);
}

/// A supporting reference back to the finding a candidate was built from, for the
/// [`nomos_model::Evidence`] a real commit carries — `OD-CORRECTIONS-002`'s own
/// declared-not-judged `Evidence` parameter, filled honestly rather than left empty.
#[must_use]
pub(crate) fn Finding_Reference(claim: &PhantomClaim<'_>) -> EvidenceRef
{
    return EvidenceRef {
        kind: "finding".to_owned(),
        locator: format!("{}::{}", claim.finding.rule, claim.finding.subject_name),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Applicability, EvidenceClass, RuleId, SubjectId};
    use nomos_model::Content_Digest;
    use nomos_platform_std::StdFileSystem;

    #[test]
    fn Test_A_Real_Phantom_Findings_Claim_Should_Be_Read()
    {
        let finding = Phantom_Finding(PhantomFixture { claimed: "Test_Ghost", path: "a.rs" });

        let claim = Phantom_Claim(&finding).expect("this is a real phantom");

        assert_eq!(claim.claimed, "Test_Ghost");
        assert_eq!(claim.path, "a.rs");
    }

    #[test]
    fn Test_A_Findings_Claim_With_A_Shortfall_Suffix_Should_Still_Be_Read()
    {
        let mut finding = Phantom_Finding(PhantomFixture { claimed: "Test_Ghost", path: "a.rs" });
        finding.summary.push_str(" — and the check index is short 1 subject(s), none of whose text spells `Test_Ghost`, so no reading of them could have declared it");

        let claim = Phantom_Claim(&finding).expect("the tail is a prefix, not the whole summary");

        assert_eq!(claim.claimed, "Test_Ghost");
    }

    #[test]
    fn Test_An_Advisory_Finding_Should_Not_Be_Read_As_A_Phantom()
    {
        let mut finding = Phantom_Finding(PhantomFixture { claimed: "Test_Ghost", path: "a.rs" });
        finding.gate = GateCategory::Advisory;

        assert!(Phantom_Claim(&finding).is_none());
    }

    #[test]
    fn Test_A_Finding_From_A_Different_Rule_Should_Not_Be_Read()
    {
        let mut finding = Phantom_Finding(PhantomFixture { claimed: "Test_Ghost", path: "a.rs" });
        finding.rule = RuleId::New(nomos_rules::NAMING_CONVENTION);

        assert!(Phantom_Claim(&finding).is_none());
    }

    #[test]
    fn Test_An_Unrecognized_Summary_Shape_Should_Not_Be_Read()
    {
        let mut finding = Phantom_Finding(PhantomFixture { claimed: "Test_Ghost", path: "a.rs" });
        finding.summary = "something else entirely".to_owned();

        assert!(Phantom_Claim(&finding).is_none());
    }

    #[test]
    fn Test_Striking_The_One_Real_Line_Should_Remove_Only_That_Line()
    {
        let before = "/// A list.\n/// Mirrored by `Test_Ghost`.\npub const TABLES: &[&str] = &[];\n";

        let after = Strike_Claim_Line(ClaimStrike { before, claimed: "Test_Ghost" }).expect("the marker appears once");

        assert_eq!(after, "/// A list.\npub const TABLES: &[&str] = &[];\n");
    }

    #[test]
    fn Test_A_Missing_Marker_Should_Be_Ambiguous_Rather_Than_Silently_A_NoOp()
    {
        let before = "pub const TABLES: &[&str] = &[];\n";

        let refusal = Strike_Claim_Line(ClaimStrike { before, claimed: "Test_Ghost" }).expect_err("nothing to strike");

        assert!(matches!(refusal, ClaimError::Ambiguous { occurrences: 0 }));
    }

    #[test]
    fn Test_Two_Identical_Markers_Should_Be_Ambiguous_Rather_Than_Guessed()
    {
        let before = "/// Mirrored by `Test_Ghost`.\npub const A: &[&str] = &[];\n/// Mirrored by `Test_Ghost`.\npub const B: &[&str] = &[];\n";

        let refusal = Strike_Claim_Line(ClaimStrike { before, claimed: "Test_Ghost" }).expect_err("two lines both match");

        assert!(matches!(refusal, ClaimError::Ambiguous { occurrences: TWO_IDENTICAL_MARKERS }));
    }

    #[test]
    fn Test_A_Similar_But_Different_Claimed_Name_Should_Not_Match()
    {
        let before = "/// Mirrored by `Test_Ghosts`.\npub const TABLES: &[&str] = &[];\n";

        let refusal = Strike_Claim_Line(ClaimStrike { before, claimed: "Test_Ghost" }).expect_err("the names differ");

        assert!(matches!(refusal, ClaimError::Ambiguous { occurrences: 0 }));
    }

    /// How many lines two distinct declarations both claiming the same mirror name produce
    /// -- named so [`Test_Two_Identical_Markers_Should_Be_Ambiguous_Rather_Than_Guessed`]'s
    /// own assertion reads as "the fixture's own count" rather than an unexplained `2`.
    const TWO_IDENTICAL_MARKERS: usize = 2;

    #[test]
    fn Test_Candidate_For_Should_Read_Through_The_Filesystem_Port()
    {
        let root = std::env::temp_dir().join("nomos-correction-orchestration-candidate-for");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        std::fs::write(root.join("a.rs"), "/// Mirrored by `Test_Ghost`.\npub const TABLES: &[&str] = &[];\n")
            .expect("the temporary root the two statements above created holds this file");

        let finding = Phantom_Finding(PhantomFixture { claimed: "Test_Ghost", path: "a.rs" });
        let claim = Phantom_Claim(&finding).expect("this is a real phantom");

        let (candidate, before, after) = Candidate_For(&root, &claim, &StdFileSystem).expect("a real file with the marker exactly once");

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(before.contains("Mirrored by"));
        assert!(!after.contains("Mirrored by"));
        assert!(candidate.Description().contains("Test_Ghost"), "{}", candidate.Description());
    }

    /// The claimed name and the path a phantom fixture names, paired so a caller cannot
    /// transpose which is which -- both are `&str` and the compiler cannot catch a swap
    /// between them on its own.
    struct PhantomFixture<'a>
    {
        claimed: &'a str,
        path: &'a str,
    }

    fn Phantom_Finding(fixture: PhantomFixture<'_>) -> Finding
    {
        let PhantomFixture { claimed, path } = fixture;

        return Finding {
            rule: RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
            subject: SubjectId::From_Digest(Content_Digest(path.as_bytes())),
            subject_name: "TABLES".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: format!("`{claimed}` resolves to no check, so this rule is declared enforced and never runs"),
            locations: vec![path.to_owned()],
        };
    }
}
