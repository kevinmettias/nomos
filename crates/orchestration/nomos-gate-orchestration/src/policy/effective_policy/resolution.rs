//! Combining layer-labelled contributions into one effective policy, field by field.
//!
//! `OD-POLICY-001` decides the shapes and this module is those shapes built: **override** for
//! a single value, **keyed union** for a set, **refusal** for a same-layer contradiction, a
//! locked override or an unreadable artifact, and **combine as a unit** for a set of fields no
//! layer can state apart.
//!
//! Nothing here reads a clock, a store, a cost or a prior materialization. `OD-ROADMAP-003`'s
//! surviving constraint is that a gate policy is a declared table the reader reads off its
//! source, never a condition; a resolved field is read off the declared contributions
//! themselves and off nothing else, which is the same constraint one level up.
//!
//! **Declaration order decides nothing.** The contributions arrive as a slice and the slice's
//! order is never consulted for precedence: they are ranked by [`ConfigurationLayer`], whose
//! declaration order `OD-POLICY-001` took from the corpus, and two statements of equal rank
//! that disagree are refused rather than settled by which came first.

use nomos_contracts::{ConfigurationLayer, SubjectId};

use super::{
    EffectivePolicy, FieldProvenance, PHASE_POLICY_UNIT, PolicyContribution, PolicyField, PolicyRefusal, PolicyUnit, RejectedOverride,
    ResolvedField,
};
use crate::policy::{
    AdoptionPolicy, BaselineDebt, BaselinePolicy, CoveragePolicy, GatePolicyFile, RuleCalibration, Suppression, SuppressionPolicy,
};
use crate::{GatePhase, PhaseApproval};

/// The artifact a value nobody stated came from.
///
/// `OD-POLICY-001` names it: "the build for a default". A field the ten layers all left alone
/// still has a source, and naming it keeps `Default` a layer a report can print rather than a
/// blank where a provenance should be.
pub(super) const BUILD_DEFAULTS: &str = "the build's own defaults";

/// One field's resolved value together with the provenance of the statement that decided it.
struct Resolution<Value>
{
    value: Value,
    field: ResolvedField,
}

/// The phase policy, resolved whole because `OD-POLICY-001` declares it a unit.
struct PhasePolicy
{
    phases: Vec<GatePhase>,
    approvals: Vec<PhaseApproval>,
    phases_field: ResolvedField,
    approvals_field: ResolvedField,
}

/// The first refusal in `contributions`, if any.
///
/// Checked before anything is combined, and in a fixed order, so that a set carrying two
/// problems reports the same one every time rather than whichever the combining happened to
/// reach first.
pub(super) fn Refusal_In(contributions: &[PolicyContribution]) -> Option<PolicyRefusal>
{
    return Unreadable_In(contributions)
        .or_else(|| return Orphan_Companion_In(contributions))
        .or_else(|| return Contradiction_In(contributions));
}

/// `contributions` combined into one effective policy.
///
/// Every field is resolved by its own shape and each returns both the value a run judges with
/// and the provenance a report reads, so the two cannot be computed from different premises.
pub(super) fn Combined(contributions: &[PolicyContribution]) -> EffectivePolicy
{
    let ordered = Ordered_By_Layer(contributions);
    let suppressions = Resolved_Suppressions(&ordered);
    let baseline = Resolved_Baseline(&ordered);
    let adoption = Resolved_Adoption(&ordered);
    let coverage = Resolved_Coverage(&ordered);
    let phase_policy = Resolved_Phase_Policy(&ordered);

    return EffectivePolicy {
        values: GatePolicyFile {
            suppressions: suppressions.value,
            baseline: baseline.value,
            adoption: adoption.value,
            coverage: coverage.value,
            phases: phase_policy.phases,
            approvals: phase_policy.approvals,
        },
        fields: vec![
            suppressions.field,
            baseline.field,
            adoption.field,
            coverage.field,
            phase_policy.phases_field,
            phase_policy.approvals_field,
        ],
        absent_layers: Absent_Layers(contributions),
    };
}

/// `contributions` ranked lowest layer first.
///
/// A stable sort, so two contributions of one layer keep the order they were handed over in
/// -- which decides nothing, because a same-layer disagreement is already refused.
fn Ordered_By_Layer(contributions: &[PolicyContribution]) -> Vec<&PolicyContribution>
{
    let mut ordered: Vec<&PolicyContribution> = contributions.iter().collect();
    ordered.sort_by_key(|contribution| return contribution.layer);

    return ordered;
}

/// Every layer with no contribution at all, in precedence order.
///
/// `Default` is never absent: the build states every default, which is what [`BUILD_DEFAULTS`]
/// names. The rest are absent exactly when no source spoke for them, and `OD-POLICY-001`
/// requires that to be a line in a report rather than an omission from one.
fn Absent_Layers(contributions: &[PolicyContribution]) -> Vec<ConfigurationLayer>
{
    return ConfigurationLayer::ALL
        .into_iter()
        .filter(|layer| return *layer != ConfigurationLayer::Default)
        .filter(|layer| return !contributions.iter().any(|contribution| return contribution.layer == *layer))
        .collect();
}

/// The first artifact that is present and could not be read.
fn Unreadable_In(contributions: &[PolicyContribution]) -> Option<PolicyRefusal>
{
    let offending = contributions.iter().find(|contribution| return contribution.unreadable.is_some())?;
    let detail = offending.unreadable.clone()?;

    return Some(PolicyRefusal::UnreadableArtifact { layer: offending.layer, artifact: offending.artifact.clone(), detail });
}

/// The first contribution stating a companion field of a unit without that unit's deciding
/// field.
///
/// A property of one contribution and not of the layer set, so it refuses whether or not some
/// other layer would have supplied the deciding field: a configuration must not become valid
/// because a layer appeared, which is `US-CONFIG-002`'s "the resolved policy is reproducible"
/// read at the resolver.
fn Orphan_Companion_In(contributions: &[PolicyContribution]) -> Option<PolicyRefusal>
{
    for contribution in contributions
    {
        if let Some(refusal) = Orphan_Companion_Of(contribution, PHASE_POLICY_UNIT)
        {
            return Some(refusal);
        }
    }

    return None;
}

/// `unit`'s companion that `contribution` states without the deciding field, if it states one.
fn Orphan_Companion_Of(contribution: &PolicyContribution, unit: PolicyUnit) -> Option<PolicyRefusal>
{
    if contribution.States(unit.deciding_field)
    {
        return None;
    }

    let orphan = unit.companion_fields.iter().find(|field| return contribution.States(**field))?;

    return Some(PolicyRefusal::CompanionWithoutADecidingField {
        unit,
        field: *orphan,
        layer: contribution.layer,
        artifact: contribution.artifact.clone(),
    });
}

/// The first field two statements of one layer disagree about.
fn Contradiction_In(contributions: &[PolicyContribution]) -> Option<PolicyRefusal>
{
    let ordered = Ordered_By_Layer(contributions);

    return Whole_Field_Contradiction(&ordered, PolicyField::Coverage, Coverage_Of)
        .or_else(|| return Whole_Field_Contradiction(&ordered, PolicyField::Phases, Phases_Of))
        .or_else(|| return Whole_Field_Contradiction(&ordered, PolicyField::Approvals, Approvals_Of))
        .or_else(|| return Suppression_Contradiction(&ordered))
        .or_else(|| return Baseline_Contradiction(&ordered))
        .or_else(|| return Adoption_Contradiction(&ordered));
}

/// Two statements of one layer that give `field` different whole values.
///
/// For a field whose value is replaced rather than merged, the key is the field itself: there
/// is no finer address at which two sources of one layer could disagree.
fn Whole_Field_Contradiction<Value: PartialEq>(
    ordered: &[&PolicyContribution],
    field: PolicyField,
    value_of: fn(&PolicyContribution) -> Option<&Value>,
) -> Option<PolicyRefusal>
{
    for (index, one) in ordered.iter().enumerate()
    {
        let Some(rest) = ordered.get(index.saturating_add(1)..)
        else
        {
            continue;
        };
        let disagreeing = rest.iter().find(|other| return Disagree_Wholly(one, other, value_of));

        if let Some(other) = disagreeing
        {
            return Some(Whole_Field_Refusal(one, other, field));
        }
    }

    return None;
}

/// Whether two contributions of one layer state `value_of` differently.
fn Disagree_Wholly<Value: PartialEq>(
    one: &PolicyContribution,
    other: &PolicyContribution,
    value_of: fn(&PolicyContribution) -> Option<&Value>,
) -> bool
{
    if one.layer != other.layer
    {
        return false;
    }

    return match (value_of(one), value_of(other))
    {
        (Some(stated), Some(also_stated)) => stated != also_stated,
        _ => false,
    };
}

/// The refusal two whole-field statements of one layer earn.
fn Whole_Field_Refusal(one: &PolicyContribution, other: &PolicyContribution, field: PolicyField) -> PolicyRefusal
{
    return PolicyRefusal::ContradictionWithinALayer {
        layer: one.layer,
        field,
        key: field.Label().to_owned(),
        artifacts: vec![one.artifact.clone(), other.artifact.clone()],
    };
}

/// The coverage floor `contribution` states, if it states one.
fn Coverage_Of(contribution: &PolicyContribution) -> Option<&CoveragePolicy>
{
    return contribution.coverage.as_ref();
}

/// The stages `contribution` states, if it states any.
fn Phases_Of(contribution: &PolicyContribution) -> Option<&Vec<GatePhase>>
{
    return contribution.phases.as_ref();
}

/// The approvals `contribution` states, if it states any.
fn Approvals_Of(contribution: &PolicyContribution) -> Option<&Vec<PhaseApproval>>
{
    return contribution.approvals.as_ref();
}

/// The dispositions `contribution` states, if it states any.
fn Suppressions_Of(contribution: &PolicyContribution) -> Option<&[Suppression]>
{
    return contribution.suppressions.as_ref().map(|policy| return policy.suppressions.as_slice());
}

/// The debt `contribution` states, if it states any.
fn Debt_Of(contribution: &PolicyContribution) -> Option<&[BaselineDebt]>
{
    return contribution.baseline.as_ref().map(|policy| return policy.debt.as_slice());
}

/// The calibrations `contribution` states, if it states any.
fn Calibrations_Of(contribution: &PolicyContribution) -> Option<&[RuleCalibration]>
{
    return contribution.adoption.as_ref().map(|policy| return policy.calibrated.as_slice());
}

/// One entry as stated, with where it came from and the key an author addresses it by.
struct StatedEntry<'a, Entry>
{
    from: &'a PolicyContribution,
    key: String,
    entry: &'a Entry,
}

/// Every entry of one merged field, flattened across `ordered` with its source kept.
fn Stated_Entries<'a, Entry>(
    ordered: &[&'a PolicyContribution],
    entries_of: fn(&'a PolicyContribution) -> Option<&'a [Entry]>,
    key_of: fn(&Entry) -> String,
) -> Vec<StatedEntry<'a, Entry>>
{
    let mut stated: Vec<StatedEntry<'a, Entry>> = Vec::new();
    for contribution in ordered
    {
        let Some(entries) = entries_of(contribution)
        else
        {
            continue;
        };

        stated.extend(entries.iter().map(|entry| return StatedEntry { from: contribution, key: key_of(entry), entry }));
    }

    return stated;
}

/// The first key two entries of one layer address with different values.
///
/// `MODEL-ROUTE-023`'s rule applied per field: equal-authority statements assigning
/// incompatible values prevent resolution, and declaration order is one of the ways of
/// settling it the corpus names as forbidden.
fn Keyed_Contradiction<Entry: PartialEq>(stated: &[StatedEntry<'_, Entry>], field: PolicyField) -> Option<PolicyRefusal>
{
    for (index, one) in stated.iter().enumerate()
    {
        let Some(rest) = stated.get(index.saturating_add(1)..)
        else
        {
            continue;
        };
        let disagreeing = rest.iter().find(|other| return Disagree_On_A_Key(one, other));

        if let Some(other) = disagreeing
        {
            return Some(Keyed_Refusal(one, other, field));
        }
    }

    return None;
}

/// Whether two stated entries of one layer address one key with different values.
fn Disagree_On_A_Key<Entry: PartialEq>(one: &StatedEntry<'_, Entry>, other: &StatedEntry<'_, Entry>) -> bool
{
    return one.from.layer == other.from.layer && one.key == other.key && one.entry != other.entry;
}

/// The refusal two entries of one layer addressing one key earn.
fn Keyed_Refusal<Entry>(one: &StatedEntry<'_, Entry>, other: &StatedEntry<'_, Entry>, field: PolicyField) -> PolicyRefusal
{
    return PolicyRefusal::ContradictionWithinALayer {
        layer: one.from.layer,
        field,
        key: one.key.clone(),
        artifacts: vec![one.from.artifact.clone(), other.from.artifact.clone()],
    };
}

/// How an author addresses one disposition: the rule, and the path they wrote it for.
fn Suppression_Key(entry: &Suppression) -> String
{
    return format!("the suppression for rule '{}' on subject {}", entry.rule.As_Str(), Subject_Label(entry.subject));
}

/// How an author addresses one baseline entry.
fn Debt_Key(entry: &BaselineDebt) -> String
{
    let path = entry.declared_path.clone().unwrap_or_else(|| return Subject_Label(entry.subject));

    return format!("the baseline entry for rule '{}' on '{path}'", entry.rule.As_Str());
}

/// How an author addresses one calibration: `RuleCalibration` matches by rule alone.
fn Calibration_Key(entry: &RuleCalibration) -> String
{
    return format!("the calibration for rule '{}'", entry.rule.As_Str());
}

/// A subject as a refusal can name it.
///
/// The digest rather than a path, because a `Suppression` keeps no spelling of the path it was
/// authored from -- `BaselineDebt::declared_path` is the one entry kind that does, and
/// [`Debt_Key`] prefers it for exactly that reason.
fn Subject_Label(subject: SubjectId) -> String
{
    return format!("{}", subject.Digest());
}

/// Suppressions from the first dispute in `ordered`, if the layer's own sources disagree.
fn Suppression_Contradiction(ordered: &[&PolicyContribution]) -> Option<PolicyRefusal>
{
    let stated = Stated_Entries(ordered, Suppressions_Of, Suppression_Key);

    return Keyed_Contradiction(&stated, PolicyField::Suppressions);
}

/// Baseline debt from the first dispute in `ordered`, if the layer's own sources disagree.
fn Baseline_Contradiction(ordered: &[&PolicyContribution]) -> Option<PolicyRefusal>
{
    let stated = Stated_Entries(ordered, Debt_Of, Debt_Key);

    return Keyed_Contradiction(&stated, PolicyField::Baseline);
}

/// Calibrations from the first dispute in `ordered`, if the layer's own sources disagree.
fn Adoption_Contradiction(ordered: &[&PolicyContribution]) -> Option<PolicyRefusal>
{
    let stated = Stated_Entries(ordered, Calibrations_Of, Calibration_Key);

    return Keyed_Contradiction(&stated, PolicyField::Adoption);
}

/// The union of one merged field across `ordered`, with the contributions it superseded.
///
/// The same key stated at two layers takes the higher layer's entry and records the lower one
/// as overridden, and no layer removes a lower layer's entry: `OD-POLICY-001` leaves a
/// retraction spelling undecided, so today a higher layer can only supersede.
///
/// The surviving entries keep the order they were first declared in, which is load-bearing
/// rather than incidental: `SuppressionPolicy::Suppressing` and `BaselinePolicy::Tolerating`
/// both take the first entry that applies, and `GateRunProvenance::policy` hashes the order
/// for that reason.
fn Merged_Entries<Entry: Clone + PartialEq>(stated: &[StatedEntry<'_, Entry>]) -> (Vec<Entry>, Vec<FieldProvenance>)
{
    let mut merge = Merge::Empty();

    for one in stated
    {
        merge.Placed(one);
    }

    return (merge.entries, merge.overrode);
}

/// The union being built for one merged field.
///
/// A value of its own rather than four locals threaded through two helpers, because those
/// helpers took five parameters between them and this crate caps a function at four -- and the
/// four move together in any case: an entry, where it came from, the key it is addressed by,
/// and who it displaced are one fact about one slot.
struct Merge<'a, Entry>
{
    /// The surviving entries, in the order their keys were first declared.
    entries: Vec<Entry>,
    /// The contribution each surviving entry came from, parallel to `entries`.
    sources: Vec<&'a PolicyContribution>,
    /// The key each surviving entry is addressed by, parallel to `entries`.
    keys: Vec<String>,
    /// Every contribution some entry of which a higher layer superseded.
    overrode: Vec<FieldProvenance>,
}

impl<'a, Entry: Clone + PartialEq> Merge<'a, Entry>
{
    /// A union with nothing in it yet.
    fn Empty() -> Self
    {
        return Self { entries: Vec::new(), sources: Vec::new(), keys: Vec::new(), overrode: Vec::new() };
    }

    /// `one` placed into the union: appended when its key is new, and superseding the entry
    /// already held under that key otherwise.
    fn Placed(&mut self, one: &StatedEntry<'a, Entry>)
    {
        let held = self.keys.iter().position(|key| return *key == one.key);

        match held
        {
            None => self.Appended(one),
            Some(index) => self.Superseded(one, index),
        }
    }

    /// A key this field has not seen yet, kept in declaration order.
    fn Appended(&mut self, one: &StatedEntry<'a, Entry>)
    {
        self.entries.push(one.entry.clone());
        self.sources.push(one.from);
        self.keys.push(one.key.clone());
    }

    /// A key a lower layer already stated, replaced and recorded as overridden.
    fn Superseded(&mut self, one: &StatedEntry<'a, Entry>, index: usize)
    {
        let displaced = self.sources.get(index).copied();

        if let Some(slot) = self.entries.get_mut(index)
        {
            *slot = one.entry.clone();
        }
        if let Some(slot) = self.sources.get_mut(index)
        {
            *slot = one.from;
        }
        if let Some(previous) = displaced
        {
            Noted_As_Overridden(previous, one.from, &mut self.overrode);
        }
    }
}

/// `previous` recorded once as a contribution `winner` superseded.
///
/// A contribution that superseded its own earlier entry is not recorded: a source restating a
/// key it already stated has overridden nobody, and reporting it as having outranked itself
/// would put a line in a report that names no disagreement.
fn Noted_As_Overridden(previous: &PolicyContribution, winner: &PolicyContribution, overrode: &mut Vec<FieldProvenance>)
{
    if previous.layer == winner.layer && previous.artifact == winner.artifact
    {
        return;
    }

    let provenance = Provenance_Of(previous, None);

    if !overrode.contains(&provenance)
    {
        overrode.push(provenance);
    }
}

/// The provenance of `contribution`'s statement, carrying `unit` when a unit decided it.
fn Provenance_Of(contribution: &PolicyContribution, unit: Option<PolicyUnit>) -> FieldProvenance
{
    return FieldProvenance { layer: contribution.layer, artifact: contribution.artifact.clone(), decided_by_unit: unit };
}

/// The provenance of a value nobody stated.
fn Default_Provenance() -> FieldProvenance
{
    return FieldProvenance { layer: ConfigurationLayer::Default, artifact: BUILD_DEFAULTS.to_owned(), decided_by_unit: None };
}

/// Every contribution that states `field` and is not locked out of stating it.
fn Admitted<'a>(ordered: &[&'a PolicyContribution], field: PolicyField) -> Vec<&'a PolicyContribution>
{
    return ordered
        .iter()
        .copied()
        .filter(|contribution| return contribution.States(field))
        .filter(|contribution| return Locking(ordered, field, contribution).is_none())
        .collect();
}

/// The higher-layer contribution that locks `field` against `offered_by`, if one does.
fn Locking<'a>(ordered: &[&'a PolicyContribution], field: PolicyField, offered_by: &PolicyContribution) -> Option<&'a PolicyContribution>
{
    return ordered
        .iter()
        .copied()
        .find(|contribution| return contribution.locks.contains(&field) && contribution.layer > offered_by.layer);
}

/// Every statement of `field` a lock refused, kept visible with its reason.
fn Rejections_For(ordered: &[&PolicyContribution], field: PolicyField) -> Vec<RejectedOverride>
{
    let mut rejected: Vec<RejectedOverride> = Vec::new();
    for contribution in ordered
    {
        let Some(locking) = Locking(ordered, field, contribution).filter(|_| return contribution.States(field))
        else
        {
            continue;
        };

        rejected.push(Rejection(contribution, locking, field));
    }

    return rejected;
}

/// One refused statement as the record `CONFIG-003` requires to stay visible.
fn Rejection(offered_by: &PolicyContribution, locked_by: &PolicyContribution, field: PolicyField) -> RejectedOverride
{
    let offered = Provenance_Of(offered_by, None);
    let refusal = PolicyRefusal::LockedOverride { field, locked_by: Provenance_Of(locked_by, None), offered_by: offered.clone() };

    return RejectedOverride { offered_by: offered, reason: refusal.Sentence() };
}

/// The coverage floor the highest layer that states one decided (`OD-POLICY-001`: override).
fn Resolved_Coverage(ordered: &[&PolicyContribution]) -> Resolution<CoveragePolicy>
{
    let admitted = Admitted(ordered, PolicyField::Coverage);
    let deciding = admitted.last().copied();
    let value = deciding.and_then(|contribution| return contribution.coverage).unwrap_or_default();

    return Resolution { value, field: Overridden_Field(&admitted, PolicyField::Coverage, ordered) };
}

/// The resolved field for a value the highest stating layer replaced whole.
fn Overridden_Field(admitted: &[&PolicyContribution], field: PolicyField, ordered: &[&PolicyContribution]) -> ResolvedField
{
    let decided_by = admitted.last().map_or_else(Default_Provenance, |contribution| return Provenance_Of(contribution, None));
    let outranked = admitted.split_last().map_or(&[][..], |(_, rest)| return rest);

    return ResolvedField {
        field,
        decided_by,
        overrode: outranked.iter().map(|contribution| return Provenance_Of(contribution, None)).collect(),
        rejected: Rejections_For(ordered, field),
    };
}

/// The dispositions every layer between them declared (`OD-POLICY-001`: keyed union).
fn Resolved_Suppressions(ordered: &[&PolicyContribution]) -> Resolution<SuppressionPolicy>
{
    let admitted = Admitted(ordered, PolicyField::Suppressions);
    let stated = Stated_Entries(&admitted, Suppressions_Of, Suppression_Key);
    let (suppressions, overrode) = Merged_Entries(&stated);

    return Resolution {
        value: SuppressionPolicy { suppressions },
        field: Merged_Field(&admitted, PolicyField::Suppressions, overrode, ordered),
    };
}

/// The debt every layer between them declared (`OD-POLICY-001`: keyed union).
fn Resolved_Baseline(ordered: &[&PolicyContribution]) -> Resolution<BaselinePolicy>
{
    let admitted = Admitted(ordered, PolicyField::Baseline);
    let stated = Stated_Entries(&admitted, Debt_Of, Debt_Key);
    let (debt, overrode) = Merged_Entries(&stated);

    return Resolution { value: BaselinePolicy { debt }, field: Merged_Field(&admitted, PolicyField::Baseline, overrode, ordered) };
}

/// The calibrations every layer between them declared (`OD-POLICY-001`: keyed union).
fn Resolved_Adoption(ordered: &[&PolicyContribution]) -> Resolution<AdoptionPolicy>
{
    let admitted = Admitted(ordered, PolicyField::Adoption);
    let stated = Stated_Entries(&admitted, Calibrations_Of, Calibration_Key);
    let (calibrated, overrode) = Merged_Entries(&stated);

    return Resolution { value: AdoptionPolicy { calibrated }, field: Merged_Field(&admitted, PolicyField::Adoption, overrode, ordered) };
}

/// The resolved field for a union, whose deciding statement is the highest layer that stated
/// the field at all.
fn Merged_Field(
    admitted: &[&PolicyContribution],
    field: PolicyField,
    overrode: Vec<FieldProvenance>,
    ordered: &[&PolicyContribution],
) -> ResolvedField
{
    let decided_by = admitted.last().map_or_else(Default_Provenance, |contribution| return Provenance_Of(contribution, None));

    return ResolvedField { field, decided_by, overrode, rejected: Rejections_For(ordered, field) };
}

/// The phases and the approvals one source declared (`OD-POLICY-001`: combine as a unit).
///
/// The highest layer that states `phases` decides both, and no other layer contributes to
/// either. `approvals` is a list and is replaced rather than unioned, because that union is
/// exactly the cross-layer pairing the coupling exists to prevent: an approval names the phase
/// it covers, so a caller's phases paired with a file's approvals would let an approval address
/// a stage its own source never declared.
///
/// Inside the unit the deciding field's statement carries the companion's emptiness, which is
/// the one place "a sentinel is not a statement" reads differently: a source stating `phases`
/// states its own approvals too, including none, so a lower layer's approvals do not fill the
/// gap.
fn Resolved_Phase_Policy(ordered: &[&PolicyContribution]) -> PhasePolicy
{
    let admitted = Admitted(ordered, PHASE_POLICY_UNIT.deciding_field);
    let deciding = admitted.last().copied();
    let outranked = Outranked_Within_The_Unit(ordered, deciding);

    return PhasePolicy {
        phases: deciding.and_then(|contribution| return contribution.phases.clone()).unwrap_or_default(),
        approvals: deciding.and_then(|contribution| return contribution.approvals.clone()).unwrap_or_default(),
        phases_field: Unit_Field(PolicyField::Phases, deciding, &outranked, ordered),
        approvals_field: Unit_Field(PolicyField::Approvals, deciding, &outranked, ordered),
    };
}

/// Every contribution to any field of the unit that the deciding statement outranked.
///
/// Recorded for *every* field of the unit rather than for the deciding one alone, so a reader
/// of `approvals` sees that a file's approvals lost to a caller's stages although only
/// `phases` was compared.
fn Outranked_Within_The_Unit<'a>(ordered: &[&'a PolicyContribution], deciding: Option<&PolicyContribution>) -> Vec<&'a PolicyContribution>
{
    return ordered
        .iter()
        .copied()
        .filter(|contribution| return PHASE_POLICY_UNIT.companion_fields.iter().any(|field| return contribution.States(*field))
            || contribution.States(PHASE_POLICY_UNIT.deciding_field))
        .filter(|contribution| return !Is_The_Deciding_Statement(contribution, deciding))
        .collect();
}

/// Whether `contribution` is the statement that decided the unit.
fn Is_The_Deciding_Statement(contribution: &PolicyContribution, deciding: Option<&PolicyContribution>) -> bool
{
    return deciding.is_some_and(|winner| return winner.layer == contribution.layer && winner.artifact == contribution.artifact);
}

/// The resolved field for one field of a unit, carrying the deciding field's provenance.
fn Unit_Field(
    field: PolicyField,
    deciding: Option<&PolicyContribution>,
    outranked: &[&PolicyContribution],
    ordered: &[&PolicyContribution],
) -> ResolvedField
{
    let decided_by =
        deciding.map_or_else(Default_Provenance, |contribution| return Provenance_Of(contribution, Some(PHASE_POLICY_UNIT)));

    return ResolvedField {
        field,
        decided_by,
        overrode: outranked.iter().map(|contribution| return Provenance_Of(contribution, Some(PHASE_POLICY_UNIT))).collect(),
        rejected: Rejections_For(ordered, field),
    };
}
