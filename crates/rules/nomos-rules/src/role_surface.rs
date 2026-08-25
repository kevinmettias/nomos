//! Whether a crate's declared role and its actual public surface agree — a judgment no
//! mechanical provider can make.
//!
//! Grounded against two real, external architecture-standards trees the workspace does not
//! own, read directly rather than assumed: `code-standards`' architecture placement
//! standards name "Module Roles and Responsibility Boundaries" as a real rule — "every
//! module has one role... the authoritative, per-module role statement" — and `xvpe`'s
//! engineering doctrine states the general shape directly: "some `MUST` rules are
//! review-enforced because they govern semantic truthfulness... in ways automation cannot
//! fully prove." This rule is that shape, for this workspace's own convention: every crate's
//! `README.md` band-table row already reads as exactly the "responsible for / explicitly not
//! responsible for" statement those standards ask for (see `nomos-agent-executor`'s own row:
//! "The first real `AgentExecutor`... Does not assemble a `WorkResult`"), and every crate's
//! committed public-surface snapshot (`tests/contract/surface/<crate>.txt`) already states
//! exactly what it actually exports. Whether the two agree is a semantic question over prose
//! and a list of signatures — dependency direction is mechanical and already checked
//! ([`crate::Check_Dependency_Direction`]); whether a crate's declared purpose matches what
//! it exports is not, the same distinction `code-standards`' own `form-vs-infrastructure`
//! rule draws for a single-file dependency edge: "a misplaced file cannot be detected in
//! general — only a human [or, here, a model] knows what a body of code means."
//!
//! # Why this rule takes plain data, not a [`nomos_analysis::FactReader`]
//!
//! Every other rule in this crate reads its subject through a capability, because a capability
//! names a contract several providers could compete to answer — `nomos.cap.syntax.items` has
//! two, `nomos.cap.dependency.edges` and `nomos.cap.controlflow.reachability` each have their
//! own real provider distinct from a hand-written alternative. A crate's `README.md` row and
//! its own committed surface snapshot have no such competing-provider question: there is one
//! way to read a file's own committed text, not a guarantee to negotiate. Routing them through
//! a capability contract would invent a competition that does not exist, the same
//! "declaration nothing enforces" shape `OD-CAPABILITY-*` records elsewhere in this workspace
//! decline to build ahead of a real second provider. [`RoleSurfacePair`] is therefore plain
//! data a composition root reads directly and hands in, the same way `nomos-check-
//! orchestration::Run` already hands this crate its `sources` rather than having a rule walk
//! a directory.
//!
//! # This rule always reports `AgentRequired`, never a verdict
//!
//! `OD-CONTRACTS-002` is exact about what a rule producing this state does and does not do:
//! "this decides only that a run can *say* a subject needs one" — no executor runs here, and
//! no judgment is reached. Every subject this rule is handed is unconditionally
//! [`nomos_contracts::Applicability::AgentRequired`], because the question this rule asks —
//! do a crate's declared role and its actual surface agree — is never answerable by this rule
//! itself; only by a model. `nomos-agent-executor` is the real, separate dispatch a caller
//! chooses to make against a finding this rule produces, not something this rule invokes.

use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_model::Subject_Of_Path;

/// This rule's own identifier.
pub const DECLARED_ROLE_MATCHES_SURFACE: &str = "declared-role-matches-surface";

/// One crate's declared role — its `README.md` band-table row, prose a person wrote about
/// what the crate is responsible for — paired with its actual public surface: the committed
/// snapshot naming exactly what the crate exports today. Whether the two agree is what a
/// model is asked to judge; this type carries what it would need to judge it, nothing more.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleSurfacePair
{
    /// The crate's manifest-relative root, e.g. `crates/agent/nomos-agent-executor` —
    /// addressed the same way `OD-MODEL-002` already derives a subject from a
    /// repository-relative path, so two crates never collide and a crate moved is a
    /// different subject rather than a silently reused one.
    pub crate_root: String,
    /// The crate's own name, for reporting.
    pub crate_name: String,
    /// `README.md`'s band-table prose for this crate.
    pub declared_role: String,
    /// The crate's committed public-surface snapshot, verbatim.
    pub actual_surface: String,
}

/// Reports every `subjects` pair as needing a model's judgment — never a verdict, and never
/// anything else. See the module doc for why `AgentRequired` is the only state this rule can
/// reach.
#[must_use]
pub fn Check_Declared_Role_Matches_Surface(subjects: &[RoleSurfacePair]) -> Vec<Finding>
{
    let mut findings: Vec<Finding> = subjects.iter().map(Agent_Required_Finding).collect();

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));

    return findings;
}

fn Agent_Required_Finding(subject: &RoleSurfacePair) -> Finding
{
    return Finding {
        rule: RuleId::New(DECLARED_ROLE_MATCHES_SURFACE),
        subject: Subject_Of_Path(&subject.crate_root),
        subject_name: subject.crate_name.clone(),
        applicability: Applicability::AgentRequired,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "whether {}'s declared role and its actual public surface agree needs a model's judgment",
            subject.crate_name
        ),
        locations: vec!["README.md".to_owned(), format!("tests/contract/surface/{}.txt", subject.crate_name)],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Pair(crate_name: &str, declared_role: &str, actual_surface: &str) -> RoleSurfacePair
    {
        return RoleSurfacePair {
            crate_root: format!("crates/example/{crate_name}"),
            crate_name: crate_name.to_owned(),
            declared_role: declared_role.to_owned(),
            actual_surface: actual_surface.to_owned(),
        };
    }

    #[test]
    fn Test_Every_Subject_Is_Reported_As_Agent_Required()
    {
        let subjects = vec![
            Pair("nomos-example-a", "Reads files.", "pub fn Read() -> String"),
            Pair("nomos-example-b", "Writes files.", "pub fn Write(text: &str)"),
        ];

        let findings = Check_Declared_Role_Matches_Surface(&subjects);

        assert_eq!(findings.len(), 2, "{findings:?}");
        for finding in &findings
        {
            assert_eq!(finding.applicability, Applicability::AgentRequired);
            assert_eq!(finding.evidence, EvidenceClass::Derived);
            assert_eq!(finding.gate, GateCategory::Advisory);
        }
    }

    /// The rule never reads `declared_role` or `actual_surface` itself — it cannot judge
    /// them, only report that judgment is needed — so a pair whose prose obviously agrees
    /// and one whose prose obviously conflicts must produce the identical finding shape.
    /// This is the test that would fail if the rule ever started guessing at a verdict.
    #[test]
    fn Test_The_Rule_Never_Reaches_A_Verdict_Regardless_Of_Content()
    {
        let agrees = Pair("nomos-consistent", "Reads files.", "pub fn Read() -> String");
        let conflicts = Pair("nomos-inconsistent", "Reads files.", "pub fn Delete_Everything()");

        let findings = Check_Declared_Role_Matches_Surface(&[agrees, conflicts]);
        let (first, second) = (findings.first().expect("two findings"), findings.get(1).expect("two findings"));

        assert_eq!(first.applicability, second.applicability);
        assert_eq!(first.evidence, second.evidence);
    }

    #[test]
    fn Test_Findings_Are_Sorted_By_Subject_Name()
    {
        let subjects = vec![Pair("nomos-z", "z", "z"), Pair("nomos-a", "a", "a")];

        let findings = Check_Declared_Role_Matches_Surface(&subjects);

        assert_eq!(findings.first().expect("two findings").subject_name, "nomos-a");
        assert_eq!(findings.get(1).expect("two findings").subject_name, "nomos-z");
    }

    #[test]
    fn Test_An_Empty_Subject_List_Produces_No_Findings()
    {
        assert!(Check_Declared_Role_Matches_Surface(&[]).is_empty());
    }

    #[test]
    fn Test_The_Locations_Name_The_Readme_And_The_Surface_Snapshot()
    {
        let subjects = vec![Pair("nomos-example-a", "Reads files.", "pub fn Read() -> String")];

        let findings = Check_Declared_Role_Matches_Surface(&subjects);

        assert_eq!(
            findings.first().expect("one finding").locations,
            vec!["README.md".to_owned(), "tests/contract/surface/nomos-example-a.txt".to_owned()]
        );
    }
}
