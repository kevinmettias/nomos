//! This workspace's own rules, as the declarations `OD-RULES-022` decided composition resolves.

use nomos_contracts::{Assurance, ContractVersion, EvidenceClass, FactVariant, Guarantee, IncrementalGranularity, PackageId, PackageKind};
use nomos_rule_package::{
    ApplicabilitySemantics, CapabilityRequirement, Judgment, PackageVersion, ProtocolRange,
    RuleContract, RulePackage,
};
use nomos_rules::{RuleDescriptor, DESCRIPTORS};

/// The package version every declaration below carries.
///
/// One version for the whole set, because these are not independently released packages: they
/// are this workspace's own rules, declared in one place and shipped in one binary. A real
/// per-package version belongs to a package a repository installs, which is what
/// `nomos_rule_package::Read_Manifest` is for and this function deliberately is not.
const DECLARED_AT: PackageVersion = PackageVersion::New(1, 0, 0);

/// The contract version every declaration below states at both ends of its protocol range.
///
/// Both ends, and the same version, because these declarations describe what this build
/// itself speaks: a range wider than one version would claim a compatibility nobody
/// measured, and the two ends would then be two more numbers to keep in step.
const DECLARED_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// Every rule this workspace declares, derived from `nomos_rules::DESCRIPTORS`.
///
/// Derived rather than authored, which is the whole point. `OD-GATE-020` named the missing
/// piece exactly — a rule's contract citation was "knowledge no export carries", so
/// `nomos-gate-orchestration` typed it a second time — and `P52` put that citation on the
/// descriptor. This function is the export that record said would satisfy `OD-GATE-011`'s
/// legitimate-exception test: one authority, named outside both artifacts, that each side
/// derives from instead of restating.
///
/// Every declaration is [`Judgment::Mechanical`], and that is a measurement rather than a
/// default: every rule in `DESCRIPTORS` is a linked function this build composes. The
/// model-judged arm exists for rules whose judgment is a model reading source — the Go
/// predecessor's own population — and this workspace declares none yet.
///
/// `applicability` is [`ApplicabilitySemantics::AlwaysSupported`] for the same kind of reason
/// and with one honest gap: `OD-PACKAGE-008` measured `Check_Unread_Reaches_A_Finding` as
/// structurally partial, and `RuleDescriptor` carries no field for that today. Recording it
/// per rule is a further increment; declaring the shape uniformly here would be a claim this
/// function cannot support, so the doc says so rather than the value implying otherwise.
#[must_use]
pub fn Declared_Rules() -> Vec<RulePackage>
{
    return DESCRIPTORS.iter().map(Declared_Package).collect();
}

/// One descriptor as the declaration a resolution reads.
fn Declared_Package(descriptor: &RuleDescriptor) -> RulePackage
{
    return RulePackage {
        package_id: PackageId::New(format!("nomos.rule.{}", descriptor.id)),
        package_kind: PackageKind::RulePackage,
        package_version: DECLARED_AT,
        protocol_range: ProtocolRange::New(DECLARED_VERSION, DECLARED_VERSION),
        rule_id: descriptor.Rule(),
        contract: Cited_Contract(descriptor),
        judgment: Judgment::Mechanical,
        applicability: ApplicabilitySemantics::AlwaysSupported,
        required_capabilities: Required_Capabilities(descriptor),
        evidence_schema: EvidenceClass::Derived,
        enhanced_implementation: Vec::new(),
        external_diagnostics: Vec::new(),
        correction_and_suppression: None,
        examples: Vec::new(),
        counterexamples: Vec::new(),
        conformance_fixtures: Vec::new(),
        evaluation_corpus: None,
        agent_guidance: Vec::new(),
        title: descriptor.id.to_owned(),
    };
}

/// The contract a descriptor cites, or `None` where the authority carries no version.
///
/// `RuleContract` pairs a record with a version, and a prose authority has none —
/// `NO_VERSIONED_RECORD` is a sentinel, not a version zero, by `OD-GATE-020`'s own words. So a
/// prose citation becomes absence here rather than a contract at version zero, which would be
/// the sentinel leaking into a type that has a better way to say it.
fn Cited_Contract(descriptor: &RuleDescriptor) -> Option<RuleContract>
{
    if !descriptor.Is_Citing_A_Versioned_Record()
    {
        return None;
    }

    return Some(RuleContract::New(descriptor.contract_record.to_owned(), descriptor.contract_record_version));
}

/// The weakest guarantee a declaration can state and still be true.
///
/// `Assurance::Unknown` on both axes is "nobody has established it", which is exactly right
/// here and is not interchangeable with `Unsound` — `nomos-lang-rust-scan`'s own guarantee doc
/// is the worked example of why those two must not be confused. `IncrementalGranularity::None`
/// claims no incremental reuse.
const NO_STATED_FLOOR: Guarantee = Guarantee::New(FactVariant::Syntactic, Assurance::Unknown, Assurance::Unknown, IncrementalGranularity::None);

/// The capabilities a descriptor's rule reads, at the floor a declaration states.
///
/// [`NO_STATED_FLOOR`] deliberately, and the name says why: `RuleDescriptor` names which
/// families a rule needs materialized and says nothing about how strong an answer it will
/// accept. A rule's real floor is its own `Requirement`, stated where the rule is —
/// `nomos_rules::Syntax_Requirement` is that for `Check_Completeness_Mirrors`, and its own doc
/// insists the floor is the rule's and not the composition root's. Inventing a stronger floor
/// here would be a second authority on a question the rule already answers, and stating one
/// this function cannot know would be worse than stating none.
fn Required_Capabilities(descriptor: &RuleDescriptor) -> Vec<CapabilityRequirement>
{
    return descriptor
        .requires
        .iter()
        .map(|family| return CapabilityRequirement::New(family.Capability(), NO_STATED_FLOOR))
        .collect();
}

#[cfg(test)]
mod tests
{
    use super::Declared_Rules;
    use crate::Composed_Rules;
    use nomos_rule_package::Judgment;

    /// Every rule this workspace composes is declared, and nothing else is.
    ///
    /// The derivation's own honesty check: `Declared_Rules` reads `DESCRIPTORS` and
    /// `Composed_Rules` reads the array `Run` executes, so this is the same both-directions
    /// comparison `tests/contract/tests/rule_descriptors.rs` already makes, asserted here
    /// against the declarations rather than against the descriptors they came from.
    #[test]
    fn Test_Every_Composed_Rule_Should_Be_Declared_And_Nothing_Else()
    {
        let mut declared: Vec<String> = Declared_Rules().iter().map(|package| return package.rule_id.As_Str().to_owned()).collect();
        let mut composed: Vec<String> = Composed_Rules().iter().map(|rule| return rule.As_Str().to_owned()).collect();

        declared.sort();
        composed.sort();

        assert!(!declared.is_empty(), "an empty declaration set would make every comparison here pass having read nothing");
        assert_eq!(declared, composed);
    }

    /// The eight rules whose authority is a versioned record carry a contract; the ported
    /// ones carry none, because a prose authority has no version to cite.
    #[test]
    fn Test_Only_A_Versioned_Authority_Should_Become_A_Contract()
    {
        let declared = Declared_Rules();

        let contracted = declared.iter().filter(|package| return package.contract.is_some()).count();
        let uncontracted = declared.len().saturating_sub(contracted);

        assert!(contracted > 0, "seven rules cite a governing record, so none is a derivation defect");
        assert!(uncontracted > 0, "most rules are ports citing prose, so none is a derivation defect");
        for package in declared.iter().filter(|package| return package.contract.is_some())
        {
            let contract = package.contract.as_ref().expect("filtered to the contracted rules above");
            assert_ne!(contract.version, 0, "{}", package.rule_id);
        }
    }

    /// Every declaration this workspace makes is mechanical, which is what makes the
    /// resolution against `Composed_Rules` meaningful rather than vacuous.
    #[test]
    fn Test_Every_Declaration_Should_Be_Mechanical()
    {
        for package in Declared_Rules()
        {
            assert_eq!(package.judgment, Judgment::Mechanical, "{}", package.rule_id);
            assert!(package.judgment.Needs_An_Implementation(), "{}", package.rule_id);
        }
    }
}
