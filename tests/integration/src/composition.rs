//! Declaring both capabilities and registering every provider that offers them.
//!
//! Not a method on the composed system. A registry is what a [`crate::Slice`] is *given*,
//! and building one reads no part of a slice — a caller that wants two slices over one
//! registry, or a registry with nothing composed around it yet, has to be able to ask for
//! it without holding the thing it is meant to compose.

use nomos_capability::Registry;

/// Declares both capabilities and registers both providers.
///
/// # Panics
///
/// If the registry refuses a declaration or an offer. Both are decided by this
/// function's own constants, so a refusal here is a contradiction in the composition
/// rather than a runtime condition — and continuing past it would produce a run whose
/// facts nobody offered.
#[must_use]
pub fn Registered() -> Registry
{
    use nomos_cap_syntax as syntax;
    use nomos_lang_rust as rust;
    use nomos_lang_rust_scan as scan;
    use crate::surface;

    let mut registry = Registry::New();

    registry
        .Declare_And_Offer(syntax::Capability_Contract(), rust::Provider_Offer())
        .expect("the syntax capability is declared once, and the Rust provider is within it");
    // The second offer against the same contract, and the reason this one is not a
    // `Declare_And_Offer`. It is accepted because the ceiling bounds what may be *claimed*,
    // not how weak an offer may be — a provider that promises less than the ceiling is
    // exactly what a ceiling is for.
    registry
        .Offer(scan::Provider_Offer())
        .expect("the scanner's offer is within the same ceiling");
    registry
        .Declare_And_Offer(surface::Capability_Contract(), surface::Provider_Offer())
        .expect("the surface capability is declared once, and the rollup is within it");

    return registry;
}
