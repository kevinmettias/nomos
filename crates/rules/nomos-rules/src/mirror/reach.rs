//! What a universe's claimed mirror amounts to against the checks that exist.

use super::{DeclaredUniverse, BTreeSet, EnforcementReach, EnforcerRef, RuleId, COMPLETENESS_MIRROR, GateCategory, EnforcementBreach};

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
        Resolution(claimed, Claim::Of(checks.contains(claimed)));

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
/// Named rather than a bool. `Resolution(claimed, true)` said nothing at the call site
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
pub(super) fn Resolution(claimed: &str, claim: Claim) -> Resolved
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
