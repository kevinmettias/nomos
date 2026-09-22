//! A struct field whose validated range contradicts the type storing it, ported from
//! code-standards' `check-scalar-range` (shared package `rules/general/style/shared/
//! scalarrange`, Rust front end `rust_scalar_range.go`).
//!
//! `#[validate(range(min = 0, max = 255))] port: i64,` states two things about `port` — its
//! range, and its type — and they disagree: nothing the range permits requires 64 signed
//! bits, and nothing in it can go negative. The rule is that a number known to sit inside a
//! range is stored in the narrowest type that holds it, and in an unsigned type when the
//! range cannot go below zero.
//!
//! Two code-standards rule-doc ids name the two distinct predicates —
//! [`NONNEGATIVE_STORAGE_IS_UNSIGNED`] and [`A_KNOWN_RANGE_PICKS_ITS_TYPE`] — and
//! code-standards' own finding model carries no per-doc rule id at all (a tool's
//! `Enforces:` line is the only place multiple ids are named). This port is two separate
//! functions, one per predicate, the same choice `concurrency_text.rs`'s three functions
//! already made reading one shared value: a field violating both predicates gets two
//! separate findings, not one merged one.
//!
//! No repository-configurable dimension — the bound is always the author's own
//! already-written attribute, never a default a repository would state differently — so a
//! leaf module, not a capability.
//!
//! # Scoped to Rust only this increment
//!
//! Go's companion front end reads a different `validate:"min=N,max=N"` struct-tag string
//! syntax that needs its own separate parsing, not a trivial extension of the
//! attribute-clause reader here, and is left as a natural follow-up.
//!
//! # A simplification found to be safe, not merely convenient
//!
//! The real front end pairs an attribute with the field beneath it by sibling position
//! inside a parsed `field_declaration_list`. This port approximates that as a linear scan
//! with no struct-body boundary tracking at all: a `#[validate(range(...))]` attribute is
//! accumulated across consecutive attribute lines and consumed by the next non-attribute,
//! non-blank line, wherever in the file that happens. A function parameter list formatted
//! one argument per line has the identical `name: Type,` shape a struct field does, but
//! nobody writes this specific attribute above a parameter — the vocabulary itself is
//! narrow enough that the boundary tracking the real parser buys is not needed here.
//!
//! # Split by responsibility
//!
//! [`bound`] reads — an attribute and the field declaration beneath it become a [`Bound`],
//! with no opinion about whether that bound is wrong. [`narrowest`] answers what type would
//! hold a stated range. This file keeps only the judgment: the two predicates, the walk over
//! each source's bounds, and the finding each one renders to.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

mod bound;
mod narrowest;

use bound::{Bound, Bounded_Fields_In};
use narrowest::Narrowest_That_Holds;

/// The code-standards non-negative-storage rule id.
pub const NONNEGATIVE_STORAGE_IS_UNSIGNED: &str = "nonnegative-storage-is-unsigned";
/// The code-standards known-range rule id.
pub const A_KNOWN_RANGE_PICKS_ITS_TYPE: &str = "a-known-range-picks-its-type";

/// Reports a signed field whose stated minimum cannot go below zero.
#[must_use]
pub fn Check_Nonnegative_Storage_Is_Unsigned(sources: &[SourceFile]) -> Vec<Finding>
{
    return Findings_For(sources, NONNEGATIVE_STORAGE_IS_UNSIGNED, |bound| {
        return bound.declared_signed && bound.min.is_some_and(|min| return min >= 0);
    });
}

/// Reports a field wider than its stated range needs.
#[must_use]
pub fn Check_A_Known_Range_Picks_Its_Type(sources: &[SourceFile]) -> Vec<Finding>
{
    return Findings_For(sources, A_KNOWN_RANGE_PICKS_ITS_TYPE, |bound| {
        let (Some(min), Some(max)) = (bound.min, bound.max) else { return false };
        return Narrowest_That_Holds(min, max).is_some_and(|(_, bits, _)| return bits < bound.declared_bits);
    });
}

fn Findings_For(sources: &[SourceFile], rule: &str, matches: fn(&Bound) -> bool) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !source.Is_Written_In(RUST_LANGUAGE) || Is_Own_Implementation_File(source)
        {
            continue;
        }

        let source_findings = Findings_For_Source(source, rule, matches);
        findings.extend(source_findings);
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Findings_For_Source(source: &SourceFile, rule: &str, matches: fn(&Bound) -> bool) -> Vec<Finding>
{
    let lines: Vec<&str> = source.text.lines().collect();
    let mut findings = Vec::new();

    for bound in Bounded_Fields_In(&lines)
    {
        if !matches(&bound)
        {
            continue;
        }

        let finding = Bound_Finding(source, rule, &bound);
        findings.push(finding);
    }

    return findings;
}

fn Bound_Finding(source: &SourceFile, rule: &str, bound: &Bound) -> Finding
{
    let line_number = Line_Number(bound.line_index);
    let span = match (bound.min, bound.max)
    {
        (Some(min), Some(max)) => format!("{min}..={max}"),
        (Some(min), None) => format!("{min} and above"),
        (None, Some(max)) => format!("{max} and below"),
        (None, None) => "an unstated range".to_owned(),
    };
    let summary = if let Some((remedy, remedy_bits, _)) = Narrowest_That_Holds(bound.min.unwrap_or(0), bound.max.unwrap_or(0))
        && bound.min.is_some()
        && bound.max.is_some()
        && remedy_bits < bound.declared_bits
    {
        format!(
            "`{}` at {}:{line_number} stays within {span}, but is stored in `{}` ({} bits) for a range that needs {remedy}",
            bound.member, source.path, bound.declared_spelling, bound.declared_bits
        )
    }
    else
    {
        format!(
            "`{}` at {}:{line_number} stays within {span}, which cannot go below zero, but is stored in the signed type `{}`",
            bound.member, source.path, bound.declared_spelling
        )
    };

    return Finding {
        address: None,
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary,
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

/// This module's own path. Every fixture below spells a real attribute/field shape inside a
/// Rust string literal, which would otherwise self-match when this crate checks its own
/// workspace — the same self-exemption every other `*_text.rs`-shaped rule here carries for
/// the identical reason.
const OWN_IMPLEMENTATION_FILE: &str = "crates/rules/nomos-rules/src/checks/scalar_range.rs";

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return source.path.replace('\\', "/") == OWN_IMPLEMENTATION_FILE;
}

#[cfg(test)]
mod tests;
