//! The completeness-mirror rule.
//!
//! `OD-COMPLETENESS-001` names the shape and `P10-FIRST-CHECK` asks for one rule that
//! runs end to end and judges real code. This is that rule, and it is the one chosen
//! because it is the only rule in this tree with three recorded historical instances to
//! test a judgment against.
//!
//! # What it judges
//!
//! A declared universe must be mirrored by a check that compares the declaration against
//! the reality it claims to enumerate. The universe declares that check by name, at the
//! site, and this rule resolves the name against the real source.
//!
//! Three outcomes, and the ordering between the last two is the whole point:
//!
//! | The universe | The rule says |
//! |---|---|
//! | names a check that exists | nothing — it is mirrored |
//! | names a check that does not exist | a **blocking** finding |
//! | names nothing | an **advisory** finding |
//!
//! A false claim of coverage is worse than an admitted gap. `enforcement.rs` already
//! says so about enforcers — a phantom is "worse than declaring no enforcer at all:
//! nothing runs, nothing can fail, and the declaration says the rule is covered so no
//! reader looks twice" — and the same asymmetry is why this rule blocks on one and
//! reports on the other. It is also what keeps the check green today: this workspace has
//! thirteen unmirrored universes, and a gate that can never be green is a gate everybody
//! learns to ignore.
//!
//! # Why the judgment is built on `EnforcementReach`
//!
//! Because it already exists and it is already right. `declared`, `expected` and
//! `computed` are exactly the three things this rule has — what the universe names, what
//! naming it amounts to as a claim, and what the source says it really amounts to — and
//! [`EnforcementReach::Is_Enforced`] and [`EnforcementReach::Is_Truthful`] are already
//! the two questions being asked. A second judgment written next to it would be a second
//! place for the same rule to be spelled, which is how two guards for one rule come to
//! disagree.

use crate::universe::{DeclaredUniverse, Reading, Read_Universes, UniverseKind};
use crate::SourceFile;
use nomos_contracts::{
    Applicability, EnforcementBreach, EnforcementReach, EnforcerRef, EvidenceClass, Finding,
    GateCategory, RuleId, SubjectId,
};
use nomos_model::Content_Digest;
use std::collections::BTreeSet;

/// The rule's stable identifier.
pub const COMPLETENESS_MIRROR: &str = "completeness-mirror";

/// Judges every declared universe in `sources`.
///
/// Findings come back sorted by subject name, which is the stable one. Sorting by path
/// would reorder the whole report when a file moves, and a report that reorders is a
/// report nobody can diff.
#[must_use]
pub fn Check_Completeness_Mirrors(sources: &[SourceFile]) -> Vec<Finding>
{
    let checks = Check_Names(sources);

    let mut universes: Vec<DeclaredUniverse> = Vec::new();
    let mut findings: Vec<Finding> = Vec::new();

    for source in sources
    {
        match Read_Universes(&source.path, &source.text)
        {
            Reading::Parsed(found) => universes.extend(found),
            // A file this rule could not read is reported, not skipped. Skipping it would
            // fold "there is nothing here" into "I could not look", which is the one
            // conflation `Applicability` exists to prevent.
            Reading::Unparseable { because } => findings.push(Unreadable(source, &because)),
        }
    }

    universes.sort();
    universes.dedup();

    findings.extend(
        universes
            .iter()
            .filter_map(|universe| return Judge(universe, &checks)),
    );
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));

    return findings;
}

/// A file the rule could not read.
///
/// Identified by the digest of its contents rather than by its name, because what could
/// not be parsed is this text — the same path holding different bytes is a different
/// fact, and a finding keyed on the name would look unchanged after an edit that fixed
/// it.
fn Unreadable(source: &SourceFile, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(Content_Digest(source.text.as_bytes())),
        subject_name: source.path.clone(),
        applicability: Applicability::Unparseable,
        evidence: EvidenceClass::Derived,
        // Advisory, and it would make no difference if it were not: `Can_Fail_A_Build`
        // consults the applicability too, and a rule that never read its subject must not
        // stop anybody. What this finding is for is being *counted* — a run that could not
        // read four files and reports clean is the defect, not the four files.
        gate: GateCategory::Advisory,
        summary: format!("could not be parsed, so no universe in it was judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

/// What one universe's declaration amounts to, and what is really true of it.
///
/// Returns `None` when the universe is mirrored, because a rule that emits a finding per
/// subject it approves of produces a report in which the defects cannot be found.
fn Judge(universe: &DeclaredUniverse, checks: &BTreeSet<String>) -> Option<Finding>
{
    let reach = Reach_Of(universe, checks);

    if reach.Is_Enforced()
    {
        return None;
    }

    // The two gates in play are not the same gate, and collapsing them is the mistake
    // this whole module is about. `reach.computed` is what the *universe's* declared
    // mirror amounts to — `Unreachable` for a phantom. The finding's gate is what *this
    // rule* does about that, and a false claim of coverage is the one outcome worth
    // failing a build over.
    let (gate, summary) = match reach.breaches.first()
    {
        Some(breach) => (GateCategory::Blocking, breach.Describe()),
        None => (
            GateCategory::Advisory,
            format!(
                "declares no mirror, so nothing compares this list against the reality it \
                 enumerates; a {} added without adding it here is outside every guard built \
                 on it, and those guards then pass by not looking",
                match universe.kind
                {
                    UniverseKind::Constant => "member",
                    UniverseKind::Enumeration => "variant",
                }
            ),
        ),
    };

    return Some(Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(Content_Digest(universe.name.as_bytes())),
        subject_name: universe.name.clone(),
        // The rule read the whole of what it binds: the declaration is in the text it was
        // handed, and so is every check name it resolved against.
        applicability: Applicability::Supported,
        // Computed from source by a deterministic rule, and no stronger than that source.
        evidence: EvidenceClass::Derived,
        gate,
        summary,
        locations: vec![universe.path.clone()],
    });
}

/// The enforcement claim a universe makes, and what the source says of it.
///
/// A universe that names no mirror declares [`EnforcerRef::Review`] and expects
/// [`GateCategory::Review`], which is *truthful* — an admitted gap is an honest
/// declaration, and `enforcement.rs` is explicit that it must not be conflated with an
/// overclaim. It is still not enforcement, which is why the finding is raised on
/// [`EnforcementReach::Is_Enforced`] and its severity read off the breaches.
fn Reach_Of(universe: &DeclaredUniverse, checks: &BTreeSet<String>) -> EnforcementReach
{
    let Some(claimed) = universe.claimed_mirror.as_ref()
    else
    {
        return EnforcementReach {
            rule: RuleId::New(COMPLETENESS_MIRROR),
            declared: vec![EnforcerRef::Review],
            expected: GateCategory::Review,
            computed: GateCategory::Review,
            breaches: Vec::new(),
        };
    };

    let enforcer = EnforcerRef::Check {
        name: claimed.clone(),
    };
    let resolves = checks.contains(claimed);

    return EnforcementReach {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        declared: vec![enforcer],
        // Naming a check is a claim that a violation would be caught. That is what
        // makes a name that resolves to nothing a false claim rather than a typo.
        expected: GateCategory::Blocking,
        computed: if resolves
        {
            GateCategory::Blocking
        }
        else
        {
            GateCategory::Unreachable
        },
        breaches: if resolves
        {
            Vec::new()
        }
        else
        {
            vec![EnforcementBreach::Phantom {
                name: claimed.clone(),
            }]
        },
    };
}

/// Every check name the sources define.
///
/// A check is a test function, by this workspace's naming convention: `fn Test_…`.
///
/// Parsed rather than matched, for the same reason discovery is. A `fn Test_X` written
/// inside a fixture string would resolve a claim that nothing actually checks, which is
/// the precise defect this rule exists to find — arriving through the rule's own back
/// door.
fn Check_Names(sources: &[SourceFile]) -> BTreeSet<String>
{
    let mut names = BTreeSet::new();

    for source in sources
    {
        if let Ok(file) = syn::parse_file(&source.text)
        {
            Check_Names_In_Items(&file.items, &mut names);
        }
    }

    return names;
}

/// Collects test function names, descending into modules and implementations.
fn Check_Names_In_Items(items: &[syn::Item], names: &mut BTreeSet<String>)
{
    for item in items
    {
        match item
        {
            syn::Item::Fn(function) => Remember(&function.sig.ident.to_string(), names),
            syn::Item::Mod(module) =>
            {
                if let Some((_, nested)) = module.content.as_ref()
                {
                    Check_Names_In_Items(nested, names);
                }
            }
            syn::Item::Impl(block) =>
            {
                for member in &block.items
                {
                    if let syn::ImplItem::Fn(function) = member
                    {
                        Remember(&function.sig.ident.to_string(), names);
                    }
                }
            }
            _ =>
            {}
        }
    }
}

/// Keeps a name if it is a check by this workspace's convention.
fn Remember(name: &str, names: &mut BTreeSet<String>)
{
    if name.starts_with("Test_")
    {
        names.insert(name.to_owned());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Findings_Over(sources: &[SourceFile]) -> Vec<Finding>
    {
        return Check_Completeness_Mirrors(sources);
    }

    fn Only(findings: &[Finding]) -> &Finding
    {
        assert_eq!(findings.len(), 1, "expected exactly one finding: {findings:?}");
        return findings.first().expect("just asserted the length is one");
    }

    /// ---- the three instances `OD-COMPLETENESS-001` analyses ----
    ///
    /// None of the three can be replayed from git; each was repaired at the site. They
    /// are reproduced here as they were originally written, which is the whole reason
    /// this rule takes source as an argument rather than reading the tree.
    ///
    /// Instance one and two: `Table::All`, a list kept beside the enum, over which
    /// `Assert_Complete` and `Assert_Landed` both quantified. A migration adding a table
    /// without adding it here left that table outside both guards.
    #[test]
    fn Test_The_Table_Universe_As_Originally_Written_Should_Be_Found_Unmirrored()
    {
        let findings = Findings_Over(&[SourceFile::New(
            "crates/spec/nomos-spec-store/src/store.rs",
            "impl Table\n\
             {\n\
             \x20   /// Every table in the schema.\n\
             \x20   pub const fn All() -> &'static [Self]\n\
             \x20   {\n\
             \x20   }\n\
             }\n",
        )]);

        let finding = Only(&findings);

        assert_eq!(finding.subject_name, "Table::All");
        assert_eq!(finding.rule.As_Str(), COMPLETENESS_MIRROR);
        assert!(finding.summary.contains("declares no mirror"), "{}", finding.summary);
    }

    /// Instance three: `GOVERNING_RECORD_IDS`, compared against a store seeded from
    /// `GOVERNING_RECORD_IDS` — a comparison that cannot fail. Six governing records sat
    /// outside it for months.
    #[test]
    fn Test_The_Governing_Universe_As_Originally_Written_Should_Be_Found_Unmirrored()
    {
        let findings = Findings_Over(&[SourceFile::New(
            "crates/spec/nomos-spec-store/src/governing.rs",
            "/// The records this store seeds itself with.\n\
             pub const GOVERNING_RECORD_IDS: &[&str] = &[\n\
             \x20   \"ARC-SPECDB-001\",\n\
             ];\n",
        )]);

        assert_eq!(Only(&findings).subject_name, "GOVERNING_RECORD_IDS");
    }

    /// And all three together, in one run, because `P10-FIRST-CHECK` asks for the rule
    /// to fail on all three rather than on each in isolation.
    #[test]
    fn Test_All_Three_Historical_Instances_Should_Fail_This_Rule()
    {
        let findings = Findings_Over(&[
            SourceFile::New(
                "crates/spec/nomos-spec-store/src/store.rs",
                "impl Table\n{\n    pub const fn All() -> &'static [Self]\n    {\n    }\n}\n",
            ),
            SourceFile::New(
                "crates/spec/nomos-spec-store/src/governing.rs",
                "pub const GOVERNING_RECORD_IDS: &[&str] = &[];\n",
            ),
            SourceFile::New(
                "tests/contract/src/gates.rs",
                "pub const CORPUS_VARIABLES: &[&str] = &[];\n",
            ),
        ]);

        let judged: Vec<&str> = findings
            .iter()
            .map(|finding| return finding.subject_name.as_str())
            .collect();

        assert_eq!(
            judged,
            vec!["CORPUS_VARIABLES", "GOVERNING_RECORD_IDS", "Table::All"],
            "all three instances OD-COMPLETENESS-001 analyses must be found"
        );
    }

    /// ---- the rule can say clean ----
    ///
    /// The control that stops the rule being a counter. A check that fires on everything
    /// is not a judgment, and a gate that can never be green is one everybody learns to
    /// ignore.
    #[test]
    fn Test_A_Universe_Whose_Mirror_Exists_Should_Produce_No_Finding()
    {
        let findings = Findings_Over(&[
            SourceFile::New(
                "a.rs",
                "/// Mirrored by `Test_Every_Table_Should_Be_Declared`.\n\
                 pub const TABLES: &[&str] = &[];\n",
            ),
            SourceFile::New(
                "a_test.rs",
                "#[test]\nfn Test_Every_Table_Should_Be_Declared()\n{\n}\n",
            ),
        ]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// ---- the negative control this rule exists for ----
    ///
    /// A universe naming a check that does not exist reads as covered and checks nothing.
    /// It is the only outcome that blocks, and it must block: an admitted gap is honest,
    /// a false claim of coverage is not.
    #[test]
    fn Test_A_Mirror_That_Resolves_To_Nothing_Should_Block()
    {
        let findings = Findings_Over(&[SourceFile::New(
            "a.rs",
            "/// Mirrored by `Test_Renamed_Away`.\npub const TABLES: &[&str] = &[];\n",
        )]);

        let finding = Only(&findings);

        assert_eq!(finding.gate, GateCategory::Blocking);
        assert!(
            finding.Can_Fail_A_Build(),
            "a claim of coverage that checks nothing must be able to stop a build"
        );
        assert!(
            finding.summary.contains("Test_Renamed_Away"),
            "the finding must name the check that resolved to nothing: {}",
            finding.summary
        );
    }

    /// The severity ordering, asserted directly, because it is the judgment call this
    /// rule makes and a later edit could quietly invert it.
    #[test]
    fn Test_A_False_Claim_Should_Outrank_An_Admitted_Gap()
    {
        let admitted = Findings_Over(&[SourceFile::New(
            "a.rs",
            "pub const TABLES: &[&str] = &[];\n",
        )]);
        let false_claim = Findings_Over(&[SourceFile::New(
            "a.rs",
            "/// Mirrored by `Test_Nowhere`.\npub const TABLES: &[&str] = &[];\n",
        )]);

        assert_eq!(Only(&admitted).gate, GateCategory::Advisory);
        assert_eq!(Only(&false_claim).gate, GateCategory::Blocking);
        assert!(
            Only(&false_claim).gate > Only(&admitted).gate,
            "a phantom mirror must outrank an admitted gap"
        );
        assert!(
            !Only(&admitted).Can_Fail_A_Build(),
            "an admitted gap is honest, and thirteen of them exist; blocking on those \
             makes a gate that can never be green"
        );
    }

    /// A finding must carry its own provenance rather than leaving the reader to assume
    /// it. `Derived` is the honest class: computed from source by a deterministic rule.
    #[test]
    fn Test_A_Finding_Should_Report_How_It_Was_Come_By()
    {
        let findings = Findings_Over(&[SourceFile::New("a.rs", "pub const T: &[&str] = &[];\n")]);
        let finding = Only(&findings);

        assert_eq!(finding.evidence, EvidenceClass::Derived);
        assert!(finding.Is_Mechanical());
        assert_eq!(finding.applicability, Applicability::Supported);
    }

    /// Identity is the name, not the path. A universe that moves file is the same
    /// universe, and a finding keyed on its location would close and reopen for free.
    #[test]
    fn Test_The_Same_Universe_In_Two_Places_Should_Keep_One_Identity()
    {
        let here = Findings_Over(&[SourceFile::New("a.rs", "pub const T: &[&str] = &[];\n")]);
        let moved = Findings_Over(&[SourceFile::New("b/c.rs", "pub const T: &[&str] = &[];\n")]);

        assert_eq!(Only(&here).subject, Only(&moved).subject);
        assert_ne!(Only(&here).locations, Only(&moved).locations);
    }

    /// An empty tree yields nothing, and that is exactly why the caller has to check for
    /// vacuity. Asserted here so the property is written down where the rule is, rather
    /// than being an unstated assumption the composition root happens to cover.
    #[test]
    fn Test_No_Sources_Should_Produce_No_Findings()
    {
        assert!(Check_Completeness_Mirrors(&[]).is_empty());
    }

    #[test]
    fn Test_A_Check_Name_Should_Be_Found_Wherever_It_Is_Defined()
    {
        let names = Check_Names(&[SourceFile::New(
            "a.rs",
            "    #[test]\n    fn Test_Something_Should_Hold()\n    {\n    }\n",
        )]);

        assert!(names.contains("Test_Something_Should_Hold"), "{names:?}");
    }

    /// The rule's own back door. A check name written inside a fixture string is not a
    /// check, and resolving it would let a claim pass while nothing checks it — the exact
    /// defect this rule exists to find, arriving through the resolver instead of the
    /// scanner. This is why check names are parsed rather than matched.
    #[test]
    fn Test_A_Check_Named_Only_Inside_A_Fixture_Should_Not_Resolve()
    {
        let findings = Findings_Over(&[
            SourceFile::New(
                "a.rs",
                "/// Mirrored by `Test_Only_In_A_Fixture`.\npub const T: &[&str] = &[];",
            ),
            SourceFile::New(
                "b.rs",
                "fn Fixture() { let source = \"fn Test_Only_In_A_Fixture() {}\"; }",
            ),
        ]);

        assert_eq!(Only(&findings).gate, GateCategory::Blocking);
    }

    /// A file that could not be read is reported, not skipped. A run that silently drops
    /// what it could not parse and reports clean is the shape this workspace keeps
    /// finding — and `Applicability` is the field that says so.
    #[test]
    fn Test_An_Unparseable_File_Should_Be_Reported_And_Not_Fail_The_Build()
    {
        let findings = Findings_Over(&[SourceFile::New("broken.rs", "pub const ??? = ;")]);
        let finding = Only(&findings);

        assert_eq!(finding.applicability, Applicability::Unparseable);
        assert!(
            !finding.Can_Fail_A_Build(),
            "a rule that could not read its subject must not stop anybody"
        );
        assert!(finding.summary.contains("could not be parsed"), "{}", finding.summary);
    }

    /// A mention is not a definition. Resolving against prose would let a comment naming
    /// a deleted test keep the claim alive, which is the defect one level up.
    #[test]
    fn Test_A_Test_Named_Only_In_Prose_Should_Not_Resolve()
    {
        let findings = Findings_Over(&[
            SourceFile::New(
                "a.rs",
                "/// Mirrored by `Test_Deleted`.\npub const T: &[&str] = &[];\n",
            ),
            SourceFile::New("b.rs", "// see Test_Deleted for the comparison\n"),
        ]);

        assert_eq!(Only(&findings).gate, GateCategory::Blocking);
    }
}
