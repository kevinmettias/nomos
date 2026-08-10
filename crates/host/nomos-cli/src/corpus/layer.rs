//! One optional corpus file, read, parsed and ingested, or accounted for.

use super::{PathBuf, Assembly, Absence, IngestError, SpecificationStore, Path, Parse_Statements, Ingest_Statements, Parse_Catalog, Ingest_Catalog};
use super::roots::STATEMENTS;
use super::roots::CATALOG;

/// One optional corpus file, and what its absence costs.
///
/// The two costs are separate because they are separate claims: the first is what the
/// reader loses when the file is not there, and the second is what they lose when it was
/// there and would not go in. Written out at each of the three sites that need them, the
/// two ingests carried six copies of four strings.
pub(super) struct Layered<'a>
{
    /// What an absence calls it.
    subject: &'a str,
    /// Where it should be.
    path: PathBuf,
    /// What is lost when it cannot be read at all.
    unread: &'a str,
    /// What is lost when it was read and refused.
    refused: &'a str,
}

/// The text of an optional input, or nothing and a recorded absence.
pub(super) fn Text_Of(assembly: &mut Assembly, input: &Layered<'_>) -> Option<String>
{
    let Ok(text) = std::fs::read_to_string(&input.path)
    else
    {
        assembly.absent.push(Absence {
            subject: input.subject.to_owned(),
            expected: input.path.display().to_string(),
            cause: "the file could not be read".to_owned(),
            cost: input.unread.to_owned(),
        });

        return None;
    };

    return Some(text);
}

/// Records an input that was found and would not go in.
pub(super) fn Refuse(assembly: &mut Assembly, input: &Layered<'_>, error: &IngestError)
{
    let refusal = Refused(
        input.subject,
        &input.path.display().to_string(),
        error,
        input.refused,
    );
    assembly.absent.push(refusal);
}

/// Reads, parses and ingests one optional corpus file.
///
/// The two callers differ only in what they parse and what they put in, and every step
/// around those two is the same: the file may not be there, it may not read, and it may not
/// go in. Written out twice, the two carried six copies of four strings between them and
/// were one edit away from disagreeing about what an absence costs.
pub(super) fn Layer<P, R>(
    assembly: &mut Assembly,
    input: &Layered<'_>,
    parse: impl FnOnce(&str) -> Result<P, IngestError>,
    ingest: impl FnOnce(&mut SpecificationStore, &P) -> Result<R, IngestError>,
) -> Option<R>
{
    let text = Text_Of(assembly, input)?;
    let parsed = match parse(&text)
    {
        Ok(parsed) => parsed,
        Err(error) =>
        {
            Refuse(assembly, input, &error);

            return None;
        }
    };

    match ingest(&mut assembly.store, &parsed)
    {
        Ok(report) => return Some(report),
        Err(error) => Refuse(assembly, input, &error),
    }

    return None;
}

/// Records what an input contributed.
pub(super) fn Note(assembly: &mut Assembly, input: &Layered<'_>, count: u32, noun: &str)
{
    assembly.read.push(format!("{count} {noun} from {}", input.path.display()));
}

pub(super) fn Ingest_Statement_File(assembly: &mut Assembly, root: &Path)
{
    let input = Layered {
        subject: "the normative statements",
        path: root.join(STATEMENTS),
        unread: "no normative statement is in this store, so a profile that projects \
                 statements has nothing to project",
        refused: "no normative statement is in this store",
    };
    let Some(report) = Layer(assembly, &input, Parse_Statements, Ingest_Statements)
    else
    {
        return;
    };

    Note(assembly, &input, report.ingested, "normative statement(s)");
}

pub(super) fn Ingest_Catalog_File(assembly: &mut Assembly, root: &Path)
{
    let input = Layered {
        subject: "the node catalog",
        path: root.join(CATALOG),
        unread: "the corpus contributes no node to this store, so an identifier it holds \
                 resolves to nothing",
        refused: "the corpus contributes no node to this store",
    };
    let Some(report) = Layer(assembly, &input, Parse_Catalog, |store, entities| return Ingest_Catalog(store, entities))
    else
    {
        return;
    };

    Note(assembly, &input, report.nodes, "catalog node(s)");
}

/// An input that was found and refused.
///
/// The same shape as one that was not there at all, deliberately. From the answer's point
/// of view a corpus that will not parse and a corpus that is missing cost exactly the same
/// rows, and a reader who is told only that something failed will read the shortfall in
/// the answer as the answer.
pub(super) fn Refused(subject: &str, path: &str, error: &IngestError, cost: &str) -> Absence
{
    return Absence {
        subject: subject.to_owned(),
        expected: path.to_owned(),
        cause: format!("it was found and refused: {error}"),
        cost: cost.to_owned(),
    };
}
