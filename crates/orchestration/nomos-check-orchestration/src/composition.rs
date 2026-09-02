//! The registry this run composes, and what it can be asked to say about itself.
//!
//! Moved verbatim (apart from visibility) from `nomos-cli::check::composition`, minus
//! `Host_Variant` -- what this binary was compiled as is read through `env!`, which resolves
//! against the *compiling* crate's own build script (`nomos-cli/build.rs`), so a build
//! variant computed here would describe this library's own compilation rather than the
//! binary that called it. The composition root still owns that value and hands it to
//! [`crate::Run`] as an argument, the same way `nomos_work_orchestration::Run` takes
//! `published` as a value rather than deriving it.

use nomos_cap_syntax::Language;
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
    Declare_Naming_Policy_Capability(&mut registry)?;
    Declare_Limits_Policy_Capability(&mut registry)?;
    Declare_Scripting_Policy_Capability(&mut registry)?;
    Declare_Goals_Policy_Capability(&mut registry)?;
    Declare_Words_Policy_Capability(&mut registry)?;

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
/// `nomos_rules::SourceFile::preferred_syntax_provider` as data (`crate::run_context`'s own
/// enrichment step) for `nomos_rules::Syntax_Requirement_For` to attach via
/// `.Preferring(...)` -- never computed inside that crate.
/// [`crate::facts::dependency_materialization::Materialize_Syntax`]'s write side calls the identical
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

/// A sixth capability, one offer against it -- `OD-RULES-011`'s first capability instance
/// wired for real. `nomos_repo_standards` reads this repository's own `standards.json`
/// through the `FileSystem` [`crate::run_context::RunContext`] now carries, the same
/// `Declare`-then-`Offer` shape [`Declare_Dependency_Policy_Capability`] already has one
/// capability over.
fn Declare_Naming_Policy_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_naming_policy::Capability_Contract())?;
    registry.Offer(nomos_repo_standards::Provider_Offer())?;

    return Ok(());
}

/// A seventh capability, one offer against it -- `OD-RULES-011`'s threshold family, read the
/// same way naming's is one function above.
///
/// Five composed rules already ask for this capability by name and, until this declaration
/// existed, every one of them fell back to its own hardcoded constant. Worth stating plainly
/// because it is easy to oversell: on *this* repository the wiring changes no finding at all,
/// since `standards.json` declares exactly the numbers the fallbacks already carry -- 500
/// review, 1500 hard, 1000 for Go's own hard trigger, 4 parameters. What it changes is that
/// those numbers are now read rather than assumed, so a repository declaring different ones
/// is finally judged by its own.
fn Declare_Limits_Policy_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_limits_policy::Capability_Contract())?;
    registry.Offer(nomos_repo_limits::Provider_Offer())?;

    return Ok(());
}

/// An eighth capability, one offer against it -- and unlike its two policy siblings above,
/// this one is a live defect being closed rather than an assumption being replaced.
///
/// `Check_Declared_Tooling_Language_For_Scripts` resolves an absent
/// `nomos.cap.scripting.policy` to *no findings at all* rather than to a prior default,
/// because the rule never existed before the capability did and there was no earlier value to
/// fall back to. So with nothing declaring this capability, that rule has been silent in every
/// real run since it landed, while `standards.json` named a tooling language and five
/// forbidden extensions that nothing enforced.
fn Declare_Scripting_Policy_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_scripting_policy::Capability_Contract())?;
    registry.Offer(nomos_repo_scripting::Provider_Offer())?;

    return Ok(());
}

/// A ninth capability, one offer against it -- the last of `OD-RULES-011`'s five families to
/// reach a real run.
///
/// Its one rule is composed by the same item that adds this declaration, which is the reverse
/// of how limits and scripting arrived: those were rules already running and starved of a
/// fact, this is a fact that had no consumer. The words family is still absent for the same
/// reason stated the other way round -- its rule cannot be composed yet, so materializing its
/// fact would be wiring something nobody reads.
fn Declare_Goals_Policy_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_goals_policy::Capability_Contract())?;
    registry.Offer(nomos_repo_goals::Provider_Offer())?;

    return Ok(());
}

/// A tenth capability, one offer against it -- the last of `OD-RULES-011`'s five families to
/// reach a real run, and the one that waited longest for a reason worth recording.
///
/// Its rule was composable only once `Check_Abbreviations` stopped judging names a trait
/// fixes (`8ac2a830`): before that it reported 148 findings against this workspace, 104 of
/// them the word `fmt`, which `core::fmt::Display` requires and no author can rename.
fn Declare_Words_Policy_Capability(registry: &mut Registry) -> Result<(), RegistryError>
{
    registry.Declare(nomos_cap_words_policy::Capability_Contract())?;
    registry.Offer(nomos_repo_words::Provider_Offer())?;

    return Ok(());
}

/// Which registered `nomos.cap.syntax.items` provider `path` belongs to, if either does --
/// `OD-CAPABILITY-009`'s corrected fix, and the one function both halves of the pipeline
/// consult so they cannot independently drift on the answer.
///
/// This crate is the caller `OD-CAPABILITY-009` names: the one place that may know
/// `nomos_lang_rust` and `nomos_lang_go` by name to answer an applicability question no
/// [`Registry::Resolve`] call could, because by the time `Resolve` is reached the subject is
/// already the opaque digest [`nomos_contracts::SubjectId`] carries. `crate::run_context`'s own
/// enrichment step calls this to populate `nomos_rules::SourceFile::preferred_syntax_provider`
/// before any rule ever sees a source, and
/// [`crate::facts::dependency_materialization::Materialize_Syntax`]'s write side calls it again over the
/// identical path to decide which provider's own `Materialize_Syntax_Fact` to run. Both call sites
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

/// Which language `path` is written in, or `None` if no registered provider recognizes it.
///
/// Sits beside [`Recognized_Syntax_Provider`] and asks the identical question of the
/// identical crates, because a path's language and the provider that reads it are decided
/// by one recognition and must not be decided by two. `OD-RULES-014` moved this question
/// out of eleven private extension tests in `nomos-rules`; splitting it back across two
/// functions here would restore the same defect one layer up.
///
/// It is deliberately not derived from [`Recognized_Syntax_Provider`]'s answer. That
/// mapping happens to be lossless today, but a provider identity names reading technology
/// -- `nomos.lang.rust.syn` and `nomos.lang.rust.scan` are one language -- so deriving a
/// language from it would encode a many-to-one collapse that a third Rust provider would
/// silently have to be added to. Each language crate declares its own `LANGUAGE`, and this
/// reads it.
#[must_use]
pub(crate) fn Recognized_Language(path: &str) -> Option<Language>
{
    if nomos_lang_rust::Recognition::Of_Path(path) == nomos_lang_rust::Recognition::Recognized
    {
        return Some(Language::New(nomos_lang_rust::LANGUAGE));
    }

    if nomos_lang_go::Recognition::Of_Path(path) == nomos_lang_go::Recognition::Recognized
    {
        return Some(Language::New(nomos_lang_go::LANGUAGE));
    }

    return None;
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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The composition this crate ships must not be self-contradictory, and it must
    /// declare exactly the ten capabilities [`Registered`]'s own body wires: syntax,
    /// dependency, controlflow, lint, dependency-policy, and all five of `OD-RULES-011`'s
    /// families -- naming, limits, scripting, goals and words.
    #[test]
    fn Test_Registered_Should_Declare_Every_Composed_Capability()
    {
        let registry = Registered().expect("this crate's own composition must not be self-contradictory");

        assert_eq!(
            registry.Declared().count(),
            10,
            "Registered() wires ten Declare calls; a changed count here means the two drifted"
        );
    }

    /// [`Recognized_Syntax_Provider`]'s whole contract: a `.rs` path resolves to
    /// `nomos_lang_rust`'s identity, a `.go` path to `nomos_lang_go`'s, and a path neither
    /// recognizes resolves to neither.
    #[test]
    fn Test_Recognized_Syntax_Provider_Should_Resolve_By_Extension()
    {
        assert_eq!(Recognized_Syntax_Provider("a.rs"), Some(ProviderId::New(nomos_lang_rust::PROVIDER)));
        assert_eq!(Recognized_Syntax_Provider("main.go"), Some(ProviderId::New(nomos_lang_go::PROVIDER)));
        assert_eq!(Recognized_Syntax_Provider("readme.md"), None);
    }

    /// [`Recognized_Language`]'s whole contract, and deliberately a separate assertion from
    /// the provider one above: the two answers come from the same recognition but are not
    /// the same fact.
    #[test]
    fn Test_Recognized_Language_Should_Resolve_By_Extension()
    {
        assert_eq!(Recognized_Language("a.rs"), Some(Language::New(nomos_lang_rust::LANGUAGE)));
        assert_eq!(Recognized_Language("main.go"), Some(Language::New(nomos_lang_go::LANGUAGE)));
        assert_eq!(Recognized_Language("readme.md"), None);
    }

    /// The guard `OD-RULES-014` requires, and the one place in this workspace that can hold
    /// it. A language-restricted rule states its own literal because `nomos-rules` may not
    /// depend on a language crate, so nothing in that crate can check the literal against
    /// what the provider actually declares. If the two ever drift the rule simply stops
    /// firing -- no findings, no reported absence -- which is exactly the silent failure the
    /// record set out to remove. This crate depends on both sides and fails loudly instead.
    #[test]
    fn Test_The_Language_Names_Rules_State_Should_Agree_With_What_The_Language_Crates_Declare()
    {
        assert_eq!(nomos_rules::RUST_LANGUAGE, nomos_lang_rust::LANGUAGE);
        assert_eq!(nomos_rules::RUST_LANGUAGE, nomos_lang_rust_scan::LANGUAGE);
        assert_eq!(nomos_rules::GO_LANGUAGE, nomos_lang_go::LANGUAGE);
    }

    /// Both Rust providers are one language, which is the measurement that decided
    /// `OD-RULES-014`: a provider identity cannot stand in for a language because this
    /// equality holds while the identities differ.
    #[test]
    fn Test_The_Two_Rust_Providers_Should_Declare_One_Language_Under_Two_Identities()
    {
        assert_eq!(nomos_lang_rust::LANGUAGE, nomos_lang_rust_scan::LANGUAGE);
        assert_ne!(nomos_lang_rust::PROVIDER, nomos_lang_rust_scan::PROVIDER);
    }

    /// [`Resolved_Configuration`] is documented as a digest of a fully resolved effective
    /// policy: the same registry must render the same identity every time, or a fact
    /// this run materializes would be filed under an address that moves under it.
    #[test]
    fn Test_Resolved_Configuration_Should_Be_Deterministic_Over_The_Same_Registry()
    {
        let registry = Registered().expect("this crate's own composition must not be self-contradictory");

        let first = Resolved_Configuration(&registry);
        let second = Resolved_Configuration(&registry);

        assert_eq!(first, second, "the same registry must render the same configuration identity");
    }
}
