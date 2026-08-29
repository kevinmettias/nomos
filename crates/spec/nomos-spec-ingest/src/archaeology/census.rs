//! How much of a revision says nothing, counted rather than sampled.

use super::{Later, FillerCensus, Template, SHARED_BY, Repetition, Get_Filler_Pattern};

pub(super) fn Census_Fillers(later: &Later) -> FillerCensus
{
    return FillerCensus {
        declared: later.declared.iter().cloned().collect(),
        templates: Shared_Templates(later),
        stubs: Stubs_Of(later),
    };
}

/// The blocks repeated widely enough to be template rather than prose, widest first.
///
/// Ordering is by reach and then by text, so the report opens on the thing worth deleting
/// and two runs over the same corpus agree on what to print.
pub(super) fn Shared_Templates(later: &Later) -> Vec<Template>
{
    let mut templates: Vec<Template> = later
        .templates
        .iter()
        .filter(|(_, repetition)| return repetition.sections >= SHARED_BY)
        .map(|(key, repetition)| return Described_Template(key, repetition))
        .collect();
    templates.sort_by(|first, second| {
        return second
            .sections
            .cmp(&first.sections)
            .then_with(|| return first.text.cmp(&second.text));
    });

    return templates;
}

/// One repeated block as the census reports it.
pub(super) fn Described_Template(key: &str, repetition: &Repetition) -> Template
{
    return Template {
        text: key.to_owned(),
        sections: repetition.sections,
        documents: repetition.documents.iter().cloned().collect(),
        declared: Get_Filler_Pattern(key),
    };
}

/// The documents carrying nothing but filler.
///
/// A document with no blocks at all is not one of them: it is empty, which is a different
/// complaint and one the reader can already see.
pub(super) fn Stubs_Of(later: &Later) -> Vec<String>
{
    let mut stubs: Vec<String> = Vec::new();

    for (path, bodies) in &later.bodies
    {
        if bodies.blocks > 0 && bodies.filler == bodies.blocks
        {
            stubs.push(path.clone());
        }
    }

    return stubs;
}
