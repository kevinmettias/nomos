//! The registry this build composes, and what it can be asked to say about itself.

use super::{Registry, BuildVariant, ConfigurationId, Content_Digest, CapabilityId, Guarantee};
use nomos_capability::RegistryError;

/// The capability this run declares and the providers it admits.
///
/// # Errors
///
/// [`RegistryError`] if the registry refuses the declaration or the offer. Both are
/// decided by this function's own constants, so a refusal is a contradiction in the
/// composition rather than a runtime condition — and it is still handed back rather than
/// unwound, because continuing past it would produce a run whose facts nobody offered and
/// the caller is the one that decides what a run it cannot compose is worth.
pub(super) fn Registered() -> Result<Registry, RegistryError>
{
    let mut registry = Registry::New();

    registry.Declare(nomos_cap_syntax::Capability_Contract())?;
    registry.Offer(nomos_lang_rust::Provider_Offer())?;

    return Ok(registry);
}

/// The build variant this binary was compiled as.
///
/// Every component is captured by `build.rs` from cargo's own environment, because none of
/// them survives into the compiled program.
pub(super) fn Host_Variant() -> BuildVariant
{
    return BuildVariant::New(
        env!("NOMOS_TARGET"),
        env!("NOMOS_PROFILE"),
        env!("NOMOS_TOOLCHAIN"),
        env!("NOMOS_FEATURES")
            .split(',')
            .filter(|feature| return !feature.is_empty()),
    );
}

/// The identity of this run's effective policy.
///
/// # Why the registry is the configuration
///
/// [`ConfigurationId`] is documented as a digest of a fully resolved effective policy, and
/// for an analysis run the resolved policy *is* which capabilities are declared and which
/// providers may answer for them, at what versions and under what guarantees — which is
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
/// to remove.
pub(super) fn Resolved_Configuration(registry: &Registry) -> ConfigurationId
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

    return ConfigurationId::From_Digest(Content_Digest(rendered.as_bytes()));
}

/// Every offer standing against one capability, each on its own line.
///
/// The offers are part of the configuration and not only the declarations, because the same
/// floor served by a different provider is a different composition and has to hash apart.
pub(super) fn Render_Offers(rendered: &mut String, registry: &Registry, capability: &CapabilityId)
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
pub(super) fn Rendered_Guarantee(guarantee: Guarantee) -> String
{
    return format!(
        "{}/{}/{}/{}",
        guarantee.variant.Label(),
        guarantee.soundness.Label(),
        guarantee.completeness.Label(),
        guarantee.incremental.Label()
    );
}
