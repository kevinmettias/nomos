//! Reading the registry, which refuses rather than skips.
//!
//! A lenient reader here would turn a typo into an assessment that silently stopped being
//! counted, and the floor would then measure a set nobody chose. So every shape that is not
//! an entry comes back as an `Err` naming what is wrong with it.

use crate::assessment::{Assessment, Site, Verdict, REGISTRY};
use std::path::Path;

/// The extension one entry carries.
const EXTENSION: &str = "assessment";

/// Every committed assessment, or a panic naming the entry that would not read.
pub(crate) fn Committed(root: &Path) -> Vec<Assessment>
{
    return Entries(&root.join(REGISTRY))
        // `Entries` is fallible so a control can hand it a bad directory and read the refusal
        // back. The committed registry has no such caller: an entry that will not read is an
        // assessment that stopped being counted, and the floor would measure a set nobody
        // chose while still reporting a number.
        .unwrap_or_else(|refusal| panic!("the committed registry must read: {refusal}"));
}

/// Every assessment in a directory, sorted by requirement.
///
/// Takes the directory rather than finding it, so a control can hand it one.
pub(crate) fn Entries(directory: &Path) -> Result<Vec<Assessment>, String>
{
    let listing = std::fs::read_dir(directory)
        .map_err(|error| return format!("{} cannot be read: {error}", directory.display()))?;
    let mut found = Vec::new();

    for entry in listing.flatten()
    {
        let path = entry.path();
        found.extend(Read_Entry(&path)?);
    }

    found.sort_by(|left, right| return left.requirement.cmp(&right.requirement));
    return Ok(found);
}

/// The assessment one directory entry holds, or `None` for a file that is not an entry.
fn Read_Entry(path: &Path) -> Result<Option<Assessment>, String>
{
    let is_entry = path
        .extension()
        .is_some_and(|extension| return extension == EXTENSION);
    if !is_entry
    {
        return Ok(None);
    }
    let stem = path
        .file_stem()
        .and_then(|stem| return stem.to_str())
        .ok_or_else(|| return format!("{} has no readable stem", path.display()))?;
    let text = std::fs::read_to_string(path)
        .map_err(|error| return format!("{} cannot be read: {error}", path.display()))?;

    return Parse(stem, &text)
        .map(Some)
        .map_err(|refusal| return format!("{}: {refusal}", path.display()));
}

/// One entry, or the reason it is not one.
///
/// Refuses rather than skips on every malformed shape. A lenient reader here would turn a
/// typo into an assessment that silently stopped being counted, and the floor would then
/// measure a set nobody chose.
pub(crate) fn Parse(stem: &str, text: &str) -> Result<Assessment, String>
{
    if !Is_Requirement_Id(stem)
    {
        return Err(format!(
            "{stem} is not a requirement identifier; a file here is named for the \
             requirement it assesses"
        ));
    }

    let read = Read_Lines(text)?;
    let verdict = read.verdict.ok_or_else(|| {
        return "no verdict; an entry with none is not an assessment".to_owned();
    })?;

    Assert_Complete(verdict, read.record.as_deref(), &read.sites)?;

    return Ok(Assessment {
        requirement: stem.to_owned(),
        verdict,
        record: read.record,
        sites: read.sites,
    });
}

/// What the lines of an entry said, before anything is required of them.
#[derive(Default)]
struct Read
{
    verdict: Option<Verdict>,
    record: Option<String>,
    sites: Vec<Site>,
}

/// Every `key: value` line an entry holds, filed under its key.
fn Read_Lines(text: &str) -> Result<Read, String>
{
    let mut read = Read::default();

    for line in text.lines()
    {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#')
        {
            Read_One(&mut read, trimmed)?;
        }
    }

    return Ok(read);
}

/// One line, filed under the key it names.
fn Read_One(read: &mut Read, line: &str) -> Result<(), String>
{
    let Some((key, value)) = line.split_once(':')
    else
    {
        return Err(format!("`{line}` is not a `key: value` line"));
    };
    let value = value.trim();

    match key.trim()
    {
        "verdict" => read.verdict = Some(Second_Verdict(read.verdict.as_ref(), value)?),
        "record" => read.record = Some(Second_Record(read.record.as_deref(), value)?),
        "site" => read.sites.push(Read_Site(value)?),
        other => return Err(format!("unknown key `{other}`")),
    }

    return Ok(());
}

/// The verdict a line names, refusing a second one: an entry says one thing.
fn Second_Verdict(held: Option<&Verdict>, value: &str) -> Result<Verdict, String>
{
    if held.is_some()
    {
        return Err("two verdict lines; an entry says one thing".to_owned());
    }

    return Read_Verdict(value);
}

/// The record a line names, refusing an empty one and a second one.
fn Second_Record(held: Option<&str>, value: &str) -> Result<String, String>
{
    if held.is_some()
    {
        return Err(
            "two record lines; one reason, in one record, or the entry does not say which"
                .to_owned(),
        );
    }
    if value.is_empty()
    {
        return Err("an empty record line".to_owned());
    }

    return Ok(value.to_owned());
}

/// What an entry owes once its lines have been read.
fn Assert_Complete(verdict: Verdict, record: Option<&str>, sites: &[Site]) -> Result<(), String>
{
    if sites.is_empty()
    {
        return Err(
            "no site; a verdict that names no place in the workspace cannot go stale and \
             cannot be checked"
                .to_owned(),
        );
    }
    if verdict.Owes_A_Record() && record.is_none()
    {
        return Err(format!(
            "{} with no record; OD-TRACE-001 makes that not a verdict but the state it \
             exists to end",
            verdict.Label()
        ));
    }

    return Ok(());
}

/// One of the three writable verdicts.
fn Read_Verdict(value: &str) -> Result<Verdict, String>
{
    for verdict in [Verdict::Met, Verdict::Diverges, Verdict::NotBinding]
    {
        if value == verdict.Label()
        {
            return Ok(verdict);
        }
    }

    if value == "Unassessed"
    {
        return Err(
            "Unassessed is held by the absence of an entry, so writing one would record \
             that somebody looked and did not look"
                .to_owned(),
        );
    }

    return Err(format!(
        "`{value}` is not a verdict; OD-TRACE-001 names Met, Diverges and NotBinding, and \
         holds the fourth by absence"
    ));
}

/// A `path#symbol` site.
fn Read_Site(value: &str) -> Result<Site, String>
{
    let Some((path, symbol)) = value.split_once('#')
    else
    {
        return Err(format!(
            "`{value}` is not a site; a site is `path#symbol`, and a path on its own \
             survives every rename that matters"
        ));
    };
    let path = path.trim();
    let symbol = symbol.trim();
    if path.is_empty() || symbol.is_empty()
    {
        return Err(format!("`{value}` has an empty path or symbol"));
    }
    if path.starts_with('/') || path.starts_with('\\') || path.contains("..")
    {
        return Err(format!(
            "`{path}` is not repo-relative; a site outside this workspace is not a site \
             this guard can see vanish"
        ));
    }

    return Ok(Site {
        path: path.to_owned(),
        symbol: symbol.to_owned(),
    });
}

/// Whether a string is a corpus requirement identifier: a family, then three digits.
///
/// The shape every family in the corpus uses — `CHK-003`, `EVID-001`, `WORK-LEDGER-005`.
/// Checked because the stem *is* the identity, so a mistyped one would silently create a
/// requirement the corpus does not have and count it toward the floor.
pub(crate) fn Is_Requirement_Id(stem: &str) -> bool
{
    let Some((family, number)) = stem.rsplit_once('-')
    else
    {
        return false;
    };
    if number.len() != 3 || !number.bytes().all(|byte| return byte.is_ascii_digit())
    {
        return false;
    }
    if family.is_empty()
    {
        return false;
    }

    return family
        .split('-')
        .all(|segment| {
            return !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| return byte.is_ascii_uppercase() || byte.is_ascii_digit());
        });
}
