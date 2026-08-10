//! Putting the catalog in, with the aliases each node answers to.

use super::{CatalogEntity, IngestError, SpecificationStore, CatalogReport, StoreError};

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
    for alias in &entity.aliases
    {
        store
            .Connection()
            .execute(
                "INSERT OR IGNORE INTO node_aliases (alias, node_uid) VALUES (?1, ?2)",
                rusqlite::params![alias, node_uid],
            )
            .map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())))?;
        report.aliases = report.aliases.saturating_add(1);
    }

    return Ok(());
}

/// I3 — the node graph.
///
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
        let node_uid = store.Upsert_Node(
            &entity.id,
            &entity.kind,
            if entity.authority.is_empty() { "unstated" } else { &entity.authority },
            "record",
            &entity.title,
        )?;
        report.nodes = report.nodes.saturating_add(1);
        Record_Aliases(store, entity, node_uid, &mut report)?;
    }

    return Ok(report);
}
