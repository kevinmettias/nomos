//! A kind: one projection the family carries, declared once.

use super::{LABELED_JUMP, SiteField};

/// One projection -- its name, the construct it projects, and the fields each record carries.
///
/// A kind says what is projected and never how (`OD-CAPABILITY-019`, fifth decision): whether a
/// provider answers it through a parser or through lines is that provider's own business and its
/// own guarantee's, so nothing here names a tree, a node or a tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SiteKind
{
    /// The kind's name, as a payload's stances and sites spell it.
    pub name: &'static str,
    /// The construct a record of this kind locates, in words a reader of a finding can check
    /// against the source.
    pub construct: &'static str,
    /// Every field a record of this kind carries besides its kind and line, in the order the
    /// writer puts them.
    pub fields: &'static [SiteField],
}

impl SiteKind
{
    /// The field this kind declares under `name`, if it declares one.
    #[must_use]
    pub fn Field(&self, name: &str) -> Option<SiteField>
    {
        return self.fields.iter().find(|field| return field.name == name).copied();
    }
}

/// Every kind this family declares. A payload naming a kind not here is refused, never passed on.
///
/// Mirrored by `Test_Every_Declared_Site_Kind_Should_Be_Listed`, which reads every `SiteKind`
/// constant this crate's source declares and compares their names with this list in both
/// directions, so a kind declared and left out of the list -- which the reader would then refuse --
/// fails a test rather than every payload that carries it.
pub const SITE_KINDS: &[SiteKind] = &[LABELED_JUMP];

/// The declared kind named `name`, if there is one.
#[must_use]
pub fn Site_Kind(name: &str) -> Option<&'static SiteKind>
{
    return SITE_KINDS.iter().find(|kind| return kind.name == name);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Site_Kind_Should_Find_A_Declared_Kind_And_Nothing_Else()
    {
        assert_eq!(Site_Kind(LABELED_JUMP.name), Some(&LABELED_JUMP));
        assert_eq!(Site_Kind("labelled-jump"), None);
    }

    #[test]
    fn Test_Field_Should_Find_A_Declared_Field_And_Nothing_Else()
    {
        let first = LABELED_JUMP.fields.first().expect("the labeled jump declares fields");

        assert_eq!(LABELED_JUMP.Field(first.name), Some(*first));
        assert_eq!(LABELED_JUMP.Field("depth"), None);
    }

    /// The names every `SiteKind` constant under `directory` declares, read off the `name:` field
    /// that follows each declaration. The declaration is spelled at run time so this function's own
    /// text is not read as one.
    fn Declared_Kind_Names(directory: &std::path::Path) -> Vec<String>
    {
        let declaration = format!(": {0} = {0} {{", "SiteKind");
        let mut names = Vec::new();
        for entry in std::fs::read_dir(directory).expect("the crate's source directory is readable")
        {
            let path = entry.expect("a directory entry").path();
            if path.is_dir()
            {
                names.extend(Declared_Kind_Names(&path));
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a source file is text");
            let mut lines = text.lines();
            while let Some(line) = lines.next()
            {
                if !line.contains(&declaration)
                {
                    continue;
                }
                let kind_name = lines.by_ref().find_map(|field| return field.trim().strip_prefix("name: \"").and_then(|rest| return rest.split('"').next()));
                names.push(kind_name.expect("a SiteKind declaration names its kind").to_owned());
            }
        }

        return names;
    }

    #[test]
    fn Test_Every_Declared_Site_Kind_Should_Be_Listed()
    {
        let mut declared = Declared_Kind_Names(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"));
        declared.sort_unstable();
        let mut listed: Vec<String> = SITE_KINDS.iter().map(|kind| return kind.name.to_owned()).collect();
        listed.sort_unstable();

        assert!(!declared.is_empty(), "the scan read no kind, so the comparison below would pass having read nothing");
        assert_eq!(declared, listed, "every SiteKind this crate declares must be in SITE_KINDS, and nothing else");
    }

    /// Two kinds sharing a name would make a stance ambiguous, and two fields sharing one would
    /// make a record's value ambiguous; the declaration is the one place either could happen.
    #[test]
    fn Test_Every_Kind_And_Every_Field_Should_Be_Named_Once()
    {
        let mut kinds: Vec<&str> = SITE_KINDS.iter().map(|kind| return kind.name).collect();
        kinds.sort_unstable();
        kinds.dedup();
        assert_eq!(kinds.len(), SITE_KINDS.len(), "{kinds:?}");

        for kind in SITE_KINDS
        {
            let mut fields: Vec<&str> = kind.fields.iter().map(|field| return field.name).collect();
            fields.sort_unstable();
            fields.dedup();
            assert_eq!(fields.len(), kind.fields.len(), "{}: {fields:?}", kind.name);
        }
    }
}
