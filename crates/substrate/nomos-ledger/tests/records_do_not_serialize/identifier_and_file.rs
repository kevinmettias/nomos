//! OD-LEDGER-016: the identifier and the file it names are one record.

use crate::board::{
    Contest, Contested, PathText, Is_Colliding, RECORD_DIRECTORY, Repository_Root,
    Two_Record_Writers, TwoWriters,
};
use nomos_ledger::{ClaimRefusal, Territory};
use std::path::Path;

/// The record identifier an item reserves, and the file that identifier was allocated for.
///
/// Both spellings written out, because the whole of the defect is that they used to be
/// three characters of shared text away from each other and `Territory::Intersect` compared
/// them as siblings. `OD-LEDGER-006` is a record that exists, which is what makes this the
/// amendment case rather than the allocation case.
const AN_IDENTIFIER: &str = "docs/records/OD-LEDGER-006";
const THE_FILE_IT_NAMES: &str =
    "docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md";

/// The fewest records that make the sweep below a sweep rather than a look at one file.
///
/// A floor, not a count: the directory holds hundreds and a number written here would be
/// wrong the next time a record is written. This is the point below which the comparison
/// has stopped covering the directory it claims to.
const FEWEST_RECORDS_ON_DISK: usize = 20;

/// The property `OD-LEDGER-016` buys, claimed through the ledger rather than argued.
///
/// One item reserves a record by the identifier `OD-LEDGER-001`'s authoring rule tells it
/// to reserve; another names the file outright, which is what an item amending a record
/// that already exists was reduced to doing when the identifier turned out to reserve
/// nothing. The second claim must be refused by name.
///
/// Stated over a records-only projection for the same reason
/// [`super::record_exclusion::Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer`] is:
/// two items on this board share code as well, and a refusal caused by `nomos-cli` would let
/// this pass while proving nothing about records.
#[test]
fn Test_An_Item_Naming_A_Records_File_Should_Be_Refused_By_Its_Identifiers_Holder()
{
    let TwoWriters {
        mut document,
        first,
        second,
    } = Two_Record_Writers();
    for item in &mut document.items
    {
        if item.id == first
        {
            item.territory = Territory::Of_Files([AN_IDENTIFIER]);
        }
        else if item.id == second
        {
            item.territory = Territory::Of_Files([THE_FILE_IT_NAMES]);
        }
    }
    let Contest {
        scratch: _scratch,
        refusal,
    } = Contested("record-stem", &document, &first, &second);

    assert!(
        matches!(refusal, ClaimRefusal::HeldBy { .. }),
        "the refusal must name the holder: {}",
        refusal.Describe()
    );
    assert!(
        refusal.Describe().contains("agent-a"),
        "{second} named `{THE_FILE_IT_NAMES}` while agent-a held `{AN_IDENTIFIER}`, and the \
         two are one record: {}",
        refusal.Describe()
    );
}

/// Every record in `docs/records`, as the identifier it declares and the file it is.
///
/// The identifier is read from the record's own front matter rather than from its filename,
/// so the two sides of the comparison below are written down independently — which is the
/// distinction `OD-SPEC-007` drew between a check and a derivation that compares a
/// directory against itself.
fn Records_On_Disk() -> Vec<(String, String)>
{
    let directory = Repository_Root().join("docs").join("records");
    let entries = std::fs::read_dir(&directory).expect("the record directory must be readable");
    let mut records = Vec::new();

    for entry in entries
    {
        let path = entry.expect("a record directory entry must read").path();
        records.extend(Record_At(&path));
    }

    return records;
}

/// One record's declared identifier and the path it lives at, or `None` for a file that is
/// not a record.
///
/// The extension is read case-insensitively, because the reservation is folded
/// case-insensitively and a record shouted onto disk is still a record.
fn Record_At(path: &Path) -> Option<(String, String)>
{
    let name = path.file_name().and_then(|name| return name.to_str())?;
    let is_record = path
        .extension()
        .is_some_and(|extension| return extension.eq_ignore_ascii_case("md"));
    if !is_record
    {
        return None;
    }
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{name} must be readable: {error}"));
    let identifier = text
        .lines()
        .find_map(|line| return line.trim().strip_prefix("id:").map(|id| return id.trim().to_owned()))
        .unwrap_or_else(|| panic!("{name} must declare an id in its front matter"));

    return Some((identifier, format!("{RECORD_DIRECTORY}/{name}")));
}

/// The rule against every record this repository actually has.
///
/// A fixture would prove the folding works on the one filename somebody thought to write
/// into it. This asks the ledger's own comparison about all of them, and so it is also the
/// guard on the identifier grammar: a record named in a shape the reduction does not read —
/// or a record whose filename has drifted from the identifier in its own front matter —
/// fails here rather than by silently reserving nothing when somebody comes to amend it.
#[test]
fn Test_Every_Record_On_Disk_Should_Be_One_Subject_With_Its_Identifier()
{
    let records = Records_On_Disk();
    assert!(
        records.len() >= FEWEST_RECORDS_ON_DISK,
        "the record directory must hold the records this compares; got {}",
        records.len()
    );
    let divergent: Vec<String> = records
        .iter()
        .filter(|(identifier, path)| {
            let reserved = format!("{RECORD_DIRECTORY}/{identifier}");

            return !Is_Colliding(PathText(&reserved), PathText(path.as_str()));
        })
        .map(|(identifier, path)| return format!("{identifier} != {path}"))
        .collect();

    assert!(
        divergent.is_empty(),
        "reserving these records by identifier reserves nothing, because the identifier and \
         the file do not fold onto one subject: {divergent:?}.\n\
         Either the filename does not begin with the identifier its front matter declares, \
         or the identifier is not in a shape `Normalize_Path` reads."
    );
}
