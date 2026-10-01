//! Rendering `nomos spec table`'s answer, or the refusal saying why it has none.
//!
//! Resolving the address and reading the rows is `nomos-spec-orchestration::Resolved_Table`'s job
//! now. This module keeps only the writing and the `ExitCode` a rendering layer is
//! responsible for.

use crate::spec::{Assembly, TableRequest, Channels, ExitCode, DocumentSource, TableLine, RowCensus, Report_Store_Error, PathMatch, Absent_Or};
use nomos_spec_orchestration::{TableAnswer, TableRefusal};

/// Phase 2's other half: the real rows.
pub(in crate::spec) fn Read_Table(assembly: &Assembly, request: &TableRequest, channels: &mut Channels<'_>) -> ExitCode
{
    return match nomos_spec_orchestration::Resolved_Table(assembly, request)
    {
        Ok(answer) => Printed_Answer(request, &answer, channels),
        Err(TableRefusal::Store(error)) => Report_Store_Error(&error, channels.notes),
        Err(TableRefusal::NoSuchDocument) => No_Such_Document(assembly, &request.document, channels.notes),
        Err(TableRefusal::AmbiguousDocument { matched, tier }) =>
        {
            Several_Documents(&request.document, matched, tier, channels.notes)
        }
        Err(TableRefusal::NoRows { document, tier, census }) =>
        {
            let resolved = ResolvedDocument { document: &document, census, tier };
            Unselected_Rows(assembly, request, &resolved, channels.notes)
        }
    };
}

/// A document that was actually read: its path, how it was matched, and how many pipe
/// lines it carries -- the three coordinates [`Note_Document`] and [`Unselected`] both need
/// and only ever receive together, from either a resolved [`TableAnswer`] or a
/// [`TableRefusal::NoRows`].
pub(super) struct ResolvedDocument<'a>
{
    pub(super) document: &'a DocumentSource,
    pub(super) census: RowCensus,
    pub(super) tier: PathMatch,
}

/// The rows an address resolved to, with a note saying which document they came from.
pub(super) fn Printed_Answer(request: &TableRequest, answer: &TableAnswer, channels: &mut Channels<'_>) -> ExitCode
{
    let resolved = ResolvedDocument { document: &answer.document, census: answer.census, tier: answer.tier };
    Note_Document(&resolved, &request.document, channels.notes);

    return Printed_Rows(&answer.lines, request, channels);
}

/// A document that resolved and was read, and the request's own narrowing selected no rows.
pub(super) fn Unselected_Rows(
    assembly: &Assembly,
    request: &TableRequest,
    resolved: &ResolvedDocument<'_>,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    Note_Document(resolved, &request.document, notes);

    let _ = writeln!(
        notes,
        "{} carries no table row{}.",
        resolved.document.path,
        Narrowed_Description(request.block, request.table)
    );

    // The document was read, so this is an answer rather than a shortfall — unless the
    // narrowing selected a table that is not there, which the census makes visible either
    // way.
    return Nothing_Selected(assembly, resolved.census.lines, notes);
}

/// Which document was read, how it was matched, and how many pipe lines it carries.
pub(super) fn Note_Document(resolved: &ResolvedDocument<'_>, asked: &str, notes: &mut dyn std::io::Write)
{
    if resolved.tier != PathMatch::Exact
    {
        let _ = writeln!(notes, "{asked} matched {} by {}", resolved.document.path, resolved.tier.Label());
    }

    let _ = writeln!(
        notes,
        "{} at revision {}: {} pipe line(s) in the whole document — {} header, {} content, \
         {} separator",
        resolved.document.path,
        resolved.document.revision,
        resolved.census.lines,
        resolved.census.header,
        resolved.census.content,
        resolved.census.separator
    );
}

/// An address that names nothing in this store.
pub(super) fn No_Such_Document(
    assembly: &Assembly,
    asked: &str,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let _ = writeln!(notes, "no document in this store is named {asked}.");

    return Absent_Or(assembly, ExitCode::NotFound, notes);
}

/// An address that names several, which is a question rather than an answer.
pub(super) fn Several_Documents(
    asked: &str,
    matched: usize,
    tier: PathMatch,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let _ = writeln!(
        notes,
        "{asked} matches {matched} documents by {}; give a whole path.",
        tier.Label()
    );

    return ExitCode::NotFound;
}

/// A document that was read and carried no row the narrowing selected.
pub(super) fn Nothing_Selected(
    assembly: &Assembly,
    pipe_lines: u32,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    if pipe_lines == 0 && !assembly.Is_Complete()
    {
        return Absent_Or(assembly, ExitCode::NotFound, notes);
    }

    return ExitCode::NotFound;
}

/// The rows themselves, and a note saying which blocks they came from.
pub(super) fn Printed_Rows(
    lines: &[TableLine],
    request: &TableRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    for line in lines
    {
        let _ = writeln!(channels.output, "{}", line.text);
    }

    let _ = writeln!(
        channels.notes,
        "printed {} row(s){} — block(s) {}",
        lines.len(),
        Narrowed_Description(request.block, request.table),
        Block_Ordinals(lines)
    );

    return ExitCode::Ok;
}

pub(super) fn Narrowed_Description(block: Option<u32>, table: Option<u32>) -> String
{
    return match (block, table)
    {
        (None, None) => String::new(),
        (Some(block), None) => format!(" in block {block}"),
        (None, Some(table)) => format!(" in table {table} of any block"),
        (Some(block), Some(table)) => format!(" in table {table} of block {block}"),
    };
}

pub(super) fn Block_Ordinals(lines: &[nomos_spec_store::TableLine]) -> String
{
    let mut seen: Vec<u32> = Vec::new();
    for line in lines
    {
        if !seen.contains(&line.block_ordinal)
        {
            seen.push(line.block_ordinal);
        }
    }

    return seen
        .iter()
        .map(u32::to_string)
        .collect::<Vec<String>>()
        .join(", ");
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_orchestration::corpus::{Assemble_Corpus, CorpusRequest, DEFAULT_REVISION};

    /// The census a fixture document carries: every pipe line it has, then how those lines
    /// the census does not already count by identity. Named so the note the test reads and the
    /// fixture it was built from are two views of one document.
    const SAMPLE_PIPE_LINES: u32 = 4;
    const SAMPLE_CONTENT_ROWS: u32 = 2;
    const SAMPLE_NON_SEPARATOR_ROWS: u32 = 3;

    /// The block the sample lines are filed under. A block is addressed by its position, so
    /// the ordinal is a label rather than a count of anything.
    const SAMPLE_BLOCK_ORDINAL: u32 = 2;

    /// How many documents a fixture says its address matched, for the message reporting a
    /// several-way match. Nothing computes it: the fixture is standing in for a real count.
    const MATCHED_DOCUMENT_COUNT: usize = 3;

    /// How many pipe lines a fixture document is said to carry when the narrowing selected
    /// none of them.
    const SELECTED_PIPE_LINES: u32 = 5;

    /// How many rows the printed output is expected to have: one per line handed to
    /// [`Printed_Rows`].
    const PRINTED_ROW_COUNT: usize = 2;

    fn Sample_Line(block_ordinal: u32, kind: &str) -> TableLine
    {
        return TableLine {
            block_ordinal,
            table_ordinal: 1,
            row_ordinal: 1,
            kind: kind.to_owned(),
            cells: vec!["x".to_owned()],
            text: "| x |".to_owned(),
            content_hash: "h".to_owned(),
        };
    }

    #[test]
    fn Test_Read_Table_Should_Report_Absent_For_A_Document_The_Store_Never_Had()
    {
        let assembly = Corpus_Unset_Assembly();
        let request = TableRequest {
            document: "does-not-exist-p23-testing-host.md".to_owned(),
            block: None,
            table: None,
            revision: None,
        };
        let mut output = Vec::new();
        let mut notes = Vec::new();
        let mut channels = Channels { output: &mut output, notes: &mut notes };

        let code = Read_Table(&assembly, &request, &mut channels);

        assert_eq!(code, ExitCode::Absent);
    }

    #[test]
    fn Test_Printed_Answer_Should_Note_The_Document_And_Print_Its_Rows()
    {
        let answer = TableAnswer {
            document: Sample_Document(),
            tier: PathMatch::Exact,
            census: RowCensus { lines: 1, header: 0, content: 1, separator: 0, non_separator: 1 },
            lines: vec![Sample_Line(1, "content")],
        };
        let request = TableRequest { document: "a.md".to_owned(), block: None, table: None, revision: None };
        let mut output = Vec::new();
        let mut notes = Vec::new();
        let mut channels = Channels { output: &mut output, notes: &mut notes };

        let code = Printed_Answer(&request, &answer, &mut channels);

        assert_eq!(code, ExitCode::Ok);
        assert!(!output.is_empty());
        assert!(String::from_utf8_lossy(&notes).contains("1 pipe line"));
    }

    #[test]
    fn Test_Unselected_Rows_Should_Note_The_Document_And_Yield_No_Rows()
    {
        let assembly = Corpus_Unset_Assembly();
        let document = Sample_Document();
        let census = RowCensus { lines: 0, header: 0, content: 0, separator: 0, non_separator: 0 };
        let resolved = ResolvedDocument { document: &document, census, tier: PathMatch::Exact };
        let request = TableRequest { document: "a.md".to_owned(), block: None, table: None, revision: None };
        let mut notes = Vec::new();

        let code = Unselected_Rows(&assembly, &request, &resolved, &mut notes);

        assert_eq!(code, ExitCode::Absent);
        assert!(String::from_utf8_lossy(&notes).contains("carries no table row"));
    }

    #[test]
    fn Test_Note_Document_Should_Note_A_Fragment_Match_And_Always_Note_The_Line_Census()
    {
        let document = Sample_Document();
        let census = RowCensus { lines: SAMPLE_PIPE_LINES, header: 1, content: SAMPLE_CONTENT_ROWS, separator: 1, non_separator: SAMPLE_NON_SEPARATOR_ROWS };
        let resolved = ResolvedDocument { document: &document, census, tier: PathMatch::Fragment };
        let mut notes = Vec::new();

        Note_Document(&resolved, "a.md", &mut notes);

        let text = String::from_utf8_lossy(&notes);
        assert!(text.contains("a.md matched docs/records/a.md by"));
        assert!(text.contains("4 pipe line"));
    }

    #[test]
    fn Test_No_Such_Document_Should_Report_Absent_Over_A_Store_Missing_Its_Corpus()
    {
        let assembly = Corpus_Unset_Assembly();
        let mut notes = Vec::new();

        let code = No_Such_Document(&assembly, "does-not-exist.md", &mut notes);

        assert_eq!(code, ExitCode::Absent);
        assert!(String::from_utf8_lossy(&notes).contains("does-not-exist.md"));
    }

    #[test]
    fn Test_Several_Documents_Should_Report_How_Many_Matched_And_By_What()
    {
        let mut notes = Vec::new();

        let code = Several_Documents("record", MATCHED_DOCUMENT_COUNT, PathMatch::FileName, &mut notes);

        assert_eq!(code, ExitCode::NotFound);
        assert!(String::from_utf8_lossy(&notes).contains("record matches 3 documents by file name"));
    }

    #[test]
    fn Test_Nothing_Selected_Should_Prefer_Absent_Only_When_The_Document_Itself_Had_No_Lines()
    {
        let assembly = Corpus_Unset_Assembly();

        let mut notes = Vec::new();
        assert_eq!(Nothing_Selected(&assembly, 0, &mut notes), ExitCode::Absent);

        let mut notes = Vec::new();
        assert_eq!(Nothing_Selected(&assembly, SELECTED_PIPE_LINES, &mut notes), ExitCode::NotFound);
    }

    #[test]
    fn Test_Printed_Rows_Should_Print_Every_Line_And_A_Row_Count_Summary()
    {
        let lines = vec![Sample_Line(1, "content"), Sample_Line(1, "content")];
        let request = TableRequest { document: "x.md".to_owned(), block: Some(1), table: None, revision: None };
        let mut output = Vec::new();
        let mut notes = Vec::new();
        let mut channels = Channels { output: &mut output, notes: &mut notes };

        let code = Printed_Rows(&lines, &request, &mut channels);

        assert_eq!(code, ExitCode::Ok);
        assert_eq!(String::from_utf8_lossy(&output).lines().count(), PRINTED_ROW_COUNT);
        let summary = String::from_utf8_lossy(&notes);
        assert!(summary.contains("printed 2 row"));
        assert!(summary.contains("in block 1"));
    }

    #[test]
    fn Test_Narrowed_Description_Should_Describe_Each_Combination_Of_Block_And_Table()
    {
        assert_eq!(Narrowed_Description(None, None), "");
        assert_eq!(Narrowed_Description(Some(SAMPLE_BLOCK_ORDINAL), None), " in block 2");
        assert_eq!(Narrowed_Description(None, Some(1)), " in table 1 of any block");
        assert_eq!(Narrowed_Description(Some(SAMPLE_BLOCK_ORDINAL), Some(1)), " in table 1 of block 2");
    }

    #[test]
    fn Test_Block_Ordinals_Should_List_Each_Distinct_Block_Once_In_First_Seen_Order()
    {
        let lines = vec![Sample_Line(SAMPLE_BLOCK_ORDINAL, "header"), Sample_Line(SAMPLE_BLOCK_ORDINAL, "content"), Sample_Line(1, "content")];

        assert_eq!(Block_Ordinals(&lines), "2, 1");
    }

    /// A store with the embedded governing records seeded and no corpus, so
    /// [`Assembly::Is_Complete`] is deterministically `false` -- `root: None` records an
    /// absence unconditionally, regardless of what any real environment variable holds.
    fn Corpus_Unset_Assembly() -> Assembly
    {
        let request = CorpusRequest {
            variable: "NOMOS_SPEC_TABLE_TEST_CORPUS_UNSET".to_owned(),
            root: None,
            revision: DEFAULT_REVISION.to_owned(),
        };

        return Assemble_Corpus(&request).expect("the only fallible step is seeding the records this binary embeds into an in-memory store");
    }

    fn Sample_Document() -> DocumentSource
    {
        return DocumentSource {
            uid: 1,
            path: "docs/records/a.md".to_owned(),
            revision: "authored".to_owned(),
            content_hash: "h".to_owned(),
            text: String::new(),
        };
    }
}
