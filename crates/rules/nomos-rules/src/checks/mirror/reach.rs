//! What a universe's claimed mirror amounts to against the checks that exist.

use super::{BTreeSet, EnforcementReach, EnforcerRef, RuleId, COMPLETENESS_MIRROR, GateCategory, EnforcementBreach};
use crate::DeclaredUniverse;

/// The enforcement claim a universe makes, and what the source says of it.
///
/// A universe that names no mirror declares [`EnforcerRef::Review`] and expects
/// [`GateCategory::Review`], which is *truthful* — an admitted gap is an honest
/// declaration, and `enforcement.rs` is explicit that it must not be conflated with an
/// overclaim. It is still not enforcement, which is why the finding is raised on
/// [`EnforcementReach::Is_Enforced`] and its severity read off the breaches.
pub(super) fn Reach_Of(universe: &DeclaredUniverse, checks: &BTreeSet<String>) -> EnforcementReach
{
    let Some(claimed) = universe.claimed_mirror.as_ref()
    else
    {
        return Reviewed();
    };

    let enforcer = EnforcerRef::Check {
        name: claimed.clone(),
    };
    let Resolved { computed, breaches } =
        Resolution_Of_Claim(claimed, Claim::Of(checks.contains(claimed)));

    return EnforcementReach {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        declared: vec![enforcer],
        // Naming a check is a claim that a violation would be caught. That is what makes a
        // name resolving to nothing a false claim rather than a typo.
        expected: GateCategory::Blocking,
        computed,
        breaches,
    };
}

/// The reach of a universe that names no mirror.
///
/// Declared and expected agree, so there is no breach: an admitted gap is a truthful
/// declaration that this universe is reviewed rather than checked.
pub(super) fn Reviewed() -> EnforcementReach
{
    return EnforcementReach {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        declared: vec![EnforcerRef::Review],
        expected: GateCategory::Review,
        computed: GateCategory::Review,
        breaches: Vec::new(),
    };
}

/// What a claimed name amounts to, and what it breached in amounting to that.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
pub(super) struct Resolved
{
    pub(super) computed: GateCategory,
    pub(super) breaches: Vec<EnforcementBreach>,
}

/// Whether the index holds the name a universe claimed.
///
/// Named rather than a bool. `Resolution_Of_Claim(claimed, true)` said nothing at the call site
/// about what `true` was true of, and the two outcomes are a build that fails and one
/// that does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Claim
{
    /// The index holds the claimed name.
    Resolves,
    /// It does not, so the claim of coverage is a phantom.
    Phantom,
}

impl Claim
{
    /// Whether the check index holds the claimed name.
    pub(super) const fn Of(resolves: bool) -> Self
    {
        if resolves
        {
            return Self::Resolves;
        }

        return Self::Phantom;
    }
}

/// What a claimed name amounts to, given whether the index holds it.
pub(super) fn Resolution_Of_Claim(claimed: &str, claim: Claim) -> Resolved
{
    if claim == Claim::Resolves
    {
        return Resolved {
            computed: GateCategory::Blocking,
            breaches: Vec::new(),
        };
    }

    return Resolved {
        computed: GateCategory::Unreachable,
        breaches: vec![EnforcementBreach::Phantom {
            name: claimed.to_owned(),
        }],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::DeclaredUniverse;

    #[test]
    fn Test_Reach_Of_Should_Report_Reviewed_When_No_Mirror_Is_Claimed()
    {
        let universe = Universe_Claiming(None);
        let checks = BTreeSet::new();

        let reach = Reach_Of(&universe, &checks);

        assert_eq!(reach.declared, vec![EnforcerRef::Review]);
        assert_eq!(reach.expected, GateCategory::Review);
        assert!(reach.breaches.is_empty());
    }

    #[test]
    fn Test_Reach_Of_Should_Name_A_Check_Enforcer_When_A_Mirror_Is_Claimed()
    {
        let universe = Universe_Claiming(Some("Test_Every_Row"));
        let mut checks = BTreeSet::new();
        checks.insert("Test_Every_Row".to_owned());

        let reach = Reach_Of(&universe, &checks);

        assert_eq!(reach.declared, vec![EnforcerRef::Check { name: "Test_Every_Row".to_owned() }]);
        assert_eq!(reach.expected, GateCategory::Blocking);
        assert!(reach.breaches.is_empty());
    }

    #[test]
    fn Test_Reviewed_Should_Declare_A_Review_Enforcer_With_No_Breach()
    {
        let reach = Reviewed();

        assert_eq!(reach.declared, vec![EnforcerRef::Review]);
        assert_eq!(reach.computed, GateCategory::Review);
        assert!(reach.breaches.is_empty());
    }

    #[test]
    fn Test_Resolution_Of_Claim_Should_Report_A_Phantom_Breach_When_The_Claim_Does_Not_Resolve()
    {
        let resolved = Resolution_Of_Claim("Test_Nowhere", Claim::Of(false));

        assert_eq!(resolved.computed, GateCategory::Unreachable);
        assert_eq!(resolved.breaches, vec![EnforcementBreach::Phantom { name: "Test_Nowhere".to_owned() }]);
    }

    #[test]
    fn Test_Resolution_Of_Claim_Should_Report_No_Breach_When_The_Claim_Resolves()
    {
        let resolved = Resolution_Of_Claim("Test_Somewhere", Claim::Of(true));

        assert_eq!(resolved.computed, GateCategory::Blocking);
        assert!(resolved.breaches.is_empty());
    }

    #[test]
    fn Test_Of_Should_Read_A_True_Bool_As_Resolves_And_A_False_One_As_Phantom()
    {
        assert_eq!(Claim::Of(true), Claim::Resolves);
        assert_eq!(Claim::Of(false), Claim::Phantom);
    }

    fn Universe_Claiming(mirror: Option<&str>) -> DeclaredUniverse
    {
        return DeclaredUniverse {
            path: "a.rs".to_owned(),
            name: "TABLES".to_owned(),
            kind: crate::UniverseKind::Constant,
            claimed_mirror: mirror.map(str::to_owned),
        };
    }
}
