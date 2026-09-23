use crate::Digest_Of_Parts;
use nomos_contracts::{Digest128, Finding};
use serde::{Deserialize, Serialize};

/// Which recurring violation a finding is an occurrence of, so that two observations taken at
/// two revisions can be asked whether they are the same one.
///
/// `OD-GATE-030` names three parts that would make a baseline's continuity provable and calls
/// this the second: *enough to say that the occurrence seen now is the one seen at adoption,
/// rather than another occurrence of the same rule at the same subject.* The record also says
/// why [`FindingOccurrenceId`](super::FindingOccurrenceId) is not that thing, and this type
/// exists because that refusal is correct rather than despite it.
///
/// # What it is made of, and why each part survives a revision
///
/// [`Finding::rule`], [`Finding::subject`] and [`Finding::summary`], length-prefixed by
/// [`Digest_Of_Parts`] so a boundary between two of them cannot be moved without moving the
/// digest.
///
/// - `rule` is a registered identifier. It changes when a rule is renamed, and a renamed rule
///   is a different rule to every declared entry in the system, so the lineage moving with it
///   agrees with what a suppression and a baseline entry already do.
/// - `subject` is a digest of a normalized path or of a qualified name, and
///   [`Finding::subject`]'s own doc states the property this relies on: it is deliberately
///   coarse so that a tolerated entry survives an unrelated edit above it. An edit elsewhere in
///   the file does not move it, and neither does moving the occurrence within the file.
/// - `summary` is the rule's own sentence about *what* is wrong. It is the only material left
///   that distinguishes two occurrences of one rule inside one subject, and for the rules that
///   describe a violation — the naming family says which name, the mirror family which mirror —
///   it moves when the violation changes and not when the file is reformatted.
///
/// # The part of that claim that was measured and did not hold everywhere
///
/// Measured 2026-09-22 over `nomos-rules`, `crates/capabilities` and `crates/languages`: of 61
/// `summary` constructions, **14 spell a position into the sentence** — `{path} line {n}`,
/// `{path}:{n}`, or a location string outright — across thirteen rule modules, the text-scanner
/// family among them. A lineage over one of those summaries moves when the line moves, which is
/// exactly the expiry [`FindingOccurrenceId`](super::FindingOccurrenceId)'s own doc refuses.
///
/// That is stated here rather than repaired by stripping numbers out of the sentence, because a
/// normalization that guessed which digits were a position would be the approximate answer
/// `OD-GATE-030` refuses at the one point a repository is relying on the gate to be exact. What
/// it costs is bounded and is spent in the safe direction: such a lineage does not survive a
/// reformatting commit, so its old identity is last seen and a new one arrives, and a consumer
/// asking [`OccurrenceTally`](super::OccurrenceTally) what that establishes is told *nothing* —
/// not that the occurrence came back. The remedy is for such a rule to put the position in
/// [`Finding::locations`], where it already belongs, and leave the summary describing the
/// violation.
///
/// # What is deliberately left out
///
/// [`Finding::locations`] and [`Finding::subject_name`], because both move when a line moves and
/// a lineage that expired on every reformatting commit would answer nothing this type exists to
/// answer.
///
/// [`Finding::applicability`], [`Finding::evidence`] and [`Finding::gate`], for the reason
/// [`FindingOccurrenceId`](super::FindingOccurrenceId) excludes them: a finding moving between
/// gate buckets is the continuity a comparison exists to observe, and an identity that moved
/// with its treatment would report that move as one lineage ending and another beginning.
///
/// [`Finding::address`], because it is the composite [`Finding::subject`]'s digest was taken
/// over — its own doc says so — and a part that varies only when another part already varies
/// adds no discrimination while adding a second thing to get wrong.
///
/// # How this differs from `FindingOccurrenceId`, which is left alone
///
/// They answer two different questions and neither can be had from the other.
///
/// [`FindingOccurrenceId`](super::FindingOccurrenceId) answers *which of the occurrences in this
/// one pinned result is this one*, and it takes the locations because two violations of one rule
/// in one file differ by nothing else. It is injective over a single run's population, measured
/// so, and [`Occurrence_Collisions_In`](super::Occurrence_Collisions_In) keeps that checked.
///
/// This one answers *which recurring violation is this an occurrence of*, across revisions, and
/// it pays for surviving a revision with the discrimination it gives up: **two occurrences of
/// one rule in one subject whose summaries agree are one lineage.** That is not a defect to be
/// detected and refused the way a collision in the other identity is — it is the property, and
/// `OD-GATE-030`'s counting bound is what answers the question it cannot: how many occurrences a
/// scope holds, against how many its entry accepted.
///
/// So a caller keying a map by this type must expect several findings per key, and a caller that
/// needs one finding per key wants the other type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OccurrenceLineageId(Digest128);

impl OccurrenceLineageId
{
    /// The lineage `finding` is an occurrence of.
    ///
    /// A projection rather than a field on [`Finding`], the same choice and for the same reason
    /// [`FindingOccurrenceId::Of`](super::FindingOccurrenceId::Of) made: an identity a caller can
    /// author is an identity a caller can get wrong, and no construction site in the workspace
    /// changes to gain one.
    #[must_use]
    pub fn Of(finding: &Finding) -> Self
    {
        let subject = finding.subject.Digest();
        let parts: [&[u8]; 3] = [finding.rule.As_Str().as_bytes(), subject.Bytes(), finding.summary.as_bytes()];

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
    use super::OccurrenceLineageId;
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

    /// The subject every finding below is attributed to.
    const BASE_SUBJECT_SEED: u8 = 3;

    /// A second subject, so "the subject is load-bearing" has something to differ from.
    const OTHER_SUBJECT_SEED: u8 = 9;

    /// The whole point: the same violation seen at a different line is one lineage.
    ///
    /// This is the edit a reformatting commit makes, and the one
    /// [`super::super::FindingOccurrenceId`] deliberately does not survive.
    #[test]
    fn Test_A_Finding_That_Only_Moved_Should_Keep_Its_Lineage()
    {
        let before = A_Finding();
        let after = Finding { locations: vec!["src/lib.rs:400".to_owned()], ..A_Finding() };

        assert_eq!(
            OccurrenceLineageId::Of(&before),
            OccurrenceLineageId::Of(&after),
            "a lineage that expired when a line moved would expire on the next reformatting commit"
        );
    }

    /// An edit elsewhere in the file can also change how many places a finding names, and that
    /// is still the same violation.
    #[test]
    fn Test_A_Finding_Whose_Location_Count_Changed_Should_Keep_Its_Lineage()
    {
        let one = A_Finding();
        let two = Finding { locations: vec!["src/lib.rs:12".to_owned(), "src/lib.rs:40".to_owned()], ..A_Finding() };
        let none = Finding { locations: Vec::new(), ..A_Finding() };

        let lineage = OccurrenceLineageId::Of(&one);

        assert_eq!(OccurrenceLineageId::Of(&two), lineage);
        assert_eq!(OccurrenceLineageId::Of(&none), lineage);
    }

    /// `subject_name` is display material and frequently a path and a line, so it moves for the
    /// same reason the locations do.
    #[test]
    fn Test_A_Finding_Shown_Under_A_Different_Name_Should_Keep_Its_Lineage()
    {
        let shown_one_way = Finding { subject_name: "src/lib.rs:12".to_owned(), ..A_Finding() };
        let shown_another = Finding { subject_name: "src/lib.rs:40".to_owned(), ..A_Finding() };

        assert_eq!(OccurrenceLineageId::Of(&shown_one_way), OccurrenceLineageId::Of(&shown_another));
    }

    /// Treatment is excluded, which is what lets a lineage be followed from one bucket to
    /// another.
    #[test]
    fn Test_A_Finding_Differing_Only_In_Its_Treatment_Should_Keep_Its_Lineage()
    {
        let lineage = OccurrenceLineageId::Of(&A_Finding());

        let advisory = Finding { gate: GateCategory::Advisory, ..A_Finding() };
        let partial = Finding { applicability: Applicability::PartiallySupported, ..A_Finding() };
        let asserted = Finding { evidence: EvidenceClass::HumanAsserted, ..A_Finding() };

        assert_eq!(OccurrenceLineageId::Of(&advisory), lineage, "gate category");
        assert_eq!(OccurrenceLineageId::Of(&partial), lineage, "applicability");
        assert_eq!(OccurrenceLineageId::Of(&asserted), lineage, "evidence class");
    }

    /// Each of the three parts is load-bearing, checked one at a time. A part that could be
    /// dropped without changing an answer is not in the identity, whatever the derivation says.
    #[test]
    fn Test_Each_Lineage_Component_Should_Change_The_Lineage()
    {
        let lineage = OccurrenceLineageId::Of(&A_Finding());

        let other_rule = Finding { rule: RuleId::New("one-public-type-per-file"), ..A_Finding() };
        let other_subject = Finding { subject: Subject_Of(OTHER_SUBJECT_SEED), ..A_Finding() };
        let other_summary = Finding { summary: "something else entirely".to_owned(), ..A_Finding() };

        assert_ne!(OccurrenceLineageId::Of(&other_rule), lineage, "rule");
        assert_ne!(OccurrenceLineageId::Of(&other_subject), lineage, "subject");
        assert_ne!(OccurrenceLineageId::Of(&other_summary), lineage, "summary");
    }

    /// The framing property at this type's own boundaries: without length prefixes a rule name
    /// ending in one string and a summary beginning with another would hash alike.
    #[test]
    fn Test_A_Boundary_Between_The_Parts_Should_Be_Significant()
    {
        let split_early = Finding { rule: RuleId::New("ab"), summary: "c".to_owned(), ..A_Finding() };
        let split_late = Finding { rule: RuleId::New("a"), summary: "bc".to_owned(), ..A_Finding() };

        assert_ne!(OccurrenceLineageId::Of(&split_early), OccurrenceLineageId::Of(&split_late));
    }

    /// The cost this type's own doc states out loud, asserted rather than left for a later
    /// reader to meet as a surprise: two occurrences of one rule in one subject that describe
    /// themselves the same way are one lineage, and the counting bound is what answers how many
    /// there are.
    #[test]
    fn Test_Two_Occurrences_Described_Alike_Should_Share_One_Lineage()
    {
        let first = A_Finding();
        let second = Finding { locations: vec!["src/lib.rs:40".to_owned()], ..A_Finding() };

        assert_eq!(
            OccurrenceLineageId::Of(&first),
            OccurrenceLineageId::Of(&second),
            "this is the property, not a collision to be detected: a lineage discriminates \
             violations, and OD-GATE-030's count discriminates populations"
        );
    }

    /// A pinned vector. If this changes, every lineage ever derived has silently changed
    /// meaning, and the failure that presents is a record reporting every occurrence as gone and
    /// every occurrence as arrived between two runs that agree.
    #[test]
    fn Test_The_Lineage_Should_Match_Its_Pinned_Vector()
    {
        assert_eq!(OccurrenceLineageId::Of(&A_Finding()).Digest().to_string(), "4bec1b8e3c4ed54b2e18c4b9753d0505");
    }

    fn Subject_Of(fill: u8) -> SubjectId
    {
        return SubjectId::From_Digest(Digest128::From_Bytes([fill; Digest128::BYTE_LENGTH]));
    }

    /// One finding every test above varies a single field of.
    fn A_Finding() -> Finding
    {
        return Finding {
            address: None,
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
