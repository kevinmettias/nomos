//! Matching what a repository declares against what this build can actually run.

use crate::RuleResolutionError;
use nomos_contracts::RuleId;
use nomos_rule_package::RulePackage;
use std::collections::BTreeSet;

/// What a declared rule set resolved to against the implementations this build registers.
///
/// The model-judged half is carried rather than dropped, which is the whole reason this is a
/// value and not a bare `Vec<RuleId>`. A caller that only wanted the runnable set could
/// filter, but then nothing downstream could tell a gate that declares forty rules and runs
/// twenty from one that declares twenty — and `OD-RULES-022`'s point is that a rule nothing
/// mechanically judges is a truthful declaration rather than an absence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleResolution
{
    /// Declared mechanical and registered: the rules a deterministic run judges by. Sorted.
    pub mechanical: Vec<RuleId>,
    /// Declared model-judged: real, citable, projectable, and contributing no findings to a
    /// deterministic run. Sorted.
    pub model_judged: Vec<RuleId>,
}

/// Matches `declared` against `registered`, or refuses and says which side is wanting.
///
/// This is the step `OD-RULES-022` decided on: "Rule composition resolves a declared rule
/// package against a linked implementation... composition's job is to match them and to
/// refuse when it cannot." `registered` is the set of rule identities this build has code
/// for — [`crate::Composed_Rules`] is the one this workspace's own run supplies — and
/// `declared` is what a repository states it wants judged.
///
/// A manifest cannot conjure a function, so this never constructs an implementation for a
/// declaration it cannot match. It reports the mismatch and lets a composition root decide
/// what to do about it, which is the same division `nomos_capability::Registry` already draws
/// between ranking offers and spending a selection.
///
/// # Errors
///
/// [`RuleResolutionError::NothingDeclared`] when `declared` is empty, and
/// [`RuleResolutionError::Disagreed`] when any declaration or registration is unmatched. Both
/// are defects in a composition rather than conditions a caller can retry.
pub fn Resolve_Rules(declared: &[RulePackage], registered: &[RuleId]) -> Result<RuleResolution, RuleResolutionError>
{
    if declared.is_empty()
    {
        return Err(RuleResolutionError::NothingDeclared);
    }

    let available: BTreeSet<&RuleId> = registered.iter().collect();
    let placements = Place_Declarations(declared, &available);
    let undeclared = Undeclared_Rules(declared, available);

    if Is_Disagreed(&placements, &undeclared)
    {
        return Err(RuleResolutionError::Disagreed {
            unimplemented: placements.unimplemented.into_iter().collect(),
            undeclared: undeclared.into_iter().collect(),
            contradicted: placements.contradicted.into_iter().collect(),
        });
    }

    return Ok(RuleResolution {
        mechanical: placements.mechanical.into_iter().collect(),
        model_judged: placements.model_judged.into_iter().collect(),
    });
}

/// Every declaration sorted into the placement its own judgment and registration give it.
///
/// The four sets are built here and handed back together, so `Resolve_Rules` reads as the
/// three steps it is -- classify, compare, answer -- rather than as the loop that does the
/// classifying.
fn Place_Declarations(declared: &[RulePackage], available: &BTreeSet<&RuleId>) -> Placements
{
    let mut placements = Placements::default();

    for package in declared
    {
        let implementation = Implementation::Of(&package.rule_id, available);
        let rule = package.rule_id.clone();

        match Placement_Of(package, implementation)
        {
            Placement::Mechanical => placements.mechanical.insert(rule),
            Placement::ModelJudged => placements.model_judged.insert(rule),
            Placement::Unimplemented => placements.unimplemented.insert(rule),
            Placement::Contradicted => placements.contradicted.insert(rule),
        };
    }

    return placements;
}

/// The four sets one declaration can be sorted into, named rather than passed around
/// separately.
#[derive(Default)]
struct Placements
{
    /// Declared mechanical, and implemented.
    mechanical: BTreeSet<RuleId>,
    /// Declared model-judged, and not implemented.
    model_judged: BTreeSet<RuleId>,
    /// Declared mechanical, and not implemented.
    unimplemented: BTreeSet<RuleId>,
    /// Declared model-judged, and implemented anyway.
    contradicted: BTreeSet<RuleId>,
}

/// Where one declaration lands, given its own judgment and whether its rule is implemented.
///
/// Private, and returned rather than written into four borrowed sets: a classification that
/// hands back an answer can be read on its own, and the caller keeps the only mutable state
/// in one place.
enum Placement
{
    /// Declared mechanical, and implemented.
    Mechanical,
    /// Declared model-judged, and not implemented.
    ModelJudged,
    /// Declared mechanical, and not implemented.
    Unimplemented,
    /// Declared model-judged, and implemented anyway.
    Contradicted,
}

/// Whether this build has an implementation registered for a declared rule.
///
/// A named two-state type rather than a `bool`, because a position is not a name: the call
/// site reads which of the two states it hands over, and no combination of arguments can
/// reach a fifth placement by accident.
enum Implementation
{
    /// This build has code for the rule.
    Registered,
    /// This build has none.
    Missing,
}

impl Implementation
{
    /// Which state `available` puts `rule` in.
    fn Of(rule: &RuleId, available: &BTreeSet<&RuleId>) -> Self
    {
        if available.contains(rule)
        {
            return Self::Registered;
        }

        return Self::Missing;
    }
}

/// Which placement one declaration takes.
///
/// The four outcomes are every combination of the two questions this step asks — does the
/// declaration need an implementation, and is one registered — so there is no fifth case and
/// no default arm.
fn Placement_Of(package: &RulePackage, implementation: Implementation) -> Placement
{
    return match (package.judgment.Needs_An_Implementation(), implementation)
    {
        (true, Implementation::Registered) => Placement::Mechanical,
        (true, Implementation::Missing) => Placement::Unimplemented,
        (false, Implementation::Missing) => Placement::ModelJudged,
        (false, Implementation::Registered) => Placement::Contradicted,
    };
}

/// Every registered rule no declaration claims.
fn Undeclared_Rules(declared: &[RulePackage], available: BTreeSet<&RuleId>) -> BTreeSet<RuleId>
{
    let claimed: BTreeSet<&RuleId> = declared.iter().map(|package| return &package.rule_id).collect();

    return available.into_iter().filter(|rule| return !claimed.contains(rule)).cloned().collect();
}

/// Whether the three disagreements this resolution refuses over stand.
///
/// The three questions are asked one at a time rather than joined by `||`, so the condition
/// at the branch reads as what it is for and each question can be read on its own.
fn Is_Disagreed(placements: &Placements, undeclared: &BTreeSet<RuleId>) -> bool
{
    if !placements.unimplemented.is_empty()
    {
        return true;
    }

    if !undeclared.is_empty()
    {
        return true;
    }

    return !placements.contradicted.is_empty();
}

#[cfg(test)]
mod tests
{
    use super::{Resolve_Rules, RuleResolution};
    use crate::{Composed_Rules, RuleResolutionError};
    use nomos_contracts::{ContractVersion, EvidenceClass, PackageId, PackageKind, RuleId};
    use nomos_rule_package::{
        ApplicabilitySemantics, Judgment, PackageVersion, ProtocolRange, RulePackage,
    };

    /// The contract version every declaration in this module states at both ends of its
    /// protocol range.
    ///
    /// One named version rather than a literal pair, for the reason the production
    /// declarations give: a range is a claim about compatibility, and stating it twice
    /// invites the two ends to drift apart.
    const DECLARED_VERSION: ContractVersion = ContractVersion::New(1, 0);

    /// Every rule this workspace's own run composes, declared mechanical, resolves.
    ///
    /// The real case, over real declarations: `Composed_Rules` is what `Run` actually judges
    /// by, not a fixture, and the assertion below that it is non-empty is what stops this
    /// from passing having resolved nothing. This is the shape a repository's own rule
    /// packages would take once they exist.
    #[test]
    fn Test_Declaring_Every_Composed_Rule_Mechanically_Should_Resolve()
    {
        let composed = Composed_Rules();
        assert!(!composed.is_empty(), "this workspace composes rules, so an empty set here is a defect in the fixture");

        let declared: Vec<RulePackage> = composed
            .iter()
            .map(|rule| return Rule_Declaration(rule.As_Str(), Judgment::Mechanical))
            .collect();

        let resolved = Resolve_Rules(&declared, &composed).expect("every declaration has an implementation");

        let mut expected = composed;
        expected.sort();
        assert_eq!(resolved, RuleResolution { mechanical: expected, model_judged: Vec::new() });
    }

    /// A declaration for `rule`, carrying `judgment` and nothing else worth varying here.
    ///
    /// Every other field is at the shape `OD-PACKAGE-008`'s four-rule measurement found
    /// convergent, because this function resolves identity and judgment and reads none of
    /// them.
    fn Rule_Declaration(rule: &str, judgment: Judgment) -> RulePackage
    {
        return RulePackage {
            package_id: PackageId::New(format!("nomos.rule.{rule}")),
            package_kind: PackageKind::RulePackage,
            package_version: PackageVersion::New(1, 0, 0),
            protocol_range: ProtocolRange::New(DECLARED_VERSION, DECLARED_VERSION),
            rule_id: RuleId::New(rule),
            contract: None,
            judgment,
            applicability: ApplicabilitySemantics::AlwaysSupported,
            required_capabilities: Vec::new(),
            evidence_schema: EvidenceClass::Derived,
            enhanced_implementation: Vec::new(),
            external_diagnostics: Vec::new(),
            correction_and_suppression: None,
            examples: Vec::new(),
            counterexamples: Vec::new(),
            conformance_fixtures: Vec::new(),
            evaluation_corpus: None,
            agent_guidance: Vec::new(),
            title: format!("the {rule} rule"),
        };
    }

    /// A model-judged declaration resolves without an implementation and is reported apart
    /// from the rules a run judges by.
    #[test]
    fn Test_A_Model_Judged_Declaration_Should_Resolve_With_No_Implementation()
    {
        let declared = vec![Rule_Declaration("a-model-decides-this", Judgment::ModelJudged)];

        let resolved = Resolve_Rules(&declared, &[]).expect("a model-judged rule needs no implementation");

        assert!(resolved.mechanical.is_empty(), "{resolved:?}");
        assert_eq!(resolved.model_judged, vec![RuleId::New("a-model-decides-this")]);
    }

    /// A mechanical declaration with no implementation is refused: the gate would report a
    /// rule it cannot perform.
    #[test]
    fn Test_A_Mechanical_Declaration_With_No_Implementation_Should_Be_Refused()
    {
        let declared = vec![Rule_Declaration("nothing-implements-this", Judgment::Mechanical)];

        let refusal = Resolve_Rules(&declared, &[]).expect_err("no implementation is registered");

        assert_eq!(
            refusal,
            RuleResolutionError::Disagreed {
                unimplemented: vec![RuleId::New("nothing-implements-this")],
                undeclared: Vec::new(),
                contradicted: Vec::new(),
            }
        );
    }

    /// An implementation nothing declares is refused: this is the shape of the rules that
    /// are exported, tested and judge nothing while reading as though they are in force.
    #[test]
    fn Test_An_Implementation_Nothing_Declares_Should_Be_Refused()
    {
        let declared = vec![Rule_Declaration("declared-and-implemented", Judgment::Mechanical)];
        let registered = vec![RuleId::New("declared-and-implemented"), RuleId::New("nobody-declared-this")];

        let refusal = Resolve_Rules(&declared, &registered).expect_err("one implementation is undeclared");

        assert_eq!(
            refusal,
            RuleResolutionError::Disagreed {
                unimplemented: Vec::new(),
                undeclared: vec![RuleId::New("nobody-declared-this")],
                contradicted: Vec::new(),
            }
        );
    }

    /// A model-judged declaration whose rule is implemented anyway is refused: a real
    /// judgment would sit unrun behind a declaration saying none exists.
    #[test]
    fn Test_A_Model_Judged_Declaration_With_An_Implementation_Should_Be_Refused()
    {
        let declared = vec![Rule_Declaration("said-model-but-code-exists", Judgment::ModelJudged)];
        let registered = vec![RuleId::New("said-model-but-code-exists")];

        let refusal = Resolve_Rules(&declared, &registered).expect_err("the declaration contradicts the registration");

        assert_eq!(
            refusal,
            RuleResolutionError::Disagreed {
                unimplemented: Vec::new(),
                undeclared: Vec::new(),
                contradicted: vec![RuleId::New("said-model-but-code-exists")],
            }
        );
    }

    /// All three disagreements are reported together, so an author fixing a moved rule set
    /// meets the whole difference once rather than one third of it three times.
    #[test]
    fn Test_Every_Disagreement_Should_Be_Reported_Together()
    {
        let declared = vec![
            Rule_Declaration("needs-code-that-is-missing", Judgment::Mechanical),
            Rule_Declaration("says-model-but-code-exists", Judgment::ModelJudged),
        ];
        let registered = vec![RuleId::New("says-model-but-code-exists"), RuleId::New("nobody-declared-this")];

        let refusal = Resolve_Rules(&declared, &registered).expect_err("three disagreements stand");

        assert_eq!(
            refusal,
            RuleResolutionError::Disagreed {
                unimplemented: vec![RuleId::New("needs-code-that-is-missing")],
                undeclared: vec![RuleId::New("nobody-declared-this")],
                contradicted: vec![RuleId::New("says-model-but-code-exists")],
            }
        );
    }

    /// Declaring nothing is refused rather than resolved to an empty set.
    ///
    /// `OD-GATE-001`'s own distinction: a run that judged nothing must not report as a run
    /// that judged everything and found nothing.
    #[test]
    fn Test_Declaring_Nothing_Should_Be_Refused_Rather_Than_Resolved_Empty()
    {
        let refusal = Resolve_Rules(&[], &[]).expect_err("an empty declaration set is not a clean one");

        assert_eq!(refusal, RuleResolutionError::NothingDeclared);
    }
}
