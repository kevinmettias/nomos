//! What `explain` answered about a [`super::FindingQuery`].

use crate::{BaselineDebt, RuleCalibration, Suppression};
use nomos_contracts::{EvidenceClass, Finding};

/// What `explain` answered about a [`super::FindingQuery`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Explanation
{
    /// No finding from `query.rule` names `query.location` among its locations, in this
    /// judgment.
    NotFound,
    /// The finding `query` names, and what it would do to a real run's disposition.
    Found
    {
        /// The finding itself, in full. Boxed: `NotFound` carries nothing, and a `Finding`
        /// inline here would make every `Explanation` pay `Found`'s size regardless of
        /// which variant it holds.
        finding: Box<Finding>,
        /// Whether this finding, on its own, could fail a build a real `run` reduces it
        /// into -- `Finding::Can_Fail_A_Build`, its evidence at or above this gate's declared
        /// floor, and no [`RuleCalibration`] or [`Suppression`] matched it.
        would_block: bool,
        /// The evidence floor that kept it from blocking, when `would_block` is `false`
        /// because this gate declared one above the class this finding's evidence carries.
        ///
        /// Checked before the three below, the same order [`crate::Run_Gate`]'s own partition
        /// reduces by, and reported apart from them for the reason `OD-GATE-034` gives: a
        /// calibration, a suppression and a baseline entry are dispositions a person authored
        /// about a finding, and this is a mechanical statement about a class of evidence with
        /// no per-finding author. `None` under [`crate::EvidenceFloor::Unset`], and `None` for
        /// a finding whose evidence clears a stated floor -- a floor that moved nothing has
        /// nothing to cite.
        floored_by: Option<EvidenceClass>,
        /// The calibration that kept it from blocking, when `would_block` is `false` because
        /// of one -- checked after the floor and before the two below, the same order
        /// [`crate::Run_Gate`] reduces by, since calibration is a coarser, rule-wide override.
        calibrated_by: Option<RuleCalibration>,
        /// The suppression that kept it from blocking, when `would_block` is `false`,
        /// `calibrated_by` is `None`, and a suppression matched.
        suppressed_by: Option<Suppression>,
        /// The baseline debt entry that kept it from blocking, when `would_block` is
        /// `false` and both `calibrated_by` and `suppressed_by` are `None` -- checked only
        /// once calibration and suppression are both ruled out, the same order
        /// [`crate::Run_Gate`] reduces by.
        baselined_by: Option<BaselineDebt>,
        /// The governing record `query.rule`'s implementation cites, and the version of
        /// that record it was written against -- `AGT-008`'s "rule version" clause,
        /// [`nomos_rules::RuleOffer`]'s own two fields, looked up from the same registry
        /// `nomos gate plan` already exposes through `RuleOfferResponse`. `None` only if
        /// the registry itself is contradictory (`Registered`'s own doc: "not reachable
        /// today") or `query.rule` names a rule this build does not register at all --
        /// never the ordinary "no `CONTRACT_RECORD`" case, which `Check_Naming_
        /// Convention` represents as a real, present citation to `README.md` version 0
        /// rather than an absence.
        contract: Option<(String, u32)>,
    },
}
