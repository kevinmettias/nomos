//! Putting the catalog in, with the aliases each node answers to.

use super::{CatalogEntity, IngestError, NodeRow, SpecificationStore, CatalogReport, StoreError};

/// # Errors
///
/// Returns [`IngestError::Parse`] if the catalog is not a JSON array of entities.
pub fn Parse_Catalog(json: &str) -> Result<Vec<CatalogEntity>, IngestError>
{
    return serde_json::from_str(json)
        .map_err(|error| IngestError::Parse(format!("catalog: {error}")));
}

/// Points every name an entity answers to at its node.
pub(super) fn Record_Aliases(
    store: &mut SpecificationStore,
    entity: &CatalogEntity,
    node_uid: i64,
    report: &mut CatalogReport,
) -> Result<(), IngestError>
{
    // The statement is prepared once and executed per alias, rather than prepared inside the
    // loop. Preparing is the parse and plan step, and it does not depend on the alias, so
    // hoisting it does the work once for the entity instead of once for every name it answers
    // to. The execute must stay per alias: this is a local file with no batch form that would
    // let one crossing insert them all.
    let mut insert = store
        .Connection()
        .prepare("INSERT OR IGNORE INTO node_aliases (alias, node_uid) VALUES (?1, ?2)")
        .map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())))?;

    for alias in &entity.aliases
    {
        insert
            .execute(rusqlite::params![alias, node_uid])
            .map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())))?;
        report.aliases = report.aliases.saturating_add(1);
    }

    return Ok(());
}

/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Catalog(
    store: &mut SpecificationStore,
    entities: &[CatalogEntity],
) -> Result<CatalogReport, IngestError>
{
    let mut report = CatalogReport::default();

    for entity in entities
    {
        let node_uid = store.Upsert_Node(NodeRow {
            node_id: &entity.id,
            kind: &entity.kind,
            authority: Stated_Authority(entity),
            representation: "record",
            title: &entity.title,
        })?;

        report.nodes = report.nodes.saturating_add(1);
        Record_Aliases(store, entity, node_uid, &mut report)?;
    }

    return Ok(report);
}

/// I3 — the node graph.
///
/// The authority a catalog entity claims, or the word for claiming none.
///
/// An empty string in the catalog is an entity that said nothing about who decided it, and
/// storing that as an empty authority would read as an authority nobody has named yet.
fn Stated_Authority(entity: &CatalogEntity) -> &str
{
    if entity.authority.is_empty()
    {
        return "unstated";
    }

    return &entity.authority;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Parse_Catalog_Should_Read_A_Json_Array_Of_Entities()
    {
        let json = r#"[{"id":"AGT-001","kind":"requirement","title":"a requirement",
            "authority":"canonical","representation":"record","aliases":["AGT-010"]}]"#;

        let entities = Parse_Catalog(json).expect("parses");

        assert_eq!(entities.len(), 1);
        assert_eq!(entities.first().expect("the assertion above confirms exactly one entity").id, "AGT-001");
        assert_eq!(
            entities.first().expect("the assertion above confirms exactly one entity").aliases,
            vec!["AGT-010".to_owned()]
        );
    }

    #[test]
    fn Test_Record_Aliases_Should_Point_Every_Alias_At_The_Node()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let node_uid = store
            .Upsert_Node(NodeRow {
                node_id: "AGT-001",
                kind: "requirement",
                authority: "canonical",
                representation: "record",
                title: "a requirement",
            })
            .expect("upserts");
        let entity = Entity();
        let mut report = CatalogReport::default();

        Record_Aliases(&mut store, &entity, node_uid, &mut report).expect("records");

        assert_eq!(report.aliases, 1);
    }

    fn Entity() -> CatalogEntity
    {
        return CatalogEntity {
            id: "AGT-001".to_owned(),
            kind: "requirement".to_owned(),
            title: "a requirement".to_owned(),
            authority: "canonical".to_owned(),
            representation: "record".to_owned(),
            aliases: vec!["AGT-010".to_owned()],
        };
    }

    #[test]
    fn Test_Ingest_Catalog_Should_Create_A_Node_Per_Entity()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let entities = vec![Entity()];

        let report = Ingest_Catalog(&mut store, &entities).expect("ingests");

        assert_eq!(report.nodes, 1);
        assert_eq!(report.aliases, 1);
    }
}
