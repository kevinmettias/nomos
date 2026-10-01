//! What this build can render, and what it was assembled from.
//!
//! `Profiles` and `Enumerated_Sources` compute nothing themselves any more -- both delegate to
//! `nomos-spec-orchestration`, `OD-HOST-002`'s family-9 seam, and keep only the text
//! formatting and the `ExitCode` a rendering layer is responsible for.

use crate::spec::{Assembly, ExitCode, ProjectError, Channels, Report_Project_Error, Profile};

/// A section that selected nothing, as the projection machinery reported it.
pub(in crate::spec) struct EmptySection<'a>
{
    /// The profile being built.
    pub(super) profile: &'a str,
    /// The section of it that came back empty.
    pub(super) section: &'a str,
    /// What that section selects.
    pub(super) content: &'static str,
}

/// A section that selected nothing, over a store that is missing its corpus.
///
/// The profile machinery already refuses to render an empty section, and its message is
/// the right one when the store is whole: declare `may_be_empty` if nothing is the honest
/// answer. Over a store the corpus never reached, that message sends the reader to change
/// a profile because of a variable that is not set.
pub(in crate::spec) fn Empty_Section(
    assembly: &Assembly,
    empty: &EmptySection<'_>,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    if assembly.Is_Complete()
    {
        let reported = ProjectError::Empty {
            profile: empty.profile.to_owned(),
            section: empty.section.to_owned(),
            content: empty.content,
        };
        let _ = writeln!(notes, "{reported}");

        return ExitCode::NotFound;
    }

    let _ = writeln!(
        notes,
        "{}: section {:?} selected no {}, and this store is not whole. Reporting the absence \
         above rather than the empty section: an empty projection over a store nothing was \
         read into is not a projection of an empty specification.",
        empty.profile, empty.section, empty.content
    );

    return ExitCode::Absent;
}

pub(in crate::spec) fn List_Profiles(channels: &mut Channels<'_>) -> ExitCode
{
    let profiles = match nomos_spec_orchestration::Profiles()
    {
        Ok(profiles) => profiles,
        Err(error) => return Report_Project_Error(&error, channels.notes),
    };

    for profile in &profiles
    {
        let _ = writeln!(
            channels.output,
            "{:<28} {:<12} {:<34} {}",
            profile.id,
            profile.format.Label(),
            profile.output,
            Section_Labels(profile)
        );
    }

    return ExitCode::Ok;
}

pub(super) fn Section_Labels(profile: &Profile) -> String
{
    return profile
        .sections
        .iter()
        .map(|section| return section.content.Label())
        .collect::<Vec<&str>>()
        .join("+");
}

/// What went into this store, and what did not.
///
/// Exits [`ExitCode::Absent`] when anything is missing, so this is a check rather than a
/// description: a script can ask whether the store it is about to read is whole.
pub(in crate::spec) fn List_Sources(assembly: &Assembly, output: &mut dyn std::io::Write) -> ExitCode
{
    let answer = nomos_spec_orchestration::Enumerated_Sources(assembly);

    for line in &answer.read
    {
        let _ = writeln!(output, "read: {line}");
    }

    if answer.Is_Complete()
    {
        let _ = writeln!(output, "nothing this store expects is missing");

        return ExitCode::Ok;
    }

    let _ = writeln!(output, "{}", answer.Describe_Absences());
    let _ = writeln!(
        output,
        "{} of this store's sources were not read",
        answer.absent.len()
    );

    return ExitCode::Absent;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_orchestration::corpus::{Assemble_Corpus, CorpusRequest, DEFAULT_REVISION};

    #[test]
    fn Test_Empty_Section_Should_Report_Absent_Over_A_Store_Missing_Its_Corpus()
    {
        let assembly = Corpus_Unset_Assembly();
        let empty = EmptySection { profile: "diagram-set", section: "records", content: "record" };
        let mut notes = Vec::new();

        let code = Empty_Section(&assembly, &empty, &mut notes);

        assert_eq!(code, ExitCode::Absent);
        assert!(!notes.is_empty());
    }

    #[test]
    fn Test_List_Profiles_Should_Print_A_Line_Per_Shipped_Profile()
    {
        let mut output = Vec::new();
        let mut notes = Vec::new();
        let mut channels = Channels { output: &mut output, notes: &mut notes };

        let code = List_Profiles(&mut channels);

        assert_eq!(code, ExitCode::Ok);
        let expected = nomos_spec_orchestration::Profiles()
            .expect("the catalogue is embedded in nomos-spec-project, so only its own bytes can refuse")
            .len();
        assert_eq!(String::from_utf8_lossy(&output).lines().count(), expected);
    }

    #[test]
    fn Test_Section_Labels_Should_Join_Every_Sections_Content_Label_With_A_Plus()
    {
        let profiles = nomos_spec_orchestration::Profiles()
            .expect("the catalogue is embedded in nomos-spec-project, so only its own bytes can refuse");
        let profile = profiles.first().expect("the catalogue ships at least one profile");

        let labels = Section_Labels(profile);

        assert!(!labels.is_empty());
        assert_eq!(labels.matches('+').count() + 1, profile.sections.len());
    }

    #[test]
    fn Test_List_Sources_Should_Report_Absent_When_The_Corpus_Was_Never_Configured()
    {
        let assembly = Corpus_Unset_Assembly();
        let mut output = Vec::new();

        let code = List_Sources(&assembly, &mut output);

        assert_eq!(code, ExitCode::Absent);
        assert!(!output.is_empty());
    }

    /// A store with the embedded governing records seeded and no corpus, so
    /// [`Assembly::Is_Complete`] is deterministically `false` -- `root: None` records an
    /// absence unconditionally, regardless of what any real environment variable holds.
    fn Corpus_Unset_Assembly() -> Assembly
    {
        let request = CorpusRequest {
            variable: "NOMOS_SPEC_LISTING_TEST_CORPUS_UNSET".to_owned(),
            root: None,
            revision: DEFAULT_REVISION.to_owned(),
        };

        return Assemble_Corpus(&request).expect("the only fallible step is seeding the records this binary embeds into an in-memory store");
    }
}
