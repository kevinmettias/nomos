//! The three things `OD-TRACE-001` said the guard asserts, against the entries this
//! repository has really committed.

use crate::predicates::{Divergences_With_No_Record, Unresolved_Records, Unresolved_Sites};
use crate::registry::Committed;
use nomos_contract_tests::Workspace;

/// How few assessments this repository may hold.
///
/// A floor, not a count, and `OD-SPEC-007` settled the trade for the identical case one
/// directory over. Adding an assessment costs no edit here, which is the whole point —
/// two people assessing two requirements must not collide on a third file. Removing one
/// costs lowering this, which is the deliberate step that keeps an entry from being quietly
/// deleted to make a divergence disappear.
///
/// Its guarantee is exact only while the count sits on it. Once the registry has grown
/// above, a deletion inside the slack is caught by nothing here — the same slack
/// `OD-SPEC-007` accepted, for the same reason: an upper bound would reintroduce the shared
/// edit on an unpredictable schedule.
pub(crate) const FEWEST_ASSESSMENTS: usize = 4;

/// Every entry names a site that exists in the workspace.
#[test]
fn Test_Every_Assessment_Should_Name_A_Site_That_Exists()
{
    let root = Workspace::Workspace_Root();
    let assessments = Committed(&root);

    assert!(
        !assessments.is_empty(),
        "no assessment was read from the registry, so every comparison in this suite would \
         pass over an empty set — reporting that the registry is sound because nothing \
         contradicted it, which is the shape of defect OD-TRACE-001 is about"
    );

    let missing = Unresolved_Sites(&root, &assessments);

    assert!(
        missing.is_empty(),
        "these assessments name a site that is not there: {missing:#?}.\n\
         Either the code moved, in which case update the entry, or the thing the verdict \
         was about is gone, in which case the verdict is about nothing and the entry has \
         to be re-authored against what replaced it."
    );
}

/// Every entry that names a record names one that exists and is registered.
///
/// Wider than `OD-TRACE-001` requires, which is only that a `Diverges` entry does. A `Met`
/// entry may omit a record; naming one that does not resolve is a different thing, and a
/// dangling citation is worth catching wherever it appears. The narrower obligation — that
/// a divergence names a record *at all* — is
/// [`Test_Every_Divergence_Should_Name_A_Governing_Record`].
///
/// "Registered" is `OD-SPEC-007`'s sense: a document under `docs/records` is not governing
/// until a file under `crates/spec/nomos-spec-store/records/` says so. An entry citing an
/// unregistered document would be citing prose.
#[test]
fn Test_Every_Named_Record_Should_Exist_And_Be_Registered()
{
    let root = Workspace::Workspace_Root();
    let assessments = Committed(&root);

    let cited = assessments
        .iter()
        .filter(|assessment| return assessment.record.is_some())
        .count();
    assert!(
        cited > 0,
        "no committed assessment names a record, so this comparison read nothing"
    );

    let unresolved = Unresolved_Records(&root, &assessments);

    assert!(
        unresolved.is_empty(),
        "these assessments name a record that does not resolve: {unresolved:#?}.\n\
         A record is registered by its own file under crates/spec/nomos-spec-store/records/ \
         — OD-SPEC-007 — and writing the document alone does not make it governing."
    );
}

/// A divergence with no record is not a verdict.
///
/// **This assertion is vacuous today and that is stated rather than hidden.** Every
/// committed entry is `Met`, so the loop below runs over an empty set. The two divergences
/// `OD-TRACE-001` hand-audited — `WORK-LEDGER-005` dropping `StaleProbeArtifact` and
/// `WORK-LEDGER-001` dropping `priority` — cannot be entered yet precisely because neither
/// has a record saying why, which is `P10-LEDGER-CORPUS`'s subject and not this suite's.
///
/// What keeps the vacuity from being a hole is
/// [`crate::controls::Test_A_Divergence_With_No_Record_Should_Be_Refused`], which runs this
/// same function over a synthetic entry. It calls `Divergences_With_No_Record`, not a copy
/// of it, so the control exercises the guard rather than something written beside it.
#[test]
fn Test_Every_Divergence_Should_Name_A_Governing_Record()
{
    let root = Workspace::Workspace_Root();
    let assessments = Committed(&root);
    let unreasoned = Divergences_With_No_Record(&assessments);

    assert!(
        unreasoned.is_empty(),
        "these assessments depart from a requirement and say nothing about why: \
         {unreasoned:#?}.\n\
         A divergence with no record is the state OD-TRACE-001 exists to end. Write the \
         record, register it, and name it here."
    );
}

/// The assessed set only grows.
#[test]
fn Test_The_Assessed_Set_Should_Not_Shrink()
{
    use std::collections::BTreeSet;

    let root = Workspace::Workspace_Root();
    let assessments = Committed(&root);

    assert!(
        assessments.len() >= FEWEST_ASSESSMENTS,
        "{} requirements are assessed and FEWEST_ASSESSMENTS says at least \
         {FEWEST_ASSESSMENTS}.\n\
         An entry cannot be quietly deleted to make a divergence disappear. If a removal \
         is deliberate, lower the floor in the same commit and say why in the message.",
        assessments.len()
    );

    let distinct: BTreeSet<&str> = assessments
        .iter()
        .map(|assessment| return assessment.requirement.as_str())
        .collect();
    assert_eq!(
        distinct.len(),
        assessments.len(),
        "two entries assess one requirement, so the floor counts a requirement twice"
    );
}

/// `CHK-003`'s verdict stops living in prose.
///
/// The entry this whole item exists to make readable. `OD-CONTRACTS-002` argued that
/// `CHK-003` binds this build, decided the seventh reporting category, and then had to say
/// **Met** in its own prose because the registry had no home yet — its territory was two
/// source files, two surface snapshots and one record, and territory cannot be widened
/// mid-claim. This is the entry that replaces that sentence.
#[test]
fn Test_CHK_003_Should_Be_Assessed_Met_By_The_Record_That_Decided_It()
{
    use crate::assessment::Verdict;

    let root = Workspace::Workspace_Root();
    let assessments = Committed(&root);
    let entry = assessments
        .iter()
        .find(|assessment| return assessment.requirement == "CHK-003")
        .expect("CHK-003 must be assessed; OD-CONTRACTS-002 says Met in prose until it is");

    assert_eq!(entry.verdict, Verdict::Met);
    assert_eq!(entry.record.as_deref(), Some("OD-CONTRACTS-002"));
    for file in [
        "crates/contracts/nomos-contracts/src/finding/applicability.rs",
        "crates/contracts/nomos-contracts/src/finding/evidence.rs",
    ]
    {
        assert!(
            entry.sites.iter().any(|site| return site.path == file),
            "CHK-003's entry must name a site in {file}; the seventh reporting category is \
             a variant in one and the evidence class it must not be confused with is in \
             the other"
        );
    }
}
