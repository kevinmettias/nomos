//! The registry a real check run holds, exercised from outside the crate through public types
//! only: the contracts `nomos_check_orchestration::Registered` declares, with the offers
//! `nomos_composer_providers::Standard_Offers` selects and every host hands it (`OD-HOST-020`).
//!
//! `composition.rs`'s own internal `#[cfg(test)] mod tests` counts what `Registered` declares,
//! but its assertions never spell a capability or provider crate's name -- it reads
//! `registry.Declared().count()` rather than comparing against each capability's own real
//! contract. So nothing outside this crate proves the registry a run holds actually carries the
//! SAME contracts and offers each capability/provider crate declares for itself, rather than a
//! stale or hand-copied version of them drifting quietly apart. This file is that proof: it
//! drives `Registered` with the standard offers -- the pair every real run's registry is built
//! from -- and checks every declared capability and every offered provider against each source
//! crate's own public `Capability_Contract()`/`Provider_Offer()`, through nothing but public API
//! on every side of the boundary.
//!
//! It also holds `OD-PACKAGE-014`'s check over this workspace's own manifests of both package
//! kinds (`OD-PACKAGE-018`), for the same reason: that claim is about the registry a run holds,
//! and this crate's unit tests hold only a copy of it.

use nomos_check_orchestration::{Check_Package_Conformance, DeclaredPackage, Registered};
use nomos_contracts::GateCategory;
use nomos_package::ProviderRegistration;
use std::path::Path;

/// Five of the capabilities `Registered` declares, each checked against the real contract its own
/// capability crate publishes -- not a value this test invents.
#[test]
fn Test_Registered_Should_Declare_Every_Capability_Crates_Own_Real_Contract()
{
    let registry: nomos_capability::Registry = Registered(nomos_composer_providers::Standard_Offers).expect("this crate's own composition must not be self-contradictory");
    let declared: Vec<_> = registry.Declared().collect();

    assert!(declared.contains(&&nomos_cap_syntax::Capability_Contract()), "the syntax capability must match nomos_cap_syntax's own contract");
    assert!(declared.contains(&&nomos_cap_dependency::Capability_Contract()), "the dependency capability must match nomos_cap_dependency's own contract");
    assert!(declared.contains(&&nomos_cap_controlflow::Capability_Contract()), "the controlflow capability must match nomos_cap_controlflow's own contract");
    assert!(declared.contains(&&nomos_cap_lint::Capability_Contract()), "the lint capability must match nomos_cap_lint's own contract");
    assert!(declared.contains(&&nomos_cap_dependency_policy::Capability_Contract()), "the dependency-policy capability must match nomos_cap_dependency_policy's own contract");
    assert!(declared.contains(&&nomos_cap_complexity::Capability_Contract()), "the complexity capability must match nomos_cap_complexity's own contract");
}

/// The syntax capability's offers, each checked against the real offer its own language
/// provider crate publishes.
#[test]
fn Test_Registered_Should_Offer_Every_Syntax_Providers_Own_Real_Offer()
{
    let registry = Registered(nomos_composer_providers::Standard_Offers).expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_syntax::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust::Provider_Offer()), "the parser offer must match nomos_lang_rust's own offer");
    assert!(offers.contains(&nomos_lang_rust_scan::Provider_Offer()), "the scanner offer must match nomos_lang_rust_scan's own offer");
    assert!(offers.contains(&nomos_lang_go::Provider_Offer()), "the Go offer must match nomos_lang_go's own offer");
    assert!(offers.contains(&nomos_lang_csharp::Provider_Offer()), "the C# offer must match nomos_lang_csharp's own offer");
}

/// The dependency capability's two offers, each checked against the real offer its own manifest
/// provider crate publishes.
#[test]
fn Test_Registered_Should_Offer_Every_Dependency_Providers_Own_Real_Offer()
{
    let registry = Registered(nomos_composer_providers::Standard_Offers).expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_dependency::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust_cargo::Provider_Offer()), "the Cargo offer must match nomos_lang_rust_cargo's own offer");
    assert!(offers.contains(&nomos_lang_go_modules::Provider_Offer()), "the Go modules offer must match nomos_lang_go_modules's own offer");
}

/// The controlflow capability's one offer, checked against `nomos_lang_rust::reachability`'s
/// own real offer.
#[test]
fn Test_Registered_Should_Offer_The_Reachability_Providers_Own_Real_Offer()
{
    let registry = Registered(nomos_composer_providers::Standard_Offers).expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_controlflow::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust::reachability::Provider_Offer()), "the reachability offer must match nomos_lang_rust::reachability's own offer");
}

/// The syntax sites family: declared as `nomos_cap_syntax` publishes it, with the offer
/// `nomos_lang_rust::sites` publishes -- one contract for every kind there will be, and one offer
/// per language that offers any.
#[test]
fn Test_Registered_Should_Declare_The_Sites_Family_And_Offer_The_Rust_Sites_Providers_Own_Real_Offer()
{
    let registry = Registered(nomos_composer_providers::Standard_Offers).expect("this crate's own composition must not be self-contradictory");

    assert!(registry.Declared().any(|contract| return *contract == nomos_cap_syntax::Sites_Capability_Contract()), "the sites family must match nomos_cap_syntax's own contract");
    let offers = registry.Offers(&nomos_cap_syntax::Sites_Capability());
    assert!(offers.contains(&nomos_lang_rust::sites::Provider_Offer()), "the Rust sites offer must match nomos_lang_rust::sites's own offer");
}

/// The complexity capability's one offer, checked against `nomos_lang_rust_complexity`'s own real
/// offer -- the offer the composer selects, not a copy of it.
#[test]
fn Test_Registered_Should_Offer_The_Complexity_Providers_Own_Real_Offer()
{
    let registry = Registered(nomos_composer_providers::Standard_Offers).expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_complexity::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust_complexity::Provider_Offer()), "the complexity offer must match nomos_lang_rust_complexity's own offer");
}

/// The lint capability's one offer, checked against `nomos_lang_rust_clippy`'s own real offer.
#[test]
fn Test_Registered_Should_Offer_The_Clippy_Providers_Own_Real_Offer()
{
    let registry = Registered(nomos_composer_providers::Standard_Offers).expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_lint::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust_clippy::Provider_Offer()), "the clippy offer must match nomos_lang_rust_clippy's own offer");
}

/// The dependency-policy capability's one offer, checked against `nomos_lang_rust_deny`'s own
/// real offer.
#[test]
fn Test_Registered_Should_Offer_The_Deny_Providers_Own_Real_Offer()
{
    let registry = Registered(nomos_composer_providers::Standard_Offers).expect("this crate's own composition must not be self-contradictory");
    let offers = registry.Offers(&nomos_cap_dependency_policy::Capability_Contract().id);

    assert!(offers.contains(&nomos_lang_rust_deny::Provider_Offer()), "the deny offer must match nomos_lang_rust_deny's own offer");
}

/// The real caller this check is for: this workspace's own six real, checked-in manifests, three
/// of each kind, against the registry a real run holds -- not a synthetic fixture.
///
/// `OD-PACKAGE-014` measured that nothing ever read these manifests; this is the first real read.
/// Read against the three `LanguagePackage` manifests alone it reported seven registered
/// providers no manifest claimed, and this test asserted them as a gap while the three language
/// packages' own docs called them a deliberate exclusion. `OD-PACKAGE-018` settled which reading
/// held: a language package declares the providers that recognize its language, a `ToolProvider`
/// package declares every other language provider under its family, and the check reads both
/// kinds. With `packages/nomos.tool.rust.json`, `.go.json` and `.csharp.json` beside the language
/// manifests, nothing registered is unclaimed and nothing claimed is unregistered.
///
/// `Test_The_Language_Manifests_Alone_Should_Leave_Exactly_The_Tool_Providers_Unclaimed` is
/// the control. The same comparison, without the tool manifests, still finds exactly the providers
/// a tool manifest declares, so the empty answer below comes from the tool manifests claiming them
/// and not from a comparison that stopped looking. A provider added later to neither kind of
/// manifest shows up here by name.
///
/// It moved here from `package_conformance.rs`'s unit tests when `OD-HOST-020` took the
/// provider set out of this crate. Those unit tests can see only the local, test-only table
/// that record's section 5 keeps, which is a copy of the standard set rather than the set a
/// run holds, and a claim about this workspace's real manifests is worth making only against
/// the real offers.
#[test]
fn Test_Check_Package_Conformance_Against_This_Workspaces_Own_Real_Manifests_And_Registry()
{
    let manifests = Real_Manifests();
    let registry = Registered(nomos_composer_providers::Standard_Offers).expect("this crate's own composition must not be self-contradictory");

    let findings = Check_Package_Conformance(&manifests.Declared(), &registry);

    assert!(findings.is_empty(), "{findings:?}");
}

/// The three language manifests alone leave exactly the providers `OD-PACKAGE-018` gives to
/// a `ToolProvider` package unclaimed -- the control that makes the empty answer above mean
/// something.
#[test]
fn Test_The_Language_Manifests_Alone_Should_Leave_Exactly_The_Tool_Providers_Unclaimed()
{
    let manifests = Real_Manifests();
    let registry = Registered(nomos_composer_providers::Standard_Offers).expect("this crate's own composition must not be self-contradictory");
    let declared = manifests.Declared();
    let languages_only: Vec<DeclaredPackage<'_>> = declared.into_iter().filter(|package| return package.manifest_path.starts_with("packages/nomos.lang.")).collect();

    let findings = Check_Package_Conformance(&languages_only, &registry);

    let names: Vec<&str> = findings.iter().map(|finding| return finding.subject_name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "nomos.lang.csharp.msbuild",
            "nomos.lang.go.modules",
            "nomos.lang.go.types",
            "nomos.lang.go.vet",
            "nomos.lang.rust.cargo",
            "nomos.lang.rust.clippy",
            "nomos.lang.rust.compiler",
            "nomos.lang.rust.complexity",
            "nomos.lang.rust.deny"
        ],
        "{findings:?}"
    );
    assert!(findings.iter().all(|finding| return finding.gate == GateCategory::Advisory), "{findings:?}");
}

/// Every manifest under `packages/`, read by its own kind's reader, as the provider lists the
/// check compares -- a `ToolProvider` registration's family set aside, since the check compares
/// identities and the family classifies a registration without changing which provider it is.
struct RealManifests
{
    declared: Vec<(&'static str, Vec<ProviderRegistration>)>,
}

impl RealManifests
{
    fn Declared(&self) -> Vec<DeclaredPackage<'_>>
    {
        return self.declared.iter().map(|(manifest_path, providers)| return DeclaredPackage { manifest_path, providers }).collect();
    }
}

fn Real_Manifests() -> RealManifests
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let language = |path: &'static str, providers: Vec<ProviderRegistration>| return (path, providers);
    let tool = |path: &'static str| {
        let package = nomos_tool_package::Read_Manifest(&root.join(path)).unwrap_or_else(|error| panic!("{path} parses: {error:?}"));
        let providers = package.providers.into_iter().map(|registration| return ProviderRegistration { provider: registration.provider, tool_version: registration.tool_version }).collect();
        return (path, providers);
    };

    let declared = vec![
        language("packages/nomos.lang.rust.json", nomos_lang_rust_package::Read_Manifest(&root.join("packages/nomos.lang.rust.json")).expect("the rust manifest parses").providers),
        language("packages/nomos.lang.go.json", nomos_lang_go_package::Read_Manifest(&root.join("packages/nomos.lang.go.json")).expect("the go manifest parses").providers),
        language("packages/nomos.lang.csharp.json", nomos_lang_csharp_package::Read_Manifest(&root.join("packages/nomos.lang.csharp.json")).expect("the csharp manifest parses").providers),
        tool("packages/nomos.tool.rust.json"),
        tool("packages/nomos.tool.go.json"),
        tool("packages/nomos.tool.csharp.json"),
    ];
    return RealManifests { declared };
}
