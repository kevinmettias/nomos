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
//! Resolution is cross-file: the check a universe names almost never sits beside it, so this
//! rule's answer depends on the whole collected source set rather than on the file the
//! universe is declared in. A run that judged a truncated set would resolve a real check to
//! nothing and report a **blocking** finding against a declaration that is in fact mirrored.
//! `nomos gate run --include` did exactly that until `OD-GATE-025` took the scope off the
//! source set, so a gate run now judges every walked file and narrows only what it reports.
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
//! reports on the other. It is also what keeps the check green today: this workspace still has
//! unmirrored universes, and a gate that can never be green is a gate everybody learns to
//! ignore. How many is deliberately not restated here:
//! `tests/contract/tests/completeness_universes/table.rs` declares the count as
//! `UNMIRRORED_TOTAL` and checks it against the table beside it, and a number copied into this
//! comment would be an unchecked second copy of that one — which is exactly how this
//! sentence came to say twelve long after the real figure had fallen to three.
//!
//! There is a third outcome above both, added by `OD-RULES-001`: **the rule saying it
//! could not answer.** A subject whose syntax fact could not be read leaves the check
//! index short, and a claimed mirror the shortfall could have accounted for is reported at
//! the reader's [`Applicability`] rather than as an [`EnforcementBreach::Phantom`] —
//! because the rule cannot tell a false claim of coverage from a name it did not get to
//! look for. The asymmetry `D-134` decided is untouched; what is new is that absence no
//! longer has to be squeezed into one of its two arms.
//!
//! `OD-RULES-002` scopes that third outcome to the claims it actually bears on. Under
//! `OD-RULES-001` incompleteness was one flag over the whole run, so one unreadable file
//! anywhere silenced every phantom in the tree — including phantoms claimed in files whose
//! facts were read perfectly well, and whose claims the unreadable file could not have
//! resolved. That is a guard that reports rather than judges, and over this workspace it
//! was permanent: `tests/corpus/analysis/gamma/broken.rs` is a fixture the parser is
//! *supposed* to refuse, so the flag was set on every run. The downgrade stays and its
//! reasoning stays; what changes is that the index is asked whether it is short **of
//! something that could have resolved this name**, and the test for that is
//! [`Unread::Can_Have_Declared`].
//!
//! # Where the check names come from
//!
//! From `nomos.cap.syntax.items` facts, one [`nomos_analysis::FactReader::Require`] per
//! subject, under the floor [`crate::Syntax_Requirement`] declares. Not from a parser
//! vendored here: `D-134` created this workspace's second `syn` front end and recorded a
//! reason — replayability — that proves a rule takes its subject as an argument and does
//! not prove that the argument is text. `OD-RULES-001` withdraws the inference and moves
//! this half of the rule onto the fact layer. Universe discovery keeps its parser, for a
//! measured reason `universe_kind.rs` states.
//!
//! What that buys is not caching and not incrementality; this rule spends neither. It
//! spends [`nomos_contracts::Guarantee::Satisfies`]: the floor is the rule's own, a
//! composition root cannot lower it, and the difference between a parser and a line
//! scanner now has a verdict attached to it.
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

mod verdict;
mod reach;
mod index;
mod unread;
mod shortfall;
#[cfg(test)]
mod tests;

use index::{CheckIndex, Check_Index_Of};
use reach::Reach_Of;
use shortfall::Shortfall;
use unread::{Unread, Unread_Subject, Unreadable_Finding};

use crate::facts::Check_Names_In;
use crate::Reading;
use crate::universe_kind::Read_Universes;
use crate::UniverseKind;
use crate::{SourceFile, Syntax_Requirement_For};
// `provider_floor.rs` reaches this transitively through `tests.rs`'s own `use super::*` --
// production code here calls `Syntax_Requirement_For` only, never the bare floor, so this
// import is test-only and would otherwise warn unused in the plain library build.
#[cfg(test)]
use crate::Syntax_Requirement;
use nomos_analysis::{FactReader, InputDigest};
use nomos_contracts::{
    Applicability, EnforcementBreach, EnforcementReach, EnforcerRef, EvidenceClass, Finding,
    GateCategory, RuleId, SubjectId,
};
// check-dependency-placement reports this crate's `nomos_model` edge as this file's alone.
// It is one function, and it is the canonical digest -- OD-MODEL-001 converged three copies
// of it onto that home precisely so a caller names it rather than carrying its own.
use nomos_model::Content_Digest;
use std::collections::BTreeSet;

/// The rule's stable identifier.
pub const COMPLETENESS_MIRROR: &str = "completeness-mirror";

/// The record whose contract this rule implements.
///
/// `PKG-014` requires every rule implementation be traceable to one contract version and
/// mechanically checked for consistency with it. `D-134` is that contract, and
/// `tests/contract/tests/rule_contract_citation.rs` reads the record's own front matter on
/// every run and compares it against [`CONTRACT_RECORD_VERSION`], so an amendment this
/// implementation has not caught up to fails the check rather than sitting asserted only in
/// prose.
pub const CONTRACT_RECORD: &str = "D-134";

/// The version of [`CONTRACT_RECORD`] this implementation was written against.
pub const CONTRACT_RECORD_VERSION: u32 = 2;

/// Judges every declared universe in `sources`, resolving claimed mirrors against facts.
///
/// `facts` is the second half of the subject and not a service the rule reaches out to.
/// It is handed in for the same reason `sources` is: a test composes one over three files
/// that no longer exist anywhere and gets the same judgment the binary gets over the real
/// tree. There is one signature and not two — a text-only entry point kept beside this one
/// would be a second place for one rule to be spelled, and it would let a shipped binary
/// consult no fact while the tests all did.
///
/// Findings come back sorted by subject name, which is the stable one. Sorting by path
/// would reorder the whole report when a file moves, and a report that reorders is a
/// report nobody can diff.
#[must_use]
pub fn Check_Completeness_Mirrors(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let index = Check_Index_Of(sources, facts);

    // Both halves of the subject now come out of one fact per file. The universes are read
    // where the fact is read rather than from `source.text`, which is what removed this
    // crate's second Rust front end — `OD-RULES-001` named that the end condition and
    // `OD-SYNTAX-002` is the schema that met it.
    let mut findings = Unobserved_Findings(sources, &index);

    // One finding per subject whose fact could not be read, before any universe is judged.
    // A run that materialized nothing must not be able to render as a clean tree, and that
    // property has to hold whether or not the tree happens to declare a universe.
    findings.extend(index.unread.iter().map(Unread_Subject));

    findings.extend(Findings_Over_Universes(&index));
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));

    return findings;
}

/// One finding per file whose provider could not see doc comments.
///
/// Reported rather than skipped: folding it into "no universe here" is the
/// phantom-becomes-admitted-gap downgrade the schema version exists to stop.
fn Unobserved_Findings(sources: &[SourceFile], index: &CheckIndex<'_>) -> Vec<Finding>
{
    let mut findings: Vec<Finding> = Vec::new();

    for (path, because) in &index.unobserved
    {
        if let Some(source) = sources.iter().find(|candidate| return &candidate.path == path)
        {
            let finding = Unreadable_Finding(source, because);
            findings.push(finding);
        }
    }

    return findings;
}

/// A finding for every declared universe that is not mirrored.
///
/// Deduplicated first, because one universe declared in two files is one claim and two
/// findings about it would double-count the same defect.
fn Findings_Over_Universes(index: &CheckIndex<'_>) -> Vec<Finding>
{
    use crate::DeclaredUniverse;
    use verdict::Judgment_For_Universe;

    let mut universes: Vec<DeclaredUniverse> = index.universes.clone();
    universes.sort();
    universes.dedup();

    return universes
        .iter()
        .filter_map(|universe| return Judgment_For_Universe(universe, index))
        .collect();
}

// `mod tests;` above is `checks/mirror/tests.rs` and its own `tests/*.rs` split, which the
// coverage rule attributes to those files' own stems ("tests", "historical", "judgments", ...)
// rather than to `mirror` — a Rust unit is the FILE, and a module POINTED AT another file is
// not the same file. This block exists solely to give `mirror.rs`'s own public entry point an
// address IN THIS FILE; the exhaustive judgment-by-judgment suite stays in `mod tests` above.
#[cfg(test)]
mod address
{
    use super::*;
    use crate::checks::test_support::{self, Test_Context, TestOffering};
    use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SubjectId};
    use nomos_model::Content_Digest;

    const PARSER: &str = "nomos.test.mirror.address.parses";

    #[test]
    fn Test_Check_Completeness_Mirrors_Should_Block_On_A_Mirror_That_Resolves_To_Nothing()
    {
        let source = SourceFile::New(
            "a.rs",
            SubjectId::From_Digest(Content_Digest(b"a.rs")),
            "/// Mirrored by `Test_Renamed_Away`.\npub const TABLES: &[&str] = &[];\n",
        );
        let TestOffering { mut store, registry, offer } = test_support::Offering(
            nomos_cap_syntax::Capability_Contract(),
            nomos_cap_syntax::Capability(),
            nomos_cap_syntax::CONTRACT_VERSION,
            PARSER,
            Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File),
        );
        test_support::Materialize(
            &mut store,
            source.subject,
            &offer,
            nomos_analysis::InputDigest::Of(&[source.text.as_bytes()]),
            nomos_cap_syntax::Payload_Schema(),
            "unexpanded\t0\nitem\t0\tConstant\tPublic\tTABLES\t+ Mirrored by `Test_Renamed_Away`.\t+slice\n"
                .as_bytes()
                .to_vec(),
        );
        let mut reader = nomos_analysis::Reader::On(&store, &registry, Test_Context());

        let findings = Check_Completeness_Mirrors(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let finding = findings.first().expect("asserted len 1 above");
        assert_eq!(finding.subject_name, "TABLES");
        assert_eq!(finding.gate, GateCategory::Blocking);
    }
}
