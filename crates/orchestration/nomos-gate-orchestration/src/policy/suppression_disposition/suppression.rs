//! One disposition over one rule's finding on one subject.

use super::{SuppressionDisposition, SuppressionStatus};
use nomos_contracts::{Finding, RuleId, SubjectId};
use nomos_platform::Timestamp;

/// One disposition over one rule's finding on one subject.
///
/// `owner`/`approver`/dates/revalidation triggers are `SUP-*`'s other required fields and
/// are deliberately not here yet -- this increment's own crate doc says why: no caller
/// constructs a `Suppression` at all today, so enforcing fields nothing populates would be
/// validating against nothing. Adding them is a later increment once something authors one
/// for real.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suppression
{
    /// The rule this disposition applies to.
    pub rule: RuleId,
    /// The subject this disposition applies to.
    pub subject: SubjectId,
    /// Which of `SUP-*`'s six dispositions this is.
    pub disposition: SuppressionDisposition,
    /// Why -- required, because a disposition with no stated reason is not distinguishable
    /// from silence.
    pub rationale: String,
    /// Who is accountable for this disposition.
    pub owner: String,
    /// When this disposition stops applying, for the one disposition whose meaning requires
    /// an end date.
    ///
    /// `Some` only for [`SuppressionDisposition::TemporaryWaiver`], which documents itself as
    /// accepted for a bounded period and distinguished from `AcceptedBaselineDebt` as the one
    /// that names an end date. Until this field existed it named nothing: a temporary waiver
    /// suppressed forever and was indistinguishable in behaviour from a
    /// `FormalRiskAcceptance`, which is a promise the engine did not keep.
    ///
    /// `None` for every other disposition, and the declared file refuses to write one there
    /// rather than carrying a field nothing reads -- `crate::policy::gate_policy_file` is
    /// where that refusal lives, because an author is the one who needs to be told.
    pub expiry: Option<Timestamp>,
}

impl Suppression
{
    /// Whether this disposition applies to `finding` -- the same `rule`/`subject` identity
    /// `Finding` already carries, so matching invents no addressing scheme of its own.
    #[must_use]
    pub fn Is_Applicable_To(&self, finding: &Finding) -> bool
    {
        return self.rule == finding.rule && self.subject == finding.subject;
    }

    /// Whether this disposition still applies at `now`.
    ///
    /// `now` is the moment the run is judged against, supplied by whoever composed the run,
    /// rather than a clock read here. That is what makes a replay answer as the run it
    /// replays: judging a past moment reaches the verdict that moment reached, not the one
    /// today's date would give.
    ///
    /// At the expiry instant the disposition has already stopped applying. A waiver good
    /// until a moment is not good at it.
    #[must_use]
    pub fn Status_At(&self, now: Timestamp) -> SuppressionStatus
    {
        return match self.expiry
        {
            Some(expiry) if now >= expiry => SuppressionStatus::Expired,
            _ => SuppressionStatus::Active,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::{Suppression, SuppressionDisposition};
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

    /// The subject byte the *other* finding in [`Test_Is_Applicable_To_Should_Not_Match_A_Different_Subject`]
    /// seeds from, named so the two findings that test tells apart differ by something a reader
    /// can read rather than by two bare digits.
    const OTHER_SUBJECT_SEED: u8 = 2;

    #[test]
    fn Test_Is_Applicable_To_Should_Match_Same_Rule_And_Subject()
    {
        let finding = Finding_For("naming-convention", 1);
        let suppression = Suppression_Of(&finding);

        assert!(suppression.Is_Applicable_To(&finding));
    }

    #[test]
    fn Test_Is_Applicable_To_Should_Not_Match_A_Different_Subject()
    {
        let finding = Finding_For("naming-convention", 1);
        let other = Finding_For("naming-convention", OTHER_SUBJECT_SEED);
        let suppression = Suppression_Of(&other);

        assert!(!suppression.Is_Applicable_To(&finding));
    }

    fn Finding_For(rule: &str, subject_seed: u8) -> Finding
    {
        return Finding {
            rule: RuleId::New(rule),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([subject_seed; Digest128::BYTE_LENGTH])),
            subject_name: "Example".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "example".to_owned(),
            locations: vec!["a.rs".to_owned()],
        };
    }

    fn Suppression_Of(finding: &Finding) -> Suppression
    {
        return Suppression {
            rule: finding.rule.clone(),
            subject: finding.subject,
            disposition: SuppressionDisposition::FalsePositiveDisposition,
            rationale: "known false positive".to_owned(),
            owner: "author".to_owned(),
            expiry: None
        };
    }
}

#[cfg(test)]
mod expiry_tests
{
    use super::{Suppression, SuppressionDisposition, SuppressionStatus};
    use nomos_contracts::{RuleId, SubjectId};
    use nomos_model::Content_Digest;
    use nomos_platform::Timestamp;

    const EXPIRY_SECONDS: i64 = 1_000;

    /// Before the instant, the waiver applies.
    #[test]
    fn Test_A_Waiver_Before_Its_Expiry_Should_Be_Active()
    {
        let waiver = Waiver_Expiring_At(EXPIRY_SECONDS);
        let before = Timestamp::From_Unix_Seconds(EXPIRY_SECONDS - 1);

        assert_eq!(waiver.Status_At(before), SuppressionStatus::Active);
        assert!(waiver.Status_At(before).Suppresses());
    }

    /// At the instant, it does not.
    ///
    /// The boundary is the case worth writing down: a waiver good until a moment is not good
    /// at it, the same way a deadline is not met by arriving on it. An implementation using a
    /// strict `>` instead of `>=` passes both neighbours of this test and fails only here.
    #[test]
    fn Test_A_Waiver_At_Its_Expiry_Should_Be_Expired()
    {
        let waiver = Waiver_Expiring_At(EXPIRY_SECONDS);
        let exactly = Timestamp::From_Unix_Seconds(EXPIRY_SECONDS);

        assert_eq!(waiver.Status_At(exactly), SuppressionStatus::Expired);
        assert!(!waiver.Status_At(exactly).Suppresses());
    }

    /// After it, it does not.
    #[test]
    fn Test_A_Waiver_After_Its_Expiry_Should_Be_Expired()
    {
        let waiver = Waiver_Expiring_At(EXPIRY_SECONDS);
        let after = Timestamp::From_Unix_Seconds(EXPIRY_SECONDS + 1);

        assert_eq!(waiver.Status_At(after), SuppressionStatus::Expired);
    }

    /// A disposition with no end date never expires, at any moment.
    ///
    /// The converse control. Without it an implementation that treated every disposition as
    /// expired would pass the two above and silently stop suppressing everything else.
    #[test]
    fn Test_A_Disposition_With_No_Expiry_Should_Never_Expire()
    {
        let mut accepted = Waiver_Expiring_At(EXPIRY_SECONDS);
        accepted.disposition = SuppressionDisposition::FormalRiskAcceptance;
        accepted.expiry = None;

        for seconds in [0, EXPIRY_SECONDS, i64::MAX]
        {
            assert_eq!(
                accepted.Status_At(Timestamp::From_Unix_Seconds(seconds)),
                SuppressionStatus::Active,
                "a disposition naming no end date stopped applying at {seconds}"
            );
        }
    }

    /// The waiver every test above shares, so its construction is written once: a temporary
    /// waiver ending at `seconds`, which each test then judges at a moment of its own.
    fn Waiver_Expiring_At(seconds: i64) -> Suppression
    {
        return Suppression {
            rule: RuleId::New("naming-convention"),
            subject: SubjectId::From_Digest(Content_Digest(b"src/lib.rs")),
            disposition: SuppressionDisposition::TemporaryWaiver,
            rationale: "bounded while the rename lands".to_string(),
            owner: "someone".to_string(),
            expiry: Some(Timestamp::From_Unix_Seconds(seconds)),
        };
    }
}
