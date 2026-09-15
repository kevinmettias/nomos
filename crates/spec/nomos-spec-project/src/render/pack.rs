//! A context pack: one section reduced to identity, title, declared status and prose.

use crate::GENERATED_FILE_NOTICE;
use crate::ProjectError;
use crate::Projection;
use crate::Section;
use serde::Serialize;

pub(super) fn Render_Contextpack(projection: &Projection) -> Result<String, ProjectError>
{
    let pack = Pack {
        nomos_generated: true,
        do_not_edit: GENERATED_FILE_NOTICE,
        profile: &projection.profile,
        title: &projection.title,
        inputs_digest: projection.Inputs_Digest(),
        sections: projection.sections.iter().map(Packed_Section).collect(),
    };

    let mut rendered = serde_json::to_string_pretty(&pack)
        .map_err(|error| return ProjectError::Malformed(error.to_string()))?;
    rendered.push('\n');

    return Ok(rendered);
}

#[derive(Serialize)]
struct Pack<'a>
{
    nomos_generated: bool,
    do_not_edit: &'a str,
    profile: &'a str,
    title: &'a str,
    inputs_digest: String,
    sections: Vec<PackSection<'a>>,
}

#[derive(Serialize)]
struct PackSection<'a>
{
    title: &'a str,
    items: Vec<PackItem<'a>>,
}

#[derive(Serialize)]
struct PackItem<'a>
{
    identity: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    title: &'a str,
    /// What the record this item names declared as its lifecycle status.
    ///
    /// `OD-PROJECT-005` decided a cited neighbour carries identity, title **and** declared
    /// status, and gave the reason: ten of this repository's governing questions declare
    /// `status: open`, and a reader handed only identities cannot tell a settled neighbour
    /// from an unsettled one — which is the distinction an implementer most needs before
    /// building on one.
    ///
    /// Absent rather than empty when the item has none, because most content kinds are not
    /// records and inventing `""` for a source block would be answering a question that was
    /// never asked of it.
    #[serde(skip_serializing_if = "str::is_empty")]
    status: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a String>,
}

/// One section reduced to what a context pack carries: identity, title, declared status
/// and prose.
///
/// The other fields are dropped on purpose. A pack is read by something with a budget, and
/// a field that only a table renderer uses costs that budget without answering anything.
fn Packed_Section(section: &Section) -> PackSection<'_>
{
    return PackSection {
        title: &section.title,
        items: section
            .items
            .iter()
            .map(|item| {
                return PackItem {
                    identity: &item.identity,
                    title: item.Field("title").unwrap_or_default(),
                    status: item.Field("status").unwrap_or_default(),
                    text: item.body.as_ref(),
                };
            })
            .collect(),
    };
}
