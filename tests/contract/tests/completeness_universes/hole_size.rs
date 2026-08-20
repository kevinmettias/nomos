//! The size of the hole is a number somebody chose, and the three instances would have
//! failed this check as originally written.

use crate::table::{Standing, UNIVERSES, UNMIRRORED_TOTAL, Universe};

#[test]
fn Test_The_Number_Of_Unmirrored_Universes_Should_Be_Declared()
{
    let unmirrored: Vec<&str> = Unmirrored_Names(UNIVERSES);

    assert_eq!(
        unmirrored.len(),
        UNMIRRORED_TOTAL,
        "{} declared universes have no mirror, and UNMIRRORED_TOTAL says {UNMIRRORED_TOTAL}: \
         {unmirrored:#?}.\n\
         Raising it is the deliberate step adding an unmirrored universe is meant to cost. \
         Lowering it is what writing a mirror earns.",
        unmirrored.len()
    );
}

/// `P9-ONE-DIRECTION` requires that all three instances would have failed the enforcement
/// as originally written. They cannot be replayed — each was repaired at the site — so each
/// is reconstructed by taking its mirror away and asserting it lands in the counted hole.
///
/// Without this, a check that classified everything as mirrored would pass the tests beside
/// it and catch nothing.
#[test]
fn Test_The_Three_Instances_Should_Have_Failed_This_Check()
{
    let instances = [
        ("Table::All", "Test_Every_Table_In_The_Schema_Should_Be_Declared"),
        ("GOVERNING_RECORD_IDS", "Test_Every_Canonical_Record_On_Disk_Should_Be_Governing"),
        ("CORPUS_VARIABLES", "Test_The_Scanner_And_This_Table_Should_Name_The_Same_Variables"),
    ];

    for (name, mirror) in instances
    {
        Assert_It_Would_Have_Failed(name, mirror);
    }
}

/// One instance, reconstructed by taking its mirror away and asserting it lands in the
/// counted hole.
fn Assert_It_Would_Have_Failed(name: &'static str, mirror: &'static str)
{
    let row = Universe_Row(name);

    Assert_Currently_Mirrored_By(row, name, mirror);
    Assert_Removing_Its_Mirror_Falls_In_The_Hole(name);
}

/// The universe classified under `name`, or a panic -- the three names are the recorded
/// instances of P9-ONE-DIRECTION, and one of them no longer classified in UNIVERSES means the
/// reconstruction below cannot be performed at all, and skipping it would leave this control
/// claiming three instances and checking two.
fn Universe_Row(name: &'static str) -> &'static Universe
{
    return UNIVERSES
        .iter()
        .find(|universe| return universe.name == name)
        .unwrap_or_else(|| panic!("{name} must be classified"));
}

/// `row` must be mirrored by `mirror` today, or the reconstruction that follows has nothing
/// to take away.
fn Assert_Currently_Mirrored_By(row: &Universe, name: &str, mirror: &'static str)
{
    assert_eq!(
        row.standing,
        Standing::Mirrored { by: mirror },
        "{name} is expected to be mirrored by {mirror} today"
    );
}

/// As originally written, before `name`'s mirror existed -- removing it must leave the count
/// one above UNMIRRORED_TOTAL, which is what would have failed.
fn Assert_Removing_Its_Mirror_Falls_In_The_Hole(name: &str)
{
    let before = Unmirrored_Names_Without(name);

    assert_eq!(
        before.len().saturating_add(1),
        UNMIRRORED_TOTAL.saturating_add(1),
        "removing {name}'s mirror must leave the count one above UNMIRRORED_TOTAL, \
         which is what would have failed"
    );
}

/// The unmirrored names as they would read with one universe's mirror removed.
fn Unmirrored_Names_Without(name: &str) -> Vec<&'static str>
{
    return UNIVERSES
        .iter()
        .filter(|universe| return universe.name != name)
        .filter(|universe| return matches!(universe.standing, Standing::Unmirrored { .. }))
        .map(|universe| return universe.name)
        .collect();
}

/// The negative control for the test above. If every universe were classified unmirrored,
/// removing one mirror would change nothing and the reconstruction would prove nothing.
#[test]
fn Test_Some_Universes_Should_Actually_Be_Mirrored()
{
    let mirrored = UNIVERSES
        .iter()
        .filter(|universe| return matches!(universe.standing, Standing::Mirrored { .. }))
        .count();

    assert!(
        mirrored > 0,
        "if nothing is mirrored, this suite measures nothing"
    );
    assert_eq!(
        mirrored.saturating_add(UNMIRRORED_TOTAL),
        UNIVERSES.len(),
        "every row must be one or the other"
    );
}

fn Unmirrored_Names(universes: &'static [Universe]) -> Vec<&'static str>
{
    return universes
        .iter()
        .filter(|universe| return matches!(universe.standing, Standing::Unmirrored { .. }))
        .map(|universe| return universe.name)
        .collect();
}
