//! JSON-serializable projections of what `nomos-gate-orchestration` produces for `run` and
//! `plan`.
//!
//! `GateRunResult`, `GateRunOutcome`, `GateOutcome` and `nomos_rules::RuleOffer` do not
//! derive `Serialize` -- nothing needed a wire format for any of them before this crate
//! existed. Adding it to them directly would grow `nomos-gate-orchestration`'s or
//! `nomos-rules`' own public surface on behalf of one caller's shape, before a second
//! transport exists to check that shape against -- the same premature-surface caution this
//! crate's own module doc names. The types here are that shape, owned in this crate and
//! built by conversion from the borrowed or owned result, so neither orchestration crate
//! changes.

use nomos_contracts::{Finding, RuleId, RunId};
use nomos_gate_orchestration::{GateOutcome, GateRunOutcome, GateRunResult};
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
