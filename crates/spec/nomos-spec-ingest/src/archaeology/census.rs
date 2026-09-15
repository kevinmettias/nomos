//! How much of a revision says nothing, counted rather than sampled.

use super::{Later, FillerCensus, Template, SHARED_BY, Is_Template_Eligible, Repetition, Get_Filler_Pattern};

pub(super) fn Census_Fillers(later: &Later) -> FillerCensus
{
    return FillerCensus {
        declared: later.declared.iter().cloned().collect(),
        templates: Shared_Templates(later),
        stubs: Stubs_Of(later),
    };
}

/// The blocks repeated widely enough to be template rather than prose, widest first.
///
/// Ordering is by reach and then by text, so the report opens on the thing worth deleting
/// and two runs over the same corpus agree on what to print.
pub(super) fn Shared_Templates(later: &Later) -> Vec<Template>
{
    let mut templates: Vec<Template> = later
        .templates
        .iter()
        .filter(|(key, repetition)| {
            // The floor is applied here and in `NSV-PRESERVE-004` from one constant. The
            // rule documents why they must not become two numbers somebody has to notice
            // disagree, and a report that still called a connective a template while the
            // rule had stopped would be exactly that divergence.
            return repetition.sections >= SHARED_BY && Is_Template_Eligible(key);
        })
        .map(|(key, repetition)| return Described_Template(key, repetition))
        .collect();
    templates.sort_by(|first, second| {
        return second
            .sections
            .cmp(&first.sections)
            .then_with(|| return first.text.cmp(&second.text));
    });

    return templates;
}

/// One repeated block as the census reports it.
pub(super) fn Described_Template(key: &str, repetition: &Repetition) -> Template
{
    return Template {
        text: key.to_owned(),
        sections: repetition.sections,
        documents: repetition.documents.iter().cloned().collect(),
        declared: Get_Filler_Pattern(key),
    };
}

/// The documents carrying nothing but filler.
///
/// A document with no blocks at all is not one of them: it is empty, which is a different
/// complaint and one the reader can already see.
pub(super) fn Stubs_Of(later: &Later) -> Vec<String>
{
    let mut stubs: Vec<String> = Vec::new();

    for (path, bodies) in &later.bodies
    {
        if bodies.blocks > 0 && bodies.filler == bodies.blocks
        {
            stubs.push(path.clone());
        }
    }

    return stubs;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn Test_Census_Fillers_Should_Combine_Repeated_Templates_With_Stub_Documents()
    {
        let documents = Documents(&[
            ("a.md", "# A\n\nRefer to the owning domain volume for this material.\n"),
            ("b.md", "# B\n\nRefer to the owning domain volume for this material.\n"),
            ("c.md", "# C\n\nRefer to the owning domain volume for this material.\n"),
        ]);
        let later = Later::Read(&documents);

        let census = Census_Fillers(&later);

        assert_eq!(census.templates.len(), 1);
        assert_eq!(census.templates.first().expect("the assertion above confirms exactly one template").sections, 3);
        assert_eq!(census.stubs, vec!["a.md".to_owned(), "b.md".to_owned(), "c.md".to_owned()]);
    }

    /// The report applies the same floor as `NSV-PRESERVE-004`, and this is what says so.
    ///
    /// Without it the two could diverge silently: the rule would stop calling a connective a
    /// form letter while the cross-revision report went on doing it, which is precisely the
    /// two-numbers-somebody-has-to-notice-disagree that the threshold is shared to prevent.
    /// The other fixtures here all repeat a body well past the floor, so none of them moves
    /// if the filter is removed.
    #[test]
    fn Test_A_Body_Below_The_Floor_Should_Not_Surface_As_A_Shared_Template()
    {
        let documents = Documents(&[
            ("a.md", "# A\n\nShort shared line.\n"),
            ("b.md", "# B\n\nShort shared line.\n"),
            ("c.md", "# C\n\nShort shared line.\n"),
        ]);
        let later = Later::Read(&documents);

        let templates = Shared_Templates(&later);

        assert!(
            templates.is_empty(),
            "eighteen characters carried by three sections is a collision, not a template: {templates:?}"
        );
    }

    #[test]
    fn Test_Shared_Templates_Should_Sort_By_Reach_Then_By_Text()
    {
        let documents = Documents(&[
            ("a.md", "# A\n\nRefer to the owning domain volume for this material.\n"),
            ("b.md", "# B\n\nRefer to the owning domain volume for this material.\n"),
            ("c.md", "# C\n\nRefer to the owning domain volume for this material.\n"),
            ("d.md", "# D\n\nSomething else entirely.\n"),
        ]);
        let later = Later::Read(&documents);

        let templates = Shared_Templates(&later);

        assert_eq!(templates.len(), 1, "a block repeated only once must not surface as a template");
        assert_eq!(templates.first().expect("the assertion above confirms exactly one template").text, "Refer to the owning domain volume for this material.");
        assert_eq!(templates.first().expect("the assertion above confirms exactly one template").sections, 3);
    }

    #[test]
    fn Test_Described_Template_Should_Attach_The_Declared_Pattern_And_Sorted_Documents()
    {
        let repetition = Repetition
        {
            sections: 2,
            documents: BTreeSet::from(["b.md".to_owned(), "a.md".to_owned()]),
        };

        let template = Described_Template("This section groups related specification material for X.", &repetition);

        assert_eq!(template.text, "This section groups related specification material for X.");
        assert_eq!(template.sections, 2);
        assert_eq!(template.documents, vec!["a.md".to_owned(), "b.md".to_owned()]);
        assert_eq!(template.declared, Some("This section groups related specification material"));
    }

    #[test]
    fn Test_Stubs_Of_Should_Only_List_Documents_Whose_Blocks_Are_All_Filler()
    {
        let documents = Documents(&[
            ("stub.md", "# Stub\n\nThis section groups related specification material for X.\n"),
            (
                "mixed.md",
                "# Mixed\n\nThis section groups related specification material for X.\n\n\
                 An original paragraph nobody repeats.\n",
            ),
        ]);
        let later = Later::Read(&documents);

        assert_eq!(Stubs_Of(&later), vec!["stub.md".to_owned()]);
    }

    fn Documents(pairs: &[(&str, &str)]) -> BTreeMap<String, String>
    {
        return pairs.iter().map(|(path, text)| return ((*path).to_owned(), (*text).to_owned())).collect();
    }
}
