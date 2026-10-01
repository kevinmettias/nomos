//! The scan and the table must claim the same mirror, universe by universe.

use crate::table::{Standing, UNIVERSES};
use std::collections::BTreeMap;

/// `nomos check` reads a universe's mirror off its doc comment at the site; this table
/// declares one in a `by:` field. Those are two places to spell one claim, and for
/// `DECLARED_RULES` they said different things for the length of `P10-FIRST-CHECK` —
/// the command counted thirteen unmirrored universes while `UNMIRRORED_TOTAL` said twelve.
/// `OD-COMPLETENESS-002`.
///
/// Named rather than counted, deliberately. Both totals matched throughout the
/// disagreement this test exists to catch, and two rows swapping standing keeps every
/// total intact while changing what the table means.
///
/// Claims are compared rather than verdicts. Whether a claim *resolves* is
/// [`super::mirrors::Test_Every_Named_Mirror_Should_Exist_In_The_Source`]'s fact; folding the
/// two together would make a phantom agree with a row that called the universe unmirrored,
/// which is the false-coverage reading `mirror.rs`'s severity ordering exists to refuse.
#[test]
fn Test_The_Scan_And_The_Table_Should_Name_The_Same_Mirror()
{
    let scanned = Scanned_Claims();
    let declared = Declared_Claims();
    let (compared, disagreements) = Compare_Claims(&scanned, &declared);

    assert!(!scanned.is_empty(), "nothing was scanned, so every comparison below would pass having read nothing");
    assert_eq!(
        compared,
        scanned.len(),
        "{} of {} scanned universes have no row here, so this test compared less than the \
         whole set. Test_The_Declared_Table_Should_Match_What_Is_Derived says which.",
        scanned.len().saturating_sub(compared),
        scanned.len()
    );
    assert!(
        disagreements.is_empty(),
        "the workspace scan and this table disagree about these universes:\n{disagreements:#?}\n\
         The site is the claim `nomos check` resolves and this table is a copy of it. Move \
         the row to match the doc comment, or write the doc comment the row already \
         promises — and if neither is true the row is Unmirrored and UNMIRRORED_TOTAL rises."
    );
}

/// The mirror each universe claims, keyed by path and name.
type Claims = BTreeMap<(String, String), Option<String>>;

/// What each universe claims at its own site, which is what `nomos check` resolves.
fn Scanned_Claims() -> Claims
{
    use nomos_contract_tests::Declared_Universes;

    return Declared_Universes()
        .into_iter()
        .map(|universe| return ((universe.path, universe.name), universe.claimed_mirror))
        .collect();
}

/// What each row of this table claims.
fn Declared_Claims() -> Claims
{
    return UNIVERSES
        .iter()
        .map(|universe| {
            let claim = match universe.standing
            {
                Standing::Mirrored { by } => Some(by.to_owned()),
                Standing::Unmirrored { .. } => None,
            };
            return ((universe.path.to_owned(), universe.name.to_owned()), claim);
        })
        .collect();
}

/// How many universes had a row to compare against, and where the two claims differ.
///
/// Membership is `Test_The_Declared_Table_Should_Match_What_Is_Derived`'s fact, and reporting
/// it here too would give one cause two red tests. The count is what stops that deferral
/// turning into a comparison of nothing.
fn Compare_Claims(scanned: &Claims, declared: &Claims) -> (usize, Vec<String>)
{
    let mut compared = 0_usize;
    let mut disagreements: Vec<String> = Vec::new();
    for (key, site) in scanned
    {
        let Some(row) = declared.get(key)
        else
        {
            continue;
        };
        let (path, name) = key;

        compared = compared.saturating_add(1);
        if site != row
        {
            disagreements.push(format!(
                "{name} ({path}): the site claims {site:?} and the table claims {row:?}"
            ));
        }
    }

    return (compared, disagreements);
}
