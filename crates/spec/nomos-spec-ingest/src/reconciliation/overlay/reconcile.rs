//! Comparing every v14 identifier's statement against what v15 carries.

use crate::Artifact;
use crate::Disposition;
use crate::IdentifierOutcome;
use crate::ReconciliationReport;
use nomos_spec_model::ContentHash;
use std::collections::BTreeMap;

/// Compares every v14 identifier against what v15 carries.
#[must_use]
pub fn Reconcile(v14: &[Artifact], v15: &BTreeMap<String, String>) -> ReconciliationReport
{
    let mut outcomes: Vec<IdentifierOutcome> = Vec::new();

    for artifact in v14
    {
        outcomes.push(IdentifierOutcome {
            id: artifact.id.clone(),
            family: artifact.family,
            disposition: Judged(artifact, v15),
        });
    }

    outcomes.sort_by(|left, right| return left.id.cmp(&right.id));

    return ReconciliationReport { outcomes };
}

/// What became of one v14 identifier in v15.
///
/// Both sides are normalized, because v14 stores a statement as folded YAML and v15 stores
/// it as one line of prose. Comparing the raw text would report every identifier reworded
/// on a difference no reader could see.
fn Judged(artifact: &Artifact, v15: &BTreeMap<String, String>) -> Disposition
{
    let mine = ContentHash::Of_Normalized(&artifact.statement);
    let Some(text) = v15.get(&artifact.id)
    else
    {
        return Disposition::Absent;
    };

    let theirs = ContentHash::Of_Normalized(text);
    if theirs == mine
    {
        return Disposition::Preserved;
    }

    return Disposition::Reworded {
        v14: mine,
        v15: theirs,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Family;
    use super::super::artifact::Parse_Artifact;

    const ARTIFACT: &str = "\u{feff}---\nid: MODEL-001\nkind: requirement\n\
                            statement: MODEL-001 Artifact represents persisted objects.\n\
                            ---\n\n# MODEL-001 - Artifact\n";

    #[test]
    fn Test_A_Preserved_Identifier_Should_Reconcile()
    {
        let v14 = vec![Parse_Artifact(ARTIFACT, Family::Requirement).expect("reads")];
        let mut v15 = BTreeMap::new();
        v15.insert(
            "MODEL-001".to_owned(),
            "MODEL-001  Artifact represents\npersisted objects.".to_owned(),
        );

        let report = Reconcile(&v14, &v15);

        assert_eq!(report.Preserved_In(Family::Requirement), 1, "{}", report.Summary());
        assert!(report.Absent().is_empty());
    }

    #[test]
    fn Test_A_Missing_Identifier_Should_Be_Absent_Not_Reworded()
    {
        let v14 = vec![Parse_Artifact(ARTIFACT, Family::Requirement).expect("reads")];

        let report = Reconcile(&v14, &BTreeMap::new());

        assert_eq!(report.Absent_In(Family::Requirement), vec!["MODEL-001"]);
        assert!(report.Reworded().is_empty(), "a disappearance was reported as an edit");
        assert!(report.Summary().contains("MODEL-001"), "the summary counts without naming");
    }

    #[test]
    fn Test_A_Changed_Statement_Should_Carry_Both_Hashes()
    {
        let v14 = vec![Parse_Artifact(ARTIFACT, Family::Requirement).expect("reads")];
        let mut v15 = BTreeMap::new();
        v15.insert("MODEL-001".to_owned(), "MODEL-001 Something else entirely.".to_owned());

        let report = Reconcile(&v14, &v15);

        let reworded = report.Reworded();
        let Some(outcome) = reworded.first()
        else
        {
            // v15 was given MODEL-001 with different text, so an empty `Reworded()` means the
            // change went unreported entirely — a worse failure than the disposition being
            // wrong, and one the `matches!` below cannot reach. The summary is printed
            // because an empty list says nothing about what the statement was reported as.
            panic!("a changed statement must be reported: {}", report.Summary());
        };
        assert!(matches!(outcome.disposition, Disposition::Reworded { .. }));
    }
}
