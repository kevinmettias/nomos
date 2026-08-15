//! The archives' answer for one registered key.
//!
//! Split by what the key is about rather than written as one match, because the families
//! measure different things. A key nothing answers still panics, which is what keeps a
//! register entry from being carried by a figure nobody takes.

#![allow(dead_code)]

use nomos_spec_ingest::{Fate, Hollow, RegressionReport, DOMAIN_VOLUMES};

pub(crate) fn Measure(key: &str, report: &RegressionReport) -> u32
{
    let measured = Documents_Figure(key, report)
        .or_else(|| return Records_Figure(key, report))
        .or_else(|| return Filler_Figure(key, report))
        .or_else(|| return Members_Figure(key, report));

    return measured.unwrap_or_else(|| {
        // All four figures answered None, so the headline register quotes a key no reading
        // here produces. Letting it pass would leave a register entry carried by a figure
        // nobody takes, which is the one thing the split into four figures cannot hide.
        panic!("{key} is in the register and nothing measures it");
    });
}

/// What became of the documents, and of the volumes among them.
fn Documents_Figure(key: &str, report: &RegressionReport) -> Option<u32>
{
    return match key
    {
        "volumes.absent" => Some(Count(
            report
                .documents
                .disappeared
                .iter()
                .filter(|path| return path.contains(DOMAIN_VOLUMES))
                .count(),
        )),
        "documents.appeared" => Some(Count(report.documents.appeared.len())),
        "documents.disappeared" => Some(Count(report.documents.disappeared.len())),
        "documents.changed" => Some(Count(report.documents.changed.len())),
        "documents.relocated" => Some(Count(report.documents.relocated.len())),
        _ => None,
    };
}

/// The same questions asked of records, which are the documents under one path.
fn Records_Figure(key: &str, report: &RegressionReport) -> Option<u32>
{
    return match key
    {
        "records.relocated" => Some(Records_Among(
            report.documents.relocated.iter().map(|moved| return moved.to.as_str()),
        )),
        "records.new" => Some(Records_Among(
            report.documents.appeared.iter().map(String::as_str),
        )),
        _ => None,
    };
}

/// How many of these paths are records.
fn Records_Among<'a>(paths: impl Iterator<Item = &'a str>) -> u32
{
    return Count(paths.filter(|path| return Is_Record(path)).count());
}

/// What the blocklist declares, and what it does not see.
fn Filler_Figure(key: &str, report: &RegressionReport) -> Option<u32>
{
    let widest = || {
        return report
            .filler
            .Widest_Undeclared()
            // Two register keys are defined as properties of the widest undeclared template.
            // With none there is no such template to have a width, and answering zero would
            // state those two figures as measured against a template that does not exist.
            .unwrap_or_else(|| panic!("v15.0 carries no undeclared template"));
    };

    return match key
    {
        "filler.declared_documents" => Some(Count(report.filler.declared.len())),
        "filler.stub_documents" => Some(Count(report.filler.stubs.len())),
        "filler.widest_undeclared_sections" => Some(widest().sections),
        "filler.widest_undeclared_documents" => Some(Count(widest().documents.len())),
        _ => None,
    };
}

/// What became of each member.
fn Members_Figure(key: &str, report: &RegressionReport) -> Option<u32>
{
    return match key
    {
        "members.gone" => Some(Members_Whose_Fate(report, |fate| return *fate == Fate::Gone)),
        "members.hollowed_by_a_declared_pattern" =>
        {
            Some(Members_Whose_Fate(report, Hollowed_By_A_Declared_Pattern))
        }
        "members.hollowed_by_an_undeclared_template" =>
        {
            Some(Members_Whose_Fate(report, Hollowed_By_An_Undeclared_Template))
        }
        _ => None,
    };
}

/// How many members' fates answer a question.
fn Members_Whose_Fate(report: &RegressionReport, wanted: impl Fn(&Fate) -> bool) -> u32
{
    let matched = report.members.iter().filter(|member| return wanted(&member.fate));

    return Count(matched.count());
}

fn Hollowed_By_A_Declared_Pattern(fate: &Fate) -> bool
{
    return matches!(
        fate,
        Fate::Hollowed {
            evidence: Hollow::Template {
                declared: Some(_), ..
            },
            ..
        }
    );
}

fn Hollowed_By_An_Undeclared_Template(fate: &Fate) -> bool
{
    return matches!(
        fate,
        Fate::Hollowed {
            evidence: Hollow::Template { declared: None, .. },
            ..
        }
    );
}

pub(crate) fn Is_Record(path: &str) -> bool
{
    return path.starts_with("records/") || path.starts_with("spec-governance/records/");
}

fn Count(value: usize) -> u32
{
    return u32::try_from(value).unwrap_or(u32::MAX);
}
