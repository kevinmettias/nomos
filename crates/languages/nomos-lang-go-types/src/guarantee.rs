//! What this provider offers, and at what guarantee.

use nomos_cap_go_types::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This provider's own name, for what answers it: `go/types`, run by the host's Go toolchain.
pub const PROVIDER: &str = "nomos.lang.go.types";

/// What this provider claims, on every axis.
///
/// Exercised by `Test_Every_Blank_Assignment_Should_Be_Typed_By_The_Real_Toolchain` in
/// `tests/guarantee.rs`, against the real toolchain: each form a value reaches the blank identifier
/// by, typed across a package boundary and through a type parameter, one fact per file.
///
/// [`FactVariant::SemanticallyResolved`]: every type is the one `go/types` resolved for the
/// package, checked against its dependencies' export data -- which is how `f`'s result is known to
/// be an `error` when `f` is declared in another package.
///
/// Soundness [`Assurance::Sound`]: every value in a fact is one the helper found assigned to `_` in
/// a package that type-checked, at the position the parser gave it and with the type the checker
/// recorded. Nothing is inferred from a name or a spelling.
///
/// Completeness [`Assurance::Sound`]: the three places Go lets a value reach the blank identifier --
/// an assignment or short declaration, a `var` or `const` declaration's initializer (in a `const`
/// block, also one repeated from the line above), and a range clause's key or value -- are each
/// walked in every file of every package the module compiles on this host, its tests included, and
/// a file is filed only once its whole package has type-checked. A `var _ T` with no initializer is
/// sent no value, and is not in the population. A file the helper did not check gets no fact at
/// all, never an empty one, so an empty fact means a file that discards nothing.
///
/// [`IncrementalGranularity::File`]: one fact per Go source file, since the rule that reads it
/// judges a file at a time.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File);
}

/// This provider's offer against [`nomos_cap_go_types::Capability_Contract`].
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
