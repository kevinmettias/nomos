//! Where the store comes from, and what to say when it is not there.
//!
//! Nothing persists a specification database. The store is assembled on every invocation
//! from two sources: this repository's own governing records, which are embedded in the
//! binary and are therefore always present, and the v14 authoring corpus, which lives
//! outside this repository and on most machines is not present at all.
//!
//! That second fact is the whole reason this module exists. A read command over a store
//! nothing was read into returns nothing, and nothing is exactly what a read command
//! returns when the identifier is genuinely unknown. The two answers are opposite — one
//! says *configure your corpus*, the other says *you have the wrong identifier* — and
//! they print the same. So an absence is a value here: it names what was expected, where
//! it was looked for, why it is not there, and what is consequently not in the store.
//!
//! This is `OD-GATE-001`'s finding one level up. Sixty-eight tests report `ok` having
//! read nothing because a check that cannot find its subject returned early instead of
//! saying so. A read command that prints an empty table for a corpus it never had is the
//! same defect wearing a different hat.

mod roots;
mod volumes;
mod layer;
#[cfg(test)]
mod tests;

pub use roots::DEFAULT_REVISION;
use roots::Corpus_Root;
use volumes::Ingest_Volumes;
use layer::{Ingest_Catalog_File, Ingest_Statement_File, Refused};

mod absence;
mod corpus_request;
mod assembly;

pub use absence::Absence;
pub use corpus_request::CorpusRequest;
pub use assembly::Assembly;

use nomos_spec_ingest::{
    Ingest_Catalog, Ingest_Source_Document, Ingest_Statements, IngestError, Parse_Catalog,
    Parse_Statements,
};
use nomos_spec_store::{Seed_Governing_Records, SpecificationStore, StoreError};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Builds the store this invocation will answer from.
///
/// The governing records are seeded first and unconditionally: they are embedded, so a
/// machine with no corpus can still ask what this repository decided about itself. The
/// corpus is layered over them, and every part of it that could not be read becomes an
/// [`Absence`] rather than a smaller answer.
///
/// # Errors
///
/// Returns [`StoreError`] if the database cannot be opened or the embedded records cannot
/// be seeded. A corpus that cannot be read is not an error: it is an absence, because the
/// commands over this store still have a true answer to give without it.
pub fn Assemble(request: &CorpusRequest) -> Result<Assembly, StoreError>
{
    let mut assembly = Seeded()?;

    let Some(root) = Corpus_Root(&mut assembly, request)
    else
    {
        return Ok(assembly);
    };

    Ingest_Volumes(&mut assembly, root, &request.revision);
    Ingest_Statement_File(&mut assembly, root);
    Ingest_Catalog_File(&mut assembly, root);

    return Ok(assembly);
}

/// The floor every command stands on: this repository's own governing records.
///
/// Embedded rather than read, because they are what makes an answer possible at all when
/// no corpus is layered over them — a store that could not find its own records would have
/// nothing true left to say.
fn Seeded() -> Result<Assembly, StoreError>
{
    let mut store = SpecificationStore::In_Memory()?;
    let seeded = Seed_Governing_Records(&mut store)?;

    return Ok(Assembly {
        store,
        read: vec![format!(
            "{} governing record(s) embedded in this binary, {} block(s)",
            seeded.records, seeded.blocks
        )],
        absent: Vec::new(),
    });
}
