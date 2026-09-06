//! The README's zone table, parsed back out of the file a reader edits.

use crate::bands::{Repository_Root, Zone, Zone_Of, ZONE_LIST};

/// The zone table the README shows a reader, parsed back out of the file they edit.
///
/// A row is `| <zone> | `<crate>` | … |`. Anything else in the README is prose as far as
/// this is concerned, including the `D-128` table in `OD-PROJECT-001` and any table whose
/// first cell does not name one of `nomos-rules`' own declared zones.
fn Readme_Zones() -> Vec<(String, Zone)>
{
    let path = Repository_Root().join("README.md");
    let text = std::fs::read_to_string(&path)
        // The README is the one document this table is parsed back out of. Unreadable, it
        // yields no rows, and a comparison against no rows finds no disagreement — the
        // zone table would be checked against nothing and report itself intact.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    let mut rows = Vec::new();
    for line in text.lines()
    {
        let row = Zone_Row(line);

        rows.extend(row);
    }

    return rows;
}

/// One row, or nothing if the line is not one.
///
/// A table row opens with the delimiter, so the first cell is the empty string before it. A
/// line that merely contains a pipe does not. The first cell must name one of
/// [`ZONE_LIST`]'s own [`Zone::Display`] strings exactly, which is what tells a real zone
/// row apart from a heading (`| Zone | Crate | Owns |`) or a Markdown separator
/// (`|---|---|---|`) without a second, separately-maintained list of what counts as a row.
fn Zone_Row(line: &str) -> Option<(String, Zone)>
{
    let mut cells = line.split('|').map(str::trim);
    if cells.next() != Some("")
    {
        return None;
    }

    let (Some(zone_text), Some(name)) = (cells.next(), cells.next())
    else
    {
        return None;
    };
    let zone = ZONE_LIST.into_iter().find(|zone| zone.to_string() == zone_text)?;
    let name = name.strip_prefix('`').and_then(|rest| rest.strip_suffix('`'))?;

    return Some((name.to_owned(), zone));
}

/// The README describes this workspace, and nothing checked that it still did.
///
/// `D-128` requires the maintained overview documents to be freshness-validated, and
/// `OD-PROJECT-001` records why this README is not one of them: no content kind in the
/// projection system selects a crate's zone, so no profile can render this file. What
/// that record gives up is freshness for the prose. What it does not give up is the
/// table, because the table restates `nomos-rules`' own `ZONES` — the one declaration
/// `OD-RULES-020`'s migration item made this workspace's architecture, read here through
/// [`crate::bands`] rather than kept as a second copy — and a restatement can be compared.
///
/// It had already drifted when this check was first written: twenty-two members, eleven of
/// them listed, and the missing eleven included `nomos-spec-project`, the crate that
/// renders the projections `OD-PROJECT-001` is about.
///
/// Compared against `ZONES` rather than against the member list, because
/// [`crate::graph::Test_Every_Member_Should_Declare_A_Band`] already ties those two
/// together. Two checks reaching the same conclusion by different routes is how they come
/// to disagree.
#[test]
fn Test_The_Readme_Should_List_Every_Member_At_Its_Declared_Band()
{
    let listed = Readme_Zones();

    assert!(
        !listed.is_empty(),
        "no zone row was found in README.md. Every assertion below iterates over these \
         rows, so an empty set passes having read nothing — and the layout tables are \
         written as `| <zone> | `<crate>` | … |`"
    );

    Assert_No_Crate_Is_Listed_Twice(&listed);
    Assert_Every_Member_Is_Listed(&listed);
    Assert_Every_Listing_Is_A_Member(&listed);
}

/// Two rows for one crate can disagree with each other, so there is only ever one.
fn Assert_No_Crate_Is_Listed_Twice(listed: &[(String, Zone)])
{
    use std::collections::BTreeSet;

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for (name, _) in listed
    {
        assert!(
            seen.insert(name.as_str()),
            "{name} appears in more than one README zone row, so the two can disagree"
        );
    }
}

/// Every crate `ZONES` declares appears in the table a reader is shown.
fn Assert_Every_Member_Is_Listed(listed: &[(String, Zone)])
{
    use crate::bands::ZONES;

    let missing: Vec<&str> = ZONES
        .iter()
        .filter(|(name, _)| return !listed.iter().any(|(listed, _)| return listed == name))
        .map(|(name, _)| return *name)
        .collect();

    assert!(
        missing.is_empty(),
        "these crates are in the workspace and not in README.md's layout tables: \
         {missing:?}.\n\
         OD-PROJECT-001 keeps this file hand-authored on the condition that the part of \
         it a machine can check is checked."
    );
}

/// The other direction, in both halves: a row naming no crate, and a row naming the wrong
/// zone for one that exists.
fn Assert_Every_Listing_Is_A_Member(listed: &[(String, Zone)])
{
    let invented: Vec<&(String, Zone)> = listed
        .iter()
        .filter(|(name, _)| return Zone_Of(name).is_none())
        .collect();
    let disagreeing: Vec<String> = listed
        .iter()
        .filter_map(|(name, zone)| {
            let declared = Zone_Of(name)?;

            return (declared != *zone)
                .then(|| return format!("{name}: README says {zone}, ZONES says {declared}"));
        })
        .collect();

    assert!(
        invented.is_empty(),
        "README.md lists crates this workspace does not have: {invented:?}"
    );
    assert!(disagreeing.is_empty(), "{disagreeing:#?}");
}
