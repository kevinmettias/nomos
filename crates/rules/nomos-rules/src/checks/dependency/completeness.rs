//! Judging whether an already-decoded dependency payload's own package has a declared zone.
//!
//! `violations.rs`'s `Violations_In` silently produces no findings for a package with no
//! entry in [`super::zones::ZONES`], naming the gap as a different defect —
//! `tests/contract`'s own `Test_Every_Member_Should_Declare_A_Band`'s subject, not this
//! rule's. This module is that subject, promoted from a contract test's hand assertion to a
//! Finding-producing judgment reachable through an ordinary `nomos check` run. A pure
//! function of an already-decoded payload, grouped apart from `reading.rs` for the same
//! reason [`super::violations`] is: testable against hand-built fixtures, no registry, no
//! store, no reader.

use crate::SourceFile;
use nomos_cap_dependency::DependencyPayload;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// Whether the repository under check declared an architecture at all -- true when at least
/// one of its members has an entry in [`super::zones::ZONES`].
///
/// A named type rather than a bare `bool` because it is the difference between two findings
/// that read almost alike and mean opposite things, and a positional `bool` at a call site
/// says neither of them.
#[derive(Clone, Copy)]
pub(super) struct DeclaresAnArchitecture(pub(super) bool);

/// A finding when `payload`'s own package has no declared zone — empty otherwise.
///
/// # Why the answer depends on the rest of the workspace
///
/// A package with no declared zone is one of two quite different things, and which one is
/// not a property of the package. Either this repository declared an architecture and this
/// member was left out of it -- a real gap, and what this rule exists to catch -- or this
/// repository declared no architecture at all, in which case the rule does not bind it.
///
/// `OD-RULES-003` decided what the second case gets, choosing [`Applicability::NotApplicable`]
/// over `MissingCapability` and `ConfigurationDisabled` with a reason given for each: the
/// capability establishing the dependency graph can be entirely present and correct while no
/// architecture has been declared, so the gap is in the policy input, not the fact input; and
/// a repository that never declared one has not switched anything off.
///
/// Before `OD-RULES-029` this rule could not tell the two apart, because `ZONES` names this
/// workspace's own crates as string literals -- so every member of any other repository
/// looked like a gap, and a foreign workspace got one advisory finding per crate saying it
/// had declared nothing, plus silence on the layering question the rule exists to answer.
/// `declares` is that distinction, computed once over the whole member set by this rule's
/// own caller, which is the only place that can see it.
#[must_use]
pub(super) fn Violations_In(payload: &DependencyPayload, source: &SourceFile, declares: DeclaresAnArchitecture) -> Vec<Finding>
{
    use super::zones::Zone_Of;

    if Zone_Of(&payload.package).is_some()
    {
        return Vec::new();
    }

    let DeclaresAnArchitecture(declared) = declares;
    if !declared
    {
        return vec![Unbound_Package(source, &payload.package)];
    }

    return vec![Violation_For_Package(source, &payload.package)];
}

/// `package` sits in a repository that declared no architecture, so this rule does not bind
/// it -- stated positively rather than left as a silence.
///
/// [`Applicability::NotApplicable`] is "the only variant that is a *positive* statement about
/// the absence of a judgment", and it is deliberately not coverage debt: a decision is not a
/// gap, so this does not make the run's claim incomplete the way an unreached subject would.
fn Unbound_Package(source: &SourceFile, package: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(super::DEPENDENCY_COMPLETENESS),
        subject: source.subject,
        subject_name: package.to_owned(),
        applicability: Applicability::NotApplicable,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "{package} is not judged: no member of this repository declares a zone, so there is \
             no declared architecture to judge its dependency direction against. This is not a \
             gap in {package} — it is the absence of the declaration the rule compares against."
        ),
        locations: vec![source.path.clone()],
    };
}

fn Violation_For_Package(source: &SourceFile, package: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(super::DEPENDENCY_COMPLETENESS),
        subject: source.subject,
        subject_name: package.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "{package} has no declared zone. Every workspace member must declare where it \
             sits in this workspace's own architecture before its dependency direction can \
             be judged against it."
        ),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Violations_In_Should_Produce_No_Finding_For_A_Declared_Package()
    {
        let payload = DependencyPayload {
            package: "nomos-rules".to_owned(),
            edges: Vec::new(),
        };

        let findings = Violations_In(&payload, &Source_File("nomos-rules"), Declared());

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_An_Undeclared_Package_Should_Produce_One_Finding()
    {
        let payload = DependencyPayload {
            package: "not-in-bands".to_owned(),
            edges: Vec::new(),
        };

        let findings = Violations_In(&payload, &Source_File("not-in-bands"), Declared());

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "not-in-bands");
        assert_eq!(found.gate, GateCategory::Advisory);
    }

    #[test]
    fn Test_Edges_Have_No_Effect_On_The_Coverage_Judgment()
    {
        // Coverage is about the declaring package alone; direction over its edges is
        // `violations.rs`'s own subject.
        let payload = DependencyPayload {
            package: "nomos-rules".to_owned(),
            edges: vec![nomos_cap_dependency::DependencyEdge {
                target: "not-in-bands".to_owned(),
                kind: nomos_cap_dependency::DependencyKind::Normal,
                optional: false,
            }],
        };

        let findings = Violations_In(&payload, &Source_File("nomos-rules"), Declared());

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A repository that did declare an architecture -- this workspace, in every test above.
    fn Declared() -> DeclaresAnArchitecture
    {
        return DeclaresAnArchitecture(true);
    }

    /// A repository whose members are none of this workspace's is not judged, and says so
    /// positively rather than reporting a gap per crate.
    ///
    /// This is the case `OD-RULES-029` measured: with `ZONES` naming this workspace's own
    /// crates as literals, every member of any other repository had no entry, so the rule
    /// reported one advisory finding per crate about a declaration that repository was never
    /// asked for. `OD-RULES-003` decided that case gets `NotApplicable`.
    #[test]
    fn Test_A_Package_In_A_Repository_That_Declared_Nothing_Should_Be_Not_Applicable()
    {
        let payload = DependencyPayload { package: "serde_json".to_owned(), edges: Vec::new() };

        let findings = Violations_In(&payload, &Source_File("serde_json"), DeclaresAnArchitecture(false));

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.applicability, Applicability::NotApplicable);
        assert!(found.summary.contains("no member of this repository declares a zone"), "{}", found.summary);
        assert!(!found.summary.contains("  "), "the summary carries a run of spaces: {}", found.summary);
    }

    /// The distinction is not "is this crate one of ours". A member missing from a
    /// declaration this repository *did* author is still a gap, and is still reported as
    /// one -- which is the half of this rule that was already right and must stay right.
    ///
    /// Written as its own test rather than left to the test above, because the cheap wrong
    /// fix -- treating every unknown package as `NotApplicable` -- passes that one and
    /// silently retires the rule.
    #[test]
    fn Test_A_New_Member_Of_A_Declaring_Repository_Should_Still_Be_A_Gap()
    {
        let payload = DependencyPayload { package: "nomos-brand-new".to_owned(), edges: Vec::new() };

        let findings = Violations_In(&payload, &Source_File("nomos-brand-new"), Declared());

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.applicability, Applicability::Supported);
        assert!(found.summary.contains("has no declared zone"), "{}", found.summary);
    }

    fn Source_File(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }
}
