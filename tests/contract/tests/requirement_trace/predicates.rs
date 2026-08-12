//! The predicates. Each is called by an assertion and by a control.
//!
//! Held apart from both so the control and the assertion run the same code. A control that
//! re-implemented the comparison would prove the copy right and say nothing about the guard.

use crate::assessment::{Assessment, Site};
use std::path::Path;

/// Every site that is not where its entry says it is.
pub(crate) fn Unresolved_Sites(root: &Path, assessments: &[Assessment]) -> Vec<String>
{
    let mut missing = Vec::new();

    for assessment in assessments
    {
        for site in &assessment.sites
        {
            let unresolved = Unresolved(root, &assessment.requirement, site);
            missing.extend(unresolved);
        }
    }

    return missing;
}

/// Why one site is not where its entry says it is, if it is not.
fn Unresolved(root: &Path, requirement: &str, site: &Site) -> Option<String>
{
    let path = root.join(&site.path);
    let Ok(text) = std::fs::read_to_string(&path)
    else
    {
        return Some(format!(
            "{requirement}: {} is not a file in this workspace",
            site.path
        ));
    };
    if text.contains(&site.symbol)
    {
        return None;
    }

    return Some(format!(
        "{requirement}: {} no longer occurs in {}",
        site.symbol, site.path
    ));
}

/// Every named record that is not a registered governing record.
pub(crate) fn Unresolved_Records(root: &Path, assessments: &[Assessment]) -> Vec<String>
{
    let mut unresolved = Vec::new();

    for assessment in assessments
    {
        let Some(record) = assessment.record.as_deref()
        else
        {
            continue;
        };

        let missing = Unregistered(root, &assessment.requirement, record);
        unresolved.extend(missing);
    }

    return unresolved;
}

/// Why one named record does not resolve to a registered governing record, if it does not.
fn Unregistered(root: &Path, requirement: &str, record: &str) -> Option<String>
{
    if !Registration_Exists(root, record)
    {
        return Some(format!(
            "{requirement}: {record} has no registration under \
             crates/spec/nomos-spec-store/records/"
        ));
    }
    if Document_Exists(root, record)
    {
        return None;
    }

    return Some(format!(
        "{requirement}: {record} is registered and its document is not under docs/records/"
    ));
}

/// Every entry that departs from a requirement without saying why.
pub(crate) fn Divergences_With_No_Record(assessments: &[Assessment]) -> Vec<String>
{
    return assessments
        .iter()
        .filter(|assessment| {
            return assessment.verdict.Owes_A_Record() && assessment.record.is_none();
        })
        .map(|assessment| {
            return format!(
                "{}: {} with no governing record",
                assessment.requirement,
                assessment.verdict.Label()
            );
        })
        .collect();
}

/// Whether a record identifier has a registration file.
fn Registration_Exists(root: &Path, record: &str) -> bool
{
    return root
        .join("crates/spec/nomos-spec-store/records")
        .join(format!("{record}.record"))
        .is_file();
}

/// Whether a record identifier has a document under `docs/records`.
///
/// By stem prefix, because the slug is not derivable from the identifier — the same reason
/// `OD-SPEC-007` puts the path inside the registration rather than computing it.
fn Document_Exists(root: &Path, record: &str) -> bool
{
    let prefix = format!("{record}-");
    let Ok(entries) = std::fs::read_dir(root.join("docs/records"))
    else
    {
        return false;
    };

    return entries.flatten().any(|entry| {
        let path = entry.path();
        if path
            .extension()
            .is_none_or(|extension| return extension != "md")
        {
            return false;
        }

        return path
            .file_name()
            .and_then(|name| return name.to_str())
            .is_some_and(|name| return name.starts_with(&prefix));
    });
}
