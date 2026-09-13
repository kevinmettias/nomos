mod query;
mod sections;

use query::{Columns, FirstColumn, Gather_Items, Narrow_To_Nodes, Query, SecondColumn};
use sections::{
    Gather_Blocks, Gather_Documents, Gather_Headings, Gather_Lineage, Gather_Nodes, Gather_Omissions, Gather_Relations,
    Gather_Rows, Gather_Statements, Gather_Suites,
};

use crate::Content;
use crate::Filter;
use crate::{Profile, SUBJECT};
use crate::Input;
use crate::Item;
use crate::Name;
use crate::Projection;
use crate::Value;
use crate::Section;
use crate::ProjectError;
use nomos_spec_store::SpecificationStore;
use rusqlite::{Connection, params_from_iter, Row};

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
            ("status", &self.status),
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
            &mut self.status,
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
                // `nodes` alone, because it is the only content kind that resolves to a
                // record, and a record is the only thing that declares a status. Every
                // other kind refuses this filter rather than ignoring it.
                "status",
            ],
            Self::Statements => &["kind", "identifier_prefix", "node_id"],
            Self::Relations => &["relation_type", "suite", "identifier_prefix", "node_id"],
            Self::Lineage => &["disposition", "document"],
            Self::Omissions => &["document"],
        };
    }
}

pub fn Select_Projection(store: &SpecificationStore, profile: &Profile) -> Result<Projection, ProjectError>
{
    let connection = store.Connection();
    let mut sections = Vec::new();
    let mut inputs = Vec::new();

    for declared in &profile.sections
    {
        let section = Selected_Section(connection, profile, declared)?;
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
fn Selected_Section(
    connection: &Connection,
    profile: &Profile,
    declared: &crate::ProfileSection,
) -> Result<Section, ProjectError>
{
    Refuse_Unhonoured(profile, declared.content, &declared.filter)?;
    let items = Gather_Items(connection, declared.content, &declared.filter)?;
    Refuse_Empty(profile, declared, &items)?;

    return Ok(Section {
        title: declared.title.clone(),
        content: declared.content,
        items,
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

/// A section that gathered nothing and did not say it might.
///
/// Refused rather than rendered empty. A profile whose corpus is absent selects nothing
/// from every section, and a projection published with the headings and none of the content
/// is indistinguishable from one whose subject genuinely has nothing to say.
fn Refuse_Empty(
    profile: &Profile,
    declared: &crate::ProfileSection,
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Named_Should_List_Only_The_Fields_A_Filter_Set()
    {
        let filter = Filter {
            kind: Some("concept".to_owned()),
            node_id: Some("CDM-ONE".to_owned()),
            ..Filter::default()
        };

        let names: Vec<&str> = filter.Named().into_iter().map(|(name, _)| return name).collect();

        assert_eq!(names, vec!["kind", "node_id"]);
    }

    #[test]
    fn Test_Substitute_Should_Replace_The_Subject_Placeholder_In_Every_Set_Field()
    {
        let mut filter = Filter {
            document: Some("{subject}.md".to_owned()),
            ..Filter::default()
        };

        filter.Substitute("AGT-EXEC-001");

        assert_eq!(filter.document.as_deref(), Some("AGT-EXEC-001.md"));
    }

    #[test]
    fn Test_Honours_Should_List_Suites_Only_Identifier_Prefix()
    {
        assert_eq!(Content::Suites.Honours(), &["identifier_prefix"]);
    }

    #[test]
    fn Test_Select_Projection_Should_Gather_Every_Declared_Section_In_Order()
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO suites (suite_id, title, authority_root) \
                 VALUES ('nomos', 'The Nomos specification', 1);",
            )
            .expect("seeds a suite");

        let profile = Profile::Parse(
            r#"{
                "id": "one", "title": "One", "format": "markdown", "output": "one.md",
                "sections": [{ "title": "Suites", "content": "suites" }]
            }"#,
        )
        .expect("parses");

        let projection = Select_Projection(&store, &profile).expect("selects");

        assert_eq!(projection.sections.len(), 1);
        assert_eq!(
            projection
                .sections
                .first()
                .expect("asserted above to contain exactly one section")
                .items
                .len(),
            1
        );
        assert_eq!(projection.inputs.len(), 1);
    }
}
