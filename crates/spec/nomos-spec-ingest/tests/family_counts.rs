//! P3-COUNTS. Every restored family measured, with the definition that produced it.
//!
//! The register at `tests/corpus/families/counts.json` is the single home for these
//! numbers. Each entry carries what it counted, how it was extracted and what it was
//! measured over, and where the plan states a different figure the plan's is recorded as
//! superseded rather than quietly replaced — D-132 makes that the rule rather than a
//! habit, and OD-SPEC-002 already settled the first two instances of it.
//!
//! A definition written only in prose is a definition nothing checks, so every entry has
//! an extractor here and an entry with none fails. The extractor is the definition; the
//! sentence in the register is its reading.

use nomos_spec_ingest::{Archive, Parse_Catalog};
use nomos_spec_model::{BlockKind, Segment, SourceBlock};
use nomos_spec_store::{RowScope, SpecificationStore};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const REGISTER: &str = include_str!("../../../../tests/corpus/families/counts.json");

const V15: &str = "nomos-spec-v15.0.zip";

const VOLUMES: &str = "01_authoring/domain_volumes";

/// The families the plan's I5 names. A register that stops carrying one of them fails,
/// so a family cannot leave the restoration by leaving the register.
const REQUIRED_FAMILIES: &[&str] = &[
    "table_row",
    "code_block",
    "canonical_domain_model",
    "roadmap_milestone",
    "scenario",
    "service",
    "appendix_d",
    "appendix_h",
    "headless_inventory",
    "ide_profile",
    "glossary_term",
    "catalog_entity",
    "v15_record",
];

#[derive(Debug, Deserialize)]
struct Entry
{
    id: String,
    family: String,
    measured: u32,
    unit: String,
    definition: String,
    corpus: String,
    plan: Option<PlanFigure>,
}

#[derive(Debug, Deserialize)]
struct PlanFigure
{
    /// Absent where the plan names the family but states no number.
    figure: Option<u32>,
    named: String,
    status: String,
    note: String,
}

fn Register() -> Vec<Entry>
{
    return serde_json::from_str(REGISTER).expect("the register does not parse");
}

fn Corpus() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_V14_CORPUS")?);
    assert!(root.is_dir(), "NOMOS_V14_CORPUS is not a directory: {}", root.display());
    return Some(root);
}

fn Archives() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_SPEC_ARCHIVES")?);
    assert!(root.is_dir(), "NOMOS_SPEC_ARCHIVES is not a directory: {}", root.display());
    return Some(root);
}

// ---------------------------------------------------------------- structure

/// Runs without a corpus, and is the half that keeps the register honest about itself.
#[test]
fn Test_Every_Entry_Should_Say_What_It_Counted_And_Over_What()
{
    for entry in Register()
    {
        for (field, value) in [
            ("family", &entry.family),
            ("unit", &entry.unit),
            ("definition", &entry.definition),
            ("corpus", &entry.corpus),
        ]
        {
            assert!(!value.trim().is_empty(), "{}: {field} is empty", entry.id);
        }
    }
}

#[test]
fn Test_Entry_Identifiers_Should_Be_Unique()
{
    let entries = Register();
    let distinct: BTreeSet<&str> = entries.iter().map(|entry| return entry.id.as_str()).collect();

    assert_eq!(distinct.len(), entries.len(), "two entries share an identifier");
}

/// Every family the restoration owes an answer for is in the register, and nothing else
/// is. A family cannot leave the restoration by leaving the register, and one cannot join
/// it without being declared.
#[test]
fn Test_The_Registered_Families_Should_Be_Exactly_The_Required_Ones()
{
    let entries = Register();
    let present: BTreeSet<String> = entries.iter().map(|entry| return entry.family.clone()).collect();

    for required in REQUIRED_FAMILIES
    {
        assert!(present.contains(*required), "{required} carries no measured count");
    }
    for family in &present
    {
        assert!(
            REQUIRED_FAMILIES.contains(&family.as_str()),
            "{family} is measured and undeclared"
        );
    }
}

/// A disagreement with the plan is recorded as one. Silently agreeing figures may not be
/// marked superseded either, or the marker stops meaning anything.
#[test]
fn Test_A_Plan_Figure_Should_Be_Superseded_Exactly_When_It_Differs()
{
    for entry in Register()
    {
        let Some(plan) = entry.plan
        else
        {
            continue;
        };

        assert!(!plan.note.trim().is_empty(), "{}: the plan's figure carries no reading", entry.id);
        assert!(!plan.named.trim().is_empty(), "{}: the plan's figure names nothing", entry.id);

        match plan.figure
        {
            None => assert_eq!(
                plan.status, "unnumbered",
                "{}: the plan states no figure, so nothing can be superseded",
                entry.id
            ),
            Some(figure) =>
            {
                assert_eq!(
                    plan.status, "superseded",
                    "{}: a stated figure is either superseded or absent from the register",
                    entry.id
                );
                assert!(
                    figure != entry.measured || plan.note.contains("count"),
                    "{}: the plan's figure equals the measurement and the note does not say \
                     what it counted instead",
                    entry.id
                );
            }
        }
    }
}

// ---------------------------------------------------------------- measurement

/// The half that needs the corpus. Every entry is re-measured; an entry with no extractor
/// fails rather than passing unmeasured.
#[test]
fn Test_Every_Registered_Count_Should_Reproduce_From_The_Corpus()
{
    let (Some(corpus), Some(archives)) = (Corpus(), Archives())
    else
    {
        assert!(
            Corpus().is_none() && Archives().is_none(),
            "one of NOMOS_V14_CORPUS and NOMOS_SPEC_ARCHIVES is set without the other, so \
             half the register would go unmeasured"
        );
        return;
    };

    let mut checked = 0_u32;
    for entry in Register()
    {
        let measured = Measure(&entry.id, &corpus, &archives);
        assert_eq!(
            measured, entry.measured,
            "{}: the register says {} {}(s) and the corpus says {measured}. \
             Definition: {}",
            entry.id, entry.measured, entry.unit, entry.definition
        );
        checked = checked.saturating_add(1);
    }

    assert_eq!(
        checked,
        u32::try_from(Register().len()).unwrap_or(u32::MAX),
        "an entry was skipped"
    );
}

/// The extractor for one entry. An unknown identifier panics: a register entry nothing
/// measures is the shape this whole item exists to remove.
fn Measure(id: &str, corpus: &Path, archives: &Path) -> u32
{
    let volumes = corpus.join(VOLUMES);

    return match id
    {
        "table.pipe_lines" => Volume_Census(&volumes).lines,
        "table.header_rows" => Volume_Census(&volumes).header,
        "table.content_rows" => Volume_Census(&volumes).content,
        "table.separator_rows" => Volume_Census(&volumes).separator,
        "table.non_separator_rows" => Volume_Census(&volumes).non_separator,
        // One delimiter per table is an invariant the store enforces on write, so the
        // delimiter count is the table count. Stated as its own definition rather than
        // read off the separator row, because the two mean different things.
        "table.tables" => Tables(&volumes),

        "code.fence_lines" => Fence_Lines(&volumes),
        "code.blocks" => Code_Blocks(&volumes),

        "domain_model.pipe_lines" =>
        {
            Table_Under(corpus, "02-core", "5. Canonical domain model").lines
        }
        "domain_model.rows" => Table_Under(corpus, "02-core", "5. Canonical domain model").content,
        "domain_model.named_models" => Named_Models(corpus),

        "roadmap.milestones" => Headings_Matching(corpus, "08-roadmap", 3, &["Foundation ", "Release "]),
        "roadmap.releases" => Headings_Matching(corpus, "08-roadmap", 3, &["Release "]),

        "scenario.appendix_g_sections" => Lettered(corpus, "09-reference", 3, 'G', 1),
        "scenario.end_to_end" => End_To_End(corpus),

        "service.section_6_headings" => Section_Six(corpus).all,
        "service.leaf_headings" => Section_Six(corpus).leaves,
        "service.service_headings" => Section_Six(corpus).services,
        "service.subsystem_table_rows" => {
            Table_Under(corpus, "02-core", "6. Systems and subsystem responsibilities").content
        }

        "appendix_d.sections" => Lettered(corpus, "09-reference", 3, 'D', 1),
        "appendix_d.report_profiles" => Lettered(corpus, "09-reference", 4, 'D', 2),
        "appendix_d.members" => Lettered(corpus, "09-reference", 3, 'D', 1)
            .saturating_add(Lettered(corpus, "09-reference", 4, 'D', 2)),
        "appendix_h.sections" => Lettered(corpus, "06-agents", 3, 'H', 1),
        "headless_inventory.sections" => Prefixed(corpus, "07-clients", 4, "E.1."),
        "ide_profiles.sections" => Prefixed(corpus, "07-clients", 4, "F.1."),

        "glossary.table_terms" => Table_Under(corpus, "09-reference", "Glossary").content,
        "glossary.extended_terms" => Under_Path(corpus, "09-reference", 4, "Extended operational terms"),
        "glossary.definitions" => Table_Under(corpus, "09-reference", "Glossary")
            .content
            .saturating_add(Under_Path(corpus, "09-reference", 4, "Extended operational terms")),

        "catalog.entities" => Catalog_Entities(corpus),
        "v15.records" => V15_Records(archives),

        other => panic!("{other} is in the register and nothing measures it"),
    };
}

// ---------------------------------------------------------------- extractors

fn Volume(corpus: &Path, stem: &str) -> String
{
    let directory = corpus.join(VOLUMES);

    for path in Markdown_Files(&directory)
    {
        let name = path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("");
        if name.starts_with(stem)
        {
            return std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        }
    }

    panic!("no volume beginning {stem} in {}", directory.display());
}

fn Markdown_Files(directory: &Path) -> Vec<PathBuf>
{
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| return entry.path())
        .filter(|path| path.extension().and_then(std::ffi::OsStr::to_str) == Some("md"))
        .collect();
    paths.sort();

    assert!(!paths.is_empty(), "{} holds no markdown", directory.display());
    return paths;
}

/// The row census over every domain volume, taken through the store so the numbers come
/// from the same segmenter and the same typing the ingest uses.
fn Volume_Census(volumes: &Path) -> nomos_spec_store::RowCensus
{
    let mut store = SpecificationStore::In_Memory().expect("opens");

    for path in Markdown_Files(volumes)
    {
        let markdown = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let name = path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("?");
        let document = store
            .Put_Source_Document(name, "v14.36", &markdown)
            .expect("stores the document");
        store
            .Put_Source_Blocks(document, &Segment(&markdown))
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }

    return store.Row_Census(RowScope::Everything).expect("takes a census");
}

fn Tables(volumes: &Path) -> u32
{
    let mut tables = 0_u32;

    for path in Markdown_Files(volumes)
    {
        let markdown = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        for block in Segment(&markdown)
        {
            let highest = nomos_spec_model::Table_Rows(&block)
                .iter()
                .map(|row| return row.table_ordinal)
                .max()
                .unwrap_or(0);
            tables = tables.saturating_add(highest);
        }
    }

    return tables;
}

fn Fence_Lines(volumes: &Path) -> u32
{
    let mut fences = 0_u32;

    for path in Markdown_Files(volumes)
    {
        let markdown = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        for line in markdown.lines()
        {
            if line.trim_start().starts_with("```")
            {
                fences = fences.saturating_add(1);
            }
        }
    }

    return fences;
}

fn Code_Blocks(volumes: &Path) -> u32
{
    let mut blocks = 0_u32;

    for path in Markdown_Files(volumes)
    {
        let markdown = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        for block in Segment(&markdown)
        {
            if block.kind == BlockKind::Code
            {
                blocks = blocks.saturating_add(1);
            }
        }
    }

    return blocks;
}

/// The pipe lines and the data rows of the one table sitting directly under a heading.
///
/// Refuses a heading carrying more than one table rather than summing them: "the table
/// under §5" would then mean something the register's sentence does not say.
fn Table_Under(corpus: &Path, stem: &str, heading: &str) -> TableCounts
{
    let markdown = Volume(corpus, stem);
    let mut found: Vec<SourceBlock> = Vec::new();

    for block in Segment(&markdown)
    {
        let directly_under = block.heading_path.last().map(String::as_str) == Some(heading);
        if directly_under && !nomos_spec_model::Table_Rows(&block).is_empty()
        {
            found.push(block);
        }
    }

    assert_eq!(found.len(), 1, "{heading} in {stem} carries {} tables, not one", found.len());

    let rows = found.first().map(nomos_spec_model::Table_Rows).unwrap_or_default();
    let lines = u32::try_from(rows.len()).unwrap_or(u32::MAX);
    let content = u32::try_from(
        rows.iter()
            .filter(|row| return row.kind == nomos_spec_model::RowKind::Content)
            .count(),
    )
    .unwrap_or(u32::MAX);

    return TableCounts { lines, content };
}

/// What one table under a heading amounts to.
///
/// Named rather than a pair. Both members are `u32` and the compiler cannot tell them
/// apart, so a call site reading the wrong position gets a number that looks right.
struct TableCounts
{
    lines: u32,
    content: u32,
}

/// Counted through the restoration's own reader, not a second implementation of the split.
/// Two readings of "does this cell name one model or three" would be two authorities on
/// how many models the corpus has.
fn Named_Models(corpus: &Path) -> u32
{
    let markdown = Volume(corpus, "02-core");
    let mut models = 0_u32;

    for block in Segment(&markdown)
    {
        if block.heading_path.last().map(String::as_str) != Some("5. Canonical domain model")
        {
            continue;
        }
        for row in nomos_spec_model::Table_Rows(&block)
            .iter()
            .filter(|row| return row.kind == nomos_spec_model::RowKind::Content)
        {
            let cell = row.cells.first().map_or("", |cell| return cell.trim());
            models = models.saturating_add(
                u32::try_from(nomos_spec_ingest::Models_In(cell).len()).unwrap_or(u32::MAX),
            );
        }
    }

    return models;
}

struct Heading
{
    depth: usize,
    title: String,
    path: Vec<String>,
}

/// Headings as the segmenter sees them, which is what keeps a `###` inside a fenced block
/// from being counted as a section.
fn Headings(markdown: &str) -> Vec<Heading>
{
    return Segment(markdown)
        .into_iter()
        .filter(|block| return block.kind == BlockKind::Heading)
        .map(|block| {
            return Heading {
                depth: block.text.chars().take_while(|character| return *character == '#').count(),
                title: block.text.trim_start_matches('#').trim().to_owned(),
                path: block.heading_path,
            };
        })
        .collect();
}

fn Headings_Matching(corpus: &Path, stem: &str, depth: usize, prefixes: &[&str]) -> u32
{
    let markdown = Volume(corpus, stem);
    let matched = Headings(&markdown)
        .iter()
        .filter(|heading| return heading.depth == depth)
        .filter(|heading| {
            return prefixes.iter().any(|prefix| {
                return heading
                    .title
                    .strip_prefix(*prefix)
                    .is_some_and(|rest| return rest.starts_with(|c: char| return c.is_ascii_digit()));
            });
        })
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

/// Appendix sections, addressed the way the documents number them: a letter, then
/// `parts` dotted numbers, then a space.
fn Lettered(corpus: &Path, stem: &str, depth: usize, letter: char, parts: usize) -> u32
{
    let markdown = Volume(corpus, stem);
    let matched = Headings(&markdown)
        .iter()
        .filter(|heading| return heading.depth == depth)
        .filter(|heading| return Is_Lettered(&heading.title, letter, parts))
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

fn Is_Lettered(title: &str, letter: char, parts: usize) -> bool
{
    let Some(rest) = title.strip_prefix(letter)
    else
    {
        return false;
    };
    let Some((numbering, _)) = rest.split_once(' ')
    else
    {
        return false;
    };

    let segments: Vec<&str> = numbering.split('.').collect();
    if segments.len() != parts.saturating_add(1)
    {
        return false;
    }

    return segments.first() == Some(&"")
        && segments
            .iter()
            .skip(1)
            .all(|segment| return !segment.is_empty() && segment.chars().all(|c| return c.is_ascii_digit()));
}

fn Prefixed(corpus: &Path, stem: &str, depth: usize, prefix: &str) -> u32
{
    let markdown = Volume(corpus, stem);
    let matched = Headings(&markdown)
        .iter()
        .filter(|heading| return heading.depth == depth)
        .filter(|heading| {
            return heading
                .title
                .strip_prefix(prefix)
                .is_some_and(|rest| return rest.starts_with(|c: char| return c.is_ascii_digit()));
        })
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

fn Under_Path(corpus: &Path, stem: &str, depth: usize, ancestor: &str) -> u32
{
    let markdown = Volume(corpus, stem);
    let matched = Headings(&markdown)
        .iter()
        .filter(|heading| return heading.depth == depth)
        .filter(|heading| return heading.path.iter().any(|step| return step == ancestor))
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

fn End_To_End(corpus: &Path) -> u32
{
    let markdown = Volume(corpus, "09-reference");
    let matched = Headings(&markdown)
        .iter()
        .filter(|heading| return heading.depth == 3)
        .filter(|heading| {
            return Is_Lettered(&heading.title, 'G', 1)
                && heading.title.contains("End-to-end scenario:");
        })
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

/// Every heading of section 6, its leaves, and the leaves naming a service.
fn Section_Six(corpus: &Path) -> SectionCounts
{
    const SECTION: &str = "6. Systems and subsystem responsibilities";

    let markdown = Volume(corpus, "02-core");
    let headings = Headings(&markdown);

    let mut inside = false;
    let (mut all, mut leaves, mut services) = (0_u32, 0_u32, 0_u32);

    for heading in &headings
    {
        if heading.title == SECTION
        {
            inside = true;
        }
        else if inside && heading.depth <= 2
        {
            break;
        }
        if !inside
        {
            continue;
        }

        all = all.saturating_add(1);
        if heading.depth == 4
        {
            leaves = leaves.saturating_add(1);
            if heading.title.split_whitespace().any(|word| return word == "Service")
            {
                services = services.saturating_add(1);
            }
        }
    }

    assert!(all > 0, "section 6 is no longer in volume 02 under that title");

    return SectionCounts {
        all,
        leaves,
        services,
    };
}

/// What section 6 amounts to: every heading, the leaves, and the leaves naming a service.
///
/// Named rather than a triple of `u32`. At three members of one type a caller is counting
/// positions and the compiler is helping with none of it.
struct SectionCounts
{
    all: u32,
    leaves: u32,
    services: u32,
}

fn Catalog_Entities(corpus: &Path) -> u32
{
    let path = corpus.join("02_machine/catalog/catalog.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let entities = Parse_Catalog(&text).unwrap_or_else(|error| panic!("{error}"));

    return u32::try_from(entities.len()).unwrap_or(u32::MAX);
}

fn V15_Records(archives: &Path) -> u32
{
    let archive =
        Archive::Open(&archives.join(V15)).unwrap_or_else(|error| panic!("{error}"));
    let matched = archive
        .Listing()
        .Ending_With(".md")
        .iter()
        .filter(|entry| return entry.contains("/records/"))
        .count();

    return u32::try_from(matched).unwrap_or(u32::MAX);
}

/// The store's own row census, reached through the same fixture, so the three counts the
/// report quotes are the same three the store answers rather than a second reading taken
/// alongside it.
#[test]
fn Test_The_Register_Should_Agree_With_The_Store_Census()
{
    let Some(corpus) = Corpus()
    else
    {
        return;
    };

    let census = Volume_Census(&corpus.join(VOLUMES));
    let register = Register();
    let value = |id: &str| {
        return register
            .iter()
            .find(|entry| return entry.id == id)
            .map_or_else(|| panic!("{id} left the register"), |entry| return entry.measured);
    };

    assert_eq!(census.lines, value("table.pipe_lines"));
    assert_eq!(census.header, value("table.header_rows"));
    assert_eq!(census.content, value("table.content_rows"));
    assert_eq!(census.separator, value("table.separator_rows"));
    assert_eq!(census.non_separator, value("table.non_separator_rows"));

    // The store's fixture and the register are only meaningfully compared when the two
    // could differ. If a fresh store answered zero for everything the assertions above
    // would still hold against a register of zeroes.
    assert!(census.lines > 0, "the census read nothing, so it agreed with nothing");
}
