//! The effective policy: what each field resolved to, and which layer's artifact decided it.
//!
//! `OD-POLICY-001` decides that Nomos policy resolves across ten
//! [`nomos_contracts::ConfigurationLayer`]s, that a field combines by its shape, and that the
//! effective policy carries **a provenance per field rather than a merged blob** -- because a
//! repository that cannot be told which layer decided a field cannot be told why its gate
//! judged as it did. This module is that decision built.
//!
//! # What this replaced
//!
//! One precedence rule existed in this workspace before it: `GatePolicyFile::Resolved_Over`,
//! two levels deep, filling in the fields a `GateCommand` left at its default from
//! `nomos-gate.json`. It was called from two places -- `crate::gate_environment` and
//! `crate::finding_query` -- each with its own handling of the unreadable case, and nothing
//! recorded which of the two sides decided a field. The rule is now one function, both call
//! sites reach it, and the two-level case is the `CommandLine`-over-`Repository` case of a
//! resolution that takes any number of layers.
//!
//! The values that case produces are unchanged, which is the point of keeping
//! `crate::policy::gate_policy_file`'s own precedence tests: they assert what `Resolved_Over`
//! asserted, through the resolver, so a change in what a run judges under would fail them.
//!
//! # Which layer a `GateCommand` speaks for
//!
//! `OD-POLICY-001` measured that "the caller" is three layers wearing one coat: `nomos-cli`'s
//! gate parsing, a workflow step's own `GateBody.command`, and a policy built in code through
//! `nomos-api` or a test. A composition root that constructs a `GateCommand` should say which
//! layer it is speaking for, and none does yet -- wiring that is the host item this increment
//! deliberately does not reach. Until then the command contributes at
//! [`nomos_contracts::ConfigurationLayer::CommandLine`], which is the case
//! `P124-POLICY-001-EFFECTIVE-POLICY-FIRST-INCREMENT-2` names, and the layer is a value the
//! contribution carries rather than a fact this module hard-codes into the resolution: every
//! other layer resolves through the same [`Effective_Gate_Policy`] with no arm of its own.
//!
//! # What a sentinel means here
//!
//! A [`PolicyContribution`] states a field or does not, and `OD-POLICY-001` is explicit that a
//! sentinel is not a statement. The two builders below apply that rule to the two sources this
//! workspace has: an empty list and `CoveragePolicy::Unset` are absences, which is exactly what
//! `Preferred_Policy` and `Resolved_Over` already treated them as. `OD-GATE-029` measured that
//! `nomos-gate.json` cannot tell `"coverage": "unset"` from an omitted key and decided
//! `AllowPartial` as the spelling that would; that variant is not in the tree, `policy/
//! coverage_policy.rs` carries two, and building it is not this increment's.

mod field_provenance;
mod policy_contribution;
mod policy_field;
mod policy_refusal;
mod policy_unit;
mod rejected_override;
mod resolution;
mod resolved_field;

#[cfg(test)]
mod tests;

pub use field_provenance::FieldProvenance;
pub use policy_contribution::PolicyContribution;
pub use policy_field::PolicyField;
pub use policy_refusal::PolicyRefusal;
pub use policy_unit::{PHASE_POLICY_UNIT, PolicyUnit};
pub use rejected_override::RejectedOverride;
pub use resolved_field::ResolvedField;

use nomos_contracts::ConfigurationLayer;

use super::GatePolicyFile;
use crate::GateCommand;

/// The artifact a policy a caller built in code is reported under.
///
/// One name for every such caller today, and `OD-POLICY-001` says why that is temporary: a
/// composition root is meant to name itself, and "the caller" is what a run currently records
/// because the run does not hold the difference.
const CALLER_BUILT_POLICY: &str = "the policy the caller built";

/// What every field of the gate policy resolved to, and what decided it.
///
/// `CONFIG-001`, the `EffectiveConfiguration` glossary entry and `MODEL-ROUTE-005`'s second
/// sentence describe one shape, and it is the shape a report needs to name the layer that
/// decided each field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectivePolicy
{
    /// One entry per field of the gate policy, in the order a run checks them.
    pub fields: Vec<ResolvedField>,
    /// Every layer that had no source at all, in precedence order.
    ///
    /// `OD-POLICY-001`: "a layer with no source contributes nothing and is reported as absent,
    /// never as an empty contribution". "Organization: no source on this host" and
    /// "Organization: declared nothing" are different facts, and only the first is this list.
    ///
    /// The record's third state -- declared-unobservable, a layer this host could never have a
    /// source for -- is not here, because it is a fact about the host rather than about a
    /// resolution: `OD-POLICY-001`'s own table is where it is written, and rendering it beside
    /// these is the host item that renders the provenance.
    pub absent_layers: Vec<ConfigurationLayer>,
    /// The values a run judges with.
    ///
    /// Crate-private because [`GatePolicyFile`] is: the policy types a run judges against are
    /// this crate's own, and the digest `crate::GateRunProvenance::policy` carries is taken
    /// from exactly these values so that a comparison attributes a difference to what judged a
    /// run and not to which layer said it.
    pub(crate) values: GatePolicyFile,
}

impl EffectivePolicy
{
    /// What decided `field`, if this policy resolved it.
    ///
    /// Every field is always resolved -- a value nobody stated is decided by the build's own
    /// defaults at [`ConfigurationLayer::Default`] -- so `None` means a field this resolution
    /// does not carry at all rather than one nobody stated.
    #[must_use]
    pub fn Deciding(&self, field: PolicyField) -> Option<&ResolvedField>
    {
        return self.fields.iter().find(|resolved| return resolved.field == field);
    }
}

/// `contributions` resolved into one effective policy, or the first statement it refuses.
///
/// # Errors
///
/// Returns [`PolicyRefusal`] for an unreadable artifact, a contribution stating a companion
/// field of a unit without that unit's deciding field, or one key stated twice within one
/// layer with different values. A locked override is refused too and is not returned here:
/// `CONFIG-003` requires a rejected override to remain visible with its reason, so it is
/// recorded in [`ResolvedField::rejected`] instead of ending the resolution.
// The refusal carries the layer, the artifact and the key at fault, and a boxed one would put
// an allocation between a reader and the sentence they are being handed. `PolicyRefusal` is
// produced once per run at most, so its size is not on any path that runs twice.
#[allow(clippy::result_large_err)]
pub fn Effective_Gate_Policy(contributions: &[PolicyContribution]) -> Result<EffectivePolicy, PolicyRefusal>
{
    if let Some(refusal) = resolution::Refusal_In(contributions)
    {
        return Err(refusal);
    }

    return Ok(resolution::Combined(contributions));
}

/// The policy a run judges under: what `nomos-gate.json` declared, with what `command` states
/// resolved over it.
///
/// The one function `crate::gate_environment` and `crate::finding_query` both call, which is
/// what `OD-GATE-011` asks for and what those two did not have: each called
/// `Resolved_Over` and wrote its own handling of the unreadable case beside it.
///
/// `from_file` is `None` for a root with no policy file and for one whose file could not be
/// read. Those are different facts and the caller keeps them apart -- an unreadable file is
/// what reaches a caller as `crate::NoVerdict::UnreadablePolicy` -- but neither contributes a
/// statement, so both reach this function the same way.
///
/// # Errors
///
/// Returns [`PolicyRefusal`] for anything [`Effective_Gate_Policy`] refuses. The one case
/// reachable from these two sources is a `command` stating `approvals` without `phases`: an
/// approval addresses a phase name only `phases` declares, so the entry could only ever take
/// effect by pairing with the file's stages, which is the defect the unit exists to prevent.
#[allow(clippy::result_large_err)] // [`Effective_Gate_Policy`]'s own reason, carried through.
pub(crate) fn Resolved_Gate_Policy(from_file: Option<&GatePolicyFile>, command: &GateCommand) -> Result<EffectivePolicy, PolicyRefusal>
{
    let mut contributions = Vec::new();

    if let Some(declared) = from_file
    {
        contributions.push(declared.Contributed());
    }
    contributions.push(Stated_By_The_Command(command));

    return Effective_Gate_Policy(&contributions);
}

/// What `command` states, as the `CommandLine` layer's own contribution.
///
/// A field left at its default states nothing, which is the rule `Preferred_Policy` already
/// applied and this module's own doc carries the reason for.
fn Stated_By_The_Command(command: &GateCommand) -> PolicyContribution
{
    use crate::policy::CoveragePolicy;

    return PolicyContribution {
        suppressions: (command.suppressions != crate::SuppressionPolicy::default()).then(|| return command.suppressions.clone()),
        baseline: (command.baseline != crate::BaselinePolicy::default()).then(|| return command.baseline.clone()),
        adoption: (command.adoption != crate::AdoptionPolicy::default()).then(|| return command.adoption.clone()),
        coverage: (command.coverage != CoveragePolicy::default()).then_some(command.coverage),
        phases: (!command.phases.is_empty()).then(|| return command.phases.clone()),
        approvals: (!command.approvals.is_empty()).then(|| return command.approvals.clone()),
        ..PolicyContribution::Silent(ConfigurationLayer::CommandLine, CALLER_BUILT_POLICY)
    };
}
