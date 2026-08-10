use crate::profile::{Content, Filter, Profile, SUBJECT};
use crate::projection::{Input, Item, Projection, Section};
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

struct Query
{
    sql: String,
    values: Vec<String>,
}

impl Query
{
    fn On(base: &str) -> Self
    {
        return Self {
            sql: base.to_owned(),
            values: Vec::new(),
        };
    }

    fn Equal(&mut self, column: &str, value: Option<&String>)
    {
        if let Some(value) = value
        {
            self.values.push(value.clone());
            let _ = write!(self.sql, " AND {column} = ?{}", self.values.len());
        }
    }

    fn Prefix(&mut self, column: &str, value: Option<&String>)
    {
        if let Some(value) = value
        {
            self.values.push(format!("{}%", value.replace('%', "\\%")));
            let _ = write!(
                self.sql,
                " AND {column} LIKE ?{} ESCAPE '\\'",
                self.values.len()
            );
        }
    }

    /// One value matched against either of two columns.
    ///
    /// A relation has two ends and a subject sits at one or the other. Narrowing only the
    /// end an edge starts from would show a subject what it declares and hide what is
    /// declared about it, which is the half of a graph a reader is usually looking for.
    fn Either(&mut self, first: &str, second: &str, value: Option<&String>)
    {
        if let Some(value) = value
        {
            self.values.push(value.clone());
            let position = self.values.len();
            let _ = write!(self.sql, " AND ({first} = ?{position} OR {second} = ?{position})");
        }
    }

    fn Ordered_By(mut self, columns: &str) -> Self
    {
        self.sql.push_str(" ORDER BY ");
        self.sql.push_str(columns);

        return self;
    }

    fn Run<F>(&self, connection: &Connection, read: F) -> Result<Vec<Item>, ProjectError>
    where
        F: Fn(&Row<'_>) -> rusqlite::Result<Item>,
    {
        let mut statement = connection.prepare(&self.sql)?;
        let rows = statement.query_map(params_from_iter(self.values.iter()), |row| return read(row))?;

        let mut items = Vec::new();
        for item in rows
        {
            items.push(item?);
        }

        return Ok(items);
    }
}

pub fn Select(store: &SpecificationStore, profile: &Profile) -> Result<Projection, ProjectError>
{
    let connection = store.Connection();
    let mut sections = Vec::new();
    let mut inputs = Vec::new();

    for declared in &profile.sections
    {
        Refuse_Unhonoured(profile, declared.content, &declared.filter)?;
        let items = Gather(connection, declared.content, &declared.filter)?;

        if items.is_empty() && !declared.may_be_empty
        {
            return Err(ProjectError::Empty {
                profile: profile.id.clone(),
                section: declared.title.clone(),
                content: declared.content.Label(),
            });
        }

        for item in &items
        {
            inputs.push(Input {
                content: declared.content,
                identity: item.identity.clone(),
                hash: item
                    .Field("hash")
                    .map_or_else(|| return item.Digest(), str::to_owned),
            });
        }

        sections.push(Section {
            title: declared.title.clone(),
            content: declared.content,
            items,
        });
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

fn Gather(
    connection: &Connection,
    content: Content,
    filter: &Filter,
) -> Result<Vec<Item>, ProjectError>
{
    return match content
    {
        Content::Suites => Suites(connection, filter),
        Content::Documents => Documents(connection, filter),
        Content::Headings => Headings(connection, filter),
        Content::Blocks => Blocks(connection, filter),
        Content::Rows => Rows(connection, filter),
        Content::Nodes => Nodes(connection, filter),
        Content::Statements => Statements(connection, filter),
        Content::Relations => Relations(connection, filter),
        Content::Lineage => Lineage(connection, filter),
        Content::Omissions => Omissions(connection, filter),
    };
}

fn Text(row: &Row<'_>, index: usize) -> rusqlite::Result<String>
{
    return row.get::<usize, Option<String>>(index).map(Option::unwrap_or_default);
}

fn Suites(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT suite_id, title, authority_root FROM suites WHERE 1 = 1",
    );
    query.Prefix("suite_id", filter.identifier_prefix.as_ref());

    return query.Ordered_By("suite_id").Run(connection, |row| {
        let root: i64 = row.get(2)?;

        return Ok(Item::Of(&Text(row, 0)?)
            .With("title", &Text(row, 1)?)
            .With("authority", if root == 1 { "root" } else { "sibling" }));
    });
}

fn Documents(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT d.path, d.revision, b.sha256,
                (SELECT count(*) FROM source_blocks WHERE document_uid = d.uid),
                (SELECT count(*) FROM source_headings WHERE document_uid = d.uid)
         FROM source_documents d JOIN blobs b ON b.uid = d.blob_uid
         WHERE 1 = 1",
    );
    query.Equal("d.path", filter.document.as_ref());
    query.Equal("d.revision", filter.revision.as_ref());

    return query.Ordered_By("d.revision, d.path").Run(connection, |row| {
        let blocks: i64 = row.get(3)?;
        let headings: i64 = row.get(4)?;

        return Ok(Item::Of(&format!("{}@{}", Text(row, 0)?, Text(row, 1)?))
            .With("path", &Text(row, 0)?)
            .With("revision", &Text(row, 1)?)
            .With("blocks", &blocks.to_string())
            .With("headings", &headings.to_string())
            .With("hash", &Text(row, 2)?));
    });
}

fn Headings(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT d.path, d.revision, h.ordinal, h.depth, h.title
         FROM source_headings h JOIN source_documents d ON d.uid = h.document_uid
         WHERE 1 = 1",
    );
    query.Equal("d.path", filter.document.as_ref());
    query.Equal("d.revision", filter.revision.as_ref());

    return query
        .Ordered_By("d.revision, d.path, h.ordinal")
        .Run(connection, |row| {
            let ordinal: i64 = row.get(2)?;
            let depth: i64 = row.get(3)?;

            return Ok(Item::Of(&format!("{}#{ordinal}", Text(row, 0)?))
                .With("revision", &Text(row, 1)?)
                .With("depth", &depth.to_string())
                .With("title", &Text(row, 4)?));
        });
}

fn Blocks(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT d.path, d.revision, b.ordinal, b.kind, b.heading_path, b.text, b.content_hash
         FROM source_blocks b JOIN source_documents d ON d.uid = b.document_uid
         WHERE 1 = 1",
    );
    query.Equal("d.path", filter.document.as_ref());
    query.Equal("d.revision", filter.revision.as_ref());
    query.Equal("b.kind", filter.kind.as_ref());

    return query
        .Ordered_By("d.revision, d.path, b.ordinal")
        .Run(connection, |row| {
            let ordinal: i64 = row.get(2)?;

            return Ok(Item::Of(&format!("{}#{ordinal}", Text(row, 0)?))
                .With("revision", &Text(row, 1)?)
                .With("kind", &Text(row, 3)?)
                .With("heading", &Text(row, 4)?)
                .With("hash", &Text(row, 6)?)
                .Carrying(&Text(row, 5)?));
        });
}

fn Rows(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT d.path, d.revision, b.ordinal, r.ordinal, r.table_ordinal, r.kind,
                r.cells_json, r.text, r.content_hash
         FROM source_table_rows r
         JOIN source_blocks b ON b.uid = r.source_block_uid
         JOIN source_documents d ON d.uid = b.document_uid
         WHERE 1 = 1",
    );
    query.Equal("d.path", filter.document.as_ref());
    query.Equal("d.revision", filter.revision.as_ref());
    query.Equal("r.kind", filter.row_kind.as_ref());

    return query
        .Ordered_By("d.revision, d.path, b.ordinal, r.ordinal")
        .Run(connection, |row| {
            let block: i64 = row.get(2)?;
            let ordinal: i64 = row.get(3)?;
            let table: i64 = row.get(4)?;
            let cells: Vec<String> = serde_json::from_str(&Text(row, 6)?).unwrap_or_default();

            return Ok(Item::Of(&format!("{}#{block}:{ordinal}", Text(row, 0)?))
                .With("revision", &Text(row, 1)?)
                .With("kind", &Text(row, 5)?)
                .With("table", &table.to_string())
                .With("cells", &cells.join(" | "))
                .With("hash", &Text(row, 8)?)
                .Carrying(&Text(row, 7)?));
        });
}

fn Nodes(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT n.node_id, n.kind, n.authority, n.representation, n.title, s.suite_id
         FROM nodes n LEFT JOIN suites s ON s.uid = n.suite_uid
         WHERE n.deleted_at IS NULL",
    );
    query.Equal("n.kind", filter.kind.as_ref());
    query.Equal("n.authority", filter.authority.as_ref());
    query.Equal("n.representation", filter.representation.as_ref());
    query.Equal("s.suite_id", filter.suite.as_ref());
    query.Prefix("n.node_id", filter.identifier_prefix.as_ref());
    query.Equal("n.node_id", filter.node_id.as_ref());

    return query.Ordered_By("n.node_id").Run(connection, |row| {
        return Ok(Item::Of(&Text(row, 0)?)
            .With("kind", &Text(row, 1)?)
            .With("authority", &Text(row, 2)?)
            .With("representation", &Text(row, 3)?)
            .With("title", &Text(row, 4)?)
            .With("suite", &Text(row, 5)?));
    });
}

fn Statements(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT s.statement_id, s.kind, n.node_id, s.canonical_text, s.canonical_hash,
                s.supersedes_hash
         FROM normative_statements s JOIN nodes n ON n.uid = s.node_uid
         WHERE 1 = 1",
    );
    query.Equal("s.kind", filter.kind.as_ref());
    query.Prefix("s.statement_id", filter.identifier_prefix.as_ref());
    query.Equal("n.node_id", filter.node_id.as_ref());

    return query.Ordered_By("s.statement_id").Run(connection, |row| {
        return Ok(Item::Of(&Text(row, 0)?)
            .With("kind", &Text(row, 1)?)
            .With("node", &Text(row, 2)?)
            .With("supersedes", &Text(row, 5)?)
            .With("hash", &Text(row, 4)?)
            .Carrying(&Text(row, 3)?));
    });
}

fn Relations(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT f.node_id, r.relation_type, t.node_id, y.tier, s.suite_id
         FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         JOIN relation_types y ON y.name = r.relation_type
         LEFT JOIN suites s ON s.uid = f.suite_uid
         WHERE 1 = 1",
    );
    query.Equal("r.relation_type", filter.relation_type.as_ref());
    query.Equal("s.suite_id", filter.suite.as_ref());
    query.Prefix("f.node_id", filter.identifier_prefix.as_ref());
    query.Either("f.node_id", "t.node_id", filter.node_id.as_ref());

    return query
        .Ordered_By("f.node_id, r.relation_type, t.node_id")
        .Run(connection, |row| {
            return Ok(Item::Of(&format!(
                "{} {} {}",
                Text(row, 0)?,
                Text(row, 1)?,
                Text(row, 2)?
            ))
            .With("from", &Text(row, 0)?)
            .With("relation", &Text(row, 1)?)
            .With("to", &Text(row, 2)?)
            .With("tier", &Text(row, 3)?));
        });
}

fn Lineage(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT l.disposition,
                coalesce(d.path, hd.path, rd.path, ''),
                coalesce(b.ordinal, rb.ordinal, -1), coalesce(r.ordinal, -1), coalesce(h.title, ''),
                coalesce(n.node_id, ''), coalesce(st.statement_id, '')
         FROM lineage l
         LEFT JOIN source_blocks b ON b.uid = l.source_block_uid
         LEFT JOIN source_documents d ON d.uid = b.document_uid
         LEFT JOIN source_headings h ON h.uid = l.source_heading_uid
         LEFT JOIN source_documents hd ON hd.uid = h.document_uid
         LEFT JOIN source_table_rows r ON r.uid = l.source_table_row_uid
         LEFT JOIN source_blocks rb ON rb.uid = r.source_block_uid
         LEFT JOIN source_documents rd ON rd.uid = rb.document_uid
         LEFT JOIN nodes n ON n.uid = l.target_node_uid
         LEFT JOIN normative_statements st ON st.uid = l.target_statement
         WHERE 1 = 1",
    );
    query.Equal("l.disposition", filter.disposition.as_ref());
    query.Equal("coalesce(d.path, hd.path, rd.path, '')", filter.document.as_ref());

    return query
        .Ordered_By(
            "coalesce(d.path, hd.path, rd.path, ''), coalesce(b.ordinal, -1), \
             coalesce(r.ordinal, -1), l.disposition, coalesce(n.node_id, ''), \
             coalesce(st.statement_id, '')",
        )
        .Run(connection, |row| {
            let block: i64 = row.get(2)?;
            let ordinal: i64 = row.get(3)?;
            let source = match (block, ordinal)
            {
                (-1, -1) => format!("{}#{}", Text(row, 1)?, Text(row, 4)?),
                (block, -1) => format!("{}#{block}", Text(row, 1)?),
                (block, ordinal) => format!("{}#{block}:{ordinal}", Text(row, 1)?),
            };
            let target = match (Text(row, 5)?, Text(row, 6)?)
            {
                (node, statement) if statement.is_empty() => node,
                (_, statement) => statement,
            };

            return Ok(Item::Of(&format!("{source} -> {}", Text(row, 0)?))
                .With("source", &source)
                .With("disposition", &Text(row, 0)?)
                .With("target", &target));
        });
}

fn Omissions(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT coalesce(d.path, hd.path, ''), coalesce(b.ordinal, -1),
                coalesce(h.title, ''), o.reason, o.justification, o.decision_record
         FROM omissions o
         LEFT JOIN source_blocks b ON b.uid = o.source_block_uid
         LEFT JOIN source_documents d ON d.uid = b.document_uid
         LEFT JOIN source_headings h ON h.uid = o.source_heading_uid
         LEFT JOIN source_documents hd ON hd.uid = h.document_uid
         WHERE 1 = 1",
    );
    query.Equal("coalesce(d.path, hd.path, '')", filter.document.as_ref());

    return query
        .Ordered_By(
            "o.decision_record, coalesce(d.path, hd.path, ''), coalesce(b.ordinal, -1), o.reason",
        )
        .Run(connection, |row| {
            let block: i64 = row.get(1)?;
            let source = if block == -1
            {
                format!("{}#{}", Text(row, 0)?, Text(row, 2)?)
            }
            else
            {
                format!("{}#{block}", Text(row, 0)?)
            };

            return Ok(Item::Of(&format!("{source} -> {}", Text(row, 5)?))
                .With("source", &source)
                .With("reason", &Text(row, 3)?)
                .With("justification", &Text(row, 4)?)
                .With("decision", &Text(row, 5)?));
        });
}
