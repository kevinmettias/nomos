//! What the shipped catalogue has to be before any single profile is looked at.
//!
//! These are the claims that fail when a profile is added, not when one is changed: an
//! identifier reused, a renderer nothing reaches, a content kind that is selectable and
//! unselected.

use crate::store::{For_Building, Populated, Shipped};
use nomos_spec_project::{Build, Content, Format};
use std::collections::BTreeSet;

/// How many profiles the shipped catalogue declares: fifteen whole-store profiles and four
/// subject-addressed ones. Written here rather than derived from `SHIPPED`, because a count
/// read off the thing under test would agree with any change to it.
const SHIPPED_PROFILE_COUNT: usize = 19;

#[test]
fn Test_Every_Shipped_Profile_Should_Parse_And_Be_Distinct()
{
    let catalogue = Shipped();

    assert_eq!(
        catalogue.Profiles().len(),
        SHIPPED_PROFILE_COUNT,
        "fifteen whole-store profiles and four subject-addressed ones"
    );
    let identifiers: BTreeSet<&str> = catalogue
        .Profiles()
        .iter()
        .map(|profile| return profile.id.as_str())
        .collect();
    assert_eq!(identifiers.len(), SHIPPED_PROFILE_COUNT, "two profiles share an identifier");
}

/// `SHIPPED`'s reality: what files actually sit in `profiles/`. A profile added there and
/// not to `SHIPPED` is invisible to `nomos spec profiles`; a name kept in `SHIPPED` after its
/// file is gone is a shipped identifier with no `include_str!` behind it that still compiled
/// because the constant, not the file, is what the build reads.
#[test]
fn Test_Every_Profile_File_Should_Be_Shipped()
{
    use nomos_spec_project::SHIPPED;

    let declared: BTreeSet<String> =
        SHIPPED.iter().map(|(name, _)| return (*name).to_owned()).collect();
    let on_disk = Profile_File_Stems();

    let undeclared: Vec<&String> = on_disk.difference(&declared).collect();
    let vanished: Vec<&String> = declared.difference(&on_disk).collect();

    assert!(!on_disk.is_empty(), "no profile files were found, so this checked nothing");
    assert!(
        undeclared.is_empty(),
        "these files sit in profiles/ with no entry in SHIPPED, so `nomos spec profiles` \
         cannot see them: {undeclared:#?}"
    );
    assert!(
        vanished.is_empty(),
        "SHIPPED names these profiles and no file backs them: {vanished:#?}"
    );
}

/// Every `.json` file stem under `profiles/`, read from disk rather than from `SHIPPED`.
/// `README.md` lives in the same directory and is excluded — it documents the profiles, it
/// is not one.
fn Profile_File_Stems() -> BTreeSet<String>
{
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("profiles");
    let entries = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("reads {}: {error}", directory.display()));

    return entries
        .filter_map(|entry| return entry.ok())
        .map(|entry| return entry.path())
        .filter(|path| return path.extension().is_some_and(|extension| extension == "json"))
        .filter_map(|path| {
            return path
                .file_stem()
                .and_then(|stem| return stem.to_str())
                .map(|stem| return stem.to_owned());
        })
        .collect();
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
            // Unreachable while every shipped profile renders over the populated fixture,
            // which is exactly the claim this loop makes. A refusal has to stop the run
            // rather than skip the profile: the three assertions below are what say the
            // catalogue was covered, and a skipped profile leaves them saying it anyway.
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
