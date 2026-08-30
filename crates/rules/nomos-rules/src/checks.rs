//! The nine rules this crate implements, one module each — `dependency` holds two,
//! [`Check_Dependency_Direction`] and [`Check_Every_Member_Declares_A_Band`], since both
//! judge the same declared architecture and observed `nomos.cap.dependency.edges` fact.
//! `lib.rs`'s own module doc walks why each one exists and in what order it was built;
//! this file only gathers them so the crate root is not itself the ninth thing that
//! grows one module per rule forever.
//!
//! [`Relay_Findings`] and [`test_support`] are the two pieces of shared plumbing more than
//! one rule needed by hand before this file existed: [`lint`] and [`policy`] both relay a
//! `ToolProvider`'s own verdict 1:1 rather than judging it a second time, and every rule's
//! test module was separately rebuilding the registry/store/fact scaffolding
//! [`test_support`] now states once.

mod crosslang;
mod dependency;
mod lint;
mod mirror;
mod naming;
mod policy;
mod reachability;
mod role_surface_pair;
#[cfg(test)]
mod test_support;

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

pub use crosslang::{Check_Cross_Language_Correspondence, CROSS_LANGUAGE_CORRESPONDENCE};
pub use dependency::{
    Check_Dependency_Direction, Check_Every_Member_Declares_A_Band, DEPENDENCY_COMPLETENESS, DEPENDENCY_CONTRACT_RECORD,
    DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION,
};
pub use lint::{Check_Lint_Diagnostics, LINT_DIAGNOSTICS};
pub use mirror::{Check_Completeness_Mirrors, COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION};
pub use naming::{Check_Naming_Convention, NAMING_CONVENTION};
pub use policy::{Check_Dependency_Policy, DEPENDENCY_POLICY};
pub use reachability::{
    Check_Unread_Reaches_A_Finding, UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
};
pub use role_surface_pair::{Check_Declared_Role_Matches_Surface, RoleSurfacePair, DECLARED_ROLE_MATCHES_SURFACE};

/// Requires and judges one payload per source, the identical "read the fact, judge the
/// payload, sort by subject then summary" shape [`lint`] and [`policy`] each rebuilt by
/// hand for their own payload type: since a `ToolProvider`'s own verdict is already a
/// judgment, both rules relay it 1:1 rather than reaching a second opinion, and the only
/// thing that differs between them is how a payload is read and how it becomes findings —
/// exactly the two functions this takes rather than reimplements.
pub(crate) fn Relay_Findings<Payload>(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
    mut payload_of: impl FnMut(&SourceFile, &mut dyn FactReader) -> Result<Payload, Finding>,
    mut findings_of: impl FnMut(&SourceFile, &Payload) -> Vec<Finding>,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match payload_of(source, &mut *facts)
        {
            Ok(payload) =>
            {
                let source_findings = findings_of(source, &payload);
                findings.extend(source_findings);
            }
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return (&left.subject_name, &left.summary).cmp(&(&right.subject_name, &right.summary)));
    return findings;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_analysis::{MemoryFactStore, Reader};
    use nomos_capability::Registry;

    fn Source(path: &str) -> SourceFile
    {
        use nomos_contracts::SubjectId;
        use nomos_model::Content_Digest;

        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), String::new());
    }

    /// An admitted registry and an empty store — `Relay_Findings` never actually calls
    /// `Require` itself (its closures do), so what this reader offers is beside the
    /// point; it only has to be a real `FactReader`, the same view every real caller has.
    fn Idle_Reader() -> (Registry, MemoryFactStore)
    {
        return (Registry::New(), MemoryFactStore::New());
    }

    #[test]
    fn Test_Relay_Findings_Should_Push_The_Payload_Error_And_Extend_The_Findings_Of_Success()
    {
        let sources = vec![Source("a.rs"), Source("b.rs")];
        let (registry, store) = Idle_Reader();
        let mut facts = Reader::On(&store, &registry, crate::checks::test_support::Test_Context());

        let findings = Relay_Findings(
            &sources,
            &mut facts,
            |source, _facts| {
                if source.path == "a.rs"
                {
                    return Err(Finding {
                        rule: nomos_contracts::RuleId::New("example"),
                        subject: source.subject,
                        subject_name: source.path.clone(),
                        applicability: nomos_contracts::Applicability::DependencyUnavailable,
                        evidence: nomos_contracts::EvidenceClass::Derived,
                        gate: nomos_contracts::GateCategory::Advisory,
                        summary: "no fact for a.rs".to_owned(),
                        locations: vec![source.path.clone()],
                    });
                }
                return Ok(2u32);
            },
            |source, payload| {
                return (0..*payload)
                    .map(|index| {
                        return Finding {
                            rule: nomos_contracts::RuleId::New("example"),
                            subject: source.subject,
                            subject_name: format!("{}#{index}", source.path),
                            applicability: nomos_contracts::Applicability::Supported,
                            evidence: nomos_contracts::EvidenceClass::Derived,
                            gate: nomos_contracts::GateCategory::Advisory,
                            summary: "relayed".to_owned(),
                            locations: vec![source.path.clone()],
                        };
                    })
                    .collect();
            },
        );

        assert_eq!(findings.len(), 3, "one pushed error plus two relayed findings: {findings:?}");
        assert!(findings.iter().any(|finding| return finding.summary == "no fact for a.rs"));
        assert_eq!(findings.iter().filter(|finding| return finding.summary == "relayed").count(), 2);
    }
}
