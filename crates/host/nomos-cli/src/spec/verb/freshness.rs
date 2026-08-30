//! Rendering `nomos spec freshness`'s answer, or the refusal saying why it examined nothing.
//!
//! `D-128`'s check, run over what `nomos-platform-std::StdFileSystem` holds. Resolving which
//! profiles to look at and comparing each one against the store is
//! `nomos-spec-orchestration::Freshness_Of_Render`'s job now; this module keeps the census over the
//! answer -- how many were checked, which requirements were kept -- and the `ExitCode` a
//! rendering layer is responsible for.
//!
//! The branch that earns its own case is the half-present pair. A body with no sidecar
//! beside it is a failure rather than something skipped, because otherwise deleting the
//! sidecar is how an edit stops being caught, and a check teaches that trick to the first
//! person who trips over it.

use crate::spec::{Assembly, Profile, FreshnessRequest, Channels, ExitCode, Report_Store_Error, Path, SIDECAR_SUFFIX, Report_Build_Error, No_Such_Profile};
use nomos_spec_orchestration::{FreshnessAnswer, FreshnessRefusal, ProfileOutcome, Verdict};

/// `D-128`'s check, run.
pub(in crate::spec) fn Freshness_Of(
    assembly: &Assembly,
    request: &FreshnessRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    use nomos_platform_std::StdFileSystem;

    return match nomos_spec_orchestration::Freshness_Of_Render(assembly, request, &StdFileSystem)
    {
        Ok(answer) => Reported_Freshness(assembly, &answer, request, channels),
        Err(FreshnessRefusal::Store(error)) => Report_Store_Error(&error, channels.notes),
        Err(FreshnessRefusal::NoSuchProfile { requested, known }) => No_Such_Profile(&requested, &known, channels.notes),
        Err(FreshnessRefusal::RequirementUnexamined { requested, only }) =>
        {
            Unexamined_Requirement(&requested, only.as_deref(), channels.notes)
        }
        Err(FreshnessRefusal::Project(error)) => Report_Build_Error(assembly, &error, channels.notes),
    };
}

/// Every profile the run examined, and what this repository was promised.
fn Reported_Freshness(
    assembly: &Assembly,
    answer: &FreshnessAnswer,
    request: &FreshnessRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let mut census = Census::Over(answer.examined.len(), &answer.required);

    for outcome in &answer.examined
    {
        let code = census.Record(outcome, assembly, channels);
        census.worst = Worse_Of(census.worst, code);
    }

    return census.Report(&request.into, request.profile.as_deref(), channels.output);
}

/// The code a run reports when its profiles disagreed about what happened.
///
/// [`ExitCode::Stale`] beats [`ExitCode::Absent`] deliberately: a definite finding about
/// one output is more actionable than a machine that could not check another, and the
/// text above has already said both.
const fn Worse_Of(carried: ExitCode, found: ExitCode) -> ExitCode
{
    return match (carried, found)
    {
        (ExitCode::StoreError, _) | (_, ExitCode::StoreError) => ExitCode::StoreError,
        (ExitCode::Stale, _) | (_, ExitCode::Stale) => ExitCode::Stale,
        (ExitCode::Absent, _) | (_, ExitCode::Absent) => ExitCode::Absent,
        _ => ExitCode::Ok,
    };
}

/// A requirement outside the profile a run was narrowed to.
///
/// `--profile a --require b` asks for one profile to be examined and a different one to be
/// guaranteed. Answering it would mean reporting success over a requirement nothing
/// checked, which is the shape this flag exists against -- so it is a usage error and not a
/// quiet pass.
fn Unexamined_Requirement(unexamined: &str, only: Option<&str>, notes: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(
        notes,
        "--require {unexamined} cannot hold while --profile {} narrows this run to one \
         other profile: the requirement would be reported as met by a run that never \
         looked for it",
        only.unwrap_or("<none>")
    );

    return ExitCode::Usage;
}

/// One profile's verdict, printed, and the code it contributes to the run.
///
/// `None` for [`Verdict::Absent`] -- printed only once its promise is known, by
/// [`Census::Record`].
fn Printed_Verdict(assembly: &Assembly, outcome: &ProfileOutcome, channels: &mut Channels<'_>) -> Option<ExitCode>
{
    return match &outcome.verdict
    {
        Verdict::Absent => None,
        Verdict::Unstamped => Some(Unstamped_Profile(&outcome.profile, channels.output)),
        Verdict::Unbodied => Some(Unbodied_Profile(&outcome.profile, channels.output)),
        Verdict::Compared(result) => Some(Compared_Profile(assembly, &outcome.profile, result, channels)),
    };
}

/// A body with no stamp beside it.
///
/// A failure rather than something skipped, because otherwise deleting the sidecar is how
/// an edit stops being caught, and a check that skipped it would teach that trick to the
/// first person who tripped over it.
fn Unstamped_Profile(profile: &Profile, output: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(
        output,
        "{}: {} is there and {}{SIDECAR_SUFFIX} is not, so nothing can say whether it is what \
         the store produced",
        profile.id, profile.output, profile.output
    );

    return ExitCode::Stale;
}

/// A stamp with no body beside it: a governed output was deleted or never written.
fn Unbodied_Profile(profile: &Profile, output: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(
        output,
        "{}: a sidecar is there and {} is not, so a governed output was deleted or never \
         written",
        profile.id, profile.output
    );

    return ExitCode::Stale;
}

/// The comparison itself, already computed -- printed, with a store that may not be whole.
fn Compared_Profile(
    assembly: &Assembly,
    profile: &Profile,
    result: &Result<nomos_spec_project::Freshness, nomos_spec_project::ProjectError>,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let freshness = match result
    {
        Ok(freshness) => freshness,
        Err(error) => return Report_Build_Error(assembly, error, channels.notes),
    };

    let report = freshness.Report(&profile.output);
    let _ = writeln!(channels.output, "{}: {report}", profile.id);

    if freshness.Is_Fresh()
    {
        return ExitCode::Ok;
    }

    return ExitCode::Stale;
}

/// Whether this repository promised the output a profile names.
///
/// Named rather than a bool. `Absent(profile, true, output)` said nothing at the call
/// site about which of the two absences was being reported, and the two are not close:
/// one is a broken promise and the other is a profile nobody builds here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Promise
{
    /// Promised, so an absence is an output that was never written or has been deleted.
    Required,
    /// Not promised, so an absence is simply a profile this build root does not carry.
    Optional,
}

impl Promise
{
    /// Whether a profile's identifier is in the required list.
    const fn Of(required: bool) -> Self
    {
        if required
        {
            return Self::Required;
        }

        return Self::Optional;
    }
}

/// What the run looked at, printed whether or not it found anything.
///
/// A freshness command that prints nothing over a directory holding no outputs reads
/// exactly like one that checked everything and was happy, which is the defect the whole
/// group exists to avoid.
struct Census<'a>
{
    /// How many profiles this run was going to look for.
    wanted: usize,
    /// How many it found both halves of and compared.
    checked: u32,
    /// Those absent from the build root and not promised by anyone.
    unbuilt: Vec<&'a str>,
    /// Those this run was told must be present.
    required: &'a [String],
    /// Those it was promised and did not get a current output for, whether because
    /// nothing was on disk or because what was there did not hold up.
    unmet: Vec<&'a str>,
    /// The worst code any profile examined so far has contributed.
    worst: ExitCode,
}

impl<'a> Census<'a>
{
    /// An empty census over a run that is about to look at `wanted` profiles.
    fn Over(wanted: usize, required: &'a [String]) -> Self
    {
        return Self {
            wanted,
            checked: 0,
            unbuilt: Vec::new(),
            required,
            unmet: Vec::new(),
            worst: ExitCode::Ok,
        };
    }

    /// One profile's outcome, printed, counted, and reduced to one code for the run.
    ///
    /// A promise is kept only by an output that is current. A half-present pair or an
    /// edited body has already printed its own line, and carrying that into the
    /// requirement summary is what stops the summary reporting a requirement as met by a
    /// file that just failed.
    fn Record(&mut self, outcome: &'a ProfileOutcome, assembly: &Assembly, channels: &mut Channels<'_>) -> ExitCode
    {
        let promise = Promise::Of(self.required.iter().any(|id| return *id == outcome.profile.id));
        let found = Printed_Verdict(assembly, outcome, channels);

        let Some(code) = found
        else
        {
            return self.Absent(outcome, promise, channels.output);
        };

        self.checked = self.checked.saturating_add(1);
        if promise == Promise::Required && !matches!(code, ExitCode::Ok)
        {
            self.unmet.push(&outcome.profile.id);
        }

        return code;
    }

    /// A profile with neither half on disk: a broken promise, or simply not built here.
    fn Absent(&mut self, outcome: &'a ProfileOutcome, promise: Promise, output: &mut dyn std::io::Write) -> ExitCode
    {
        if promise == Promise::Optional
        {
            self.unbuilt.push(&outcome.profile.id);

            return ExitCode::Ok;
        }

        let _ = writeln!(
            output,
            "{}: required here, and neither {} nor its stamp is on disk, so an output this \
             repository promises to ship was never written or has been deleted",
            outcome.profile.id, outcome.profile.output
        );
        self.unmet.push(&outcome.profile.id);

        return ExitCode::Stale;
    }

    fn Report(&self, into: &Path, only: Option<&str>, output: &mut dyn std::io::Write) -> ExitCode
    {
        let _ = writeln!(
            output,
            "checked {} of {} governed output(s) under {}",
            self.checked,
            self.wanted,
            into.display()
        );

        if !self.unbuilt.is_empty()
        {
            let _ = writeln!(output, "not built here: {}", self.unbuilt.join(", "));
        }

        self.Requirements(output);

        return self.Outcome(into, only, output);
    }

    /// What this run was promised, named whether or not it was kept.
    ///
    /// The satisfied case prints too. A gate step whose green output does not say which
    /// outputs it enforced is indistinguishable from one that enforced nothing, and this
    /// whole flag exists because `checked 0 of 14` already exits zero.
    fn Requirements(&self, output: &mut dyn std::io::Write)
    {
        if self.required.is_empty()
        {
            return;
        }

        if self.unmet.is_empty()
        {
            let _ = writeln!(output, "required and current: {}", self.required.join(", "));

            return;
        }

        let _ = writeln!(
            output,
            "required and not current: {}",
            self.unmet.join(", ")
        );
    }

    /// The code the run reports, once everything it looked at has been named.
    ///
    /// Asking about one profile that is not there is a question about a named file, and
    /// "no such file" is its answer. Asking about all of them over a build root that holds
    /// three is the ordinary case and not a failure. A profile that was *required* is
    /// neither: it has already been reported as a missing promise, and letting this answer
    /// for it would downgrade that finding to a lookup miss.
    fn Outcome(&self, into: &Path, only: Option<&str>, output: &mut dyn std::io::Write) -> ExitCode
    {
        let Some(id) = only
        else
        {
            return self.worst;
        };

        if self.checked != 0 || !self.unmet.is_empty()
        {
            return self.worst;
        }

        let _ = writeln!(
            output,
            "{id} has not been built under {}, so there was nothing to compare",
            into.display()
        );

        return ExitCode::NotFound;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_orchestration::corpus::{Assemble_Corpus, CorpusRequest, DEFAULT_REVISION};

    /// A store with the embedded governing records seeded and no corpus, so
    /// [`Assembly::Is_Complete`] is deterministically `false` -- `root: None` records an
    /// absence unconditionally, regardless of what any real environment variable holds.
    fn Corpus_Unset_Assembly() -> Assembly
    {
        let request = CorpusRequest {
            variable: "NOMOS_SPEC_FRESHNESS_TEST_CORPUS_UNSET".to_owned(),
            root: None,
            revision: DEFAULT_REVISION.to_owned(),
        };

        return Assemble_Corpus(&request).expect("the embedded governing records always seed");
    }

    #[test]
    fn Test_Freshness_Of_Should_Report_An_Unknown_Profile()
    {
        let assembly = Corpus_Unset_Assembly();
        let request = FreshnessRequest {
            into: std::env::temp_dir(),
            profile: Some("no-such-profile-p23-testing-host".to_owned()),
            require: Vec::new(),
        };
        let mut output = Vec::new();
        let mut notes = Vec::new();
        let mut channels = Channels { output: &mut output, notes: &mut notes };

        let code = Freshness_Of(&assembly, &request, &mut channels);

        assert_eq!(code, ExitCode::NotFound);
        assert!(String::from_utf8_lossy(&notes).contains("no-such-profile-p23-testing-host"));
    }
}
