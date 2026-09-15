//! `OD-PACKAGE-014`: a `LanguagePackage`'s activation semantics, checked.
//!
//! The record decided what activation means for a manifest carrying `ProviderRegistration`
//! entries: the conformance claim that its declared providers match what
//! [`nomos_capability::Registry`] actually offers, checked and reported, never enacted.
//! `Check_Package_Conformance` is that check, in `Check_Every_Member_Declares_A_Band`'s own
//! shape (`OD-RULES-003`, applied a third time) -- declared data compared against an
//! observed fact, judged into a [`Finding`] on mismatch in either direction, and nothing
//! else. It never adds or removes an [`nomos_capability::ProviderOffer`]; `OD-HOST-004`
//! already forecloses that reading of "activation."
//!
//! # Why the unclaimed direction is scoped to `nomos.lang.`
//!
//! The registry this crate composes ([`crate::Registered`]) offers far more than language
//! providers -- `nomos.repo.*` policy providers, the review connector, the requirement-trace
//! provider -- and none of those belong to a `LanguagePackage` manifest; there is no
//! `PackageKind` that would ever claim them, and the record's own title is about a *language*
//! package's activation semantics specifically. Comparing every declared package against the
//! whole registry would report every one of those as "no manifest claims this," which is true
//! only in the sense that no `LanguagePackage` was ever meant to. So the direction that asks
//! "is anything registered that no manifest claims" is scoped to providers whose identity
//! carries the `nomos.lang.` prefix every real language provider already uses (verified
//! directly: `nomos.lang.rust.syn`, `.scan`, `.cargo`, `.clippy`, `.deny`,
//! `nomos.lang.go.tree-sitter`, `.modules` -- every one of them, and no non-language provider
//! carries it). The other direction -- a manifest declaring a provider the registry never
//! registers for anything -- needs no such scoping: a manifest claiming an unregistered
//! provider is wrong regardless of which namespace it claimed.
//!
//! # Where this is not yet wired
//!
//! Nothing calls this from a real host today. `OD-PACKAGE-014` names the shape a future
//! increment takes and builds nothing; this item builds the shape, and wiring it into a real
//! caller (`nomos-cli`, reading `packages/*.json` off disk) is deliberately left to a
//! separate item, the same "build the check, compose it later" split
//! `P46-UNCOMPOSED-RULES-ARE-COUNTED` already drew for `nomos-rules`' own checks -- this one
//! has no natural `RuleId`/`RulePackage` entry to begin with, since every existing
//! [`crate::run_context`] `ComposedRule` wraps a `nomos_rules` export and this check's
//! implementation deliberately does not live there (nothing in `nomos-rules` may depend on a
//! package-manifest crate).

use nomos_capability::Registry;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, ProviderId, RuleId};
use nomos_model::Content_Digest;
use nomos_package::ProviderRegistration;
use std::collections::BTreeSet;

/// Identity of the conformance check itself.
pub const PACKAGE_CONFORMANCE: &str = "package-manifest-matches-the-registry";

/// A prefix shared by every real language provider identity this workspace registers, and by
/// no other kind -- checked directly against every `PROVIDER` constant in
/// `crates/languages/*`, none of which is shared with `crates/repository/nomos-repo-policy`,
/// the review connector or the requirement-trace provider.
const LANGUAGE_PROVIDER_PREFIX: &str = "nomos.lang.";

/// One manifest's declared providers, named by the manifest a caller read them from.
///
/// Deliberately generic over `nomos_package::ProviderRegistration` rather than over
/// `nomos_lang_rust_package::LanguagePackage` or its Go sibling: the two manifest types are
/// distinct structs in distinct crates with no shared trait between them (`OD-CAPABILITY-002`
/// licenses the duplication), and `providers` is the one field this check needs, already
/// carried at the shared, language-agnostic layer both delegate to.
#[derive(Debug)]
pub struct DeclaredPackage<'a>
{
    /// Where this package's providers were declared -- a manifest path, reported so a finding
    /// says where to look.
    pub manifest_path: &'a str,
    /// The providers this manifest registers.
    pub providers: &'a [ProviderRegistration],
}

/// Compares every package's declared providers against what `registry` actually offers, and
/// reports a mismatch in either direction: a manifest claiming a provider the registry never
/// registers for any capability, and a registered language provider no manifest claims.
///
/// This is a report, never an action -- it reads `registry` and never mutates it, and neither
/// direction changes what a real run composes.
#[must_use]
pub fn Check_Package_Conformance(packages: &[DeclaredPackage<'_>], registry: &Registry) -> Vec<Finding>
{
    let registered = Registered_Providers(registry);
    let declared = Declared_Providers(packages);
    let mut findings = Unregistered_Provider_Findings(packages, &registered);
    let unclaimed = Unclaimed_Provider_Findings(&registered, &declared, packages);
    findings.extend(unclaimed);
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Every provider identity `registry` offers, under any capability.
fn Registered_Providers(registry: &Registry) -> BTreeSet<ProviderId>
{
    return registry
        .Declared()
        .flat_map(|contract| registry.Offers(&contract.id))
        .map(|offer| return offer.provider.clone())
        .collect();
}

/// Every provider identity the checked manifests declare.
fn Declared_Providers(packages: &[DeclaredPackage<'_>]) -> BTreeSet<ProviderId>
{
    return packages
        .iter()
        .flat_map(|package| package.providers.iter())
        .map(|registration| return registration.provider.clone())
        .collect();
}

/// One finding per declared provider the registry never registers for any capability.
///
/// Unscoped, unlike the other direction: a manifest claiming a provider nothing registers is
/// wrong regardless of which namespace it claimed, so there is no prefix to filter on.
fn Unregistered_Provider_Findings(packages: &[DeclaredPackage<'_>], registered: &BTreeSet<ProviderId>) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for package in packages
    {
        for registration in package.providers
        {
            if !registered.contains(&registration.provider)
            {
                let finding = Unregistered_Provider_Finding(package.manifest_path, &registration.provider);
                findings.push(finding);
            }
        }
    }

    return findings;
}

/// A manifest declares a provider the registry never registers for any capability.
fn Unregistered_Provider_Finding(manifest_path: &str, provider: &ProviderId) -> Finding
{
    return Finding {
        rule: RuleId::New(PACKAGE_CONFORMANCE),
        subject: Subject_Of_Provider(provider),
        subject_name: provider.As_Str().to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "{manifest_path} declares provider {provider}, which the registry never registers for any capability."
        ),
        locations: vec![manifest_path.to_owned()],
    };
}

/// One finding per registered language provider no checked manifest claims.
fn Unclaimed_Provider_Findings(registered: &BTreeSet<ProviderId>, declared: &BTreeSet<ProviderId>, packages: &[DeclaredPackage<'_>]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for provider in registered
    {
        if Is_Unclaimed_Language_Provider(provider, declared)
        {
            let finding = Unclaimed_Provider_Finding(provider, packages);
            findings.push(finding);
        }
    }

    return findings;
}

/// Whether `provider` is one this check owes an unclaimed finding for.
///
/// The two questions are asked one at a time rather than joined by `&&` at the loop, so the
/// loop reads as one question put to each provider. The prefix half is why this direction is
/// scoped at all -- see this module's own doc: the registry also offers `nomos.repo.*` policy
/// providers, the review connector and the requirement-trace provider, and no
/// `LanguagePackage` manifest was ever meant to claim any of them.
fn Is_Unclaimed_Language_Provider(provider: &ProviderId, declared: &BTreeSet<ProviderId>) -> bool
{
    if !provider.As_Str().starts_with(LANGUAGE_PROVIDER_PREFIX)
    {
        return false;
    }

    return !declared.contains(provider);
}

/// A registered language provider that no manifest among `packages` claims.
fn Unclaimed_Provider_Finding(provider: &ProviderId, packages: &[DeclaredPackage<'_>]) -> Finding
{
    return Finding {
        rule: RuleId::New(PACKAGE_CONFORMANCE),
        subject: Subject_Of_Provider(provider),
        subject_name: provider.As_Str().to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("the registry registers provider {provider}, which no checked manifest declares."),
        locations: packages.iter().map(|package| return package.manifest_path.to_owned()).collect(),
    };
}

/// A provider identity reduced to the subject a finding about it is filed under.
///
/// Mirrors [`Subject_Of_Path`]'s own reasoning (`OD-MODEL-001`) at one remove: two findings
/// naming the same provider must carry the same identity so they are comparable across runs,
/// and a provider identity is not a repository path, so this hashes the identity's own text
/// directly rather than routing it through path normalization that does not apply to it.
fn Subject_Of_Provider(provider: &ProviderId) -> nomos_contracts::SubjectId
{
    return nomos_contracts::SubjectId::From_Digest(Content_Digest(provider.As_Str().as_bytes()));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_capability::{CapabilityContract, ProviderOffer};
    use nomos_contracts::{
        Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    };
    use nomos_package::PackageVersion;
    use std::path::Path;

    #[test]
    fn Test_Check_Package_Conformance_Should_Report_Nothing_When_Declared_And_Registered_Agree()
    {
        let registry = Test_Registry(&["nomos.lang.rust.syn"]);
        let rust = Registration("nomos.lang.rust.syn");
        let packages = [DeclaredPackage { manifest_path: "packages/nomos.lang.rust.json", providers: &[rust] }];

        let findings = Check_Package_Conformance(&packages, &registry);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Package_Conformance_Should_Report_A_Manifest_Provider_The_Registry_Never_Registers()
    {
        let registry = Test_Registry(&["nomos.lang.rust.syn"]);
        let rust = Registration("nomos.lang.rust.syn");
        let phantom = Registration("nomos.lang.rust.phantom");
        let packages = [DeclaredPackage { manifest_path: "packages/nomos.lang.rust.json", providers: &[rust, phantom] }];

        let findings = Check_Package_Conformance(&packages, &registry);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "nomos.lang.rust.phantom");
        assert_eq!(found.locations, vec!["packages/nomos.lang.rust.json".to_owned()]);
    }

    #[test]
    fn Test_Check_Package_Conformance_Should_Report_A_Registered_Language_Provider_No_Manifest_Claims()
    {
        let registry = Test_Registry(&["nomos.lang.rust.syn", "nomos.lang.rust.cargo"]);
        let rust = Registration("nomos.lang.rust.syn");
        let packages = [DeclaredPackage { manifest_path: "packages/nomos.lang.rust.json", providers: &[rust] }];

        let findings = Check_Package_Conformance(&packages, &registry);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.subject_name, "nomos.lang.rust.cargo");
        assert_eq!(found.locations, vec!["packages/nomos.lang.rust.json".to_owned()]);
    }

    /// The scoping this module's own doc names: a registered provider outside the
    /// `nomos.lang.` namespace is never reported as unclaimed, because no `LanguagePackage`
    /// manifest was ever meant to claim it.
    #[test]
    fn Test_Check_Package_Conformance_Should_Not_Report_A_Registered_Provider_Outside_The_Language_Namespace()
    {
        let registry = Test_Registry(&["nomos.lang.rust.syn", "nomos.repo.naming"]);
        let rust = Registration("nomos.lang.rust.syn");
        let packages = [DeclaredPackage { manifest_path: "packages/nomos.lang.rust.json", providers: &[rust] }];

        let findings = Check_Package_Conformance(&packages, &registry);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Package_Conformance_Should_Report_Both_Directions_At_Once()
    {
        let registry = Test_Registry(&["nomos.lang.rust.syn", "nomos.lang.rust.cargo"]);
        let phantom = Registration("nomos.lang.rust.phantom");
        let packages = [DeclaredPackage { manifest_path: "packages/nomos.lang.rust.json", providers: &[phantom] }];

        let findings = Check_Package_Conformance(&packages, &registry);

        // `phantom` is unregistered; `cargo` and `syn` are both registered but this package
        // declares neither -- three findings, not one of each direction, because a package
        // that misses every real provider still owes an unclaimed finding per provider it
        // missed.
        let names: Vec<&str> = findings.iter().map(|finding| return finding.subject_name.as_str()).collect();
        assert_eq!(
            names,
            vec!["nomos.lang.rust.cargo", "nomos.lang.rust.phantom", "nomos.lang.rust.syn"],
            "{findings:?}"
        );
    }

    /// The real caller this check is for: this workspace's own two real, checked-in
    /// manifests, against this crate's own real, composed registry -- not a synthetic
    /// fixture. `OD-PACKAGE-014` measured, at the time it was written, that nothing ever read
    /// these; this is the first real read, and the four names below are the real, current
    /// gap it surfaces -- `nomos.lang.rust.cargo`, `.clippy` and `.deny`, and
    /// `nomos.lang.go.modules`, none of which either manifest lists yet. A future item that
    /// closes that gap by editing `packages/nomos.lang.rust.json` and
    /// `packages/nomos.lang.go.json` will need to shrink this list, not this test's shape.
    #[test]
    fn Test_Check_Package_Conformance_Against_This_Workspaces_Own_Real_Manifests_And_Registry()
    {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let rust_manifest_path = repository_root.join("packages/nomos.lang.rust.json");
        let go_manifest_path = repository_root.join("packages/nomos.lang.go.json");

        let rust = nomos_lang_rust_package::Read_Manifest(&rust_manifest_path).expect("this workspace's own rust manifest parses");
        let go = nomos_lang_go_package::Read_Manifest(&go_manifest_path).expect("this workspace's own go manifest parses");
        let registry = crate::Registered().expect("this crate's own composition must not be self-contradictory");

        let packages = [
            DeclaredPackage { manifest_path: "packages/nomos.lang.rust.json", providers: &rust.providers },
            DeclaredPackage { manifest_path: "packages/nomos.lang.go.json", providers: &go.providers },
        ];

        let findings = Check_Package_Conformance(&packages, &registry);

        let names: Vec<&str> = findings.iter().map(|finding| return finding.subject_name.as_str()).collect();
        assert_eq!(
            names,
            vec!["nomos.lang.go.modules", "nomos.lang.rust.cargo", "nomos.lang.rust.clippy", "nomos.lang.rust.deny"],
            "{findings:?}"
        );
        assert!(findings.iter().all(|finding| return finding.gate == GateCategory::Advisory), "{findings:?}");
    }

    /// A registry declaring one synthetic capability and offering `providers` under it.
    fn Test_Registry(providers: &[&str]) -> Registry
    {
        let capability = CapabilityId::New("nomos.cap.test.language");
        let mut registry = Registry::New();
        Declare_Test_Capability(&mut registry, &capability);
        Offer_Each_Provider(&mut registry, &capability, providers);
        return registry;
    }

    /// Declares the one capability every offer below is made under.
    fn Declare_Test_Capability(registry: &mut Registry, capability: &CapabilityId)
    {
        registry
            .Declare(CapabilityContract {
                id: capability.clone(),
                version: ContractVersion::New(1, 0),
                summary: "a synthetic capability for this test".to_owned(),
                ceiling: Test_Guarantee(),
            })
            .expect("a fresh registry's first declaration cannot conflict");
    }

    /// Offers `providers` the declared capability, one offer each.
    fn Offer_Each_Provider(registry: &mut Registry, capability: &CapabilityId, providers: &[&str])
    {
        for provider in providers
        {
            registry
                .Offer(ProviderOffer {
                    provider: ProviderId::New(*provider),
                    capability: capability.clone(),
                    version: ContractVersion::New(1, 0),
                    guarantee: Test_Guarantee(),
                })
                .expect("one offer per distinct provider cannot conflict");
        }
    }

    /// The guarantee these fixtures state, at the strongest assurance a real provider could.
    fn Test_Guarantee() -> Guarantee
    {
        return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Sound, IncrementalGranularity::WholeWorkspace);
    }

    /// A manifest registration for `provider`, at a version none of these tests reads.
    fn Registration(provider: &str) -> ProviderRegistration
    {
        return ProviderRegistration { provider: ProviderId::New(provider), tool_version: PackageVersion::New(0, 1, 0) };
    }
}
