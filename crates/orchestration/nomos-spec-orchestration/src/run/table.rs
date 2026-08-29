//! Resolving `nomos spec table`'s request against an assembled store.

use nomos_spec_store::{DocumentSource, PathMatch, RowCensus, RowScope, StoreError, TableLine};

use crate::corpus::Assembly;
use crate::spec_outcome::{TableAnswer, TableRefusal};
use crate::request::TableRequest;

/// The rows `request` selects, or why none answered.
///
/// Moved verbatim from `nomos-cli::spec::verb::table::Table`, minus the writing: the address
/// is resolved to one document first, and only a document that resolved and then carried no
/// selected row is [`TableRefusal::NoRows`] -- an address matching zero or several documents
/// never reaches that far.
///
/// # Errors
///
/// Returns [`TableRefusal::NoSuchDocument`] when `request.document` matches nothing,
/// [`TableRefusal::AmbiguousDocument`] when it matches more than one document,
/// [`TableRefusal::NoRows`] when the document resolved and `request`'s own narrowing
/// selected no row, and [`TableRefusal::Store`] when the store could not be read at all.
pub fn Resolved_Table(assembly: &Assembly, request: &TableRequest) -> Result<TableAnswer, TableRefusal>
{
    let (uid, tier) = Addressed_Document(assembly, request)?;
    let read = Read_Table(assembly, uid, request)?;

    if read.lines.is_empty()
    {
        return Err(TableRefusal::NoRows {
            document: read.document,
            tier,
            census: read.census,
        });
    }

    return Ok(TableAnswer {
        document: read.document,
        tier,
        census: read.census,
        lines: read.lines,
    });
}

/// The one document `request.document` names, or the refusal saying why it names none or
/// several.
fn Addressed_Document(assembly: &Assembly, request: &TableRequest) -> Result<(i64, PathMatch), TableRefusal>
{
    let revision = request.revision.as_deref();
    let (matched, tier) = assembly
        .store
        .Documents_Named(&request.document, revision)
        .map_err(TableRefusal::Store)?;

    return match matched.as_slice()
    {
        [uid] => Ok((*uid, tier)),
        [] => Err(TableRefusal::NoSuchDocument),
        many => Err(TableRefusal::AmbiguousDocument {
            matched: many.len(),
            tier,
        }),
    };
}

/// A document, the rows the narrowing selected from it, and the census of the whole of it.
///
/// The census counts the document rather than the selection deliberately: it is what lets a
/// renderer distinguish "this document has no tables" from "the block you named has none".
struct ReadTable
{
    document: DocumentSource,
    lines: Vec<TableLine>,
    census: RowCensus,
}

/// Everything a `table` run reads, once one document has been addressed.
fn Read_Table(assembly: &Assembly, uid: i64, request: &TableRequest) -> Result<ReadTable, TableRefusal>
{
    let document = match assembly.store.Document(uid)
    {
        Ok(Some(document)) => document,
        Ok(None) => return Err(TableRefusal::Store(Vanished_Document(uid))),
        Err(error) => return Err(TableRefusal::Store(error)),
    };
    let lines = assembly
        .store
        .Table_Lines(uid, request.block, request.table)
        .map_err(TableRefusal::Store)?;
    let census = assembly
        .store
        .Row_Census(RowScope::Document(uid))
        .map_err(TableRefusal::Store)?;

    return Ok(ReadTable { document, lines, census });
}

/// A document that resolved and then could not be read back.
///
/// Its own function because the situation is a store defect rather than a caller's mistake:
/// the surrogate came out of a query against the same connection.
fn Vanished_Document(uid: i64) -> StoreError
{
    return StoreError::Sql(format!(
        "document {uid} resolved and then could not be read back from the same connection"
    ));
}
