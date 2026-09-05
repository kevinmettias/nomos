//! What this provider offers, and at what guarantee.

use crate::contract::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This provider's own name.
///
/// Named for the mechanism it actually runs, the same way `nomos.lang.rust.deny` and
/// `nomos.lang.rust.clippy` are: a future provider answering the same capability through
/// a different compiler frontend (a real `rustc` embedding, a different version of
/// `ra_ap_hir`) would need to disagree with this name honestly.
pub const PROVIDER: &str = "nomos.lang.rust.compiler";

/// What this provider claims, on every axis.
///
/// [`FactVariant::SemanticallyResolved`] at the ceiling: every finding this provider
/// reports comes from `ra_ap_hir::Semantics::type_of_expr` and `Type::is_copy`, both of
/// which resolve names and look up trait implementations rather than read syntax.
///
/// Soundness [`Assurance::Sound`]: a finding this provider reports names a receiver
/// `ra_ap_hir` actually resolved and actually found `Copy` for; nothing here infers one.
///
/// Completeness [`Assurance::Unknown`], deliberately not `Sound` -- this provider walks
/// only `ast::MethodCallExpr` nodes literally spelled `.clone()`; a call reached through
/// a macro expansion, a fully-qualified `Clone::clone(&value)` form, or a trait object
/// this analysis pass could not resolve is not a case this provider can honestly claim
/// it would have caught, the same "no floor no provider can honestly meet" reasoning
/// `nomos_lang_rust_deny::guarantee::Declared_Guarantee` already gives for a different
/// underlying reason.
///
/// [`IncrementalGranularity::Project`]: [`crate::contract::Ceiling`] states why one
/// analyzed crate is the unit that must recompute together.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );
}

/// This provider's offer against [`crate::contract::Capability_Contract`].
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(PROVIDER),
        capability: Capability(),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The offer must satisfy the contract's own ceiling -- `contract.ceiling.Satisfies(&
    /// offer.guarantee)` is the exact check `Registry::Offer` runs at composition time,
    /// asserted here in the same direction so a future weakening of either constant is
    /// caught beside the constants rather than only at whatever composition root happens
    /// to run first.
    #[test]
    fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
    {
        use crate::contract::Ceiling;

        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        use crate::contract::Capability_Contract;
        use nomos_capability::Registry;

        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }
}
