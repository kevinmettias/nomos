//! The declared source a real run resolves its policies from.
//!
//! Every policy type in this module's siblings documented that nothing constructed a
//! non-empty one, and each was reachable only from a unit test. This is the authoring
//! surface that closes that: one JSON file at the run's own root, read through the
//! [`FileSystem`] port [`crate::Run_Gate`] already carries.
//!
//! # Why resolution happens inside `Run_Gate` rather than in each composition root
//!
//! `Run_Gate` already takes both the root and the filesystem port, so resolving here serves
//! the CLI, `nomos-api`, the transport and MCP at once. Resolving in each composition root
//! instead would put the same file-format knowledge in four places, which is the duplication
//! `OD-GATE-011` names as a defect class, and would leave any surface that forgot it silently
//! running on defaults.
//!
//! # Why entries name a path and not a subject
//!
//! [`crate::Suppression`] and [`crate::BaselineDebt`] both match on `SubjectId`, which is a
//! digest no person can author by hand. `nomos_model::Subject_Of_Path` is the kernel's
//! canonical path-to-subject mapping -- `OD-MODEL-001`, and the one every real walker already
//! uses to build `SourceFile::subject` -- so a file names the path a reader can see and this
//! module computes the identity a run matches on, by calling the same function the walk does
//! rather than reimplementing it.
//!
//! # What a path-authored entry does not reach
//!
//! A finding is not always addressed by the subject of the file it was found in. Measured
//! directly against a real run rather than assumed: `no-single-line-function-bodies` and
//! `todo-format-is-todo-name-description-ticket` address the file's own subject, so a path
//! names them; `completeness-mirror` addresses the mirrored item and `single-letter-names`
//! addresses the name, so a path does not. `crate::tests`'
//! `Test_A_Suppressed_Finding_Should_Not_Block` already documented this for the first of
//! those, which is why it reads its subject off a real run instead of computing one.
//!
//! So this file suppresses and baselines findings a rule addresses by file, and silently
//! matches nothing for the rest. That is a real limit and not a safe default -- an author who
//! writes an entry for `completeness-mirror` gets no error and no effect. It is left here
//! rather than papered over with a digest field, because a file naming a raw `SubjectId` is
//! the unauthorable identity this module exists to avoid, and the honest fix is for
//! sub-item findings to carry an addressable name that a person can write. That is a
//! question about `Finding` rather than about this file, and it belongs to whichever item
//! takes it up.
//!
//! # Why the file loses to an explicitly constructed policy
//!
//! A caller that built a policy in code has said something more specific than a file sitting
//! in a tree, and a test that pins a policy must not have it silently replaced by whatever
//! the working directory happens to contain. This file therefore does not resolve at all any
//! more: it *contributes*, through [`GatePolicyFile::Contributed`], at the `Repository` layer
//! of `crate::policy::effective_policy`, and the precedence is that resolver's one rule rather
//! than a second one written here. `GatePolicyFile::Resolved_Over` was that second rule, two
//! levels deep, with two call sites that each re-derived the unreadable case; `OD-POLICY-001`
//! re-homed it and the values it produced are unchanged, which is what this module's own
//! precedence tests still assert.
//!
//! `Default` already means "unset" for every family here -- `CoveragePolicy::Unset` says so in
//! its own name -- so a field left at its default states nothing, the same way
//! `ScopeSelector`'s empty-means-everything already does. That is the mapping
//! [`GatePolicyFile::Contributed`] performs, and `OD-POLICY-001`'s "a sentinel is not a
//! statement" is the rule it performs it under.
//!
//! # The two phase families are one declaration
//!
//! `phases` and `approvals` resolve from one source rather than field by field, because an
//! approval names the phase it covers. `OD-POLICY-001` version 2 calls that shape a *unit* and
//! declares this one: `crate::policy::effective_policy::PHASE_POLICY_UNIT` names the fields,
//! names `phases` as the deciding field, and carries the reason
//! `Preferred_Phase_Policy` used to state in code here. [`declared_phases`]' own doc carries
//! the rest of that shape.

mod declared_phases;

use declared_phases::{DeclaredApproval, DeclaredPhase, Phase_Problem, Resolved_Approvals, Resolved_Phases};
use nomos_contracts::{ConfigurationLayer, RuleId};
use nomos_model::Subject_Of_Path;
use nomos_platform::{FileSystem, FileSystemError};
use serde::Deserialize;
use std::path::Path;

use super::effective_policy::PolicyContribution;
use super::{
    AdoptionPolicy, BaselineAllowance, BaselineDebt, BaselinePolicy, CoveragePolicy, RuleCalibration, Suppression, SuppressionDisposition,
    SuppressionPolicy,
};
use crate::{GatePhase, NoVerdict, PhaseApproval};

/// The file a run resolves its policies from, relative to the run's own root.
///
/// JSON rather than TOML because this workspace already reads JSON for `standards.json` and
/// `work/ledger.json` and parses none of the tool-configuration TOML it sits beside, so JSON
/// is the format a reader here has already met.
pub(crate) const GATE_POLICY_FILE: &str = "nomos-gate.json";

/// What [`GATE_POLICY_FILE`] said, resolved into the policy types a run judges against.
///
/// Holds the domain types rather than the wire shapes: the deserialized forms below exist
/// only long enough to be converted, so nothing downstream of this type has to know a file
/// was involved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GatePolicyFile
{
    /// Dispositions the file declared.
    pub(crate) suppressions: SuppressionPolicy,
    /// Existing debt the file declared.
    pub(crate) baseline: BaselinePolicy,
    /// Rule calibrations the file declared.
    pub(crate) adoption: AdoptionPolicy,
    /// The coverage floor the file declared.
    pub(crate) coverage: CoveragePolicy,
    /// The ordered stages the file declared, in the order it declared them.
    pub(crate) phases: Vec<GatePhase>,
    /// The approvals the file declared, each naming one of `phases`.
    pub(crate) approvals: Vec<PhaseApproval>,
}

impl GatePolicyFile
{
    /// What this file states, as the `Repository` layer's own contribution.
    ///
    /// A field left at its default states nothing rather than stating the default:
    /// `OD-POLICY-001`'s "a sentinel is not a statement", which for this file is the rule
    /// `Preferred_Policy` already applied under another name. The resolver is what decides
    /// precedence over it; this only says what the file said.
    ///
    /// `phases` and `approvals` are mapped the same way and are *not* coupled here. The
    /// coupling is `crate::policy::effective_policy::PHASE_POLICY_UNIT`, declared once where
    /// the resolution happens, so that a second source of either field meets the same rule
    /// rather than one this reader re-derives.
    #[must_use]
    pub(crate) fn Contributed(&self) -> PolicyContribution
    {
        return PolicyContribution {
            suppressions: (self.suppressions != SuppressionPolicy::default()).then(|| return self.suppressions.clone()),
            baseline: (self.baseline != BaselinePolicy::default()).then(|| return self.baseline.clone()),
            adoption: (self.adoption != AdoptionPolicy::default()).then(|| return self.adoption.clone()),
            coverage: (self.coverage != CoveragePolicy::default()).then_some(self.coverage),
            phases: (!self.phases.is_empty()).then(|| return self.phases.clone()),
            approvals: (!self.approvals.is_empty()).then(|| return self.approvals.clone()),
            ..PolicyContribution::Silent(ConfigurationLayer::Repository, GATE_POLICY_FILE)
        };
    }
}

/// Why a policy file that exists could not become a policy.
///
/// Absence is not represented here: a tree with no policy file is the ordinary case, not a
/// failure, and [`Resolve_Gate_Policy`] reports it as `Ok(None)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GatePolicyError
{
    /// The file is there and could not be read.
    Unreadable(String),
    /// The file was read and is not the shape this module expects.
    Malformed(String),
}

impl GatePolicyError
{
    /// This failure as the reason a run reached no verdict.
    ///
    /// The mapping lives here rather than at the call site so that a variant added to this
    /// enum has to answer what it means to a run, in the file that added it, instead of
    /// silently folding into whatever arm the caller wrote last.
    pub(crate) fn As_No_Verdict(&self) -> NoVerdict
    {
        return match self
        {
            Self::Unreadable(detail) => NoVerdict::UnreadablePolicy(detail.clone()),
            Self::Malformed(detail) => NoVerdict::MalformedPolicy(detail.clone()),
        };
    }
}

/// The policy [`GATE_POLICY_FILE`] under `root` declares, or `Ok(None)` when there is none.
///
/// # Errors
///
/// Returns [`GatePolicyError`] when a file is present but cannot be turned into a policy.
/// A present-but-broken file is deliberately not treated as an absent one: a repository that
/// meant to suppress a finding and misspelled the file would otherwise get a build that
/// passes for a reason nobody chose, which is the failure this whole item exists to stop.
pub(crate) fn Resolve_Gate_Policy<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<Option<GatePolicyFile>, GatePolicyError>
{
    let text = match filesystem.Read_To_String(&root.join(GATE_POLICY_FILE))
    {
        Ok(text) => text,
        Err(FileSystemError::NotFound { .. }) => return Ok(None),
        Err(error) => return Err(GatePolicyError::Unreadable(format!("{}: {error:?}", error.Path()))),
    };

    let declared: DeclaredPolicy = serde_json::from_str(&text).map_err(|error| return GatePolicyError::Malformed(error.to_string()))?;

    if let Some(problem) = Problem_In(&declared)
    {
        return Err(GatePolicyError::Malformed(problem));
    }

    return Ok(Some(declared.Resolved()));
}

/// What is wrong with `declared`, if anything -- every refusal this module owes an author,
/// in the order the families are declared above.
///
/// One function rather than a block per family inside [`Resolve_Gate_Policy`], so a family
/// gaining a refusal has one place to be registered. `serde` owns the refusals that are about
/// shape -- an unknown key, a spelling no variant has, a count that is not a number -- and
/// reports each with the offending value, the spellings that would have been accepted and the
/// line and column it sits at. What is left here is every rule `serde` cannot state.
fn Problem_In(declared: &DeclaredPolicy) -> Option<String>
{
    return declared
        .suppressions
        .iter()
        .find_map(DeclaredSuppression::Problem)
        .or_else(|| return declared.baseline.iter().find_map(DeclaredDebt::Problem))
        .or_else(|| return Phase_Problem(&declared.phases, &declared.approvals));
}

/// [`GATE_POLICY_FILE`]'s own shape, as written.
///
/// Separate from [`GatePolicyFile`] so `serde` never reaches the domain types. Deriving
/// `Deserialize` on [`Suppression`] and its siblings would make a file able to name a
/// `SubjectId` digest directly, which is exactly the unauthorable identity this module
/// exists to compute rather than accept, and would widen the crate's public surface on
/// behalf of one caller's format.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct DeclaredPolicy
{
    suppressions: Vec<DeclaredSuppression>,
    baseline: Vec<DeclaredDebt>,
    adoption: Vec<DeclaredCalibration>,
    coverage: DeclaredCoverage,
    /// The ordered stages this file declares, `WF-001`'s "required phases ... thresholds ...
    /// and blocking behavior". An absent key is no phase policy, which is the state every
    /// caller was in before it existed.
    phases: Vec<DeclaredPhase>,
    /// The approvals this file declares, `WF-001`'s "approvals". Each names a phase in
    /// `phases`, and one that does not is refused rather than stored.
    approvals: Vec<DeclaredApproval>,
}

impl DeclaredPolicy
{
    /// This declaration as the policy types a run judges against.
    fn Resolved(self) -> GatePolicyFile
    {
        return GatePolicyFile {
            suppressions: SuppressionPolicy { suppressions: self.suppressions.into_iter().map(DeclaredSuppression::Resolved).collect() },
            baseline: BaselinePolicy { debt: self.baseline.into_iter().map(DeclaredDebt::Resolved).collect() },
            adoption: AdoptionPolicy { calibrated: self.adoption.into_iter().map(DeclaredCalibration::Resolved).collect() },
            coverage: self.coverage.Resolved(),
            phases: Resolved_Phases(self.phases),
            approvals: Resolved_Approvals(self.approvals),
        };
    }
}

/// One suppression, as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredSuppression
{
    rule: String,
    path: String,
    disposition: DeclaredDisposition,
    rationale: String,
    owner: String,
    /// When the disposition stops applying, as whole Unix seconds.
    ///
    /// Required for `temporary-waiver` and refused for every other spelling --
    /// [`DeclaredSuppression::Problem`] is where that is said to the author, because an
    /// author is who needs to hear it. A waiver with no end date is the contradiction this
    /// field exists to remove: the disposition means bounded and, without one, it suppressed
    /// forever.
    ///
    /// Whole seconds rather than a calendar date, matching how this workspace already writes
    /// a `Timestamp` into `work/ledger.json`. A friendlier spelling would mean a date-parsing
    /// dependency and a supply-chain decision nobody has asked for yet.
    #[serde(default)]
    expiry: Option<i64>,
}

impl DeclaredSuppression
{
    /// What is wrong with this entry, if anything.
    ///
    /// The one rule `serde` cannot state: which field is required depends on which
    /// disposition was named. A `temporary-waiver` without an `expiry` is a waiver that never
    /// ends, which is a `formal-risk-acceptance` wearing another name. Any other disposition
    /// *with* one is carrying a date nothing reads, and saying so is better than accepting it
    /// silently -- the same reason `deny_unknown_fields` refuses a misspelled key rather than
    /// dropping it.
    fn Problem(&self) -> Option<String>
    {
        let waiver = matches!(self.disposition, DeclaredDisposition::TemporaryWaiver);

        if waiver && self.expiry.is_none()
        {
            return Some(format!(
                "the temporary-waiver for rule '{}' on '{}' names no expiry. A temporary waiver without an end date never ends, which is a formal-risk-acceptance wearing another name: give it an expiry, or declare the disposition you actually mean.",
                self.rule, self.path
            ));
        }

        if !waiver && self.expiry.is_some()
        {
            return Some(format!(
                "the suppression for rule '{}' on '{}' names an expiry, and only a temporary-waiver has one. Every other disposition applies until it is removed, so a date here would be read by nothing and would tell a later reader something untrue.",
                self.rule, self.path
            ));
        }

        return None;
    }

    /// This entry as the domain type, with its subject computed from its path.
    fn Resolved(self) -> Suppression
    {
        use nomos_platform::Timestamp;

        return Suppression {
            rule: RuleId::New(&self.rule),
            subject: Subject_Of_Path(&self.path),
            disposition: self.disposition.Resolved(),
            rationale: self.rationale,
            owner: self.owner,
            expiry: self.expiry.map(Timestamp::From_Unix_Seconds),
        };
    }
}

/// One baseline entry, as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredDebt
{
    rule: String,
    path: String,
    rationale: String,
    /// How many occurrences of this rule at this path were accepted at adoption.
    ///
    /// Optional, and its absence is not an oversight to be defaulted away: `OD-GATE-030`
    /// decides that an entry naming no count keeps the meaning it was written under, which is
    /// unbounded. Every entry authored before the key existed is in that state, and reading
    /// them as one occurrence would start blocking builds over debt a repository did adopt.
    ///
    /// Spelled out rather than shortened because this is a file a person writes by hand once
    /// and reads much later, and `accepted` alone would not say accepted *when* or *how many*.
    #[serde(default)]
    accepted_occurrence_count: Option<u32>,
}

impl DeclaredDebt
{
    /// What is wrong with this entry, if anything -- the baseline counterpart to
    /// [`DeclaredSuppression::Problem`], and refused at the same point for the same reason.
    ///
    /// Zero is the only refusable value. An entry accepting no occurrences tolerates nothing,
    /// so its only possible effect is to block exactly what writing it claims to permit, and
    /// an author who wrote it meant something else. Not writing the entry is already how a
    /// repository tolerates none, so there is nothing this value could mean that is not
    /// already said more clearly another way.
    fn Problem(&self) -> Option<String>
    {
        if self.accepted_occurrence_count != Some(0)
        {
            return None;
        }

        return Some(format!(
            "the baseline entry for rule '{}' on '{}' accepts zero occurrences. An entry that accepts none tolerates nothing, which is what leaving the entry out already does: give it the number of occurrences you adopted, or remove it.",
            self.rule, self.path
        ));
    }

    /// This entry as the domain type, with its subject computed from its path.
    ///
    /// Both the subject and the declared path come off the one field, and they are not
    /// redundant: the subject folds every spelling of a path to one digest, and the declared
    /// path is the spelling this author happened to write. The subject is what a run matches
    /// on; the path is what a report names the scope by, because the fold is one way and a
    /// reader cannot recover their own spelling from a digest.
    fn Resolved(self) -> BaselineDebt
    {
        return BaselineDebt {
            rule: RuleId::New(&self.rule),
            subject: Subject_Of_Path(&self.path),
            rationale: self.rationale,
            allowance: self.accepted_occurrence_count.map_or(BaselineAllowance::Unbounded, BaselineAllowance::AtMost),
            // `self.path` is moved here and borrowed above. A struct literal evaluates its
            // fields in the order they are written, so the borrow has already been taken by
            // the time this line runs; reordering these two would not compile.
            declared_path: Some(self.path),
        };
    }
}

/// One rule calibration, as written.
///
/// Names no path, because [`RuleCalibration`] matches by rule alone -- a calibration is the
/// coarse, rule-wide override, and giving it a path here would suggest a narrowing it does
/// not perform.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredCalibration
{
    rule: String,
    rationale: String,
}

impl DeclaredCalibration
{
    /// This entry as the domain type.
    fn Resolved(self) -> RuleCalibration
    {
        return RuleCalibration { rule: RuleId::New(&self.rule), rationale: self.rationale };
    }
}

/// The six dispositions, as written.
///
/// Spelled in `kebab-case` rather than the Rust variant names because this is a hand-authored
/// file and not a serialization of the enum; the mapping is stated once, here.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DeclaredDisposition
{
    InlineSuppression,
    RepositoryPolicyException,
    TemporaryWaiver,
    AcceptedBaselineDebt,
    FalsePositive,
    FormalRiskAcceptance,
}

impl DeclaredDisposition
{
    /// This spelling as the domain variant.
    fn Resolved(self) -> SuppressionDisposition
    {
        return match self
        {
            Self::InlineSuppression => SuppressionDisposition::InlineSuppression,
            Self::RepositoryPolicyException => SuppressionDisposition::RepositoryPolicyException,
            Self::TemporaryWaiver => SuppressionDisposition::TemporaryWaiver,
            Self::AcceptedBaselineDebt => SuppressionDisposition::AcceptedBaselineDebt,
            Self::FalsePositive => SuppressionDisposition::FalsePositiveDisposition,
            Self::FormalRiskAcceptance => SuppressionDisposition::FormalRiskAcceptance,
        };
    }
}

/// The coverage floor, as written.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DeclaredCoverage
{
    /// Coverage debt does not affect disposition -- the state a file that says nothing is in.
    #[default]
    Unset,
    /// Coverage debt downgrades an otherwise-`Passed` run to `Indeterminate`.
    RequireCompleteness,
}

impl DeclaredCoverage
{
    /// This spelling as the domain variant.
    fn Resolved(self) -> CoveragePolicy
    {
        return match self
        {
            Self::Unset => CoveragePolicy::Unset,
            Self::RequireCompleteness => CoveragePolicy::RequireCompleteness,
        };
    }
}

#[cfg(test)]
mod tests;
