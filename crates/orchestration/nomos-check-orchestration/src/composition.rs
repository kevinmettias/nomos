//! The registry this run composes, and what it can be asked to say about itself.
//!
//! Moved verbatim (apart from visibility) from `nomos-cli::check::composition`, minus
//! `Host_Variant` -- what this binary was compiled as is read through `env!`, which resolves
//! against the *compiling* crate's own build script (`nomos-cli/build.rs`), so a build
//! variant computed here would describe this library's own compilation rather than the
//! binary that called it. The composition root still owns that value and hands it to
//! [`crate::Run`] as an argument, the same way `nomos_work_orchestration::Run` takes
//! `published` as a value rather than deriving it.

use nomos_capability::{Registry, RegistryError};
use nomos_contracts::{CapabilityId, ConfigurationId, Guarantee};

/// The capability this run declares and the providers it admits.
///
/// # Errors
///
/// [`RegistryError`] if the registry refuses the declaration or the offer. Both are
/// decided by this function's own constants, so a refusal is a contradiction in the
/// composition rather than a runtime condition -- and it is still handed back rather than
/// unwound, because continuing past it would produce a run whose facts nobody offered and
/// the caller is the one that decides what a run it cannot compose is worth.
pub fn Registered() -> Result<Registry, RegistryError>
{
    let mut registry = Registry::New();

    Declare_Syntax_Capability(&mut registry)?;
    Declare_Dependency_Capability(&mut registry)?;
    Declare_Controlflow_Capability(&mut registry)?;

    return Ok(registry);
}

/// The syntax capability, and its two competing offers.
///
/// No selection mechanism is added alongside the second offer -- `OD-HOST-004` decided a
/// second offer is composition, not choice, and `nomos_capability::Registry::Resolve` ranks
/// between the two on its own: the parser's guarantee is strictly stronger on every axis the
/// scanner differs on, so it remains the offer `Resolve` chooses with no preference named.
///
/// `nomos_lang_go` is deliberately *not* a third offer here. `Registry::Resolve` ranks
/// purely by guarantee strength with no notion of which subjects a provider can even
/// attempt, and `nomos_lang_go`'s declared guarantee -- `Assurance::Sound` on both axes,
/// where both Rust providers are weaker on completeness -- is strictly stronger than
/// either. Offering it here was tried and reverted: `nomos_rules::Syntax_Requirement`'s own
/// resolution (deliberately unpreferenced, per that function's own doc) then resolved to
/// `nomos_lang_go` for `.rs` files too, and every `.rs` fact this composition already wrote
/// under `nomos_lang_rust`'s provider identity stopped matching what the rule's index
/// believed had answered -- four real tests in this crate's own `tests.rs` went from green
/// to `DependencyUnavailable` findings, not from a mistake at this call site but from
/// `ProviderOffer` itself carrying no subject or domain scope for `Resolve` to rank within.
///
/// This is not `OD-CAPABILITY-006`'s cross-language case -- that record is explicit that it
/// governs two *different* languages' subjects joined by a declared correspondence, and
/// says so precisely to keep `OD-CAPABILITY-001`'s same-capability ranking out of its own
/// scope: "There is no ranking between a Rust ownership fact and a Python ownership fact...
/// neither is a weaker or stronger offer of the same fact, because they are not offers of
/// the same fact." `nomos_lang_rust` and `nomos_lang_go` *are* offers of the same fact,
/// `nomos.cap.syntax.items` -- the same shape `OD-CAPABILITY-001` already governs -- except
/// that unlike the parser and the scanner, neither can actually answer for the other's
/// subjects at all. No existing record states what a caller-unpreferenced `Resolve` owes a
/// capability whose real offers partition by subject rather than compete over one. Wiring
/// this provider into the registry waits on that decision; `P14-LANG-GO-SYNTAX-PROVIDER`'s
/// own follow-up item reserves it.
fn Declare_Syntax_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_syntax::Capability_Contract())?;
    registry.Offer(nomos_lang_rust::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_scan::Provider_Offer())?;

    return Ok(());
}

/// A second capability, one offer against it -- `OD-RULES-003`'s design, wired for real.
fn Declare_Dependency_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_dependency::Capability_Contract())?;
    registry.Offer(nomos_lang_rust_cargo::Provider_Offer())?;

    return Ok(());
}

/// A third capability, one tier-1 offer against it -- `OD-RULES-008`'s design, wired for
/// real. `nomos_lang_rust::reachability::Provider_Offer` states its own guarantee at
/// `Syntactic`, below the ceiling `nomos_cap_controlflow::Capability_Contract` states at
/// `SemanticallyResolved`; `Check_Unread_Reaches_A_Finding`'s own `Reachability_Requirement`
/// asks for exactly what this offer delivers.
fn Declare_Controlflow_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_controlflow::Capability_Contract())?;
    registry.Offer(nomos_lang_rust::reachability::Provider_Offer())?;

    return Ok(());
}

/// The identity of this run's effective policy.
///
/// # Why the registry is the configuration
///
/// [`ConfigurationId`] is documented as a digest of a fully resolved effective policy, and
/// for an analysis run the resolved policy *is* which capabilities are declared and which
/// providers may answer for them, at what versions and under what guarantees -- which is
/// exactly what a [`Registry`] holds once composition is finished. Inventing a second
/// policy object beside it would give the run two answers to what it is configured to do.
///
/// Line-oriented, tab-separated and hand-written, because nothing derived may sit between
/// the data and its digest: a `Debug` implementation changing its spacing in a point
/// release would re-address every fact this run produces.
///
/// This is the second rendering of a registry in the workspace;
/// `tests/integration/src/context.rs` has the other, at band 100 where this crate cannot
/// reach it. Two renderings of one policy is a real duplication and it is not this item's
/// to remove -- `OD-HOST-002` names it and says so.
#[must_use]
pub fn Resolved_Configuration(registry: &Registry) -> ConfigurationId
{
    let mut rendered = String::from("nomos.check.configuration.v1\n");

    for contract in registry.Declared()
    {
        rendered.push_str("capability\t");
        rendered.push_str(contract.id.As_Str());
        rendered.push('\t');
        rendered.push_str(&contract.version.to_string());
        rendered.push('\t');
        rendered.push_str(&Rendered_Guarantee(contract.ceiling));
        rendered.push('\n');
        Render_Offers(&mut rendered, registry, &contract.id);
    }

    return ConfigurationId::From_Digest(nomos_model::Content_Digest(rendered.as_bytes()));
}

/// Every offer standing against one capability, each on its own line.
///
/// The offers are part of the configuration and not only the declarations, because the same
/// floor served by a different provider is a different composition and has to hash apart.
fn Render_Offers(rendered: &mut String, registry: &Registry, capability: &CapabilityId)
{
    for offer in registry.Offers(capability)
    {
        rendered.push_str("offer\t");
        rendered.push_str(capability.As_Str());
        rendered.push('\t');
        rendered.push_str(offer.provider.As_Str());
        rendered.push('\t');
        rendered.push_str(&offer.version.to_string());
        rendered.push('\t');
        rendered.push_str(&Rendered_Guarantee(offer.guarantee));
        rendered.push('\n');
    }
}

/// A guarantee as one field, by its four stable labels.
fn Rendered_Guarantee(guarantee: Guarantee) -> String
{
    return format!(
        "{}/{}/{}/{}",
        guarantee.variant.Label(),
        guarantee.soundness.Label(),
        guarantee.completeness.Label(),
        guarantee.incremental.Label()
    );
}
