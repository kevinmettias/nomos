//! Reading the registry, which refuses rather than skips.
//!
//! Ported from `tests/contract/tests/requirement_trace/registry.rs`, with one change: every
//! `std::fs::read_to_string`/`std::fs::read_dir` call is now a
//! [`nomos_platform::FileSystem`] port call, because this module is reachable from a real
//! provider (`crate::provider::Materialize_Workspace`) as well as from tests. A lenient
//! reader here would turn a typo into an assessment that silently stopped being counted, and
//! the floor would then measure a set nobody chose. So every shape that is not an entry
//! comes back as an `Err` naming what is wrong with it — this module's own leniency about a
//! *missing* registry directory is `crate::provider`'s decision, made one layer up, not
//! this one's.

use crate::assessment::{Assessment, Site, Verdict};
use nomos_platform::FileSystem;
use std::path::Path;

/// The extension one entry carries.
const EXTENSION: &str = "assessment";

/// Every assessment in a directory, sorted by requirement.
///
/// Takes the directory rather than finding it, so a caller (a test, or
/// [`crate::provider::Discover_Workspace`]) can hand it one.
///
/// # Errors
///
/// A `String` naming what is wrong, when the directory cannot be enumerated or any entry in
/// it fails to parse. This function does not skip a bad entry and continue — a typo that
/// silently stopped being counted is exactly the defect `OD-TRACE-001` exists to end.
pub fn Entries<Fs: FileSystem>(directory: &Path, filesystem: &Fs) -> Result<Vec<Assessment>, String>
{
    let listing = filesystem
        .Read_Directory(directory)
        .map_err(|error| return format!("{} cannot be read: {error}", directory.display()))?;
    let mut found = Vec::new();

    for path in listing
    {
        found.extend(Read_Entry(&path, filesystem)?);
    }

    found.sort_by(|left, right| return left.requirement.cmp(&right.requirement));
    return Ok(found);
}

/// The assessment one directory entry holds, or `None` for a file that is not an entry.
fn Read_Entry<Fs: FileSystem>(path: &Path, filesystem: &Fs) -> Result<Option<Assessment>, String>
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
    let text = filesystem
        .Read_To_String(path)
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
///
/// # Errors
///
/// A `String` describing the first way `text` fails to be a well-formed entry for
/// `stem`.
pub fn Parse(stem: &str, text: &str) -> Result<Assessment, String>
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

    Assert_Complete(verdict, read.record.as_deref(), &read.sites, &read.gaps)?;

    return Ok(Assessment {
        requirement: stem.to_owned(),
        verdict,
        record: read.record,
        sites: read.sites,
        gaps: read.gaps,
    });
}

/// What the lines of an entry said, before anything is required of them.
#[derive(Default)]
struct Read
{
    verdict: Option<Verdict>,
    record: Option<String>,
    sites: Vec<Site>,
    gaps: Vec<Site>,
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
        "gap" => read.gaps.push(Read_Site(value)?),
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
fn Assert_Complete(
    verdict: Verdict,
    record: Option<&str>,
    sites: &[Site],
    gaps: &[Site],
) -> Result<(), String>
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
    if verdict == Verdict::Partial && gaps.is_empty()
    {
        return Err(
            "Partial with no gap; OD-TRACE-003 makes that not a verdict but a softer Met, \
             which is the pressure a half-finished requirement creates and the one this \
             entry must not give in to"
                .to_owned(),
        );
    }

    return Ok(());
}

/// One of the four writable verdicts.
fn Read_Verdict(value: &str) -> Result<Verdict, String>
{
    for verdict in [Verdict::Met, Verdict::Diverges, Verdict::NotBinding, Verdict::Partial]
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
        "`{value}` is not a verdict; OD-TRACE-001 names Met, Diverges and NotBinding, \
         OD-TRACE-003 adds Partial, and Unassessed is held by absence"
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
///
/// A stem whose leading segment is `US` is refused rather than matched. The corpus writes a
/// User Story's own id as `US-` in front of the requirement id it narrates — `US-AGT-001`
/// beside `AGT-001`, `US-CHK-001` beside `CHK-003` — so the leading segment is the shape's
/// own tell for the kind. `OD-TRACE-004` decided a User Story is narrative evidence for a
/// requirement's own assessment, not a second statement this registry assesses, and this
/// check is what keeps that decision and this reader from disagreeing.
#[must_use]
pub fn Is_Requirement_Id(stem: &str) -> bool
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
    if family.split('-').next() == Some("US")
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
