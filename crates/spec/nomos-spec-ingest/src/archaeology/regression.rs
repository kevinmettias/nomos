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
