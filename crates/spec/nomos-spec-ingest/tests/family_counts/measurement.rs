//! Every registered count, re-measured against the corpus it was taken over.
//!
//! Both corpus-gated tests are here with the two roots that gate them, and that is forced
//! rather than tidy: `tests/contract/src/gates.rs` resolves a test to the corpora it reaches
//! by following calls **within one file**, and neither test names `NOMOS_V14_CORPUS` or
//! `NOMOS_SPEC_ARCHIVES` itself. A gated test in a sibling module would be gated in fact and
//! counted by nobody.
//!
//! An entry with no extractor fails rather than passing unmeasured. That is the whole point
//! of the register: a definition written only in prose is a definition nothing checks.

use crate::catalog::{Catalog_Entities, V15_Records};
use crate::register::{Register, VOLUMES};
use crate::headings::{
    Appendix, End_To_End, Headings_Matching, Lettered, Prefixed, Section_Six, Under_Path,
};
use crate::volumes::{Code_Blocks, Fence_Lines, Named_Models, Table_Under, Tables, Volume_Census};
use std::path::{Path, PathBuf};

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
    let measured = Table_Figure(id, corpus)
        .or_else(|| return Volume_Figure(id, corpus))
        .or_else(|| return Appendix_Figure(id, corpus))
        .or_else(|| return Glossary_Figure(id, corpus))
        .or_else(|| return Whole_Corpus_Figure(id, corpus, archives));

    return measured.unwrap_or_else(|| {
        // Every extractor above answered None, so the register names a figure this file holds
        // no definition for. Skipping the entry would leave a count in the register measured
        // by nothing, which is the exact shape the register exists to make impossible.
        panic!("{id} is in the register and nothing measures it");
    });
}

/// The tables and the code across all ten volumes.
fn Table_Figure(id: &str, corpus: &Path) -> Option<u32>
{
    let volumes = corpus.join(VOLUMES);

    return match id
    {
        "table.pipe_lines" => Some(Volume_Census(&volumes).lines),
        "table.header_rows" => Some(Volume_Census(&volumes).header),
        "table.content_rows" => Some(Volume_Census(&volumes).content),
        "table.separator_rows" => Some(Volume_Census(&volumes).separator),
        "table.non_separator_rows" => Some(Volume_Census(&volumes).non_separator),
        // One delimiter per table is an invariant the store enforces on write, so the
        // delimiter count is the table count. Stated as its own definition rather than
        // read off the separator row, because the two mean different things.
        "table.tables" => Some(Tables(&volumes)),
        "code.fence_lines" => Some(Fence_Lines(&volumes)),
        "code.blocks" => Some(Code_Blocks(&volumes)),
        _ => None,
    };
}

/// The figures a single volume answers: the domain model, the roadmap, the services.
fn Volume_Figure(id: &str, corpus: &Path) -> Option<u32>
{
    return match id
    {
        "domain_model.pipe_lines" =>
        {
            Some(Table_Under(corpus, "02-core", "5. Canonical domain model").lines)
        }
        "domain_model.rows" =>
        {
            Some(Table_Under(corpus, "02-core", "5. Canonical domain model").content)
        }
        "domain_model.named_models" => Some(Named_Models(corpus)),
        "roadmap.milestones" =>
        {
            Some(Headings_Matching(corpus, "08-roadmap", 3, &["Foundation ", "Release "]))
        }
        "roadmap.releases" => Some(Headings_Matching(corpus, "08-roadmap", 3, &["Release "])),
        "scenario.end_to_end" => Some(End_To_End(corpus)),
        "service.section_6_headings" => Some(Section_Six(corpus).all),
        "service.leaf_headings" => Some(Section_Six(corpus).leaves),
        "service.service_headings" => Some(Section_Six(corpus).services),
        "service.subsystem_table_rows" =>
        {
            Some(Table_Under(corpus, "02-core", "6. Systems and subsystem responsibilities").content)
        }
        _ => None,
    };
}

/// The lettered appendices, counted by the depth their sections sit at.
fn Appendix_Figure(id: &str, corpus: &Path) -> Option<u32>
{
    let sections = Appendix {
        letter: 'D',
        parts: 1,
    };
    let profiles = Appendix {
        letter: 'D',
        parts: 2,
    };

    return match id
    {
        "scenario.appendix_g_sections" =>
        {
            Some(Lettered(corpus, "09-reference", 3, Appendix { letter: 'G', parts: 1 }))
        }
        "appendix_d.sections" => Some(Lettered(corpus, "09-reference", 3, sections)),
        "appendix_d.report_profiles" => Some(Lettered(corpus, "09-reference", 4, profiles)),
        "appendix_d.members" =>
        {
            let under = Lettered(corpus, "09-reference", 3, sections);
            let reports = Lettered(corpus, "09-reference", 4, profiles);

            Some(under.saturating_add(reports))
        }
        "appendix_h.sections" =>
        {
            Some(Lettered(corpus, "06-agents", 3, Appendix { letter: 'H', parts: 1 }))
        }
        "headless_inventory.sections" => Some(Prefixed(corpus, "07-clients", 4, "E.1.")),
        "ide_profiles.sections" => Some(Prefixed(corpus, "07-clients", 4, "F.1.")),
        _ => None,
    };
}

/// The glossary, which is a table and a prose section that together define the terms.
fn Glossary_Figure(id: &str, corpus: &Path) -> Option<u32>
{
    let extended = "Extended operational terms";

    return match id
    {
        "glossary.table_terms" =>
        {
            Some(Table_Under(corpus, "09-reference", "Glossary").content)
        }
        "glossary.extended_terms" =>
        {
            Some(Under_Path(corpus, "09-reference", 4, extended))
        }
        "glossary.definitions" =>
        {
            let tabled = Table_Under(corpus, "09-reference", "Glossary").content;
            let prose = Under_Path(corpus, "09-reference", 4, extended);

            Some(tabled.saturating_add(prose))
        }
        _ => None,
    };
}

/// The figures taken over something other than the volumes.
fn Whole_Corpus_Figure(id: &str, corpus: &Path, archives: &Path) -> Option<u32>
{
    return match id
    {
        "catalog.entities" => Some(Catalog_Entities(corpus)),
        "v15.records" => Some(V15_Records(archives)),
        _ => None,
    };
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
            // The five assertions below name their register ids literally. An id that has
            // left the register would otherwise resolve to a default, and the census would
            // look like it agreed with a figure nobody states any more.
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
