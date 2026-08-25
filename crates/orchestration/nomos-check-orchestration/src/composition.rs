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
use nomos_contracts::{CapabilityId, ConfigurationId, Guarantee, ProviderId};

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
    Declare_Lint_Capability(&mut registry)?;
    Declare_Dependency_Policy_Capability(&mut registry)?;

    return Ok(registry);
}

/// The syntax capability, and its three offers -- two competing, one partitioned.
///
/// No selection mechanism is added between `nomos_lang_rust` and `nomos_lang_rust_scan` --
/// `OD-HOST-004` decided a second offer is composition, not choice, and
/// `nomos_capability::Registry::Resolve` ranks between the two on its own: the parser's
/// guarantee is strictly stronger on every axis the scanner differs on, so it remains the
/// offer `Resolve` chooses with no preference named.
///
/// `nomos_lang_go` is a real third offer here, safely, because it is no longer the
/// unpreferenced ranking `Resolve` performs between the first two that decides which
/// answers for a `.rs` file. Registering it that way was tried once and reverted: with no
/// preference named, `Resolve` ranks purely by guarantee strength, `nomos_lang_go`'s
/// declared guarantee -- `Assurance::Sound` on both axes, where both Rust providers are
/// weaker on completeness -- is strictly stronger than either, and it was chosen for `.rs`
/// files too. Every `.rs` fact this composition already wrote under `nomos_lang_rust`'s
/// provider identity stopped matching what the rule's index believed had answered -- four
/// real tests in this crate's own `tests.rs` went from green to `DependencyUnavailable`
/// findings, not from a mistake at this call site but from `ProviderOffer` itself carrying
/// no subject or domain scope for `Resolve` to rank within.
///
/// `OD-CAPABILITY-009` decided what a capability whose real offers partition by subject
/// rather than compete over one owes `Resolve`: nothing, because `Resolve` never receives a
/// subject to partition on by the time it is asked -- only the opaque digest `SubjectId`
/// already is. The caller narrows it instead, and this composition root is that caller --
/// the one place allowed to know both `nomos_lang_rust` and `nomos_lang_go` by name, unlike
/// `nomos_rules`, which never depends on a language-provider crate.
/// [`Recognized_Syntax_Provider`] computes `Recognition::Of_Path` against a real path here,
/// once, before it is ever digested into a `SubjectId`, and carries the result into
/// `nomos_rules::SourceFile::preferred_syntax_provider` as data (`crate::run`'s own
/// enrichment step) for `nomos_rules::Syntax_Requirement_For` to attach via
/// `.Preferring(...)` -- never computed inside that crate.
/// [`crate::facts::materialize::Materialize_Syntax`]'s write side calls the identical
/// [`Recognized_Syntax_Provider`], so the two sides agree on which identity a `.rs` or a
/// `.go` fact is filed under by construction, not by coincidence. With both sides narrowing
/// to the same provider by the same function, `Resolve` is never asked to rank
/// `nomos_lang_go` against the other two for a subject it cannot answer for, and the hazard
/// that reverted this offer the first time cannot recur.
///
/// This is not `OD-CAPABILITY-006`'s cross-language case -- that record is explicit that it
/// governs two *different* languages' subjects joined by a declared correspondence, and
/// says so precisely to keep `OD-CAPABILITY-001`'s same-capability ranking out of its own
/// scope: "There is no ranking between a Rust ownership fact and a Python ownership fact...
/// neither is a weaker or stronger offer of the same fact, because they are not offers of
/// the same fact." `nomos_lang_rust` and `nomos_lang_go` *are* offers of the same fact,
/// `nomos.cap.syntax.items` -- the same shape `OD-CAPABILITY-001` already governs, resolved
/// the way `OD-CAPABILITY-009` decided rather than by widening what `Resolve` ranks.
fn Declare_Syntax_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_syntax::Capability_Contract())?;
    registry.Offer(nomos_lang_rust::Provider_Offer())?;
    registry.Offer(nomos_lang_rust_scan::Provider_Offer())?;
    registry.Offer(nomos_lang_go::Provider_Offer())?;

    return Ok(());
}

/// Which registered `nomos.cap.syntax.items` provider `path` belongs to, if either does --
/// `OD-CAPABILITY-009`'s corrected fix, and the one function both halves of the pipeline
/// consult so they cannot independently drift on the answer.
///
/// This crate is the caller `OD-CAPABILITY-009` names: the one place that may know
/// `nomos_lang_rust` and `nomos_lang_go` by name to answer an applicability question no
/// [`Registry::Resolve`] call could, because by the time `Resolve` is reached the subject is
/// already the opaque digest [`nomos_contracts::SubjectId`] carries. `crate::run`'s own
/// enrichment step calls this to populate `nomos_rules::SourceFile::preferred_syntax_provider`
/// before any rule ever sees a source, and
/// [`crate::facts::materialize::Materialize_Syntax`]'s write side calls it again over the
/// identical path to decide which provider's own `Materialize` to run. Both call sites
/// reach this one function rather than each recomputing `Recognition::Of_Path` for
/// themselves, so read and write agree on a subject's provider identity by construction.
#[must_use]
pub(crate) fn Recognized_Syntax_Provider(path: &str) -> Option<ProviderId>
{
    if nomos_lang_rust::Recognition::Of_Path(path) == nomos_lang_rust::Recognition::Recognized
    {
        return Some(ProviderId::New(nomos_lang_rust::PROVIDER));
    }

    if nomos_lang_go::Recognition::Of_Path(path) == nomos_lang_go::Recognition::Recognized
    {
        return Some(ProviderId::New(nomos_lang_go::PROVIDER));
    }

    return None;
}

/// A second capability, and its second real offer.
///
/// `nomos-lang-go-modules` is a genuinely safe second offer here, unlike `nomos-lang-go`'s
/// own attempt at `Declare_Syntax_Capability` above: its declared completeness
/// (`Assurance::Unknown`, its own module doc says why) does not clear
/// `nomos_rules::Dependency_Requirement`'s real floor (`Assurance::Sound` on both axes), so
/// `nomos_capability::registry::resolving::Usable` filters it out of ranking before
/// `Registry::Resolve` ever has two comparable offers to choose between for any subject,
/// Rust or Go. `OD-CAPABILITY-009` measured this directly: the hazard it names is a
/// wrongly-*chosen* provider, and a provider that never clears a real caller's floor is
/// never chosen at all. Verified against this crate's own real test suite before this
/// offer was kept, not assumed.
fn Declare_Dependency_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_dependency::Capability_Contract())?;
    registry.Offer(nomos_lang_rust_cargo::Provider_Offer())?;
    registry.Offer(nomos_lang_go_modules::Provider_Offer())?;

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

/// A fourth capability, one offer against it -- `OD-RULES-010`'s design, the first real
/// `ToolProvider` wired for real.
fn Declare_Lint_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_lint::Capability_Contract())?;
    registry.Offer(nomos_lang_rust_clippy::Provider_Offer())?;

    return Ok(());
}

/// A fifth capability, one offer against it -- `OD-RULES-010`'s second real `ToolProvider`
/// wired for real, the identical shape [`Declare_Lint_Capability`] already has one
/// capability over.
fn Declare_Dependency_Policy_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_dependency_policy::Capability_Contract())?;
    registry.Offer(nomos_lang_rust_deny::Provider_Offer())?;

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
