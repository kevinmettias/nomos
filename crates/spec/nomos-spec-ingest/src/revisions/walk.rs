//! What changed from each revision to the next.

use super::{RevisionFingerprint, PairChange, BTreeSet, Between_Revisions};

/// The four sets for every adjacent pair.
///
/// `reappeared` is computed against every revision before `from`, which is why this takes
/// the whole sequence rather than two revisions: a pairwise diff cannot see it.
#[must_use]
pub fn Walk(revisions: &[RevisionFingerprint]) -> Vec<PairChange>
{
    let mut pairs = Vec::new();
    let mut seen_before: BTreeSet<&str> = BTreeSet::new();

    for window in revisions.windows(2)
    {
        let (Some(from), Some(to)) = (window.first(), window.get(1))
        else
        {
            continue;
        };

        let change = Between_Revisions(from, to, &seen_before);
        pairs.push(change);
        seen_before.extend(from.documents.keys().map(String::as_str));
    }

    return pairs;
}
