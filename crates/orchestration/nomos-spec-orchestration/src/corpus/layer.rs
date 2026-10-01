//! One optional corpus file, read, parsed and ingested, or accounted for.

use super::{PathBuf, Assembly, Absence, IngestError, SpecificationStore, Path, Parse_Statements, Ingest_Statements, Parse_Catalog, Ingest_Catalog};

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
pub(super) fn Refuse_Input(assembly: &mut Assembly, input: &Layered<'_>, error: &IngestError)
{
    let refusal = Refused_Absence(
        Subject(input.subject),
        Expected(&input.path.display().to_string()),
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
pub(super) fn Ingest_Optional_Layer<Parsed, Report>(
    assembly: &mut Assembly,
    input: &Layered<'_>,
    parse: impl FnOnce(&str) -> Result<Parsed, IngestError>,
    ingest: impl FnOnce(&mut SpecificationStore, &Parsed) -> Result<Report, IngestError>,
) -> Option<Report>
{
    let text = Text_Of(assembly, input)?;
    let parsed = match parse(&text)
    {
        Ok(parsed) => parsed,
        Err(error) =>
        {
            Refuse_Input(assembly, input, &error);

            return None;
        }
    };

    match ingest(&mut assembly.store, &parsed)
    {
        Ok(report) => return Some(report),
        Err(error) => Refuse_Input(assembly, input, &error),
    }

    return None;
}

/// Records what an input contributed.
pub(super) fn Note_Contribution(assembly: &mut Assembly, input: &Layered<'_>, count: u32, noun: &str)
{
    assembly.read.push(format!("{count} {noun} from {}", input.path.display()));
}

pub(super) fn Ingest_Statement_File(assembly: &mut Assembly, root: &Path)
{
    use super::roots::STATEMENTS;

    let input = Layered {
        subject: "the normative statements",
        path: root.join(STATEMENTS),
        unread: "no normative statement is in this store, so a profile that projects \
                 statements has nothing to project",
        refused: "no normative statement is in this store",
    };
    let Some(report) = Ingest_Optional_Layer(assembly, &input, Parse_Statements, Ingest_Statements)
    else
    {
        return;
    };

    Note_Contribution(assembly, &input, report.ingested, "normative statement(s)");
}

pub(super) fn Ingest_Catalog_File(assembly: &mut Assembly, root: &Path)
{
    use super::roots::CATALOG;

    let input = Layered {
        subject: "the node catalog",
        path: root.join(CATALOG),
        unread: "the corpus contributes no node to this store, so an identifier it holds \
                 resolves to nothing",
        refused: "the corpus contributes no node to this store",
    };
    let Some(report) = Ingest_Optional_Layer(assembly, &input, Parse_Catalog, |store, entities| return Ingest_Catalog(store, entities))
    else
    {
        return;
    };

    Note_Contribution(assembly, &input, report.nodes, "catalog node(s)");
}

/// What an absence calls the input that refused -- distinct from [`Expected`] so the two
/// adjacent `&str` positions in [`Refused_Absence`] cannot be passed in the wrong order.
pub(super) struct Subject<'a>(pub &'a str);

/// Where the refused input was expected -- distinct from [`Subject`] for the same reason.
pub(super) struct Expected<'a>(pub &'a str);

/// An input that was found and refused.
///
/// The same shape as one that was not there at all, deliberately. From the answer's point
/// of view a corpus that will not parse and a corpus that is missing cost exactly the same
/// rows, and a reader who is told only that something failed will read the shortfall in
/// the answer as the answer.
pub(super) fn Refused_Absence(subject: Subject<'_>, path: Expected<'_>, error: &IngestError, cost: &str) -> Absence
{
    return Absence {
        subject: subject.0.to_owned(),
        expected: path.0.to_owned(),
        cause: format!("it was found and refused: {error}"),
        cost: cost.to_owned(),
    };
}

#[cfg(test)]
mod tests
{
    use super::{
        Expected, Ingest_Catalog_File, Ingest_Optional_Layer, Ingest_Statement_File, Layered,
        Note_Contribution, Refuse_Input, Refused_Absence, Subject, Text_Of,
    };
    use crate::corpus::Assembly;
    use nomos_spec_ingest::IngestError;
    use nomos_spec_store::SpecificationStore;

    /// The count one formatting case passes. Named once because the call and the line it asserts
    /// against have to agree on it, and two bare `3`s would let an edit to one drift from the other.
    const CONTRIBUTED_STATEMENTS: u32 = 3;

    #[test]
    fn Test_Text_Of_Should_Read_A_File_That_Exists()
    {
        let mut assembly = Empty_Assembly();
        let path = std::env::temp_dir().join("nomos-spec-orchestration-layer-text-of.txt");
        std::fs::write(&path, "the text")
            .expect("std::env::temp_dir() is an existing directory this process may write into");

        let text = Text_Of(&mut assembly, &Input_At(path));

        assert_eq!(text, Some("the text".to_owned()));
        assert!(assembly.absent.is_empty());
    }

    #[test]
    fn Test_Text_Of_Should_Record_An_Absence_When_The_File_Cannot_Be_Read()
    {
        let mut assembly = Empty_Assembly();
        let path = std::path::PathBuf::from("no/such/file/anywhere.txt");

        let text = Text_Of(&mut assembly, &Input_At(path));

        assert_eq!(text, None);
        assert_eq!(assembly.absent.len(), 1);
        assert_eq!(assembly.absent.first().expect("the assertion above proves one absence was recorded").subject, "an optional input");
        assert_eq!(assembly.absent.first().expect("the assertion above proves one absence was recorded").cost, "it is not read");
    }

    #[test]
    fn Test_Refuse_Input_Should_Record_A_Refusal_Against_The_Refused_Cost()
    {
        let mut assembly = Empty_Assembly();
        let input = Input_At(std::path::PathBuf::from("somewhere.txt"));
        let error = IngestError::Parse("not valid".to_owned());

        Refuse_Input(&mut assembly, &input, &error);

        assert_eq!(assembly.absent.len(), 1);
        assert_eq!(assembly.absent.first().expect("the assertion above proves one absence was recorded").subject, "an optional input");
        assert_eq!(assembly.absent.first().expect("the assertion above proves one absence was recorded").cost, "it is not in");
        assert!(
            assembly.absent.first().expect("the assertion above proves one absence was recorded").cause.contains("not valid"),
            "{}",
            assembly.absent.first().expect("the assertion above proves one absence was recorded").cause
        );
    }

    #[test]
    fn Test_Ingest_Optional_Layer_Should_Report_The_Ingest_Outcome_When_The_File_Is_Readable()
    {
        let mut assembly = Empty_Assembly();
        let path = std::env::temp_dir().join("nomos-spec-orchestration-layer-ingest-optional.txt");
        std::fs::write(&path, "parsed text")
            .expect("std::env::temp_dir() is an existing directory this process may write into");
        let input = Input_At(path);

        let report = Ingest_Optional_Layer(
            &mut assembly,
            &input,
            |text| return Ok::<String, IngestError>(text.to_owned()),
            |_store, parsed| return Ok::<String, IngestError>(parsed.clone()),
        );

        assert_eq!(report, Some("parsed text".to_owned()));
        assert!(assembly.absent.is_empty());
    }

    #[test]
    fn Test_Ingest_Optional_Layer_Should_Refuse_A_Parse_Failure_Without_Ingesting()
    {
        let mut assembly = Empty_Assembly();
        let path = std::env::temp_dir().join("nomos-spec-orchestration-layer-ingest-optional-bad.txt");
        std::fs::write(&path, "unparseable")
            .expect("std::env::temp_dir() is an existing directory this process may write into");
        let input = Input_At(path);

        let report = Ingest_Optional_Layer(
            &mut assembly,
            &input,
            |_text| -> Result<String, IngestError> { return Err(IngestError::Parse("cannot parse".to_owned())); },
            |_store, parsed| return Ok::<String, IngestError>(parsed.clone()),
        );

        assert_eq!(report, None);
        assert_eq!(assembly.absent.len(), 1);
        assert_eq!(assembly.absent.first().expect("the assertion above proves one absence was recorded").cost, "it is not in");
    }

    #[test]
    fn Test_Note_Contribution_Should_Format_The_Count_Noun_And_Path()
    {
        let mut assembly = Empty_Assembly();
        let input = Input_At(std::path::PathBuf::from("a/path.txt"));

        Note_Contribution(&mut assembly, &input, CONTRIBUTED_STATEMENTS, "statement(s)");

        assert_eq!(assembly.read, vec![format!("{CONTRIBUTED_STATEMENTS} statement(s) from a/path.txt")]);
    }

    #[test]
    fn Test_Ingest_Statement_File_Should_Record_An_Absence_When_The_Root_Has_No_Statements()
    {
        let mut assembly = Empty_Assembly();
        let root = std::env::temp_dir().join("nomos-spec-orchestration-layer-no-statements");
        std::fs::create_dir_all(&root)
            .expect("std::env::temp_dir() exists and this process may create directories under it");

        Ingest_Statement_File(&mut assembly, &root);

        assert_eq!(assembly.absent.len(), 1);
        assert_eq!(assembly.absent.first().expect("the assertion above proves one absence was recorded").subject, "the normative statements");
    }

    #[test]
    fn Test_Ingest_Catalog_File_Should_Record_An_Absence_When_The_Root_Has_No_Catalog()
    {
        let mut assembly = Empty_Assembly();
        let root = std::env::temp_dir().join("nomos-spec-orchestration-layer-no-catalog");
        std::fs::create_dir_all(&root)
            .expect("std::env::temp_dir() exists and this process may create directories under it");

        Ingest_Catalog_File(&mut assembly, &root);

        assert_eq!(assembly.absent.len(), 1);
        assert_eq!(assembly.absent.first().expect("the assertion above proves one absence was recorded").subject, "the node catalog");
    }

    #[test]
    fn Test_Refused_Absence_Should_Carry_The_Subject_Path_And_Error()
    {
        let error = IngestError::Parse("bad bytes".to_owned());

        let absence = Refused_Absence(Subject("the node catalog"), Expected("a/path.json"), &error, "nothing is in this store");

        assert_eq!(absence.subject, "the node catalog");
        assert_eq!(absence.expected, "a/path.json");
        assert_eq!(absence.cost, "nothing is in this store");
        assert!(absence.cause.contains("bad bytes"), "{}", absence.cause);
    }

    fn Empty_Assembly() -> Assembly
    {
        return Assembly {
            store: SpecificationStore::In_Memory().expect("Connection::open_in_memory() opens a database with no file behind it"),
            read: Vec::new(),
            absent: Vec::new(),
        };
    }

    fn Input_At(path: std::path::PathBuf) -> Layered<'static>
    {
        return Layered { subject: "an optional input", path, unread: "it is not read", refused: "it is not in" };
    }
}
