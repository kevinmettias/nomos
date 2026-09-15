use crate::Digest_Of_Parts;
use nomos_contracts::{Digest128, Finding};
use serde::{Deserialize, Serialize};

/// Which concrete occurrence of a violation a finding is, within one pinned result.
///
/// A rule emits one finding per occurrence — rules push inside per-line loops, each finding
/// carrying a single location — while [`Finding::subject`] is deliberately the *file*, coarse
/// enough that a tolerated entry survives an unrelated edit above it. So five violations in one
/// file are five findings that agree on `rule` and `subject`, and before this type nothing in
/// the system could tell them apart. Any consumer keying them on that pair kept one and
/// discarded four, which on this repository's own tree discarded 103 of 253.
///
/// # A projection, never a stored field
///
/// [`Of`](FindingOccurrenceId::Of) reads a finding and returns its identity. It is not a field
/// on [`Finding`], so none of the workspace's finding constructions changes and no construction
/// site can set it wrong, forget it, or set it to something the rest of the finding contradicts.
/// An identity a caller can author is an identity a caller can get wrong.
///
/// # What it is made of, and what it must never be made of
///
/// `rule`, `subject`, `summary` and `locations` — *which violation instance occurred*.
///
/// Nothing describing a finding's **treatment** enters the derivation.
/// [`Finding::applicability`], [`Finding::evidence`] and [`Finding::gate`] are excluded by
/// construction, and a test holds them out. This is the whole point rather than an economy: a
/// finding moving from blocking to suppressed is the continuity a comparison exists to observe,
/// and an identity that moved with the treatment would report that move as one occurrence
/// vanishing and an unrelated one arriving — which is the failure the identity was introduced to
/// stop, reintroduced one layer down.
///
/// [`Finding::subject_name`] is excluded for a different reason. `OD-ANALYSIS-011` records that
/// it is what to *show a person* — frequently finer than the subject, sometimes a file and line
/// — and not identity. Including it would make an occurrence's identity move whenever a
/// message's rendering was improved. The cost of leaving it out is stated below rather than
/// hidden.
///
/// # Taking the locations is not "identity is a path and a line"
///
/// [`Finding::locations`]'s own doc says *never identity*, and `identity.rs` states the rule for
/// the whole system. Both are about **what a claim is attributed to**, and neither is bent here:
/// [`Finding::subject`] remains a digest, remains deliberately coarse, and remains what a
/// suppression or a baseline entry matches on, so a tolerated entry survives an unrelated edit
/// above it.
///
/// This identity answers the question *after* that one — given the subject, which occurrence
/// within it — and two violations of one rule in one file differ by nothing else, so the
/// location is the only occurrence-discriminating material there is. The cost is real and
/// bounded: an occurrence identity moves when the line moves. That is tolerable only because it
/// is scoped to one pinned result and is never used to match a tolerance across revisions, which
/// is why suppression and baseline keep their rule-and-subject addressing rather than adopting
/// this. A change that gave either of them occurrence scope would expire every tolerated entry
/// on the next reformatting commit, and would be that separate decision, not a consequence of
/// this one.
///
/// # Why the parts are framed
///
/// [`Digest_Of_Parts`] length-prefixes every part, so the boundaries between the rule, the
/// subject, the summary and each location are part of the digest. Without that framing a summary
/// ending in one string and a location beginning with another would hash identically to their
/// neighbours' split, and two genuinely different occurrences would collide. Concatenation is
/// not a serialization.
///
/// # How far the injectivity claim reaches
///
/// Stated narrowly, because the useful version of this claim is the one a later reader cannot
/// over-read:
///
/// **This identity is injective over the currently representable [`Finding`] model, and over
/// this repository's own findings it was measured so — 256 findings to 256 distinct identities
/// on 2026-09-14.** It is not injective by construction, and nothing here makes it so.
///
/// Two occurrences that agree on all four components are one identity. That is reachable today:
/// a rule that put its discriminating position only in `subject_name` and left `locations` empty
/// would emit two findings this type cannot separate. No registered rule does, and a future one
/// might. So the answer is not to trust the property — it is
/// [`Occurrence_Collisions_In`](super::Occurrence_Collisions_In), which reports such a pair by
/// name instead of letting a map insert silently keep the last. A model change that lets two
/// distinct occurrences share `rule`, `subject`, `summary` and `locations` must extend the
/// identity material *before* it is published, and the detector is what turns that from a thing
/// somebody has to remember into a thing that fails.
///
/// # This is not a lifecycle
///
/// A `FindingOccurrenceId` names one concrete occurrence **in one pinned result**.
/// [`IdentityTransitionKind`](super::IdentityTransitionKind) names a **relation between
/// identities across snapshots** — `ExactContinuity`, `ProbableMove`, `Recreated` and the rest.
/// Asking this type whether an occurrence persisted or recurred is asking the wrong type;
/// `OD-GATE-030` names the run history that question additionally needs, and names it as
/// unbuilt. A second lifecycle vocabulary beside the one this module already declares would be
/// exactly the duplicated authority this repository files records about.
///
/// It is also not a [`CompositeIdentity`](super::CompositeIdentity), which is built from
/// language, provider-native identity, qualified name, signature and a structural fingerprint —
/// components a finding does not have. Coupling the two would claim a mapping nothing has
/// justified.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FindingOccurrenceId(Digest128);

impl FindingOccurrenceId
{
    /// The identity of the occurrence `finding` reports.
    #[must_use]
    pub fn Of(finding: &Finding) -> Self
    {
        let subject = finding.subject.Digest();
        let mut parts: Vec<&[u8]> = Vec::new();

        parts.push(finding.rule.As_Str().as_bytes());
        parts.push(subject.Bytes());
        parts.push(finding.summary.as_bytes());

        for location in &finding.locations
        {
            parts.push(location.as_bytes());
        }

        return Self(Digest_Of_Parts(&parts));
    }

    /// The underlying digest.
    #[must_use]
    pub const fn Digest(&self) -> Digest128
    {
        return self.0;
    }
}

#[cfg(test)]
mod tests
{
    use super::FindingOccurrenceId;
    use nomos_contracts::{
        Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId,
    };

    /// The subject every test but one derives its finding from.
    const BASE_SUBJECT_SEED: u8 = 3;

    /// A subject distinct from [`BASE_SUBJECT_SEED`], so that "the subject is load-bearing" has a
    /// second value to compare against.
    const OTHER_SUBJECT_SEED: u8 = 9;

    #[test]
    fn Test_The_Same_Finding_Should_Yield_The_Same_Identity()
    {
        assert_eq!(FindingOccurrenceId::Of(&A_Finding()), FindingOccurrenceId::Of(&A_Finding()));
    }

    /// The reason this type exists. Two occurrences of one rule in one file differ only in where
    /// they are, and before this they were one thing to every consumer.
    #[test]
    fn Test_Two_Occurrences_In_One_Subject_Should_Have_Different_Identities()
    {
        let first = A_Finding();
        let second = Finding { locations: vec!["src/lib.rs:40".to_owned()], ..A_Finding() };

        assert_ne!(
            FindingOccurrenceId::Of(&first),
            FindingOccurrenceId::Of(&second),
            "two findings the same rule made about one subject must be two occurrences to \
             compare; keying them the same is what discarded 103 of 253 findings"
        );
    }

    /// Treatment is excluded, and this is the assertion that says so.
    ///
    /// A finding moving between gate buckets is the continuity a comparison exists to observe.
    /// An identity that moved with the bucket would report that move as one occurrence vanishing
    /// and another arriving, which is the defect this type was introduced to remove.
    #[test]
    fn Test_A_Finding_Differing_Only_In_Its_Treatment_Should_Have_One_Identity()
    {
        let blocking = A_Finding();
        let advisory = Finding { gate: GateCategory::Advisory, ..A_Finding() };
        let partial = Finding { applicability: Applicability::PartiallySupported, ..A_Finding() };
        let asserted = Finding { evidence: EvidenceClass::HumanAsserted, ..A_Finding() };

        let identity = FindingOccurrenceId::Of(&blocking);

        assert_eq!(FindingOccurrenceId::Of(&advisory), identity, "gate category must not move the identity");
        assert_eq!(FindingOccurrenceId::Of(&partial), identity, "applicability must not move the identity");
        assert_eq!(FindingOccurrenceId::Of(&asserted), identity, "evidence class must not move the identity");
    }

    /// Each of the four identity components is load-bearing, checked one at a time.
    ///
    /// A component that could be dropped without changing any answer is a component that is not
    /// in the identity, whatever the derivation says.
    #[test]
    fn Test_Each_Identity_Component_Should_Change_The_Identity()
    {
        let identity = FindingOccurrenceId::Of(&A_Finding());

        let other_rule = Finding { rule: RuleId::New("one-public-type-per-file"), ..A_Finding() };
        let other_subject = Finding { subject: Subject_Of(OTHER_SUBJECT_SEED), ..A_Finding() };
        let other_summary = Finding { summary: "something else entirely".to_owned(), ..A_Finding() };
        let other_locations = Finding { locations: vec!["src/other.rs:1".to_owned()], ..A_Finding() };

        assert_ne!(FindingOccurrenceId::Of(&other_rule), identity, "rule");
        assert_ne!(FindingOccurrenceId::Of(&other_subject), identity, "subject");
        assert_ne!(FindingOccurrenceId::Of(&other_summary), identity, "summary");
        assert_ne!(FindingOccurrenceId::Of(&other_locations), identity, "locations");
    }

    /// A second location is not free, and neither is dropping one.
    ///
    /// The locations are a sequence, so their count and their order are part of the occurrence.
    /// Hashing them as a joined string would lose both.
    #[test]
    fn Test_The_Location_Sequence_Should_Be_Significant()
    {
        let one = A_Finding();
        let two = Finding { locations: vec!["src/lib.rs:12".to_owned(), "src/lib.rs:40".to_owned()], ..A_Finding() };
        let reversed = Finding { locations: vec!["src/lib.rs:40".to_owned(), "src/lib.rs:12".to_owned()], ..A_Finding() };
        let none = Finding { locations: Vec::new(), ..A_Finding() };

        assert_ne!(FindingOccurrenceId::Of(&one), FindingOccurrenceId::Of(&two), "a second location is a different occurrence");
        assert_ne!(FindingOccurrenceId::Of(&two), FindingOccurrenceId::Of(&reversed), "location order is significant");
        assert_ne!(FindingOccurrenceId::Of(&one), FindingOccurrenceId::Of(&none), "having no location is not the same as having one");
    }

    /// The framing property, at this type's own boundaries rather than at [`Digest_Of_Parts`]'s.
    ///
    /// Without length prefixes a summary's tail and a location's head would be interchangeable,
    /// and two different occurrences would share an identity. `Digest_Of_Parts` tests this for
    /// itself; this asserts that the parts were actually handed over separately, which is the
    /// half a caller can get wrong.
    #[test]
    fn Test_A_Boundary_Between_Summary_And_Location_Should_Be_Significant()
    {
        let split_early = Finding { summary: "ab".to_owned(), locations: vec!["c".to_owned()], ..A_Finding() };
        let split_late = Finding { summary: "a".to_owned(), locations: vec!["bc".to_owned()], ..A_Finding() };

        assert_ne!(
            FindingOccurrenceId::Of(&split_early),
            FindingOccurrenceId::Of(&split_late),
            "part boundaries must be part of the digest, or two distinct occurrences collide"
        );
    }

    /// `subject_name` is excluded, deliberately and at a cost this states out loud.
    ///
    /// It is display material rather than identity (`OD-ANALYSIS-011`), so including it would
    /// move an occurrence's identity whenever a message's rendering improved. The price is that
    /// two occurrences distinguished *only* there are one identity — which is reachable, which
    /// is why `Occurrence_Collisions_In` exists, and which is asserted here rather than left for
    /// a later reader to discover as a surprise.
    #[test]
    fn Test_A_Finding_Differing_Only_In_Its_Subject_Name_Should_Have_One_Identity()
    {
        let shown_one_way = Finding { subject_name: "src/lib.rs:12".to_owned(), locations: Vec::new(), ..A_Finding() };
        let shown_another = Finding { subject_name: "src/lib.rs:40".to_owned(), locations: Vec::new(), ..A_Finding() };

        assert_eq!(
            FindingOccurrenceId::Of(&shown_one_way),
            FindingOccurrenceId::Of(&shown_another),
            "subject_name is display material and is not identity material; a rule that needs \
             to distinguish two occurrences must say so in its locations"
        );
    }

    /// A pinned vector.
    ///
    /// If this changes, every occurrence identity this system has ever derived has silently
    /// changed meaning. The failure that would otherwise present is a comparison reporting every
    /// finding as removed and every finding as added, between two runs that agree.
    #[test]
    fn Test_The_Identity_Should_Match_Its_Pinned_Vector()
    {
        assert_eq!(
            FindingOccurrenceId::Of(&A_Finding()).Digest().to_string(),
            "f4b03938e2ff3a4f9cfbabd3c49d6fe1"
        );
    }

    fn Subject_Of(fill: u8) -> SubjectId
    {
        return SubjectId::From_Digest(Digest128::From_Bytes([fill; Digest128::BYTE_LENGTH]));
    }

    /// One finding every test above varies a single field of.
    ///
    /// A base plus struct update, rather than a constructor taking each field, so that each test
    /// states the *one* difference it is about and no reader has to diff two argument lists to
    /// find it.
    fn A_Finding() -> Finding
    {
        return Finding {
            rule: RuleId::New("file-name-matches-declared-type"),
            subject: Subject_Of(BASE_SUBJECT_SEED),
            subject_name: "src/lib.rs:12".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "the file names no type it declares".to_owned(),
            locations: vec!["src/lib.rs:12".to_owned()],
        };
    }
}
