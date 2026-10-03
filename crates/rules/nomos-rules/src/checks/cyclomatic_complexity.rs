//! `cyclomatic-complexity` -- a function with more independent paths through it than the
//! repository's declared limit.
//!
//! # A verdict no line count reaches
//!
//! This workspace's other size judgments count something else: `1500-lines` and its siblings count
//! a file's lines, and `nesting-depth` counts how deep one construct sits inside another. None of
//! them counts the paths through a function. A flat dispatcher of forty sequential `if`s is
//! shallow and can be short, and has forty-one paths; a long function of straight-line setup has
//! one. So a file can pass `1500-lines` holding a function this rule reports, and fail it holding
//! only functions this rule passes -- `nomos-check-orchestration`'s
//! `Test_Complexity_And_The_Line_Count_Should_Each_Report_The_File_The_Other_Passes` runs both
//! rules over exactly those two files, which is the evidence that neither subsumes the other. That is the trigger `OD-ROADMAP-004` named for
//! the metric family: a verdict the existing reading does not reach.
//!
//! # It reads a metric, and reads it only as the descriptor allows
//!
//! The number comes from `nomos.cap.metric.complexity`, whose every answer carries the
//! descriptor `MET-006` requires. This rule compares each function's value against the limit and
//! nothing else: it never sums two functions, never reports a file's or a crate's total, and
//! never reads a function the payload does not list as simple -- the three readings the
//! descriptor's `not-aggregable` aggregation and its missing-data clause forbid, which is `MET-007`
//! applied by the one reader that exists. [`Violations_In`] refuses a payload whose descriptor
//! says otherwise rather than reading it under terms it was not written for.
//!
//! # The limit is declared, or nothing is judged
//!
//! The limit is the `cyclomatic-complexity-max` axis of the limits family, read from
//! `nomos-limits.json` through the family's one resolver. Its undeclared meaning is written on the
//! axis as reported-as-undeclared (`OD-RULES-035` section 3): with no limit declared this rule
//! judges nothing and reports one advisory finding saying so, with what it measured, because a
//! default this crate chose would be judged as though the repository had.
//!
//! # Its floor, and what a finding claims
//!
//! [`Complexity_Requirement`] asks for `Syntactic` and a `Sound` answer and does not ask for
//! completeness: a reported value must be real, and this rule knows it may be low where a macro
//! hides a branch. So a function reported over the limit really is over it, a function not
//! reported may still be, and every breach carries `PartiallySupported` -- judged over the part of
//! the function a parse can see.

use super::structure::{Resolve_Count, Undeclared_Limit};
use crate::rule_descriptor::policy_axis::CYCLOMATIC_COMPLEXITY_MAX;
use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_complexity::{Aggregation, ComplexityPayload, Directionality, FunctionComplexity};
use nomos_capability::Requirement;
use nomos_contracts::{Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory, Guarantee, IncrementalGranularity, RuleId};

/// This rule's own identifier.
pub const CYCLOMATIC_COMPLEXITY: &str = "cyclomatic-complexity";

/// The record this implementation's contract is written against: `OD-ROADMAP-004` placed the metric
/// family, named the trigger this rule fires and fixed the shape its capability takes.
/// `tests/contract/tests/rule_contract_citation.rs` compares this against the record's own front
/// matter on every run.
pub const CYCLOMATIC_COMPLEXITY_CONTRACT_RECORD: &str = "OD-ROADMAP-004";

/// The version of [`CYCLOMATIC_COMPLEXITY_CONTRACT_RECORD`] this implementation was written against.
pub const CYCLOMATIC_COMPLEXITY_CONTRACT_RECORD_VERSION: u32 = 1;

/// What this rule needs from `nomos.cap.metric.complexity` before it will believe an answer:
/// `Syntactic`, sound, completeness not required, per file. The module doc says why each.
#[must_use]
pub(crate) fn Complexity_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    return Requirement::New(nomos_cap_complexity::Capability(), nomos_cap_complexity::CONTRACT_VERSION, guarantee);
}

/// Reports every Rust function whose cyclomatic complexity passes the declared limit -- or, with
/// no limit declared, one finding saying the limit is undeclared. Judges nothing, and reports
/// nothing, where there is no Rust source.
#[must_use]
pub fn Check_Cyclomatic_Complexity(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let rust: Vec<&SourceFile> = sources.iter().filter(|source| return crate::checks::populations::CYCLOMATIC_COMPLEXITY_POPULATION.Holds(source)).collect();
    if rust.is_empty()
    {
        return Vec::new();
    }

    let Some(limit) = Resolve_Count(facts, None, &CYCLOMATIC_COMPLEXITY_MAX)
    else
    {
        return vec![Undeclared_With_Measurement(&rust, facts)];
    };

    let mut findings = Vec::new();
    for source in rust
    {
        match Payload_Of(source, facts)
        {
            Ok(payload) => findings.extend(Violations_In(&payload, source, limit)),
            Err(finding) => findings.push(*finding),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Every function in `payload` whose complexity passes `limit`, as findings -- or one finding
/// refusing the payload if its descriptor does not allow reading values one function at a time.
///
/// A pure function of a decoded payload, testable against hand-built fixtures with no registry,
/// the split `reachability::Violations_In` draws for the same reason.
fn Violations_In(payload: &ComplexityPayload, source: &SourceFile, limit: usize) -> Vec<Finding>
{
    if payload.descriptor.aggregation != Aggregation::NotAggregable || payload.descriptor.directionality != Directionality::HigherIsWorse
    {
        let refusal = format!(
            "its descriptor declares {} aggregation, {}, which is not the reading this rule was written for",
            payload.descriptor.aggregation.Label(),
            payload.descriptor.directionality.Label()
        );
        return vec![Unread_Finding(source, Applicability::Unparseable, &refusal)];
    }

    return payload
        .functions
        .iter()
        .filter(|function| return function.complexity > limit)
        .map(|function| return Breach_Finding(source, function, limit))
        .collect();
}

fn Breach_Finding(source: &SourceFile, function: &FunctionComplexity, limit: usize) -> Finding
{
    let location = format!("{}:{}", source.path, function.line);
    return Finding {
        address: None,
        rule: RuleId::New(CYCLOMATIC_COMPLEXITY),
        subject: source.subject,
        subject_name: location.clone(),
        // Not `Supported`: the count is a floor wherever a macro hides a branch, so the function
        // was judged over the part of it a parse can see. The breach itself is certain.
        applicability: Applicability::PartiallySupported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "`{}` at {location} has cyclomatic complexity {} -- {} independent paths through it -- past the declared limit of {limit}; \
             each path is a case a reader must hold and a test must reach, so split the decisions into named helpers or replace a chain of them with a table",
            function.function, function.complexity, function.complexity
        ),
        locations: vec![location],
    };
}

/// The one finding an undeclared limit produces, carrying what was measured so a person choosing
/// a limit has the distribution's top in front of them.
fn Undeclared_With_Measurement(rust: &[&SourceFile], facts: &mut dyn FactReader) -> Finding
{
    let mut finding = Undeclared_Limit(CYCLOMATIC_COMPLEXITY, &CYCLOMATIC_COMPLEXITY_MAX);
    let mut measured = 0usize;
    let mut read = 0usize;
    let mut highest: Option<(String, FunctionComplexity)> = None;

    for source in rust
    {
        let Ok(payload) = Payload_Of(source, facts)
        else
        {
            // Nothing is judged here, so a file that could not be read changes no verdict; it only
            // leaves the measurement short, and the summary says by how much: `read` of the total.
            continue;
        };
        read = read.saturating_add(1);
        measured = measured.saturating_add(payload.functions.len());
        for function in payload.functions
        {
            if highest.as_ref().is_none_or(|(_, top)| return function.complexity > top.complexity)
            {
                highest = Some((source.path.clone(), function));
            }
        }
    }

    let most_complex = highest.map_or_else(String::new, |(path, top)| {
        return format!(", the most complex `{}` at {path}:{} with {}", top.function, top.line, top.complexity);
    });
    finding.summary = format!("{} -- {measured} function(s) were measured in {read} of {} Rust source(s){most_complex}", finding.summary, rust.len());
    return finding;
}

/// One source's decoded complexity payload, or a finding saying why it could not be read --
/// boxed, because a `Finding` is large and the payload is what the ordinary path carries.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<ComplexityPayload, Box<Finding>>
{
    let fact = Require_Fact(source, facts)?;
    if fact.payload.schema != nomos_cap_complexity::Payload_Schema()
    {
        let because = format!("the fact carries payload schema `{}`, which this build does not read", fact.payload.schema);
        return Err(Box::new(Unread_Finding(source, Applicability::Unparseable, &because)));
    }

    return nomos_cap_complexity::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Box::new(Unread_Finding(source, Applicability::Unparseable, &refusal.to_string())));
}

/// Requires `source`'s complexity fact, keyed by its text the way the provider files it.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Box<Finding>>
{
    let inputs = InputDigest::Of(&[source.text.as_bytes()]);
    return facts.Require(&nomos_cap_complexity::Capability(), &source.subject, inputs, &Complexity_Requirement()).map_err(|applicability| {
        return Box::new(Unread_Finding(source, applicability, &format!("no admitted provider answered for it ({})", applicability.Label())));
    });
}

fn Unread_Finding(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(CYCLOMATIC_COMPLEXITY),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this source's functions could not be judged for complexity: {because}"),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests;
