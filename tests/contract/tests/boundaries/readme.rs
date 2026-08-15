//! The README's band table, parsed back out of the file a reader edits.

use crate::bands::{Declared_Band, Repository_Root, BANDS};

/// The band table the README shows a reader, parsed back out of the file they edit.
///
/// A row is `| <band> | `<crate>` | … |`. Anything else in the README is prose as far as
/// this is concerned, including the `D-128` table in `OD-PROJECT-001` and any table whose
/// first cell is not a number.
fn Readme_Bands() -> Vec<(String, u32)>
{
    let path = Repository_Root().join("README.md");
    let text = std::fs::read_to_string(&path)
        // The README is the one document this table is parsed back out of. Unreadable, it
        // yields no rows, and a comparison against no rows finds no disagreement — the band
        // table would be checked against nothing and report itself intact.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    let mut rows = Vec::new();
    for line in text.lines()
    {
        let row = Band_Row(line);

        rows.extend(row);
    }

    return rows;
}

/// One row, or nothing if the line is not one.
///
/// A table row opens with the delimiter, so the first cell is the empty string before it. A
/// line that merely contains a pipe does not.
fn Band_Row(line: &str) -> Option<(String, u32)>
{
    let mut cells = line.split('|').map(str::trim);
    if cells.next() != Some("")
    {
        return None;
    }

    let (Some(band), Some(name)) = (cells.next(), cells.next())
    else
    {
        return None;
    };
    let (Ok(band), Some(name)) = (
        band.parse::<u32>(),
        name.strip_prefix('`').and_then(|rest| rest.strip_suffix('`')),
    )
    else
    {
        return None;
    };

    return Some((name.to_owned(), band));
}

/// The README describes this workspace, and nothing checked that it still did.
///
/// `D-128` requires the maintained overview documents to be freshness-validated, and
/// `OD-PROJECT-001` records why this README is not one of them: no content kind in the
/// projection system selects a crate's band, so no profile can render this file. What
/// that record gives up is freshness for the prose. What it does not give up is the
/// table, because the table restates `BANDS` — declared in [`crate::common`] — and a
/// restatement can be compared.
///
/// It had already drifted when this was written: twenty-two members, eleven of them
/// listed, and the missing eleven included `nomos-spec-project`, the crate that renders
/// the projections `OD-PROJECT-001` is about.
///
/// Compared against `BANDS` rather than against the member list, because
/// [`crate::graph::Test_Every_Member_Should_Declare_A_Band`] already ties those two
/// together. Two checks reaching the same conclusion by different routes is how they come
/// to disagree.
#[test]
fn Test_The_Readme_Should_List_Every_Member_At_Its_Declared_Band()
{
    let listed = Readme_Bands();

    assert!(
        !listed.is_empty(),
        "no band row was found in README.md. Every assertion below iterates over these \
         rows, so an empty set passes having read nothing — and the layout tables are \
         written as `| <band> | `<crate>` | … |`"
    );

    Assert_No_Crate_Is_Listed_Twice(&listed);
    Assert_Every_Member_Is_Listed(&listed);
    Assert_Every_Listing_Is_A_Member(&listed);
}

/// Two rows for one crate can disagree with each other, so there is only ever one.
fn Assert_No_Crate_Is_Listed_Twice(listed: &[(String, u32)])
{
    use std::collections::BTreeSet;

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for (name, _) in listed
    {
        assert!(
            seen.insert(name.as_str()),
            "{name} appears in more than one README band row, so the two can disagree"
        );
    }
}

/// Every crate `BANDS` declares appears in the table a reader is shown.
fn Assert_Every_Member_Is_Listed(listed: &[(String, u32)])
{
    let missing: Vec<&str> = BANDS
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
/// band for one that exists.
fn Assert_Every_Listing_Is_A_Member(listed: &[(String, u32)])
{
    let invented: Vec<&(String, u32)> = listed
        .iter()
        .filter(|(name, _)| return Declared_Band(name).is_none())
        .collect();
    let disagreeing: Vec<String> = listed
        .iter()
        .filter_map(|(name, band)| {
            let declared = Declared_Band(name)?;

            return (declared != *band)
                .then(|| return format!("{name}: README says {band}, BANDS says {declared}"));
        })
        .collect();

    assert!(
        invented.is_empty(),
        "README.md lists crates this workspace does not have: {invented:?}"
    );
    assert!(disagreeing.is_empty(), "{disagreeing:#?}");
}
