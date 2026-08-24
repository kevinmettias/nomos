//! JSON-serializable projections of what `nomos-gate-orchestration` produces for `run`,
//! `plan` and `explain`.
//!
//! `GateRunResult`, `GateRunOutcome`, `GateOutcome`, `Explanation` and its own
//! `RuleCalibration`/`Suppression`/`SuppressionDisposition`/`BaselineDebt`, and
//! `nomos_rules::RuleOffer`, do not derive `Serialize` -- nothing needed a wire format for
//! any of them before this crate existed. Adding it to them directly would grow
//! `nomos-gate-orchestration`'s or `nomos-rules`' own public surface on behalf of one
//! caller's shape, before a second transport exists to check that shape against -- the same
//! premature-surface caution this crate's own module doc names. The types here are that
//! shape, owned in this crate and built by conversion from the borrowed or owned result, so
//! neither orchestration crate changes.

use nomos_contracts::{Finding, RuleId, RunId, SubjectId};
use nomos_gate_orchestration::{
    BaselineDebt, Explanation, GateOutcome, GateRunOutcome, GateRunResult, RuleCalibration, Suppression,
    SuppressionDisposition,
};
use nomos_rules::RuleOffer;
use serde::Serialize;
use std::path::PathBuf;

/// What a real gate run over `root` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
pub struct GateRunResponse
{
    /// The identity of this execution. `RunId` already derives `Serialize` -- unlike
    /// [`GateRunOutcome`], it needs no local twin.
    pub run: RunId,
    /// The tree this run judged.
    pub root: PathBuf,
    /// The reduced verdict.
    pub disposition: Disposition,
    /// Exactly the findings that failed this build. Empty whenever `disposition` is not
    /// [`Disposition::Failed`].
    pub blocking_findings: Vec<Finding>,
    /// Findings an `AdoptionPolicy` calibration kept from blocking.
    pub calibrated_findings: Vec<Finding>,
    /// Findings a `Suppression` kept from blocking.
    pub suppressed_findings: Vec<Finding>,
    /// Findings a `BaselineDebt` kept from blocking.
    pub baselined_findings: Vec<Finding>,
}

impl GateRunResponse
{
    pub(crate) fn From(result: GateRunResult) -> Self
    {
        return Self {
            run: result.run,
            root: result.root,
            disposition: Disposition::From(result.disposition),
            blocking_findings: result.blocking_findings,
            calibrated_findings: result.calibrated_findings,
            suppressed_findings: result.suppressed_findings,
            baselined_findings: result.baselined_findings,
        };
    }
}

/// A serializable twin of [`nomos_gate_orchestration::GateRunOutcome`].
///
/// A twin rather than a re-export because the type it mirrors does not derive `Serialize`,
/// for the reason this module's own doc gives; kept to the same three variants, in the same
/// order, so a mismatch between the two is a compile error in [`Disposition::From`] rather
/// than a silent divergence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition
{
    /// No finding this run saw can fail a build.
    Passed,
    /// At least one finding can fail a build.
    Failed,
    /// The check behind this run could not produce an authoritative judgment.
    Indeterminate,
}

impl Disposition
{
    fn From(outcome: GateRunOutcome) -> Self
    {
        return match outcome
        {
            GateRunOutcome::Passed => Disposition::Passed,
            GateRunOutcome::Failed => Disposition::Failed,
            GateRunOutcome::Indeterminate => Disposition::Indeterminate,
        };
    }
}

/// What a real `nomos gate plan` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum GatePlanResponse
{
    /// This gate's rule registry composed cleanly.
    Planned
    {
        /// Every rule this gate's registry holds.
        rules: Vec<RuleOfferResponse>,
    },
    /// This gate's own rule composition is self-contradictory -- a defect in the
    /// composition, not in anything a caller supplied. Not reachable today; see
    /// `nomos_gate_orchestration::Registered`'s own doc.
    Contradictory
    {
        /// What went wrong, as `RuleRegistryError`'s own `Debug` renders it -- it does not
        /// derive `Display`, the same reason `crates/host/nomos-cli/src/gate/report.rs`'s
        /// own `Render_Plan` prints `{error:?}` too.
        cause: String,
    },
}

impl GatePlanResponse
{
    pub(crate) fn From(outcome: GateOutcome) -> Self
    {
        return match outcome
        {
            GateOutcome::Planned(plan) => Self::Planned {
                rules: plan.rules.into_iter().map(RuleOfferResponse::From).collect(),
            },
            GateOutcome::Contradictory(error) => Self::Contradictory { cause: format!("{error:?}") },
        };
    }
}

/// A serializable twin of [`nomos_rules::RuleOffer`], for the same reason [`Disposition`]
/// twins [`GateRunOutcome`]: the type it mirrors does not derive `Serialize`, and growing
/// `nomos-rules`' own public surface on behalf of one caller's wire shape is not this
/// increment's to spend.
#[derive(Debug, Serialize)]
pub struct RuleOfferResponse
{
    /// The rule's own identity.
    pub rule: RuleId,
    /// The governing record this rule's implementation cites, e.g. `"D-134"`.
    pub contract_record: String,
    /// The version of `contract_record` this implementation was written against.
    pub contract_record_version: u32,
}

impl RuleOfferResponse
{
    fn From(offer: RuleOffer) -> Self
    {
        return Self {
            rule: offer.rule,
            contract_record: offer.contract_record,
            contract_record_version: offer.contract_record_version,
        };
    }
}

/// What a real `nomos gate explain` produced, in a shape `serde_json` can hand across a
/// wire.
///
/// Carries no `root` and no `check_outcome`: this crate's caller already supplied `root`,
/// and `GateRunResponse` already sets the precedent of dropping `check_outcome` from its own
/// wire shape entirely, rather than re-exposing `CheckOutcome` for a caller to reconstruct a
/// distinction `Explain_Gate` itself does not draw -- see [`Self::NotFound`]'s own doc for
/// the one it collapses.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum GateExplainResponse
{
    /// No finding from the query's rule at the query's location exists in this judgment.
    ///
    /// Two different causes collapse into this one value, by `Explain_Gate`'s own design
    /// (`crates/orchestration/nomos-gate-orchestration/src/explain.rs`'s `Explained`): a
    /// tree that was never judged at all, and a judged tree whose real findings simply do
    /// not include this rule and location. This type preserves that collapse rather than
    /// inventing a finer distinction the orchestration crate does not draw.
    NotFound,
    /// The finding the query names, and what it would do to a real run's disposition.
    Found
    {
        /// The finding itself, in full. `Finding` already derives `Serialize`. Boxed, the
        /// same reason `WorkShowResponse::Found::item` is: `NotFound` carries nothing, and
        /// an unboxed `Finding` here would size every `GateExplainResponse` to `Found`'s own
        /// width regardless of which variant it holds.
        finding: Box<Finding>,
        /// Whether this finding, on its own, could fail a build a real `run` reduces it
        /// into.
        would_block: bool,
        /// The calibration that kept it from blocking, when `would_block` is `false`
        /// because of one.
        calibrated_by: Option<RuleCalibrationResponse>,
        /// The suppression that kept it from blocking, when `would_block` is `false`,
        /// `calibrated_by` is `None`, and a suppression matched. Boxed: `SuppressionResponse`
        /// is the largest of the three calibration/suppression/baseline fields (it alone
        /// carries two owned `String`s beside `rule` and `subject`), so it is the one
        /// `clippy::large_enum_variant` names to shrink `Found`'s own width by.
        suppressed_by: Box<Option<SuppressionResponse>>,
        /// The baseline debt entry that kept it from blocking, when `would_block` is
        /// `false` and both `calibrated_by` and `suppressed_by` are `None`.
        baselined_by: Option<BaselineDebtResponse>,
        /// The governing record `finding.rule`'s implementation cites, and the version
        /// of that record it was written against -- `AGT-008`'s "rule version" clause.
        /// `None` only for a rule this build's registry does not hold; see
        /// [`nomos_gate_orchestration::Explanation::Found`]'s own doc for why the
        /// ordinary "no `CONTRACT_RECORD`" case is a real citation, not this.
        contract_record: Option<String>,
        /// The version of `contract_record` this rule's implementation was written
        /// against. Always `Some` exactly when `contract_record` is.
        contract_record_version: Option<u32>,
    },
}

impl GateExplainResponse
{
    pub(crate) fn From(explanation: Explanation) -> Self
    {
        return match explanation
        {
            Explanation::NotFound => Self::NotFound,
            Explanation::Found { finding, would_block, calibrated_by, suppressed_by, baselined_by, contract } =>
            {
                let (contract_record, contract_record_version) = match contract
                {
                    Some((record, version)) => (Some(record), Some(version)),
                    None => (None, None),
                };

                Self::Found {
                    finding,
                    would_block,
                    calibrated_by: calibrated_by.map(RuleCalibrationResponse::From),
                    suppressed_by: Box::new(suppressed_by.map(SuppressionResponse::From)),
                    baselined_by: baselined_by.map(BaselineDebtResponse::From),
                    contract_record,
                    contract_record_version,
                }
            }
        };
    }
}

/// A serializable twin of [`nomos_gate_orchestration::RuleCalibration`], for the same reason
/// [`Disposition`] twins [`GateRunOutcome`].
#[derive(Debug, Serialize)]
pub struct RuleCalibrationResponse
{
    /// The rule this calibration applies to.
    pub rule: RuleId,
    /// Why this rule is not yet blocking for this repository.
    pub rationale: String,
}

impl RuleCalibrationResponse
{
    fn From(calibration: RuleCalibration) -> Self
    {
        return Self { rule: calibration.rule, rationale: calibration.rationale };
    }
}

/// A serializable twin of [`nomos_gate_orchestration::Suppression`], for the same reason
/// [`Disposition`] twins [`GateRunOutcome`].
#[derive(Debug, Serialize)]
pub struct SuppressionResponse
{
    /// The rule this disposition applies to.
    pub rule: RuleId,
    /// The subject this disposition applies to.
    pub subject: SubjectId,
    /// Which of `SUP-*`'s six dispositions this is.
    pub disposition: SuppressionDispositionResponse,
    /// Why.
    pub rationale: String,
    /// Who is accountable for this disposition.
    pub owner: String,
}

impl SuppressionResponse
{
    fn From(suppression: Suppression) -> Self
    {
        return Self {
            rule: suppression.rule,
            subject: suppression.subject,
            disposition: SuppressionDispositionResponse::From(suppression.disposition),
            rationale: suppression.rationale,
            owner: suppression.owner,
        };
    }
}

/// A serializable twin of [`nomos_gate_orchestration::SuppressionDisposition`], kept to the
/// same six variants, in the same order, so a mismatch between the two is a compile error in
/// [`SuppressionDispositionResponse::From`] rather than a silent divergence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionDispositionResponse
{
    /// Suppressed at the finding's own site, e.g. an inline annotation.
    InlineSuppression,
    /// Exempted by a repository-wide policy rather than a per-finding annotation.
    RepositoryPolicyException,
    /// Accepted for a bounded period, expected to be revisited.
    TemporaryWaiver,
    /// Existing debt a baseline tolerates rather than blocks.
    AcceptedBaselineDebt,
    /// The finding does not hold; the rule (or its inputs) were wrong here.
    FalsePositiveDisposition,
    /// A deliberate, owned decision to accept the risk the finding names.
    FormalRiskAcceptance,
}

impl SuppressionDispositionResponse
{
    fn From(disposition: SuppressionDisposition) -> Self
    {
        return match disposition
        {
            SuppressionDisposition::InlineSuppression => Self::InlineSuppression,
            SuppressionDisposition::RepositoryPolicyException => Self::RepositoryPolicyException,
            SuppressionDisposition::TemporaryWaiver => Self::TemporaryWaiver,
            SuppressionDisposition::AcceptedBaselineDebt => Self::AcceptedBaselineDebt,
            SuppressionDisposition::FalsePositiveDisposition => Self::FalsePositiveDisposition,
            SuppressionDisposition::FormalRiskAcceptance => Self::FormalRiskAcceptance,
        };
    }
}

/// A serializable twin of [`nomos_gate_orchestration::BaselineDebt`], for the same reason
/// [`Disposition`] twins [`GateRunOutcome`].
#[derive(Debug, Serialize)]
pub struct BaselineDebtResponse
{
    /// The rule this debt applies to.
    pub rule: RuleId,
    /// The subject this debt applies to.
    pub subject: SubjectId,
    /// Why this finding is tolerated rather than fixed.
    pub rationale: String,
}

impl BaselineDebtResponse
{
    fn From(debt: BaselineDebt) -> Self
    {
        return Self { rule: debt.rule, subject: debt.subject, rationale: debt.rationale };
    }
}
