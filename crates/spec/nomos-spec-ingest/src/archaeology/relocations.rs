//! Where content that left one document turned up.

use super::{PairChange, RevisionFingerprint, DocumentFate, BTreeMap, BTreeSet, Relocation};

#[must_use]
pub fn Relocations(
    pair: &PairChange,
    from: &RevisionFingerprint,
    to: &RevisionFingerprint,
) -> DocumentFate
{
    let origins = Origins_By_Content(pair, from);
    let mut fate = DocumentFate {
        changed: pair.changed.clone(),
        ..DocumentFate::default()
    };
    let moved = Place_Appeared(pair, to, &origins, &mut fate);

    fate.disappeared = pair
        .disappeared
        .iter()
        .filter(|path| return !moved.contains(path.as_str()))
        .cloned()
        .collect();

    return fate;
}

/// The paths that disappeared, indexed by the content they held.
///
/// A move is only recognisable as one because the content survived under another name, so
/// content is the key and the old paths are what it answers with.
pub(super) fn Origins_By_Content<'a>(
    pair: &'a PairChange,
    from: &'a RevisionFingerprint,
) -> BTreeMap<&'a str, Vec<String>>
{
    let mut origins: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for path in &pair.disappeared
    {
        if let Some(hash) = from.documents.get(path)
        {
            origins.entry(hash.as_str()).or_default().push(path.clone());
        }
    }

    return origins;
}

/// Sorts each appeared path into a relocation or a genuine arrival, and says which origins
/// were accounted for — those are the ones the caller must not also report as disappeared.
pub(super) fn Place_Appeared(
    pair: &PairChange,
    to: &RevisionFingerprint,
    origins: &BTreeMap<&str, Vec<String>>,
    fate: &mut DocumentFate,
) -> BTreeSet<String>
{
    let mut moved: BTreeSet<String> = BTreeSet::new();

    for path in &pair.appeared
    {
        let origin = to.documents.get(path).and_then(|hash| return origins.get(hash.as_str()));
        match origin
        {
            Some(from_paths) =>
            {
                moved.extend(from_paths.iter().cloned());
                fate.relocated.push(Relocation {
                    to: path.clone(),
                    from: from_paths.clone(),
                });
            }
            None => fate.appeared.push(path.clone()),
        }
    }

    return moved;
}
