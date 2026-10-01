//! The dependency folder's own fixture: the one payload shape its rule test modules each built
//! by hand, case by case, and the one assertion they drew from it.
//!
//! Held here rather than beside any one rule because no rule owns it. [`super::violations`],
//! [`super::completeness`] and [`super::write_authority`] all judge the same decoded
//! [`DependencyPayload`] against the same declared [`ArchitecturePayload`], and their own test
//! modules are where the duplication showed: the same payload literal, the same call, and the
//! same three assertions about what came back, written out once per case in each of them.
//!
//! Distinct from [`crate::checks::test_support`], which builds the registry, store and reader
//! every fixture reaches a fact through. Nothing here needs a registry: a payload and a
//! declaration are values, and the rules that judge them are pure functions of both.
//!
//! Declared behind `#[cfg(test)]` at its `mod test_support;` site in [`super`] rather than gated
//! again here, for the reason `crate::checks::test_support` gives for the same choice.

use crate::SourceFile;
use nomos_cap_architecture::ArchitecturePayload;
use nomos_cap_dependency::{DependencyEdge, DependencyKind, DependencyPayload};
use nomos_contracts::{Finding, SubjectId};
use nomos_model::Content_Digest;

/// The one finding `judge` produces for `package`'s own payload — a single normal edge to
/// `target` — judged against `declared`.
///
/// The "exactly one" assertion belongs here rather than at each call site: a case that produced
/// none would otherwise read its subject or its summary off an empty slice, and panic somewhere
/// other than the line that was wrong.
///
/// `judge` is whichever rule's own `Violations_In` the case is about. It is named at the call
/// site rather than chosen here because the two rules that take this shape have identical
/// signatures, so a case accidentally passed the wrong one would compile: naming the judge is
/// what keeps a test about the authority rule from quietly passing against the direction one.
pub(in crate::checks) fn Sole_Violation(
    judge: fn(&ArchitecturePayload, &DependencyPayload, &SourceFile) -> Vec<Finding>,
    declared: &ArchitecturePayload,
    package: &str,
    target: &str,
) -> Finding
{
    let payload = DependencyPayload {
        package: package.to_owned(),
        edges: vec![DependencyEdge { target: target.to_owned(), kind: DependencyKind::Normal, optional: false }],
    };

    let findings = judge(declared, &payload, &Source_File(package));
    assert_eq!(findings.len(), 1, "{findings:?}");

    return findings.into_iter().next().expect("asserted len 1 above");
}

/// A source for `package`, keyed the way this folder's rules address a member: its own name as
/// both its path and its subject digest, with no text, since none of them reads `source.text`.
fn Source_File(package: &str) -> SourceFile
{
    return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
}
