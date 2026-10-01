//! `uncompiled-conditional-branch` -- a branch of a C# `#if` chain that none of the repository's
//! declared builds compiles.
//!
//! # A verdict the syntax provider cannot reach
//!
//! `nomos-lang-csharp` reads C# on its face: a `#if`/`#elif`/`#else` chain holds every branch's
//! declarations at once, and nothing in the file says which one the compiler reads, so it
//! declines to read inside any of them. Whether a branch is compiled turns on the symbols a build
//! defines -- a configuration's `DEBUG`, a framework's `NET8_0_OR_GREATER`, a project's own
//! `DefineConstants` -- and no parse tree holds them. So this is the one question about a
//! conditional that a syntax reading cannot answer and a build-aware one can: is this branch
//! compiled by anything this repository builds?
//!
//! It is chosen for what the answer costs a repository when it is *no*. A branch no declared build
//! compiles is never type-checked, never bound to the code around it and never run by a test that
//! builds any of those configurations, so it rots unseen -- the `#else` of a
//! `#if NET8_0_OR_GREATER` long after the last `net48` target was dropped, or a `#if DEBUG_TRACE`
//! nobody defines. Every other rule in this workspace reads the file with that branch in it and
//! judges it as ordinary code. This rule is the one that says it is not.
//!
//! # What it reads, and what "declared" means
//!
//! `nomos.cap.csharp.conditional_compilation`, one fact per file per declared build that
//! compiles it -- the repository names its builds in `nomos-csharp-builds.json`, and `MSBuild`
//! says which files each compiles. A branch is reported only when every one of those facts calls
//! it `Skipped`. A branch any build calls `Unevaluated` is not reported: that build's compiler
//! reports its condition as an error rather than skipping it, and "compiled by nothing" would be a
//! guess about a build that does not finish. A branch nested inside one already reported is not
//! reported again, since it is dead because its parent is.
//!
//! Every build's answer is needed before any branch can be called dead, since the build that
//! did not answer may be the one that compiles it. The composition therefore hands this rule the
//! facts of every declared build or of none, and says why when it is none; a file one of whose
//! facts cannot be read here is left unjudged and reported as unread.
//!
//! # Its floor, and what a finding claims
//!
//! [`Conditional_Requirement`] asks for `SemanticallyResolved` and a `Sound` answer, and does not
//! ask for completeness. Resolved, because a state read without the build's symbols is the guess
//! this rule exists to replace; sound, because a branch reported skipped must really be skipped by
//! every build, or the finding tells somebody to delete code that ships. A branch missing from an
//! answer would leave a dead one unreported and never report a live one, so completeness is not
//! what makes a finding true -- and an answer that disagrees with a sibling build's about which
//! branches the file has is refused rather than aligned by guesswork.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_csharp_semantics::{Branch, BranchState, BuildSelection, ConditionalPayload, ConditionalRegion};
use nomos_capability::Requirement;
use nomos_contracts::{Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory, Guarantee, IncrementalGranularity, RuleId};
use std::collections::BTreeMap;

/// This rule's own identifier.
pub const UNCOMPILED_CONDITIONAL_BRANCH: &str = "uncompiled-conditional-branch";

/// The record this implementation's contract is written against: `OD-ROADMAP-006` section 5
/// requires a compiler-backed provider for C#, and this is the rule that reads what it answers.
/// `tests/contract/tests/rule_contract_citation.rs` compares this against the record's own front
/// matter on every run.
pub const UNCOMPILED_CONDITIONAL_BRANCH_CONTRACT_RECORD: &str = "OD-ROADMAP-006";

/// The version of [`UNCOMPILED_CONDITIONAL_BRANCH_CONTRACT_RECORD`] this implementation was written
/// against.
pub const UNCOMPILED_CONDITIONAL_BRANCH_CONTRACT_RECORD_VERSION: u32 = 1;

/// What this rule needs from `nomos.cap.csharp.conditional_compilation` before it will believe an
/// answer: `SemanticallyResolved`, sound, completeness not required, per file. The module doc says
/// why each.
#[must_use]
pub(crate) fn Conditional_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    return Requirement::New(nomos_cap_csharp_semantics::Capability(), nomos_cap_csharp_semantics::CONTRACT_VERSION, guarantee);
}

/// Reports every branch of every C# conditional chain that none of the declared builds compiling
/// its file compiles.
///
/// `sources` holds one entry per file per build: the file's path, and the subject that build's
/// answer for it is filed under. Judges nothing, and reports nothing, when there are none.
#[must_use]
pub fn Check_Uncompiled_Conditional_Branch(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let mut answers: BTreeMap<&str, Option<Vec<ConditionalPayload>>> = BTreeMap::new();
    let mut findings = Vec::new();
    for source in sources
    {
        let entry = answers.entry(source.path.as_str()).or_insert_with(|| return Some(Vec::new()));
        match Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                if let Some(payloads) = entry
                {
                    payloads.push(payload);
                }
            }
            Err(finding) =>
            {
                findings.push(*finding);
                *entry = None;
            }
        }
    }

    for (path, payloads) in answers
    {
        if let Some(payloads) = payloads
        {
            findings.extend(Uncompiled_Branches(path, &payloads));
        }
    }

    findings.sort_by(|left, right| return (&left.subject_name, &left.summary).cmp(&(&right.subject_name, &right.summary)));
    return findings;
}

/// Every branch of `path` that each of `payloads` -- one per build compiling the file -- calls
/// skipped, outermost first, or one finding refusing answers that disagree about the file's own
/// branches.
///
/// A pure function of decoded payloads, testable against hand-built ones with no registry.
fn Uncompiled_Branches(path: &str, payloads: &[ConditionalPayload]) -> Vec<Finding>
{
    let Some(first) = payloads.first()
    else
    {
        return Vec::new();
    };
    let disagrees = payloads.iter().any(|payload| {
        return payload.regions.len() != first.regions.len()
            || payload.regions.iter().zip(&first.regions).any(|(one, other)| return (one.line, one.branch) != (other.line, other.branch));
    });
    if disagrees
    {
        let because = "its builds' answers disagree about which branches the file has, which one text cannot";
        return vec![Unread_Finding(path, Applicability::Unparseable, because)];
    }

    // A branch's own lines lie strictly between its directive and `end_line`, the directive that
    // closes it -- which is also where the next branch of the same chain opens. So a region opening
    // before the last reported one's end sits inside it, and one opening at that end is its sibling.
    let mut findings = Vec::new();
    let mut enclosing_end = 0usize;
    for (index, region) in first.regions.iter().enumerate()
    {
        let skipped_everywhere = payloads.iter().all(|payload| return payload.regions.get(index).is_some_and(|region| return region.state == BranchState::Skipped));
        if skipped_everywhere && region.line >= enclosing_end
        {
            findings.push(Uncompiled_Finding(path, region, payloads));
            enclosing_end = region.end_line;
        }
    }

    return findings;
}

fn Uncompiled_Finding(path: &str, region: &ConditionalRegion, payloads: &[ConditionalPayload]) -> Finding
{
    let location = format!("{path}:{}", region.line);
    let directive = match region.branch
    {
        Branch::Else => "#else".to_owned(),
        branch => format!("#{} {}", branch.Label(), region.condition),
    };
    let builds: Vec<String> = payloads.iter().map(|payload| return Build_Label(&payload.selection)).collect();

    return Finding {
        address: None,
        rule: RuleId::New(UNCOMPILED_CONDITIONAL_BRANCH),
        subject: nomos_model::Subject_Of_Path(path),
        subject_name: location.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "the `{directive}` branch at {location} is compiled by none of the {} declared build(s) that compile this file ({}), \
             so nothing this repository builds type-checks its lines; delete it, or declare the build that needs it in nomos-csharp-builds.json",
            builds.len(),
            builds.join(", ")
        ),
        locations: vec![location],
    };
}

/// A build as a person names it: its project, configuration and framework.
fn Build_Label(selection: &BuildSelection) -> String
{
    return format!("{} {} {}", selection.project, selection.configuration, selection.target_framework);
}

/// One source's decoded answer, or a finding saying why it could not be read -- boxed, because a
/// `Finding` is large and the payload is what the ordinary path carries.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<ConditionalPayload, Box<Finding>>
{
    let fact = Require_Fact(source, facts)?;
    if fact.payload.schema != nomos_cap_csharp_semantics::Payload_Schema()
    {
        let because = format!("the fact carries payload schema `{}`, which this build does not read", fact.payload.schema);
        return Err(Box::new(Unread_Finding(&source.path, Applicability::Unparseable, &because)));
    }

    return nomos_cap_csharp_semantics::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Box::new(Unread_Finding(&source.path, Applicability::Unparseable, &refusal.to_string())));
}

/// Requires `source`'s answer, under the subject its build's fact is filed under. The key's
/// inputs are empty: the provider files them so, because the build's symbols are an input no
/// reader holds.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Box<Finding>>
{
    let capability = nomos_cap_csharp_semantics::Capability();
    return facts.Require(&capability, &source.subject, InputDigest::Of(&[]), &Conditional_Requirement()).map_err(|applicability| {
        return Box::new(Unread_Finding(&source.path, applicability, &format!("no admitted provider answered for one of its builds ({})", applicability.Label())));
    });
}

fn Unread_Finding(path: &str, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(UNCOMPILED_CONDITIONAL_BRANCH),
        subject: nomos_model::Subject_Of_Path(path),
        subject_name: path.to_owned(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this file's conditional branches could not be judged against its declared builds: {because}"),
        locations: vec![path.to_owned()],
    };
}

#[cfg(test)]
mod tests;
