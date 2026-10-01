//! How a profile names which model it wants.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-003`'s five ways a profile may select a model: "A profile may select an
/// exact model identity, an allowed model set, a backend family, a capability
/// predicate, or a policy-ranked candidate set."
///
/// All five variants carry raw, unresolved strings for the same reason
/// [`crate::ModelSelection::Catalog`] does: there is no live provider, catalog, or
/// entitlement system anywhere in this workspace yet to resolve an identity, a family
/// name or a predicate against (`OD-PACKAGE-011`), so a typed resolution result would be
/// invented rather than transcribed. What this type provides is the *shape* of the
/// choice a profile author states, not a resolved answer -- resolving one, when a real
/// provider exists to resolve it against, is `OD-PACKAGE-012`'s own constraint:
/// check `nomos_capability::Registry` first rather than inventing a parallel mechanism.
///
/// `Pinned` is `MODEL-ROUTE-003`'s second sentence, kept on [`ExactIdentity`] rather
/// than as a sixth variant: "Exact identities shall be pinned where reproducibility is
/// required; aliases such as `latest` shall resolve to and record the actual provider
/// identity when exposed" describes a property an exact identity has (pinned or not),
/// not a different way of selecting a model.
///
/// [`ExactIdentity`]: ModelSelector::ExactIdentity
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelSelector
{
    /// One specific model identity, such as `"acme-large-2024-06"` or the alias
    /// `"latest"`.
    ExactIdentity
    {
        identity: String,
        /// Whether this identity must resolve to the exact same provider revision on
        /// every use, per `MODEL-ROUTE-003`'s reproducibility clause. An alias like
        /// `"latest"` is never pinned; a dated revision string typically is.
        pinned: bool,
    },
    /// Any model from a named, closed set.
    AllowedSet(Vec<String>),
    /// Any model belonging to a named backend family, such as `"acme-family"`.
    BackendFamily(String),
    /// A predicate a model must satisfy, stated in a provider-neutral vocabulary this
    /// workspace has not yet typed further than the raw expression an author wrote.
    CapabilityPredicate(String),
    /// A candidate set a policy ranks at resolution time, rather than a fixed choice
    /// stated here.
    PolicyRankedCandidates(Vec<String>),
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Variants_With_Equal_Content_Should_Be_Equal()
    {
        assert_eq!(
            ModelSelector::ExactIdentity { identity: "acme-large".to_owned(), pinned: true },
            ModelSelector::ExactIdentity { identity: "acme-large".to_owned(), pinned: true }
        );
        assert_ne!(
            ModelSelector::ExactIdentity { identity: "latest".to_owned(), pinned: false },
            ModelSelector::ExactIdentity { identity: "latest".to_owned(), pinned: true }
        );
        assert_ne!(
            ModelSelector::BackendFamily("acme-family".to_owned()),
            ModelSelector::CapabilityPredicate("acme-family".to_owned())
        );
    }
}
