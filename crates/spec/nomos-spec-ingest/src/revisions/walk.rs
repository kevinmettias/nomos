//! What changed from each revision to the next.

use super::{RevisionFingerprint, PairChange, BTreeSet, Between_Revisions};

/// A window over two neighbours, which is what "adjacent" means to a sequence.
const ADJACENT_PAIR: usize = 2;

/// The four sets for every adjacent pair.
///
/// `reappeared` is computed against every revision before `from`, which is why this takes
/// the whole sequence rather than two revisions: a pairwise diff cannot see it.
#[must_use]
pub fn Walk_Revisions(revisions: &[RevisionFingerprint]) -> Vec<PairChange>
{
    let mut pairs = Vec::new();
    let mut seen_before: BTreeSet<&str> = BTreeSet::new();

    for window in revisions.windows(ADJACENT_PAIR)
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

#[cfg(test)]
mod tests
{
    use super::*;

    fn Revision(label: &str, documents: &[(&str, &str)]) -> RevisionFingerprint
    {
        return RevisionFingerprint {
            label: label.to_owned(),
            documents: documents
                .iter()
                .map(|(path, hash)| return ((*path).to_owned(), (*hash).to_owned()))
                .collect(),
        };
    }

    #[test]
    fn Test_Walk_Revisions_Should_Produce_One_Pair_Change_Per_Adjacent_Revision()
    {
        let walk = Walk_Revisions(&[
            Revision("v14.1", &[("a.md", "sha256:01")]),
            Revision("v14.2", &[("a.md", "sha256:02"), ("b.md", "sha256:03")]),
        ]);

        assert_eq!(walk.len(), 1);
        let pair = walk.first().expect("has one pair");
        assert_eq!(pair.from, "v14.1");
        assert_eq!(pair.to, "v14.2");
        assert_eq!(pair.changed, vec!["a.md".to_owned()]);
        assert_eq!(pair.appeared, vec!["b.md".to_owned()]);
    }
}
