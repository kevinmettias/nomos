//! Where content that left one document turned up.

use super::{BTreeMap, BTreeSet, DocumentFate, PairChange, Relocation, RevisionFingerprint};

#[must_use]
pub(crate) fn Relocations_Between(
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

#[cfg(test)]
mod tests
{
    use super::*;

    fn Fingerprint(label: &str, entries: &[(&str, &str)]) -> RevisionFingerprint
    {
        return RevisionFingerprint {
            label: label.to_owned(),
            documents: entries.iter().map(|(path, hash)| return ((*path).to_owned(), (*hash).to_owned())).collect(),
        };
    }

    #[test]
    fn Test_Relocations_Between_Should_Report_A_Moved_Path_As_Relocated_Not_Disappeared()
    {
        let from = Fingerprint("v14.35", &[("old/record.md", "hash-1")]);
        let to = Fingerprint("v14.36", &[("new/record.md", "hash-1")]);
        let pair = PairChange {
            from: "v14.35".to_owned(),
            to: "v14.36".to_owned(),
            appeared: vec!["new/record.md".to_owned()],
            disappeared: vec!["old/record.md".to_owned()],
            changed: Vec::new(),
            reappeared: Vec::new(),
        };

        let fate = Relocations_Between(&pair, &from, &to);

        assert_eq!(fate.relocated.len(), 1);
        assert_eq!(fate.relocated.first().expect("the assertion above confirms exactly one relocation").to, "new/record.md");
        assert_eq!(
            fate.relocated.first().expect("the assertion above confirms exactly one relocation").from,
            vec!["old/record.md".to_owned()]
        );
        assert!(fate.disappeared.is_empty());
        assert!(fate.appeared.is_empty());
    }

    #[test]
    fn Test_Origins_By_Content_Should_Index_Disappeared_Paths_By_Their_Hash()
    {
        let from = Fingerprint("v14.35", &[("old/one.md", "hash-1"), ("old/two.md", "hash-1"), ("kept.md", "hash-2")]);
        let pair = PairChange {
            from: "v14.35".to_owned(),
            to: "v14.36".to_owned(),
            appeared: Vec::new(),
            disappeared: vec!["old/one.md".to_owned(), "old/two.md".to_owned()],
            changed: Vec::new(),
            reappeared: Vec::new(),
        };

        let origins = Origins_By_Content(&pair, &from);

        assert_eq!(origins.get("hash-1"), Some(&vec!["old/one.md".to_owned(), "old/two.md".to_owned()]));
        assert_eq!(origins.get("hash-2"), None, "a path that did not disappear contributes no origin");
    }

    #[test]
    fn Test_Place_Appeared_Should_Split_Arrivals_Into_Relocations_And_Genuine_Ones()
    {
        let to = Fingerprint("v14.36", &[("new/record.md", "hash-1"), ("brand-new.md", "hash-3")]);
        let mut origins: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        origins.insert("hash-1", vec!["old/record.md".to_owned()]);
        let pair = PairChange {
            from: "v14.35".to_owned(),
            to: "v14.36".to_owned(),
            appeared: vec!["new/record.md".to_owned(), "brand-new.md".to_owned()],
            disappeared: vec!["old/record.md".to_owned()],
            changed: Vec::new(),
            reappeared: Vec::new(),
        };
        let mut fate = DocumentFate::default();

        let moved = Place_Appeared(&pair, &to, &origins, &mut fate);

        assert_eq!(moved, BTreeSet::from(["old/record.md".to_owned()]));
        assert_eq!(fate.relocated.len(), 1);
        assert_eq!(fate.appeared, vec!["brand-new.md".to_owned()]);
    }
}
