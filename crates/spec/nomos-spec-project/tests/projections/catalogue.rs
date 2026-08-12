//! What the shipped catalogue has to be before any single profile is looked at.
//!
//! These are the claims that fail when a profile is added, not when one is changed: an
//! identifier reused, a renderer nothing reaches, a content kind that is selectable and
//! unselected.

use crate::common::{For_Building, Populated, Shipped};
use nomos_spec_project::{Build, Content, Format};
use std::collections::BTreeSet;

#[test]
fn Test_Every_Shipped_Profile_Should_Parse_And_Be_Distinct()
{
    let catalogue = Shipped();

    assert_eq!(
        catalogue.Profiles().len(),
        18,
        "fourteen whole-store profiles and four subject-addressed ones"
    );
    let identifiers: BTreeSet<&str> = catalogue
        .Profiles()
        .iter()
        .map(|profile| return profile.id.as_str())
        .collect();
    assert_eq!(identifiers.len(), 18, "two profiles share an identifier");
}

#[test]
fn Test_Every_Format_Should_Be_Reached_By_A_Shipped_Profile()
{
    let catalogue = Shipped();

    for format in Format::All()
    {
        assert!(
            catalogue
                .Profiles()
                .iter()
                .any(|profile| return profile.format == *format),
            "{} is a renderer no profile can reach",
            format.Label()
        );
    }
}

#[test]
fn Test_Every_Content_Kind_Should_Be_Reached_By_A_Shipped_Profile()
{
    let catalogue = Shipped();

    for content in Content::All()
    {
        assert!(
            catalogue.Profiles().iter().any(|profile| {
                return profile
                    .sections
                    .iter()
                    .any(|section| return section.content == *content);
            }),
            "{} is selectable and nothing selects it",
            content.Label()
        );
    }
}

#[test]
fn Test_Every_Shipped_Profile_Should_Render_Content_From_The_Store()
{
    let store = Populated();

    for profile in Shipped().Profiles()
    {
        let output = Build(&store, &For_Building(profile))
            .unwrap_or_else(|error| panic!("{error}"));

        assert!(!output.body.trim().is_empty(), "{} rendered nothing", profile.id);
        assert!(
            output.stamp.sections.iter().all(|(_, items)| return *items > 0),
            "{} rendered a section with no item: {:?}",
            profile.id,
            output.stamp.sections
        );
        assert!(!output.stamp.inputs.is_empty(), "{} consumed nothing", profile.id);
    }
}
