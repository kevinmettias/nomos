//! What one revision lost against the one before it.

use super::{Revision, RegressionReport, IngestError, Later, Relocations_Between, Census_Fillers, BTreeMap, DOMAIN_VOLUMES, MemberFate, Extract_Members, Judge_Member, RevisionFingerprint, PairChange, Walk_Revisions};

pub fn Regression_Between_Revisions(from: &Revision, to: &Revision) -> Result<RegressionReport, IngestError>
{
    let volumes = Volumes_Of(from)?;
    let earlier = from.Fingerprint()?;
    let later_print = to.Fingerprint()?;
    let pair = One_Pair(&earlier, &later_print)?;
    let later = Later::Read(&to.documents);
    let members = Judged_Members(&volumes, &later, &to.documents)?;

    return Ok(RegressionReport {
        from: from.label.clone(),
        to: to.label.clone(),
        documents: Relocations_Between(&pair, &earlier, &later_print),
        members,
        filler: Census_Fillers(&later),
    });
}

/// The domain volumes a revision carries, refusing one that carries none.
///
/// Without the refusal, a revision that never held a family reports every member of every
/// family gone — which reads as catastrophic loss rather than as the wrong input.
pub(super) fn Volumes_Of(from: &Revision) -> Result<BTreeMap<String, String>, IngestError>
{
    let volumes = from.Volumes();
    if volumes.is_empty()
    {
        return Err(IngestError::Parse(format!(
            "{} has no {DOMAIN_VOLUMES}, so there is no family to ask after. Refusing to \
             report every member gone from a revision that never carried one",
            from.label
        )));
    }

    return Ok(volumes);
}

/// Every member the earlier volumes declare, each with what became of it.
pub(super) fn Judged_Members(
    volumes: &BTreeMap<String, String>,
    later: &Later,
    documents: &BTreeMap<String, String>,
) -> Result<Vec<MemberFate>, IngestError>
{
    use nomos_spec_store::DocumentPath;

    let mut members = Vec::new();
    for (document, markdown) in volumes
    {
        for member in Extract_Members(DocumentPath(document), markdown)?
        {
            let judged = Judge_Member(&member, later, documents);
            members.push(judged);
        }
    }

    return Ok(members);
}

pub(super) fn One_Pair(
    from: &RevisionFingerprint,
    to: &RevisionFingerprint,
) -> Result<PairChange, IngestError>
{
    let walk = Walk_Revisions(&[from.clone(), to.clone()]);

    return walk.into_iter().next().ok_or_else(|| {
        return IngestError::Parse(format!(
            "{} and {} are not a pair the walk recognises",
            from.label, to.label
        ));
    });
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Fate;

    const CORE: &str = "# Core\n\n## 5. Canonical domain model\n\n\
                        | Model | Responsibility |\n| --- | --- |\n\
                        | WorkspaceContext | Repository, branch, configuration. |\n";

    #[test]
    fn Test_Regression_Between_Revisions_Should_Report_Documents_Members_And_Filler()
    {
        let to = Revision {
            label: "v15.0".to_owned(),
            documents: Documents(&[("a.md", "# A\n\nThe WorkspaceContext is discussed.\n")]),
        };

        let report = Regression_Between_Revisions(&From_Revision(), &to)
            .expect("From_Revision carries a domain volume, so Volumes_Of does not refuse it");

        assert_eq!(report.from, "v14.36");
        assert_eq!(report.to, "v15.0");
        assert_eq!(report.members.len(), 1);
        assert_eq!(report.members.first().expect("the assertion above confirms exactly one member").name, "WorkspaceContext");
        assert_eq!(
            report.members.first().expect("the assertion above confirms exactly one member").fate,
            Fate::Mentioned { documents: vec!["a.md".to_owned()] }
        );
        assert!(report.documents.appeared.contains(&"a.md".to_owned()));
    }

    fn From_Revision() -> Revision
    {
        return Revision {
            label: "v14.36".to_owned(),
            documents: Documents(&[(format!("{DOMAIN_VOLUMES}02-core-architecture.md").as_str(), CORE)]),
        };
    }

    #[test]
    fn Test_Volumes_Of_Should_Refuse_A_Revision_With_No_Domain_Volumes()
    {
        let revision = Revision {
            label: "v14.36".to_owned(),
            documents: Documents(&[("records/one.md", "# Record\n\nText.\n")]),
        };

        let refusal = Volumes_Of(&revision).expect_err("must refuse");

        assert!(matches!(refusal, IngestError::Parse(_)));
        assert!(format!("{refusal}").contains("no family to ask after"));
    }

    #[test]
    fn Test_Judged_Members_Should_Extract_And_Judge_Each_Domain_Model()
    {
        let mut volumes = BTreeMap::new();
        volumes.insert("02-core-architecture.md".to_owned(), CORE.to_owned());
        let to_documents = Documents(&[("a.md", "# A\n\nThe WorkspaceContext is discussed.\n")]);
        let later = Later::Read(&to_documents);

        let members = Judged_Members(&volumes, &later, &to_documents)
            .expect("CORE holds the well-formed domain-model table Extract_Members reads");

        assert_eq!(members.len(), 1);
        assert_eq!(members.first().expect("the assertion above confirms exactly one member").name, "WorkspaceContext");
        assert_eq!(
            members.first().expect("the assertion above confirms exactly one member").fate,
            Fate::Mentioned { documents: vec!["a.md".to_owned()] }
        );
    }

    #[test]
    fn Test_One_Pair_Should_Find_The_Single_Adjacent_Pair()
    {
        let from = RevisionFingerprint { label: "v14.35".to_owned(), documents: BTreeMap::new() };
        let to = RevisionFingerprint { label: "v14.36".to_owned(), documents: BTreeMap::new() };

        let pair = One_Pair(&from, &to).expect("finds the pair");

        assert_eq!(pair.from, "v14.35");
        assert_eq!(pair.to, "v14.36");
    }

    fn Documents(pairs: &[(&str, &str)]) -> BTreeMap<String, String>
    {
        return pairs.iter().map(|(path, text)| return ((*path).to_owned(), (*text).to_owned())).collect();
    }
}
