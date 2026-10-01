//! One distinct provenance tuple behind the reads a rule performed, as an editor receives it.
//!
//! A projection of `nomos_check_orchestration::FactRead` and nothing else -- every field below
//! is that tuple's own field carried through. The reduction that produced it, and the store
//! lookup that resolved the answering fact's guarantee and evidence class, both happened inside
//! the run while the generation was unambiguous and the store was still in hand (`OD-HOST-016`).
//! Nothing here re-reads a store, and this crate holds none.
//!
//! The kernel's own value is carried wherever the kernel already serializes it --
//! `nomos_contracts::ContractVersion`, `nomos_contracts::Guarantee` and
//! `nomos_contracts::EvidenceClass` all derive `Serialize` there, so respelling any of them
//! here would be this crate inventing a second vocabulary for somebody else's. The one
//! exception is `nomos_analysis::ReadOutcome`, which derives none: it is projected to its own
//! variant name the way [`crate::GoverningRule`] projects `nomos_rules::SubjectKind`, with the
//! `Applicability` behind a degraded read carried beside the word rather than folded into it.

use nomos_analysis::ReadOutcome;
use nomos_contracts::{ContractVersion, EvidenceClass, Guarantee};
use serde::Serialize;

/// One distinct provenance tuple behind a rule's reads: which capability was asked, which
/// provider answered it at which contract version, how the read came back, and -- for a read
/// that was answered -- the guarantee and evidence class of the fact that answered it.
///
/// Provenance, not identity. The `nomos_analysis::FactKey`'s own digest components are
/// deliberately absent here because they were deliberately absent from the tuple this projects:
/// `OD-HOST-016` decided they address a fact and answer no question a reader can ask, and that
/// dropping the subject in particular is what makes this honest at rule grain, since a finding's
/// subject is not always the subject its rule read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FactRead
{
    /// The capability family the rule asked for, as its real wire identifier
    /// (`nomos.cap.syntax.items`, and so on) -- never a label invented here.
    pub capability: String,
    /// The provider whose offer this read was addressed to, as its real wire identifier.
    pub provider: String,
    /// The contract version that provider's offer names.
    pub provider_version: ContractVersion,
    /// How the read came back: `Materialized`, `Absent`, `Superseded` or `Degraded`. A miss is
    /// carried as readily as a hit, because a rule whose optional policy read missed was judged
    /// against a compiled-in default and `OD-HOST-016` measured this as the only place that
    /// says so.
    pub outcome: &'static str,
    /// The applicability behind a `Degraded` read, and `None` for every other outcome -- the
    /// reason the read did not answer, kept beside the word rather than folded into it.
    pub degraded_by: Option<&'static str>,
    /// The declared completeness of the fact that answered, or `None` for a read that did not
    /// answer.
    pub guarantee: Option<Guarantee>,
    /// How the fact that answered was come by, or `None` for a read that did not answer.
    pub evidence: Option<EvidenceClass>,
    /// How many reads stood behind this tuple. A rule reading one family over a thousand
    /// sources reduces to one tuple carrying a thousand, which is the whole reason the
    /// reduction exists.
    pub reads: usize,
}

impl FactRead
{
    /// `read` in the shape an editor receives, field for field.
    #[must_use]
    pub(crate) fn Of(read: &nomos_check_orchestration::FactRead) -> Self
    {
        return Self {
            capability: read.capability.As_Str().to_owned(),
            provider: read.provider.As_Str().to_owned(),
            provider_version: read.provider_version,
            outcome: Outcome_Label(read.outcome),
            degraded_by: Degraded_By(read.outcome),
            guarantee: read.guarantee,
            evidence: read.evidence,
            reads: read.reads,
        };
    }
}

/// `outcome`'s own variant name, since [`ReadOutcome`] carries neither a label nor a
/// `Serialize` of its own.
fn Outcome_Label(outcome: ReadOutcome) -> &'static str
{
    return match outcome
    {
        ReadOutcome::Materialized => "Materialized",
        ReadOutcome::Absent => "Absent",
        ReadOutcome::Superseded => "Superseded",
        ReadOutcome::Degraded(_) => "Degraded",
    };
}

/// The applicability a degraded read carries, and `None` for an outcome that is not degraded.
///
/// Beside [`Outcome_Label`] rather than inside it: a composite word would be a spelling this
/// crate invented, and `Applicability::Label` is the one this workspace already publishes for
/// the same vocabulary on [`crate::WalkOutward::applicability`].
fn Degraded_By(outcome: ReadOutcome) -> Option<&'static str>
{
    return match outcome
    {
        ReadOutcome::Degraded(applicability) => Some(applicability.Label()),
        ReadOutcome::Materialized | ReadOutcome::Absent | ReadOutcome::Superseded => None,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Applicability, Assurance, CapabilityId, FactVariant, IncrementalGranularity, ProviderId};

    /// The capability and provider every fixture read below is addressed to.
    ///
    /// Deliberately not a real capability this workspace offers. A capability id is an
    /// agreement between the crates that spell it, and
    /// `Test_A_Capability_Id_Should_Be_Written_In_One_Crate` in `tests/contract` refuses a
    /// second crate writing one -- which a fixture here would be, for a string that is only
    /// ever a placeholder to this module.
    const FIXTURE_CAPABILITY: &str = "nomos.cap.test.projection";
    const FIXTURE_PROVIDER: &str = "nomos.provider.test.projection";

    /// The contract version every fixture read below names. Nothing asserts on the number
    /// itself -- it exists so each fixture carries a real version rather than one that means
    /// only itself.
    const FIXTURE_VERSION: ContractVersion = ContractVersion::New(1, 0);

    /// How many reads the fixture tuple stands for.
    const FIXTURE_READS: usize = 3;

    #[test]
    fn Test_Of_Should_Carry_An_Answered_Read_With_Its_Guarantee_And_Evidence()
    {
        let answered = nomos_check_orchestration::FactRead {
            capability: CapabilityId::New(FIXTURE_CAPABILITY),
            provider: ProviderId::New(FIXTURE_PROVIDER),
            provider_version: FIXTURE_VERSION,
            outcome: ReadOutcome::Materialized,
            guarantee: Some(Fixture_Guarantee()),
            evidence: Some(EvidenceClass::Derived),
            reads: FIXTURE_READS,
        };

        let carried = FactRead::Of(&answered);

        assert_eq!(carried.capability, FIXTURE_CAPABILITY);
        assert_eq!(carried.provider, FIXTURE_PROVIDER);
        assert_eq!(carried.provider_version, FIXTURE_VERSION);
        assert_eq!(carried.outcome, "Materialized");
        assert_eq!(carried.degraded_by, None);
        assert_eq!(carried.guarantee, Some(Fixture_Guarantee()));
        assert_eq!(carried.evidence, Some(EvidenceClass::Derived));
        assert_eq!(carried.reads, FIXTURE_READS);
    }

    /// A miss is the case the whole tuple exists to make visible, so the reason it missed must
    /// survive the projection rather than being flattened into the word `Degraded`.
    #[test]
    fn Test_Of_Should_Carry_Why_A_Degraded_Read_Did_Not_Answer()
    {
        let missed = Read_With(ReadOutcome::Degraded(Applicability::DependencyUnavailable));

        let carried = FactRead::Of(&missed);

        assert_eq!(carried.outcome, "Degraded");
        assert_eq!(carried.degraded_by, Some(Applicability::DependencyUnavailable.Label()));
        assert_eq!(carried.guarantee, None, "a read that did not answer has no answering fact");
        assert_eq!(carried.evidence, None, "a read that did not answer has no answering fact");
    }

    /// The three outcomes that are not `Degraded` must each keep their own word, or a reader
    /// cannot tell a fact that was never written from one a later generation superseded.
    #[test]
    fn Test_Of_Should_Keep_Every_Outcome_That_Is_Not_Degraded_Apart()
    {
        let absent = FactRead::Of(&Read_With(ReadOutcome::Absent));
        let superseded = FactRead::Of(&Read_With(ReadOutcome::Superseded));
        let materialized = FactRead::Of(&Read_With(ReadOutcome::Materialized));

        assert_eq!(absent.outcome, "Absent");
        assert_eq!(superseded.outcome, "Superseded");
        assert_eq!(materialized.outcome, "Materialized");
        assert_eq!(absent.degraded_by, None);
        assert_eq!(superseded.degraded_by, None);
        assert_eq!(materialized.degraded_by, None);
    }

    /// A tuple whose only interesting field is its outcome.
    fn Read_With(outcome: ReadOutcome) -> nomos_check_orchestration::FactRead
    {
        return nomos_check_orchestration::FactRead {
            capability: CapabilityId::New(FIXTURE_CAPABILITY),
            provider: ProviderId::New(FIXTURE_PROVIDER),
            provider_version: FIXTURE_VERSION,
            outcome,
            guarantee: None,
            evidence: None,
            reads: 1,
        };
    }

    /// A guarantee whose four axes differ enough that carrying the wrong one would show.
    fn Fixture_Guarantee() -> Guarantee
    {
        return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    }
}
