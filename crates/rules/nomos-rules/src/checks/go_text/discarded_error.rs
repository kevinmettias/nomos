//! `a-discarded-error-is-explained` -- an error a Go file assigns to the blank identifier with no
//! comment saying why it cannot matter.
//!
//! # A verdict the text cannot reach
//!
//! Whether a value thrown away with `_` is an error is a fact about its type, and the call that
//! produced it may be declared in another package. Read on its face, this rule could only guess:
//! it flagged every `_ = f()` line -- a discarded `fmt.Sprintf` string among them -- and never saw
//! `n, _ := strconv.Atoi(s)`, `_, _ = w.Write(b)` or `var _ = os.Chdir(d)`, which are how most Go
//! code discards an error. So it reads `nomos.cap.go.discarded_values` instead: every value each
//! file assigns to `_`, with the type the type checker resolved for it, and reports each one whose
//! type is an error.
//!
//! # What it still reads from the text
//!
//! The explanation. A comment is not a type, and "a comment on the same line or the line above" is
//! a question about the file's lines, answered by the same reading its sibling rules share. Two
//! errors discarded on one line need one explanation, so a line is reported once.
//!
//! # Its floor, and a file with no answer
//!
//! [`Discarded_Values_Requirement`] asks for `SemanticallyResolved` and a `Sound` answer. Resolved,
//! because a type read from the spelling is the guess this rule exists to replace; sound, because a
//! finding says the value *is* an error. Completeness is not asked: a value missing from an answer
//! leaves an error unreported and never reports a non-error. A file whose fact cannot be read is
//! reported as unread and not judged -- never judged from its text instead, since that is the
//! reading already measured to be wrong more often than right.

use super::{Finding_For_Line, Has_Adjacent_Explanation, Lines_Of};
use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_go_types::DiscardedValue;
use nomos_capability::Requirement;
use nomos_contracts::{Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory, Guarantee, IncrementalGranularity, RuleId};
use std::collections::BTreeSet;

/// The code-standards discarded-error rule id.
pub const A_DISCARDED_ERROR_IS_EXPLAINED: &str = "a-discarded-error-is-explained";

/// What this rule needs from `nomos.cap.go.discarded_values` before it will believe an answer:
/// `SemanticallyResolved`, sound, completeness not required, per file. The module doc says why
/// each.
#[must_use]
pub(crate) fn Discarded_Values_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    return Requirement::New(nomos_cap_go_types::Capability(), nomos_cap_go_types::CONTRACT_VERSION, guarantee);
}

/// Reports every error a Go source discards to `_` with no adjacent comment explaining why it
/// cannot matter, each line once.
///
/// `sources` holds one entry per Go file the provider answered for, with its text, under the
/// subject its fact is filed under. Judges nothing, and reports nothing, when there are none.
#[must_use]
pub fn Check_A_Discarded_Error_Is_Explained(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        match Discarded_Values_Of(source, facts)
        {
            Ok(values) => findings.extend(Unexplained_Errors(source, &values)),
            Err(finding) => findings.push(*finding),
        }
    }
    findings.sort_by(|left, right| return (&left.subject_name, &left.summary).cmp(&(&right.subject_name, &right.summary)));
    return findings;
}

/// Each line of `source` discarding an error among `values` that no adjacent comment explains.
///
/// A pure function of the source and a decoded payload, testable against hand-built values with
/// no registry.
fn Unexplained_Errors(source: &SourceFile, values: &[DiscardedValue]) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut reported = BTreeSet::new();
    let mut findings = Vec::new();
    for value in values.iter().filter(|value| return value.is_error)
    {
        let Some(line) = usize::try_from(value.line).ok().filter(|line| return *line > 0)
        else
        {
            continue;
        };
        if Has_Adjacent_Explanation(&lines, line.saturating_sub(1)) || !reported.insert(line)
        {
            continue;
        }
        let because = format!("discards an error (`{}`) to `_` with no comment saying why it cannot matter", value.type_name);
        findings.push(Finding_For_Line(source, A_DISCARDED_ERROR_IS_EXPLAINED, line, &because));
    }

    return findings;
}

/// `source`'s discarded values, or a finding saying why they could not be read -- boxed, because a
/// `Finding` is large and the values are what the ordinary path carries.
fn Discarded_Values_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<Vec<DiscardedValue>, Box<Finding>>
{
    let capability = nomos_cap_go_types::Capability();
    // The key's inputs are empty: the provider files them so, because the type checker's reading of
    // the whole package is an input no reader holds.
    let fact = facts.Require(&capability, &source.subject, InputDigest::Of(&[]), &Discarded_Values_Requirement()).map_err(|applicability| {
        return Box::new(Unread_Finding(source, applicability, &format!("no admitted provider answered for it ({})", applicability.Label())));
    })?;
    if fact.payload.schema != nomos_cap_go_types::Payload_Schema()
    {
        let because = format!("the fact carries payload schema `{}`, which this build does not read", fact.payload.schema);
        return Err(Box::new(Unread_Finding(source, Applicability::Unparseable, &because)));
    }

    return nomos_cap_go_types::Parse_Payload(&fact.payload.bytes)
        .map(|payload| return payload.values)
        .map_err(|refusal| return Box::new(Unread_Finding(source, Applicability::Unparseable, &refusal.to_string())));
}

fn Unread_Finding(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(A_DISCARDED_ERROR_IS_EXPLAINED),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this file's discarded values could not be judged for errors: {because}"),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests;
