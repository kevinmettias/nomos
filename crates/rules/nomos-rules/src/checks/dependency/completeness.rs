//! Judging whether an already-decoded dependency payload's own package is placed by the
//! repository's declared architecture.
//!
//! `violations.rs`'s `Violations_In` silently produces no findings for a package the
//! declaration does not place, naming the gap as a different defect — this module's. A pure
//! function of an already-decoded payload, grouped apart from `reading.rs` for the same reason
//! [`super::violations`] is: testable against hand-built fixtures, no registry, no store, no
//! reader.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_architecture::ArchitecturePayload;
use nomos_cap_dependency::DependencyPayload;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// Judges whether every workspace member `sources` names is placed by the architecture its own
/// repository declares — the coverage half of architecture conformance.
///
/// Reads the identical two facts [`super::Check_Dependency_Direction`] does, through the same
/// readers, and files an unread subject under its own identifier rather than direction's.
///
/// Spelled here rather than in [`super`] because this is the coverage half of the judgment and
/// [`Violations_In`] beside it is the whole of what it reaches.
#[must_use]
pub fn Check_Every_Member_Declares_A_Band(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let architecture = match super::reading::Architecture_Of(sources, facts, super::DEPENDENCY_COMPLETENESS)
    {
        Ok(architecture) => architecture,
        Err(unread) => return unread,
    };

    return super::Judged_Members(sources, facts, super::DEPENDENCY_COMPLETENESS, &|payload, source| {
        return Violations_In(&architecture, payload, source);
    });
}

/// A finding when `payload`'s own package is not placed by `architecture` — empty otherwise.
///
/// # Why the answer depends on the declaration and not on the rest of the member set
///
/// A package with no component is one of two quite different things. Either this repository
/// declared an architecture and this member was left out of it — a real gap, and what this
/// rule exists to catch — or this repository declared none at all, in which case the rule does
/// not bind it.
///
/// `OD-RULES-003` decided what the second case gets, choosing [`Applicability::NotApplicable`]
/// over `MissingCapability` and `ConfigurationDisabled` with a reason given for each: the
/// capability establishing the dependency graph can be entirely present and correct while no
/// architecture has been declared, so the gap is in the policy input, not the fact input; and a
/// repository that never declared one has not switched anything off.
///
/// Before the declaration left this crate, telling the two apart was impossible and then merely
/// indirect. `OD-RULES-029` measured the first state: `ZONES` named this workspace's own crates
/// as string literals, so every member of any other repository looked like a gap. The fix that
/// followed inferred "did this repository declare anything" from whether *any* member resolved,
/// which was right in every case anyone had and was still an inference. The declaration answers
/// it outright now — [`ArchitecturePayload::Has_An_Architecture`] — so a repository that
/// writes the file, names its components and has placed nothing yet is correctly read as
/// declaring an architecture, which the member-set inference would have called silence.
#[must_use]
pub(super) fn Violations_In(architecture: &ArchitecturePayload, payload: &DependencyPayload, source: &SourceFile) -> Vec<Finding>
{
    if architecture.Component_Of(&payload.package).is_some()
    {
        return Vec::new();
    }

    if !architecture.Has_An_Architecture()
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
            "{package} is not judged: this repository declares no architecture, so there is no \
             set of components to place it in and nothing to judge its dependency direction \
             against. This is not a gap in {package} — it is the absence of the declaration the \
             rule compares against."
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
            "{package} is placed in none of the components this repository declares. Every \
             workspace member must be placed before its dependency direction can be judged \
             against the architecture."
        ),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_architecture::Membership;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Violations_In_Should_Produce_No_Finding_For_A_Placed_Package()
    {
        let payload = DependencyPayload { package: "billing".to_owned(), edges: Vec::new() };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_An_Unplaced_Package_Should_Produce_One_Finding()
    {
        let payload = DependencyPayload { package: "unplaced".to_owned(), edges: Vec::new() };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("unplaced"));

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "unplaced");
        assert_eq!(found.gate, GateCategory::Advisory);
    }

    #[test]
    fn Test_Edges_Have_No_Effect_On_The_Coverage_Judgment()
    {
        // Coverage is about the declaring package alone; direction over its edges is
        // `violations.rs`'s own subject.
        let payload = DependencyPayload {
            package: "billing".to_owned(),
            edges: vec![nomos_cap_dependency::DependencyEdge {
                target: "unplaced".to_owned(),
                kind: nomos_cap_dependency::DependencyKind::Normal,
                optional: false,
            }],
        };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("billing"));

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// A repository whose members are none of this one's is not judged, and says so positively
    /// rather than reporting a gap per crate.
    ///
    /// This is the case `OD-RULES-029` measured: with a compiled table naming this workspace's
    /// own crates, every member of any other repository had no entry, so the rule reported one
    /// advisory finding per crate about a declaration that repository was never asked for.
    #[test]
    fn Test_A_Package_In_A_Repository_That_Declared_Nothing_Should_Be_Not_Applicable()
    {
        let payload = DependencyPayload { package: "serde_json".to_owned(), edges: Vec::new() };

        let findings = Violations_In(&ArchitecturePayload::default(), &payload, &Source_File("serde_json"));

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.applicability, Applicability::NotApplicable);
        assert!(found.summary.contains("declares no architecture"), "{}", found.summary);
        assert!(!found.summary.contains("  "), "the summary carries a run of spaces: {}", found.summary);
    }

    /// The distinction is not "is this crate one of ours". A member missing from a declaration
    /// this repository *did* author is still a gap, and is still reported as one -- which is the
    /// half of this rule that was already right and must stay right.
    ///
    /// Written as its own test rather than left to the test above, because the cheap wrong fix
    /// -- treating every unplaced package as `NotApplicable` -- passes that one and silently
    /// retires the rule.
    #[test]
    fn Test_A_New_Member_Of_A_Declaring_Repository_Should_Still_Be_A_Gap()
    {
        let payload = DependencyPayload { package: "brand-new".to_owned(), edges: Vec::new() };

        let findings = Violations_In(&Declaration(), &payload, &Source_File("brand-new"));

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.applicability, Applicability::Supported);
        assert!(found.summary.contains("placed in none of the components"), "{}", found.summary);
    }

    /// The case the member-set inference this replaces got wrong: a repository that has named
    /// its components and placed nothing has declared an architecture, and its members are gaps
    /// rather than out of scope.
    #[test]
    fn Test_A_Declaration_With_Components_And_No_Members_Should_Still_Report_A_Gap()
    {
        let declared_but_empty = ArchitecturePayload { components: vec!["Domain".to_owned()], ..ArchitecturePayload::default() };
        let payload = DependencyPayload { package: "billing".to_owned(), edges: Vec::new() };

        let findings = Violations_In(&declared_but_empty, &payload, &Source_File("billing"));

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.applicability, Applicability::Supported, "components declared is an architecture declared");
    }

    fn Declaration() -> ArchitecturePayload
    {
        return ArchitecturePayload {
            components: vec!["Domain".to_owned(), "Api".to_owned()],
            membership: vec![
                Membership { package: "billing".to_owned(), component: "Domain".to_owned() },
                Membership { package: "http".to_owned(), component: "Api".to_owned() },
            ],
            ..ArchitecturePayload::default()
        };
    }

    fn Source_File(package: &str) -> SourceFile
    {
        return SourceFile::New(package, SubjectId::From_Digest(Content_Digest(package.as_bytes())), String::new());
    }
}
