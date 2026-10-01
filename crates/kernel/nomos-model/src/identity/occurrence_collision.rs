use super::FindingOccurrenceId;
use nomos_contracts::Finding;
use std::collections::BTreeMap;

/// Two findings in one population that share a [`FindingOccurrenceId`].
///
/// Both findings are carried rather than a count or a key, because the only useful thing to do
/// with a collision is look at the two things that collided. A report saying "some identity
/// occurred twice" leaves a reader to re-derive which two, from a population that may hold
/// hundreds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OccurrenceCollision
{
    /// The identity both findings derived to.
    pub occurrence: FindingOccurrenceId,
    /// The finding that reached that identity first, in the order the population was given.
    pub first: Finding,
    /// The finding that reached it again.
    pub second: Finding,
}

/// Every pair in `findings` that shares one [`FindingOccurrenceId`].
///
/// Empty means the identity is injective over this population — which is the answer a caller
/// about to key a map by it needs, and the one it must ask *before* building the map rather than
/// discover from it.
///
/// # Why a caller cannot just use a map and see
///
/// Because `BTreeMap::insert` returns the displaced value and almost every caller drops it, so
/// the discovery mechanism for a lost finding is an ignored return value. That is precisely how
/// 103 of this repository's 253 findings were being discarded before comparison began: not by a
/// decision anybody made, but by an insert nobody read. A collision is a defect in the identity
/// material rather than a condition the caller can handle, so it is reported by name and early,
/// not absorbed.
///
/// # What it does when three collide
///
/// Reports two collisions, each naming the first-seen finding and the one that collided with it.
/// Pairing every later finding against the first, rather than against its immediate predecessor,
/// keeps the report readable when a whole family shares an identity: the repeated element is the
/// one to look at, and it appears in every line.
///
/// The order of `findings` therefore decides which finding is `first`. That is deliberate and it
/// is the caller's order, not an ordering this function invents — a function that sorted the
/// population first would report a different pair than the one the caller is about to build.
#[must_use]
pub fn Occurrence_Collisions_In(findings: &[&Finding]) -> Vec<OccurrenceCollision>
{
    let mut seen: BTreeMap<FindingOccurrenceId, &Finding> = BTreeMap::new();
    let mut collisions = Vec::new();

    for finding in findings
    {
        Absorb_Finding(&mut seen, &mut collisions, finding);
    }

    return collisions;
}

/// Fold one finding into a population being scanned for collisions.
///
/// A finding whose identity has already been seen is reported against the one that reached it
/// first; any other is recorded so that a later finding can be reported against it. This is the
/// loop body of [`Occurrence_Collisions_In`], named so that the policy it encodes — report, do not
/// absorb — is readable without the iteration around it.
fn Absorb_Finding<'a>(
    seen: &mut BTreeMap<FindingOccurrenceId, &'a Finding>,
    collisions: &mut Vec<OccurrenceCollision>,
    finding: &'a Finding,
)
{
    let occurrence = FindingOccurrenceId::Of(finding);

    if let Some(first) = seen.get(&occurrence)
    {
        collisions.push(OccurrenceCollision {
            occurrence,
            first: (*first).clone(),
            second: finding.clone(),
        });

        return;
    }

    seen.insert(occurrence, finding);
}

#[cfg(test)]
mod tests
{
    use super::{Occurrence_Collisions_In, OccurrenceCollision};
    use crate::FindingOccurrenceId;
    use nomos_contracts::{
        Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId,
    };

    /// The subject seed every fixture finding is built from.
    const BASE_SUBJECT_SEED: u8 = 3;

    /// How many collisions three findings sharing one identity report: the second and the third
    /// each collide with the first, so two pairs name the finding that got there first.
    const COLLISIONS_AMONG_THREE_SHARING_ONE_IDENTITY: usize = 2;

    /// The population this repository actually has: many findings, no two alike.
    ///
    /// The vacuity half. Without it, a detector that returned an empty vector unconditionally
    /// would pass every other assertion here.
    #[test]
    fn Test_A_Population_With_No_Duplicate_Should_Report_No_Collision()
    {
        let first = A_Finding();
        let second = Another_Occurrence();
        let third = Finding { rule: RuleId::New("one-public-type-per-file"), ..A_Finding() };

        assert_eq!(Occurrence_Collisions_In(&[&first, &second, &third]), Vec::new());
    }

    /// The negative control, and the assertion that the detector can fail at all.
    ///
    /// Two findings agreeing on rule, subject, summary and locations are one identity — which the
    /// identity's own documentation states as the limit of its injectivity. This is that limit
    /// being reported rather than silently absorbed.
    #[test]
    fn Test_Two_Indistinguishable_Findings_Should_Report_One_Collision_Naming_Both()
    {
        let first = A_Finding();
        let second = An_Indistinguishable_Finding();

        let collisions = Occurrence_Collisions_In(&[&first, &second]);

        assert_eq!(collisions.len(), 1, "{collisions:?}");

        let collision = collisions.first().expect("asserted len 1 above");

        assert_eq!(collision.occurrence, FindingOccurrenceId::Of(&first));
        assert_eq!(collision.first, first, "the collision must name the finding that got there first");
        assert_eq!(collision.second, second, "and the one that collided with it");
    }

    /// A collision among many is found, not drowned.
    ///
    /// The realistic shape: one duplicated pair inside a population that is otherwise clean. A
    /// detector that only worked on a population of two would pass the test above and nothing
    /// real.
    #[test]
    fn Test_A_Collision_Should_Be_Found_Among_Distinct_Findings()
    {
        let first = A_Finding();
        let distinct = Another_Occurrence();
        let other_rule = Finding { rule: RuleId::New("one-public-type-per-file"), ..A_Finding() };
        let duplicate = An_Indistinguishable_Finding();

        let collisions = Occurrence_Collisions_In(&[&first, &distinct, &other_rule, &duplicate]);

        assert_eq!(collisions.len(), 1, "{collisions:?}");
        assert_eq!(collisions.first().expect("asserted len 1 above").second, duplicate);
    }

    /// Three sharing one identity is two collisions, each naming the first.
    ///
    /// Stated as a test because the alternative pairing — each against its predecessor — is just
    /// as defensible and produces different output, so which one this is must be pinned rather
    /// than left to whoever reads the loop next.
    #[test]
    fn Test_Three_Colliding_Findings_Should_Report_Two_Collisions_Against_The_First()
    {
        let first = A_Finding();
        let second = An_Indistinguishable_Finding();
        let third = Finding { applicability: Applicability::PartiallySupported, ..A_Finding() };

        let collisions = Occurrence_Collisions_In(&[&first, &second, &third]);

        assert_eq!(collisions.len(), COLLISIONS_AMONG_THREE_SHARING_ONE_IDENTITY, "{collisions:?}");

        for collision in &collisions
        {
            assert_eq!(collision.first, first, "every collision names the finding that got there first");
        }
    }

    /// An empty population has no collisions, and says so without panicking.
    #[test]
    fn Test_An_Empty_Population_Should_Report_No_Collision()
    {
        assert_eq!(Occurrence_Collisions_In(&[]), Vec::<OccurrenceCollision>::new());
    }

    fn A_Finding() -> Finding
    {
        return Finding {
            address: None,
            rule: RuleId::New("file-name-matches-declared-type"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([BASE_SUBJECT_SEED; Digest128::BYTE_LENGTH])),
            subject_name: "src/lib.rs:12".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "the file names no type it declares".to_owned(),
            locations: vec!["src/lib.rs:12".to_owned()],
        };
    }

    /// The same rule and subject, at a different place: distinct occurrences, no collision.
    fn Another_Occurrence() -> Finding
    {
        return Finding { locations: vec!["src/lib.rs:40".to_owned()], ..A_Finding() };
    }

    /// A finding indistinguishable from `A_Finding` in every identity component.
    ///
    /// Differs in treatment alone, which is excluded from the identity by construction — so this
    /// is the shape a real collision takes rather than a contrived duplicate.
    fn An_Indistinguishable_Finding() -> Finding
    {
        return Finding { gate: GateCategory::Advisory, ..A_Finding() };
    }
}
