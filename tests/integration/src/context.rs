//! Where a fact's context comes from.
//!
//! A [`nomos_analysis::FactKey`] names a snapshot, a build variant and a configuration, and
//! every fact in the store is filed under all three. The slice used to supply them as three
//! byte-fill constants — `[0x51; 16]`, `[0x52; 16]`, `[0x53; 16]` — which compiled, ran,
//! and produced a corpus of facts whose identity rested on nothing.
//!
//! That is worse than it sounds. An identity component nobody derived from anything cannot
//! be *wrong*, so it can never be observed to be wrong: two machines, two toolchains and
//! two policies all agree, and the disagreement they should have had is the one the fact
//! key exists to detect. This module is the other half — each of the three comes from
//! something real, and changing that thing changes the identity.
//!
//! The snapshot comes from [`nomos_workspace::Workspace`] and is the slice's own doing. The
//! other two are here.

use nomos_capability::Registry;
use nomos_contracts::{ConfigurationId, Guarantee};
use nomos_workspace::BuildVariant;

/// The schema of the configuration rendering below.
///
/// Part of the digested bytes, so that changing how a configuration is written down is
/// itself a configuration change rather than a silent re-addressing of every fact in the
/// store under the old spelling.
pub const CONFIGURATION_SCHEMA: &str = "nomos.slice.configuration.v1";

/// The build variant this binary is compiled as.
///
/// Every component is captured by `build.rs` from cargo's own environment, because none of
/// them survives into the compiled program: `std::env::consts` knows an architecture and an
/// operating system, and `x86_64-pc-windows-msvc` and `x86_64-pc-windows-gnu` agree on both
/// while being two different programs.
#[must_use]
pub fn Host_Variant() -> BuildVariant
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

/// A guarantee as one field, by its four stable labels.
///
/// Written out rather than derived. `{guarantee:?}` would put a `Debug` implementation
/// between the data and the digest, and a `Debug` output is not a stable format — a
/// derive's spacing changing in a point release would re-address every fact in the store.
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

/// The effective policy of one composition, written out.
///
/// # Why the registry is the configuration
///
/// [`ConfigurationId`] is documented as "a digest of a fully resolved effective policy",
/// and for an analysis run the resolved policy *is* which capabilities are declared and
/// which providers may answer for them, at what versions and under what guarantees. That is
/// exactly what a [`Registry`] holds once composition is finished. Inventing a second
/// policy object beside it would give the run two answers to what it is configured to do.
///
/// Contract summaries are deliberately absent: prose is documentation, and rewording a
/// sentence must not invalidate a corpus.
///
/// Line-oriented and tab-separated, hand-written, for the reason every payload in this
/// system is: nothing derived may sit between the data and its digest.
#[must_use]
pub fn Configuration_Rendering(registry: &Registry) -> String
{
    let mut rendered = String::new();
    rendered.push_str(CONFIGURATION_SCHEMA);
    rendered.push('\n');

    for contract in registry.Declared()
    {
        rendered.push_str("capability\t");
        rendered.push_str(contract.id.As_Str());
        rendered.push('\t');
        rendered.push_str(&contract.version.to_string());
        rendered.push('\t');
        rendered.push_str(&Rendered_Guarantee(contract.ceiling));
        rendered.push('\n');
        Render_The_Offers(&mut rendered, registry, &contract.id);
    }

    return rendered;
}

/// Every offer against one capability, in the order the registry holds them.
fn Render_The_Offers(
    rendered: &mut String,
    registry: &Registry,
    capability: &nomos_contracts::CapabilityId,
)
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

/// The identity of that policy.
#[must_use]
pub fn Resolved_Configuration(registry: &Registry) -> ConfigurationId
{
    use nomos_model::Content_Digest;

    return ConfigurationId::From_Digest(Content_Digest(
        Configuration_Rendering(registry).as_bytes(),
    ));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_capability::{CapabilityContract, ProviderOffer};
    use nomos_contracts::{
        Assurance, CapabilityId, ContractVersion, FactVariant, IncrementalGranularity, ProviderId,
    };

    const VERSION: ContractVersion = ContractVersion::New(1, 0);

    fn Syntactic() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    fn Registry_With(providers: &[&str]) -> Registry
    {
        let mut registry = Declaring_The_Test_Capability();
        for provider in providers
        {
            registry
                .Offer(ProviderOffer {
                    provider: ProviderId::New(*provider),
                    capability: CapabilityId::New("nomos.cap.test"),
                    version: VERSION,
                    guarantee: Syntactic(),
                })
                .expect("within the ceiling");
        }

        return registry;
    }

    /// A registry holding one capability and no offer against it yet.
    fn Declaring_The_Test_Capability() -> Registry
    {
        let mut registry = Registry::New();
        registry
            .Declare(CapabilityContract {
                id: CapabilityId::New("nomos.cap.test"),
                version: VERSION,
                summary: "a capability for this test".to_owned(),
                ceiling: Guarantee::New(
                    FactVariant::Syntactic,
                    Assurance::Sound,
                    Assurance::Sound,
                    IncrementalGranularity::Region,
                ),
            })
            .expect("declared once");

        return registry;
    }

    /// The variant describes this build, not a build somebody wrote down once.
    #[test]
    fn Test_The_Host_Variant_Should_Describe_This_Build()
    {
        let variant = Host_Variant();

        assert!(
            variant.target.contains(std::env::consts::ARCH),
            "`{}` is not a triple for the architecture this test is running on ({})",
            variant.target,
            std::env::consts::ARCH
        );
        assert!(
            !variant.profile.is_empty() && !variant.toolchain.is_empty(),
            "a variant with an empty component is a variant that identifies nothing: {variant:?}"
        );
        assert_eq!(
            variant,
            Host_Variant(),
            "the same build must produce the same variant, or every run is a new one"
        );
    }

    /// A different set of providers is a different configuration. This is the property that
    /// makes registering a second provider invalidate the facts the first one produced.
    #[test]
    fn Test_A_Different_Composition_Should_Be_A_Different_Configuration()
    {
        let one = Registry_With(&["nomos.test.one"]);
        let two = Registry_With(&["nomos.test.one", "nomos.test.two"]);

        assert_ne!(
            Resolved_Configuration(&one),
            Resolved_Configuration(&two),
            "two providers answering one capability is not the policy one provider was"
        );
        assert_ne!(
            Resolved_Configuration(&one),
            Resolved_Configuration(&Registry::New()),
            "an empty composition is not the one that declared something"
        );
    }

    /// The positive control for the test above. If the rendering included anything that
    /// varies between two identical compositions, nothing would ever be reused.
    #[test]
    fn Test_The_Same_Composition_Should_Be_The_Same_Configuration()
    {
        assert_eq!(
            Resolved_Configuration(&Registry_With(&["nomos.test.one"])),
            Resolved_Configuration(&Registry_With(&["nomos.test.one"]))
        );
    }

    /// Offer order is a fact about how a composition root was written, not about what it
    /// resolved to. The registry sorts its offers; this asserts that the rendering reads
    /// them in that order rather than in arrival order.
    #[test]
    fn Test_Offer_Order_Should_Not_Reach_The_Configuration()
    {
        assert_eq!(
            Resolved_Configuration(&Registry_With(&["nomos.test.alpha", "nomos.test.beta"])),
            Resolved_Configuration(&Registry_With(&["nomos.test.beta", "nomos.test.alpha"]))
        );
    }

    /// A guarantee reaches the configuration. A provider quietly weakening what it promises
    /// must not go on answering under the identity it earned when it promised more.
    #[test]
    fn Test_A_Weakened_Guarantee_Should_Be_A_Different_Configuration()
    {
        let mut weakened = Registry::New();
        weakened
            .Declare(CapabilityContract {
                id: CapabilityId::New("nomos.cap.test"),
                version: VERSION,
                summary: "a capability for this test".to_owned(),
                ceiling: Guarantee::New(
                    FactVariant::Syntactic,
                    Assurance::Sound,
                    Assurance::Sound,
                    IncrementalGranularity::Region,
                ),
            })
            .expect("declared once");
        weakened
            .Offer(ProviderOffer {
                provider: ProviderId::New("nomos.test.one"),
                capability: CapabilityId::New("nomos.cap.test"),
                version: VERSION,
                guarantee: Guarantee::New(
                    FactVariant::Syntactic,
                    Assurance::Unknown,
                    Assurance::Unknown,
                    IncrementalGranularity::Project,
                ),
            })
            .expect("within the ceiling");

        assert_ne!(
            Resolved_Configuration(&weakened),
            Resolved_Configuration(&Registry_With(&["nomos.test.one"]))
        );
    }
}
