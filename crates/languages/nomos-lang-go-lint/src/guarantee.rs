//! What this provider offers, and at what guarantee.

use nomos_cap_lint::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This provider's own name, for the tool it runs: `go vet`. A second Go linter answering the
/// same capability -- `staticcheck`, `golangci-lint` -- would be a second provider with its own
/// name, never this one widened.
pub const PROVIDER: &str = "nomos.lang.go.vet";

/// What this provider claims, on every axis -- the same four `nomos-lang-rust-clippy` claims,
/// each for the same kind of reason.
///
/// Exercised by `Test_A_Real_Tree_Should_Be_Filed_Module_By_Module_With_Each_Gap_Reported` in
/// `tests/guarantee.rs`, against the real toolchain: a finding only a type-checked reading reaches,
/// relayed exactly as `go vet` reported it, one fact per module.
///
/// [`FactVariant::SemanticallyResolved`]: `go vet` loads each package through the type checker,
/// and its analyzers run over resolved types, not text -- a `printf` verb is judged against the
/// argument's type.
///
/// Soundness [`Assurance::Sound`]: every diagnostic in a fact is one `go vet` itself reported,
/// at the position it gave; nothing here infers, merges or synthesizes one. That is a claim about
/// the relay, not about `go vet`'s heuristics, which its own documentation says do not guarantee
/// every report is a genuine problem.
///
/// Completeness [`Assurance::Unknown`]: `go vet` runs a fixed default suite of analyzers and makes
/// no claim to find every instance of what they look for, and no bound exists that would let
/// this provider claim more.
///
/// [`IncrementalGranularity::Project`]: one fact per Go module, from one `go vet` over the whole
/// module. Nothing below a module is materialized on its own.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::Project);
}

/// This provider's offer against [`nomos_cap_lint::Capability_Contract`].
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
