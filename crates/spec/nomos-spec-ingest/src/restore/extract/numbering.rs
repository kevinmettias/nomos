//! What a heading names: how deep it sits, what its numbering says, and the identifier that
//! name mints.
//!
//! These are pure readings of a heading's text, and they sit apart from the volume and row
//! passes in [`super`] because nothing here walks a document or touches a store. They came
//! out of `extract.rs` when it crossed the file-size review trigger, and their tests came
//! with them -- a Rust unit test's companion is the file the test is written in, so a
//! function and its test have to move together.

use super::super::{EXTENDED_TERMS, Restored, SYSTEMS_HEADING, SourceBlock};

/// A heading recognised as a family member: the family, the key its identifier is minted
/// from, and the alias it answers to when it answers to one.
pub(super) type Recognition = (Restored, String, Option<String>);

/// What a heading names, judged by how deep it sits.
///
/// Depth is what separates a family member from a section that merely mentions one, so a
/// heading at any other level names nothing at all.
pub(super) fn Recognized_In_Heading(block: &SourceBlock, title: &str) -> Vec<Recognition>
{
    let depth = block.text.chars().take_while(|character| return *character == '#').count();

    return match depth
    {
        LEVEL_3 => At_Depth_3(title),
        LEVEL_4 => At_Depth_4(title, &block.heading_path),
        _ => Vec::new(),
    };
}

/// The two heading levels a family member can sit at. Anything shallower is a volume and
/// anything deeper is a subsection of a member rather than a member.
const LEVEL_3: usize = 3;
const LEVEL_4: usize = 4;

/// A level-4 heading's numbering carries two parts, `D.1.2`, where a level-3 one carries one.
const PARTS_AT_DEPTH_4: usize = 2;

/// The level-3 numbered series, and the family each one restores.
const DEPTH_3: &[(char, Restored)] = &[('D', Restored::AppendixD), ('H', Restored::AppendixH)];

/// The level-4 numbered series, and the family each one restores.
const DEPTH_4: &[(char, Restored)] = &[
    ('D', Restored::AppendixD),
    ('E', Restored::HeadlessInventory),
    ('F', Restored::IdeProfile),
];

/// Every family in `series` whose letter this title carries at `parts` levels of numbering.
pub(super) fn Numbered_Families(title: &str, parts: usize, series: &[(char, Restored)]) -> Vec<Recognition>
{
    let mut found = Vec::new();

    for (letter, family) in series
    {
        if let Some(numbering) = Numbering_In(title, *letter, parts)
        {
            found.push((*family, numbering, None));
        }
    }

    return found;
}

/// What a level-3 heading names.
///
/// A `G` numbering is a scenario only when the title says so, because appendix G numbers
/// its prose sections in the same series as its scenarios.
pub(super) fn At_Depth_3(title: &str) -> Vec<Recognition>
{
    let mut found = Numbered_Families(title, 1, DEPTH_3);

    if let Some(numbering) = Milestone_Numbering(title)
    {
        found.push((Restored::RoadmapMilestone, numbering, None));
    }
    if let Some(numbering) = Numbering_In(title, 'G', 1)
        && title.contains("End-to-end scenario:")
    {
        found.push((Restored::Scenario, numbering, None));
    }

    return found;
}

/// What a level-4 heading names.
///
/// Services and glossary terms are recognised by where they sit rather than by a numbering,
/// because neither carries one — the heading path is the only thing that distinguishes a
/// service from any other level-4 heading in the same volume.
pub(super) fn At_Depth_4(title: &str, path: &[String]) -> Vec<Recognition>
{
    let mut found = Numbered_Families(title, PARTS_AT_DEPTH_4, DEPTH_4);

    if Has_Ancestor(path, SYSTEMS_HEADING) && Has_The_Word_Service(title)
    {
        found.push((Restored::Service, title.to_owned(), None));
    }
    if Has_Ancestor(path, EXTENDED_TERMS)
    {
        found.push((Restored::GlossaryTerm, title.to_owned(), Some(title.to_owned())));
    }

    return found;
}

/// Whether a heading calls its subject a service in so many words.
fn Has_The_Word_Service(title: &str) -> bool
{
    return title.split_whitespace().any(|word| return word == "Service");
}

pub(super) fn Has_Ancestor(path: &[String], heading: &str) -> bool
{
    return path.iter().any(|step| return step == heading);
}

/// `Foundation 0` and `Release 3`, as the roadmap numbers itself.
///
/// Both are milestones; only seven of the eight are Releases, which is why the family is
/// named for the milestone and not for the release.
pub(in crate::restore) fn Milestone_Numbering(title: &str) -> Option<String>
{
    let mut words = title.split_whitespace();
    let word = words.next()?;
    let number = words.next()?;

    if word != "Foundation" && word != "Release"
    {
        return None;
    }
    if !Is_All_Digits(number)
    {
        return None;
    }

    let initial = word.get(..1)?;
    return Some(format!("{initial}.{number}"));
}

/// `D.7.1` from `D.7.1 atlas-profile.build-cost`, when it has exactly `parts` numbers.
pub(in crate::restore) fn Numbering_In(title: &str, letter: char, parts: usize) -> Option<String>
{
    let rest = title.strip_prefix(letter)?;
    let (numbering, _) = rest.split_once(' ')?;

    let segments: Vec<&str> = numbering.split('.').collect();
    if segments.len() != parts.checked_add(1)?
    {
        return None;
    }
    if segments.first() != Some(&"")
    {
        return None;
    }
    if !segments.iter().skip(1).all(|segment| return Is_All_Digits(segment))
    {
        return None;
    }

    return Some(format!("{letter}{numbering}"));
}

/// The identifier, minted from the family's prefix and what the document already says.
pub(super) fn Identify_Member(family: Restored, name: &str) -> String
{
    return format!("{}-{}", family.Prefix(), Slug_Of(name));
}

pub(super) fn Slug_Of(name: &str) -> String
{
    let mut slug = String::new();
    let mut pending = false;

    for character in name.chars()
    {
        if character.is_ascii_alphanumeric()
        {
            if pending && !slug.is_empty()
            {
                slug.push('-');
            }
            pending = false;
            slug.extend(character.to_uppercase());
        }
        else
        {
            pending = true;
        }
    }

    return slug;
}

/// A non-empty run of ASCII digits and nothing else.
fn Is_All_Digits(text: &str) -> bool
{
    return !text.is_empty() && text.bytes().all(|byte| return byte.is_ascii_digit());
}

#[cfg(test)]
mod tests
{
    use super::super::GLOSSARY;
    use super::super::Block;
    use super::*;

    #[test]
    fn Test_Recognized_In_Heading_Should_Read_The_Heading_Depth()
    {
        let deep_enough = Block("### D.7 Profiles");
        let recognitions = Recognized_In_Heading(&deep_enough, "D.7 Profiles");
        assert_eq!(recognitions, vec![(Restored::AppendixD, "D.7".to_owned(), None)]);

        let shallow = Block("## Not a member");
        assert!(Recognized_In_Heading(&shallow, "Not a member").is_empty());
    }

    #[test]
    fn Test_Numbered_Families_Should_Match_Every_Letter_In_The_Series()
    {
        let found = Numbered_Families("D.7 Profiles", 1, DEPTH_3);
        assert_eq!(found, vec![(Restored::AppendixD, "D.7".to_owned(), None)]);

        assert!(Numbered_Families("Z.7 Nothing", 1, DEPTH_3).is_empty());
    }

    #[test]
    fn Test_At_Depth_3_Should_Recognize_A_Numbered_Family_A_Milestone_And_A_Scenario()
    {
        assert_eq!(At_Depth_3("D.7 Profiles"), vec![(Restored::AppendixD, "D.7".to_owned(), None)]);
        assert_eq!(
            At_Depth_3("Foundation 0 — Protocol"),
            vec![(Restored::RoadmapMilestone, "F.0".to_owned(), None)]
        );
        assert_eq!(
            At_Depth_3("G.2 End-to-end scenario: add a strategy"),
            vec![(Restored::Scenario, "G.2".to_owned(), None)]
        );
    }

    #[test]
    fn Test_At_Depth_4_Should_Recognize_A_Numbered_Family_A_Service_And_A_Glossary_Term()
    {
        let systems_path = vec![SYSTEMS_HEADING.to_owned()];
        let extended_path = vec![EXTENDED_TERMS.to_owned()];

        assert_eq!(
            At_Depth_4("E.1.1 Legacy client", &[]),
            vec![(Restored::HeadlessInventory, "E.1.1".to_owned(), None)]
        );
        assert_eq!(
            At_Depth_4("Counterfactual Analysis Service", &systems_path),
            vec![(Restored::Service, "Counterfactual Analysis Service".to_owned(), None)]
        );
        assert_eq!(
            At_Depth_4("SavedView", &extended_path),
            vec![(Restored::GlossaryTerm, "SavedView".to_owned(), Some("SavedView".to_owned()))]
        );
    }

    #[test]
    fn Test_Has_Ancestor_Should_Find_A_Step_Anywhere_In_The_Path()
    {
        let path = vec!["Reference".to_owned(), GLOSSARY.to_owned(), "Extended operational terms".to_owned()];

        assert!(Has_Ancestor(&path, GLOSSARY));
        assert!(!Has_Ancestor(&path, SYSTEMS_HEADING));
    }

    #[test]
    fn Test_Milestone_Numbering_Should_Read_Foundation_And_Release_Titles()
    {
        assert_eq!(Milestone_Numbering("Foundation 0 — Protocol"), Some("F.0".to_owned()));
        assert_eq!(Milestone_Numbering("Release 7 — Advanced"), Some("R.7".to_owned()));
        assert_eq!(Milestone_Numbering("Release notes"), None);
    }

    #[test]
    fn Test_Numbering_In_Should_Require_The_Exact_Depth_Of_Numbers()
    {
        assert_eq!(Numbering_In("D.7 Profiles", 'D', 1), Some("D.7".to_owned()));
        assert_eq!(Numbering_In("D.7 Profiles", 'D', PARTS_AT_DEPTH_4), None);
        assert_eq!(Numbering_In("D.7.1 atlas", 'D', PARTS_AT_DEPTH_4), Some("D.7.1".to_owned()));
        assert_eq!(Numbering_In("Design notes", 'D', 1), None);
    }

    #[test]
    fn Test_Identify_Member_Should_Mint_An_Identifier_From_The_Family_Prefix_And_Slug()
    {
        assert_eq!(
            Identify_Member(Restored::Service, "Counterfactual Analysis Service"),
            "SVC-COUNTERFACTUAL-ANALYSIS-SERVICE"
        );
        assert_eq!(Identify_Member(Restored::GlossaryTerm, "Applicability"), "GLS-APPLICABILITY");
    }

    #[test]
    fn Test_Slug_Of_Should_Join_Alnum_Runs_With_A_Single_Dash()
    {
        assert_eq!(Slug_Of("WorkspaceContext"), "WORKSPACECONTEXT");
        assert_eq!(Slug_Of("Counterfactual Analysis Service"), "COUNTERFACTUAL-ANALYSIS-SERVICE");
        assert_eq!(Slug_Of("  Leading and trailing  "), "LEADING-AND-TRAILING");
    }
}
