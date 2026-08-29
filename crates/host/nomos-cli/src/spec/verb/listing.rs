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
