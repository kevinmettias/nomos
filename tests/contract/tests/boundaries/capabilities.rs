//! A capability id is written in one crate's library source and nowhere else.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The prefix every capability id in this workspace carries.
const CAPABILITY_PREFIX: &str = "nomos.cap.";

/// A capability id is written in one crate's library source and nowhere else.
///
/// A capability contract is the agreed meaning of a question and the ceiling on what any
/// answer may claim, and an agreement is not the property of one party to it. Two crates
/// spelling one capability id are two parties who agree because somebody retyped a string,
/// and nothing notices the day one of them is retyped differently.
///
/// That is not hypothetical. `nomos.cap.syntax.items` was declared in `nomos-lang-rust` and
/// spelled again in `nomos-lang-rust-scan`, which offers against it and cannot name its
/// peer — two providers of one capability sit at the same band and the downward rule in
/// [`crate::graph`] forbids the edge. The remedy is a home below both, and `nomos-cap-syntax`
/// is it.
///
/// # Why only `src`
///
/// A declaration is library code. A test may name any capability it likes, including one
/// another crate declares, because naming is not declaring — `tests/integration` asserts
/// over `nomos.cap.syntax.items of alpha/one.rs` and is not a second party to anything.
#[test]
fn Test_A_Capability_Id_Should_Be_Written_In_One_Crate()
{
    let spelled_by = Capability_Ids_By_Crate();

    assert!(
        !spelled_by.is_empty(),
        "no capability id was found in any crate's source. Every assertion below iterates \
         over this map, so an empty one passes having checked nothing — and this workspace \
         has capabilities"
    );

    let shared: Vec<(&String, &BTreeSet<String>)> = spelled_by
        .iter()
        .filter(|(_, crates)| crates.len() > 1)
        .collect();

    assert!(
        shared.is_empty(),
        "these capability ids are written in more than one crate: {shared:#?}.\n\
         A capability contract is an agreement, and an agreement is not the property of a \
         party to it. Move the id to a crate below everything that offers against it, and \
         let the parties import it."
    );
}

/// Every capability id written in library source, and the crates that write it.
fn Capability_Ids_By_Crate() -> BTreeMap<String, BTreeSet<String>>
{
    use crate::bands::Source_Files;
    use nomos_contract_tests::Workspace;

    let workspace = Workspace::Load();
    let mut spelled_by: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for member in workspace.Members()
    {
        let source_root = member.root.join("src");
        if !source_root.is_dir()
        {
            continue;
        }
        for file in Source_Files(&source_root)
        {
            Note_The_Ids_In(&file, &member.name, &mut spelled_by);
        }
    }

    return spelled_by;
}

/// The ids one file writes, filed under the crate that wrote them.
fn Note_The_Ids_In(file: &Path, member: &str, spelled: &mut BTreeMap<String, BTreeSet<String>>)
{
    let Ok(text) = std::fs::read_to_string(file)
    else
    {
        return;
    };

    for id in Capability_Ids_In(&text)
    {
        spelled.entry(id).or_default().insert(member.to_owned());
    }
}

/// Every `nomos.cap.…` id a file declares, as written.
///
/// A capability named in prose is a reference, not a declaration, so commented lines are
/// passed over. Every doc comment in this workspace that explains a capability would
/// otherwise read as a second party to it.
fn Capability_Ids_In(text: &str) -> Vec<String>
{
    let mut found = Vec::new();

    for line in text.lines()
    {
        let trimmed = line.trim();
        if trimmed.starts_with("//")
        {
            continue;
        }

        let spelled = Capability_Ids(trimmed);
        found.extend(spelled);
    }

    return found;
}

/// Every `nomos.cap.…` id in a line, as written.
fn Capability_Ids(line: &str) -> Vec<String>
{
    let mut found = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find(CAPABILITY_PREFIX)
    {
        let Some(after) = rest.get(start..)
        else
        {
            break;
        };
        let id = Id_At(after);
        let Some(remainder) = after.get(id.len()..)
        else
        {
            break;
        };

        found.push(id);
        rest = remainder;
    }

    return found;
}

/// The id that starts here: every character an id may be made of, up to the first that is not.
fn Id_At(after: &str) -> String
{
    return after
        .chars()
        .take_while(|character| {
            return character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-');
        })
        .collect();
}
