//! Deciding what one declared universe is owed, and saying it in a finding.

use super::{CheckIndex, Finding, Reach_Of, Applicability, GateCategory, RuleId, COMPLETENESS_MIRROR, SubjectId, Content_Digest, EvidenceClass, EnforcementReach, EnforcementBreach, UniverseKind};
use crate::DeclaredUniverse;

/// What one universe's declaration amounts to, and what is really true of it.
///
/// Returns `None` when the universe is mirrored, because a rule that emits a finding per
/// subject it approves of produces a report in which the defects cannot be found.
pub(super) fn Judgment_For_Universe(universe: &DeclaredUniverse, index: &CheckIndex<'_>) -> Option<Finding>
{
    let reach = Reach_Of(universe, &index.names);

    if reach.Is_Enforced()
    {
        return None;
    }

    let verdict = Verdict_For_Reach(universe, &reach, index);

    return Some(Shortcoming_Finding(universe, verdict));
}

/// A universe's shortfall as a finding.
///
/// An admitted gap does not depend on the index at all — nothing was resolved, so nothing
/// could have been missed — and stays `Supported` however short the index is. Only a claim
/// that failed to resolve inherits the doubt. The evidence is `Derived` either way: computed
/// from source by a deterministic rule, and no stronger than that source.
///
/// The subject is hashed from the *qualified* name (`D-134`), not `universe.name` alone.
/// Two universes named alike in two different crates are two different subjects; without
/// the qualifier they would hash to the same one and be indistinguishable in every finding,
/// suppression, or history keyed on it.
pub(super) fn Shortcoming_Finding(universe: &DeclaredUniverse, verdict: Judgment) -> Finding
{
    let Judgment {
        applicability,
        gate,
        summary,
    } = verdict;

    let qualified = match Qualifier_Of(&universe.path)
    {
        Some(qualifier) => format!("{qualifier}::{}", universe.name),
        None => universe.name.clone(),
    };

    return Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: universe.name.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate,
        summary,
        locations: vec![universe.path.clone()],
    };
}

/// The crate or test-suite directory a universe's path belongs to, if the path names one.
///
/// A pure function of the path string — no filesystem access and no `Cargo.toml` read.
/// `universe_kind.rs`'s module doc explains why this crate carries no parser and reads no
/// files; the same discipline applies here. This workspace's real paths are
/// `crates/<band>/<crate-name>/src/...` or `<suite-root>/src/...` /
/// `<suite-root>/tests/...` (`tests/contract/tests/completeness_universes/table.rs`'s
/// `UNIVERSES` has no exception among its nineteen entries), so the qualifier is the path
/// segment immediately before the last `/src/` or `/tests/` marker — the crate or suite
/// directory the file lives in, which stays the same as the file moves within it and
/// differs across crates that happen to share a bare universe name.
///
/// `None` when neither marker is present. Every real universe path in this workspace has
/// one; a path without either is a synthetic fixture path from a unit test (`"a.rs"`,
/// `"b/c.rs"`), not a shape this rule is ever really handed. Guessing a qualifier there —
/// say, the whole path — would turn "the same universe moved file" into two identities,
/// which is exactly what this module's own same-universe-keeps-one-identity test pins
/// against. `None` leaves the name unqualified instead, which is the old behaviour and
/// the right one for a path that names no crate at all.
fn Qualifier_Of(path: &str) -> Option<&str>
{
    let split_at = ["/src/", "/tests/"]
        .iter()
        .filter_map(|marker| return path.rfind(marker))
        .max()?;

    let before = &path[..split_at];
    let leaf = before.rsplit('/').next()?;

    if leaf.is_empty()
    {
        return None;
    }

    return Some(leaf);
}

/// How the universe's shortfall is reported.
///
/// The claimed name is read back off the universe rather than off the breach, because the
/// breach's payload is `nomos-contracts`' shape and this is the rule's own claim. The two
/// cannot disagree: [`Reach_Of`] produces a breach only in the arm where `claimed_mirror`
/// is `Some` and none in the arm where it is `None`, so the `unwrap_or_default` below is
/// unreachable rather than a fallback with a meaning. An empty name matches every text,
/// which would downgrade rather than block — the safe direction, for the reason
/// [`Unread::Can_Have_Declared`] gives.
pub(super) fn Verdict_For_Reach(
    universe: &DeclaredUniverse,
    reach: &EnforcementReach,
    index: &CheckIndex<'_>,
) -> Judgment
{
    let Some(breach) = reach.breaches.first()
    else
    {
        return Admitted_Gap(universe);
    };

    let claimed = universe.claimed_mirror.as_deref().unwrap_or_default();

    return Unresolved_Claim(breach, claimed, index);
}

/// How to report a claimed mirror that did not resolve against the index.
///
/// The two gates in play are not the same gate, and collapsing them is the mistake this
/// whole module is about. `reach.computed` is what the *universe's* declared mirror amounts
/// to — `Unreachable` for a phantom. The gate returned here is what *this rule* does about
/// that, and a false claim of coverage is the one outcome worth failing a build over.
///
/// Unless the index is short of something that could have resolved *this* name. Then the
/// claim is not established as false — the name may be in the subject that was not read —
/// and reporting it as a phantom would be the rule manufacturing the one finding it is
/// entitled to stop a build over out of its own inability to look, which is the same defect
/// as reporting clean wearing the other face. `Can_Fail_A_Build` consults the applicability
/// as well as the gate, so the refusal is machinery `D-134` already built rather than a
/// second rule about severity.
///
/// `OD-RULES-001` asked that question of the whole run and this asks it of the claim, which
/// is the whole of `OD-RULES-002`. When the index is short of subjects that *could not*
/// have resolved this name, the judgment is made and the shortfall travels with it in the
/// summary rather than suppressing it: a reader who wants to know what the run did not see
/// is owed that on the finding, not instead of it.
pub(super) fn Unresolved_Claim(
    breach: &EnforcementBreach,
    claimed: &str,
    index: &CheckIndex<'_>,
) -> Judgment
{
    let Some(shortfall) = index.Shortfall_For(claimed)
    else
    {
        return Judgment {
            applicability: Applicability::Supported,
            gate: GateCategory::Blocking,
            summary: if index.unread.is_empty()
            {
                breach.Describe()
            }
            else
            {
                format!(
                    "{} — and the check index is short {} subject(s), none of whose text \
                     spells `{claimed}`, so no reading of them could have declared it",
                    breach.Describe(),
                    index.unread.len()
                )
            },
        };
    };

    return Judgment {
        applicability: shortfall.Applicability(),
        gate: GateCategory::Advisory,
        summary: format!("{} — and {}", breach.Describe(), shortfall.Describe()),
    };
}

/// How to report a universe that claims no mirror at all.
///
/// An admitted gap does not depend on the index in any way — nothing was resolved, so
/// nothing could have been missed — which is why no shortfall is consulted here.
pub(super) fn Admitted_Gap(universe: &DeclaredUniverse) -> Judgment
{
    return Judgment {
        applicability: Applicability::Supported,
        gate: GateCategory::Advisory,
        summary: format!(
            "declares no mirror, so nothing compares this list against the reality it \
             enumerates; a {} added without adding it here is outside every guard built \
             on it, and those guards then pass by not looking",
            match universe.kind
            {
                UniverseKind::Constant => "member",
                UniverseKind::Enumeration => "variant",
            }
        ),
    };
}

/// What one universe is owed: how far the rule can stand behind it, what the rule does
/// about it, and what it says.
///
/// Named rather than a triple. At three members a caller is counting positions, and these
/// three travel together through four functions — [`Verdict_For_Reach`], [`Unresolved_Claim`],
/// [`Admitted_Gap`] and [`Shortcoming_Finding`] — which is four places for a position to slip.
pub(super) struct Judgment
{
    pub(super) applicability: Applicability,
    pub(super) gate: GateCategory,
    pub(super) summary: String,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Judgment_For_Universe_Should_Produce_No_Finding_When_The_Reach_Is_Enforced()
    {
        let universe = Declared_Universe(Some("Test_Every_Row"));
        let mut names = std::collections::BTreeSet::new();
        names.insert("Test_Every_Row".to_owned());
        let index = Empty_Index_With_Names(names);

        let judgment = Judgment_For_Universe(&universe, &index);

        assert!(judgment.is_none(), "{judgment:?}");
    }

    #[test]
    fn Test_Judgment_For_Universe_Should_Produce_A_Finding_When_The_Reach_Is_Not_Enforced()
    {
        let universe = Declared_Universe(None);
        let index = Empty_Index_With_Names(std::collections::BTreeSet::new());

        let judgment = Judgment_For_Universe(&universe, &index).expect("an admitted gap is still reported");

        assert_eq!(judgment.subject_name, "TABLES");
        assert_eq!(judgment.gate, GateCategory::Advisory);
    }

    #[test]
    fn Test_Shortcoming_Finding_Should_Carry_The_Judgments_Applicability_Gate_And_Summary()
    {
        let universe = Declared_Universe(None);
        let judgment = Admitted_Gap(&universe);
        let expected_summary = judgment.summary.clone();

        let finding = Shortcoming_Finding(&universe, judgment);

        assert_eq!(finding.applicability, Applicability::Supported);
        assert_eq!(finding.gate, GateCategory::Advisory);
        assert_eq!(finding.summary, expected_summary);
        assert_eq!(finding.subject_name, "TABLES");
    }

    #[test]
    fn Test_Verdict_For_Reach_Should_Admit_A_Gap_When_The_Reach_Has_No_Breach()
    {
        let universe = Declared_Universe(None);
        let reach = EnforcementReach {
            rule: RuleId::New(COMPLETENESS_MIRROR),
            declared: vec![nomos_contracts::EnforcerRef::Review],
            expected: GateCategory::Review,
            computed: GateCategory::Review,
            breaches: Vec::new(),
        };
        let index = Empty_Index_With_Names(std::collections::BTreeSet::new());

        let judgment = Verdict_For_Reach(&universe, &reach, &index);

        assert_eq!(judgment.gate, GateCategory::Advisory);
        assert!(judgment.summary.contains("declares no mirror"), "{}", judgment.summary);
    }

    #[test]
    fn Test_Verdict_For_Reach_Should_Resolve_The_Claim_When_The_Reach_Has_A_Breach()
    {
        let universe = Declared_Universe(Some("Test_Nowhere"));
        let reach = EnforcementReach {
            rule: RuleId::New(COMPLETENESS_MIRROR),
            declared: vec![nomos_contracts::EnforcerRef::Check { name: "Test_Nowhere".to_owned() }],
            expected: GateCategory::Blocking,
            computed: GateCategory::Unreachable,
            breaches: vec![EnforcementBreach::Phantom { name: "Test_Nowhere".to_owned() }],
        };
        let index = Empty_Index_With_Names(std::collections::BTreeSet::new());

        let judgment = Verdict_For_Reach(&universe, &reach, &index);

        assert_eq!(judgment.gate, GateCategory::Blocking);
        assert_eq!(judgment.applicability, Applicability::Supported);
    }

    #[test]
    fn Test_Unresolved_Claim_Should_Block_When_Nothing_Was_Left_Unread()
    {
        let breach = EnforcementBreach::Phantom { name: "Test_Nowhere".to_owned() };
        let index = Empty_Index_With_Names(std::collections::BTreeSet::new());

        let judgment = Unresolved_Claim(&breach, "Test_Nowhere", &index);

        assert_eq!(judgment.gate, GateCategory::Blocking);
        assert_eq!(judgment.applicability, Applicability::Supported);
    }

    #[test]
    fn Test_Unresolved_Claim_Should_Downgrade_When_The_Index_Is_Short_Of_A_Subject_That_Could_Have_Resolved_It()
    {
        // Test-only: production code here never constructs an `Unread` directly, only ever
        // an index that already carries one.
        use super::super::Unread;

        let breach = EnforcementBreach::Phantom { name: "Test_Renamed_Away".to_owned() };
        let index = CheckIndex {
            names: std::collections::BTreeSet::new(),
            universes: Vec::new(),
            unobserved: Vec::new(),
            unread: vec![Unread {
                path: "b.rs".to_owned(),
                text: "fn Test_Renamed_Away() {}",
                inputs: SubjectId::From_Digest(Content_Digest(b"b.rs")),
                applicability: Applicability::DependencyUnavailable,
                because: "no admitted provider answered for it".to_owned(),
            }],
        };

        let judgment = Unresolved_Claim(&breach, "Test_Renamed_Away", &index);

        assert_eq!(judgment.gate, GateCategory::Advisory);
        assert_eq!(judgment.applicability, Applicability::DependencyUnavailable);
    }

    #[test]
    fn Test_Admitted_Gap_Should_Name_The_Kind_Of_Member_The_List_Enumerates()
    {
        let universe = DeclaredUniverse {
            path: "a.rs".to_owned(),
            name: "Table::All".to_owned(),
            kind: UniverseKind::Enumeration,
            claimed_mirror: None,
        };

        let judgment = Admitted_Gap(&universe);

        assert_eq!(judgment.gate, GateCategory::Advisory);
        assert_eq!(judgment.applicability, Applicability::Supported);
        assert!(judgment.summary.contains("variant"), "{}", judgment.summary);
    }

    fn Empty_Index_With_Names(names: std::collections::BTreeSet<String>) -> CheckIndex<'static>
    {
        return CheckIndex { names, universes: Vec::new(), unobserved: Vec::new(), unread: Vec::new() };
    }

    fn Declared_Universe(claimed_mirror: Option<&str>) -> DeclaredUniverse
    {
        return DeclaredUniverse {
            path: "a.rs".to_owned(),
            name: "TABLES".to_owned(),
            kind: UniverseKind::Constant,
            claimed_mirror: claimed_mirror.map(str::to_owned),
        };
    }
}
