//! How many pairs of record writers exclude each other, and why.
//!
//! Split out of `serializers.rs`, which says what still serializes the board — the register
//! of declared serializers and the search that finds an undeclared one. This is the other
//! question, over the same board: of the pairs the board holds, how many exclude each other,
//! and how many of those are excluded only by a path the register declares. Two subjects
//! with two answers, so the census reads as its own file rather than as the second half of
//! one that was already the longest here.

use crate::board::{PathText, Paths_Collide, RECORD_DIRECTORY};
use nomos_ledger::{LedgerItem, Normalize_Path};

use super::Declared;

/// How many pairs of record writers there are, how many exclude each other, and how many of
/// those are excluded only by a path the register declares.
#[derive(Default)]
pub(crate) struct Exclusions
{
    pub(crate) pairs: usize,
    pub(crate) blocked: usize,
    pub(crate) structural: usize,
}

pub(crate) fn Exclusions_Among(writers: &[&LedgerItem]) -> Exclusions
{
    let declared = Declared();
    let mut counted = Exclusions::default();

    for (index, left) in writers.iter().enumerate()
    {
        for right in writers.iter().skip(index.saturating_add(1))
        {
            counted.pairs = counted.pairs.saturating_add(1);
            Count_Exclusion(&mut counted, (left, right), &declared);
        }
    }

    return counted;
}

/// Whether one pair excludes the other, and whether the register is the only reason.
///
/// Anything else is two items wanting the same crate, which is territory doing its job and
/// resolves when one of them finishes.
fn Count_Exclusion(counted: &mut Exclusions, pair: (&LedgerItem, &LedgerItem), declared: &[&str])
{
    let (left, right) = pair;
    if left.territory.Intersect(&right.territory).Permits_Concurrency()
    {
        return;
    }

    counted.blocked = counted.blocked.saturating_add(1);
    if Shared_Paths(left, right).iter().all(|path| {
        return declared
            .iter()
            .any(|known| return Paths_Collide(PathText(known), PathText(path)));
    })
    {
        counted.structural = counted.structural.saturating_add(1);
    }
}

/// The non-record paths two items share, narrower spelling first.
fn Shared_Paths(left: &LedgerItem, right: &LedgerItem) -> Vec<String>
{
    let mut shared = Vec::new();

    for mine in &left.territory.paths
    {
        for broader in Broader_Of(mine, right)
        {
            if !shared.contains(&broader)
            {
                shared.push(broader);
            }
        }
    }

    shared.sort();
    return shared;
}

/// The broader spelling of every non-record path in `right` that `mine` collides with.
fn Broader_Of(mine: &str, right: &LedgerItem) -> Vec<String>
{
    if Normalize_Path(mine).starts_with(RECORD_DIRECTORY)
    {
        return Vec::new();
    }

    return right
        .territory
        .paths
        .iter()
        .filter(|theirs| return !Normalize_Path(theirs).starts_with(RECORD_DIRECTORY))
        .filter(|theirs| return Paths_Collide(PathText(mine), PathText(theirs)))
        .map(|theirs| return Broader(PathText(mine), PathText(theirs)))
        .collect();
}

/// The broader of two colliding paths, which is the one that serializes.
///
/// An item reserving a whole crate is what a file inside it collides with, and naming the
/// file would report the symptom. The shorter normalized spelling is the container — the
/// same tie-break `Covers` takes the direction of a containment from.
///
/// Both positions are a [`PathText`] rather than a `&str`, and the same one, because this
/// asks a symmetric question: the answer is whichever of the two is broader, so a caller
/// that swaps them is asking the same question and cannot be given a different answer.
fn Broader(mine: PathText<'_>, theirs: PathText<'_>) -> String
{
    if Normalize_Path(mine.0).len() <= Normalize_Path(theirs.0).len()
    {
        return Normalize_Path(mine.0);
    }

    return Normalize_Path(theirs.0);
}
