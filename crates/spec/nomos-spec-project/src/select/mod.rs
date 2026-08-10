mod query;
mod sections;

use query::{Gather, Narrow_To_Nodes, Query, Text};
use sections::{Blocks, Documents, Headings, Lineage, Nodes, Omissions, Relations, Rows, Statements, Suites};

use crate::content::Content;
use crate::filter::Filter;
use crate::profile::Profile;
use crate::profile::SUBJECT;
use crate::input::Input;
use crate::item::Item;
use crate::projection::Projection;
use crate::section::Section;
use crate::ProjectError;
use core::fmt::Write as _;
use nomos_spec_store::SpecificationStore;
use rusqlite::{params_from_iter, Connection, Row};

impl Filter
{
    #[must_use]
    pub fn Named(&self) -> Vec<(&'static str, &String)>
    {
        let mut named = Vec::new();
        for (name, value) in [
            ("kind", &self.kind),
            ("authority", &self.authority),
            ("representation", &self.representation),
            ("suite", &self.suite),
            ("document", &self.document),
            ("revision", &self.revision),
            ("relation_type", &self.relation_type),
            ("disposition", &self.disposition),
            ("row_kind", &self.row_kind),
            ("identifier_prefix", &self.identifier_prefix),
            ("node_id", &self.node_id),
        ]
        {
            if let Some(set) = value
            {
                named.push((name, set));
            }
        }

        return named;
    }

    /// Replaces the subject placeholder in every value this filter carries.
    ///
    /// Every value rather than `node_id` alone. A subject narrows different content in
    /// different ways — a node by identity, its statements by the node they belong to, a
    /// document by path — and deciding here which of those is allowed would put the
    /// profile's vocabulary in the substitution rather than in the profile.
    pub fn Substitute(&mut self, subject: &str)
    {
        for set in [
            &mut self.kind,
            &mut self.authority,
            &mut self.representation,
            &mut self.suite,
            &mut self.document,
            &mut self.revision,
            &mut self.relation_type,
            &mut self.disposition,
            &mut self.row_kind,
            &mut self.identifier_prefix,
            &mut self.node_id,
        ]
        .into_iter()
        .flatten()
        {
            *set = set.replace(SUBJECT, subject);
        }
    }
}

impl Content
{
    #[must_use]
    pub const fn Honours(self) -> &'static [&'static str]
    {
        return match self
        {
            Self::Suites => &["identifier_prefix"],
            Self::Documents | Self::Headings => &["document", "revision"],
            Self::Blocks => &["document", "revision", "kind"],
            Self::Rows => &["document", "revision", "row_kind"],
            Self::Nodes => &[
                "kind",
                "authority",
                "representation",
                "suite",
                "identifier_prefix",
                "node_id",
            ],
            Self::Statements => &["kind", "identifier_prefix", "node_id"],
            Self::Relations => &["relation_type", "suite", "identifier_prefix", "node_id"],
            Self::Lineage => &["disposition", "document"],
            Self::Omissions => &["document"],
        };
    }
}

pub fn Select(store: &SpecificationStore, profile: &Profile) -> Result<Projection, ProjectError>
{
    let connection = store.Connection();
    let mut sections = Vec::new();
    let mut inputs = Vec::new();

    for declared in &profile.sections
    {
        let section = Selected(connection, profile, declared)?;
        let contributed = Inputs_Of(declared.content, &section.items);
        inputs.extend(contributed);
        sections.push(section);
    }

    return Ok(Projection {
        profile: profile.id.clone(),
        title: profile.title.clone(),
        format: profile.format,
        output: profile.output.clone(),
        sections,
        inputs,
    });
}

/// One declared section, gathered and checked.
///
/// Both refusals come before the rows are used, because a section that cannot be honoured
/// or that came back empty is a defect in the profile rather than a thin projection.
fn Selected(
    connection: &Connection,
    profile: &Profile,
    declared: &crate::profile_section::Section,
) -> Result<Section, ProjectError>
{
    Refuse_Unhonoured(profile, declared.content, &declared.filter)?;
    let items = Gather(connection, declared.content, &declared.filter)?;
    Refuse_Empty(profile, declared, &items)?;

    return Ok(Section {
        title: declared.title.clone(),
        content: declared.content,
        items,
    });
}

/// What a section's items contribute to the projection's input set.
///
/// An item's own hash where it has one, a digest of its fields where it has none. The
/// freshness check compares this set, so an item that carries no hash still has to move the
/// set when its content changes or the output would report itself current over stale rows.
fn Inputs_Of(content: Content, items: &[Item]) -> Vec<Input>
{
    return items
        .iter()
        .map(|item| {
            return Input {
                content,
                identity: item.identity.clone(),
                hash: item.Field("hash").map_or_else(|| return item.Digest(), str::to_owned),
            };
        })
        .collect();
}

/// A section that gathered nothing and did not say it might.
///
/// Refused rather than rendered empty. A profile whose corpus is absent selects nothing
/// from every section, and a projection published with the headings and none of the content
/// is indistinguishable from one whose subject genuinely has nothing to say.
fn Refuse_Empty(
    profile: &Profile,
    declared: &crate::profile_section::Section,
    items: &[Item],
) -> Result<(), ProjectError>
{
    if !items.is_empty() || declared.may_be_empty
    {
        return Ok(());
    }

    return Err(ProjectError::Empty {
        profile: profile.id.clone(),
        section: declared.title.clone(),
        content: declared.content.Label(),
    });
}

fn Refuse_Unhonoured(
    profile: &Profile,
    content: Content,
    filter: &Filter,
) -> Result<(), ProjectError>
{
    for (name, _) in filter.Named()
    {
        if !content.Honours().contains(&name)
        {
            return Err(ProjectError::UnsupportedFilter {
                profile: profile.id.clone(),
                content: content.Label(),
                filter: name,
            });
        }
    }

    return Ok(());
}
