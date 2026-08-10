//! Whether a committed projection still stands against the store it came from.

use super::{Catalogue, Profile, ExitCode, Assembly, FreshnessRequest, Channels, Report_Project_Error, Path, SIDECAR_SUFFIX, Rendered, Check, Report_Build_Error};

/// The shipped profile that identifier names, or the message saying which ones exist.
pub(super) fn Resolved<'a>(
    catalogue: &'a Catalogue,
    profile: &str,
    notes: &mut dyn std::io::Write,
) -> Result<&'a Profile, ExitCode>
{
    let Some(declared) = catalogue.Named(profile)
    else
    {
        let _ = writeln!(
            notes,
            "no shipped profile is named {profile}. There are {}: {}",
            catalogue.Profiles().len(),
            catalogue
                .Profiles()
                .iter()
                .map(|shipped| return shipped.id.clone())
                .collect::<Vec<String>>()
                .join(", ")
        );

        return Err(ExitCode::NotFound);
    };

    return Ok(declared);
}

/// `D-128`'s check, run over what is on disk.
///
/// [`nomos_spec_project::Check`] shipped with the renderers and was reachable from that
/// crate's own unit tests and from nothing else, so a hand edit to a generated output was
/// detectable in principle and detected by nobody — the shape `OD-GATE-001` is about. This
/// is the command that runs it.
///
/// The branch that earns its own case is the half-present pair. A body with no sidecar
/// beside it is a failure rather than something skipped, because otherwise deleting the
/// sidecar is how an edit stops being caught, and a check teaches that trick to the first
/// person who trips over it.
pub(super) fn Freshness_Of(
    assembly: &Assembly,
    request: &FreshnessRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let catalogue = match Catalogue::Shipped()
    {
        Ok(catalogue) => catalogue,
        Err(error) => return Report_Project_Error(&error, channels.notes),
    };

    let only = request.profile.as_deref();
    let (required, wanted) = match Examined(&catalogue, request, channels.notes)
    {
        Ok(examined) => examined,
        Err(code) => return code,
    };

    let mut census = Census::Over(wanted.len(), required);
    let worst = census.Survey(assembly, &wanted, &request.into, channels);

    return census.Report(&request.into, only, worst, channels.output);
}

/// Which profiles this run will look at, and which of them it was promised.
///
/// Both are resolved before any disk is read, so an unknown identifier stays a question
/// about a profile rather than becoming an answer about a file.
pub(super) fn Examined<'a>(
    catalogue: &'a Catalogue,
    request: &FreshnessRequest,
    notes: &mut dyn std::io::Write,
) -> Result<(Vec<&'a str>, Vec<&'a Profile>), ExitCode>
{
    let only = request.profile.as_deref();
    let required = Required_Profiles(catalogue, &request.require, notes)?;

    let wanted: Vec<&Profile> = match only
    {
        Some(id) => vec![Resolved(catalogue, id, notes)?],
        None => catalogue.Profiles().iter().collect(),
    };

    Every_Requirement_Examined(&required, &wanted, only, notes)?;

    return Ok((required, wanted));
}

/// The profiles a run was told it must find, resolved before any disk is read.
///
/// Resolving first is what keeps an unknown `--require` a question about a profile rather
/// than an answer about a file. Reporting `diagram-sett` as a missing output would send a
/// reader looking for something that was never nameable, and the catalogue already knows
/// how to refuse an identifier by listing the ones that exist.
pub(super) fn Required_Profiles<'a>(
    catalogue: &'a Catalogue,
    require: &[String],
    notes: &mut dyn std::io::Write,
) -> Result<Vec<&'a str>, ExitCode>
{
    let mut required: Vec<&str> = Vec::new();

    for id in require
    {
        required.push(Resolved(catalogue, id, notes)?.id.as_str());
    }

    return Ok(required);
}

/// Refuses a run that was promised an output it would never have looked at.
///
/// `--profile a --require b` asks for one profile to be examined and a different one to be
/// guaranteed. Answering it would mean reporting success over a requirement nothing
/// checked, which is the shape this flag exists against — so it is a usage error and not a
/// quiet pass.
pub(super) fn Every_Requirement_Examined(
    required: &[&str],
    wanted: &[&Profile],
    only: Option<&str>,
    notes: &mut dyn std::io::Write,
) -> Result<(), ExitCode>
{
    let Some(unexamined) = required
        .iter()
        .find(|id| return !wanted.iter().any(|profile| return profile.id == **id))
    else
    {
        return Ok(());
    };

    let _ = writeln!(
        notes,
        "--require {unexamined} cannot hold while --profile {} narrows this run to one \
         other profile: the requirement would be reported as met by a run that never \
         looked for it",
        only.unwrap_or("<none>")
    );

    return Err(ExitCode::Usage);
}

/// One profile's answer, or [`None`] when neither half of the pair is on disk.
pub(super) fn Verdict(
    assembly: &Assembly,
    profile: &Profile,
    into: &Path,
    channels: &mut Channels<'_>,
) -> Option<ExitCode>
{
    let body_path = into.join(&profile.output);
    let sidecar_path = into.join(format!("{}{SIDECAR_SUFFIX}", profile.output));
    let body = std::fs::read_to_string(&body_path).ok();
    let sidecar = std::fs::read_to_string(&sidecar_path).ok();

    return match (body, sidecar)
    {
        (None, None) => None,
        (Some(_), None) => Some(Unstamped(profile, channels.output)),
        (None, Some(_)) => Some(Unbodied(profile, channels.output)),
        (Some(body), Some(sidecar)) =>
        {
            let rendered = Rendered {
                body: &body,
                sidecar: &sidecar,
            };

            Some(Compared(assembly, profile, &rendered, channels))
        }
    };
}

/// A body with no stamp beside it.
///
/// A failure rather than something skipped, because otherwise deleting the sidecar is how
/// an edit stops being caught, and a check that skipped it would teach that trick to the
/// first person who tripped over it.
pub(super) fn Unstamped(profile: &Profile, output: &mut dyn std::io::Write) -> ExitCode
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
pub(super) fn Unbodied(profile: &Profile, output: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(
        output,
        "{}: a sidecar is there and {} is not, so a governed output was deleted or never \
         written",
        profile.id, profile.output
    );

    return ExitCode::Stale;
}

/// The comparison itself, with a store that may not be whole.
pub(super) fn Compared(
    assembly: &Assembly,
    profile: &Profile,
    rendered: &Rendered<'_>,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let body = Some(rendered.body);
    let sidecar = Some(rendered.sidecar);
    let freshness = match Check(&assembly.store, profile, body, sidecar)
    {
        Ok(freshness) => freshness,
        Err(error) => return Report_Build_Error(assembly, &error, channels.notes),
    };

    let report = freshness.Report(&profile.output);
    let _ = writeln!(channels.output, "{}: {report}", profile.id);

    if freshness.Is_Fresh()
    {
        return ExitCode::Ok;
    }

    return ExitCode::Stale;
}

/// What the run looked at, printed whether or not it found anything.
///
/// A freshness command that prints nothing over a directory holding no outputs reads
/// exactly like one that checked everything and was happy, which is the defect the whole
/// group exists to avoid.
pub(super) struct Census<'a>
{
    /// How many profiles this run was going to look for.
    wanted: usize,
    /// How many it found both halves of and compared.
    checked: u32,
    /// Those absent from the build root and not promised by anyone.
    unbuilt: Vec<&'a str>,
    /// Those this run was told must be present.
    required: Vec<&'a str>,
    /// Those it was promised and did not get a current output for, whether because
    /// nothing was on disk or because what was there did not hold up.
    unmet: Vec<&'a str>,
}

impl<'a> Census<'a>
{
    /// An empty census over a run that is about to look at `wanted` profiles.
    fn Over(wanted: usize, required: Vec<&'a str>) -> Self
    {
        return Self {
            wanted,
            checked: 0,
            unbuilt: Vec::new(),
            required,
            unmet: Vec::new(),
        };
    }

    /// Every wanted profile, examined, counted, and reduced to one code for the run.
    fn Survey(
        &mut self,
        assembly: &Assembly,
        wanted: &[&'a Profile],
        into: &Path,
        channels: &mut Channels<'_>,
    ) -> ExitCode
    {
        let mut worst = ExitCode::Ok;

        for profile in wanted
        {
            let found = Verdict(assembly, profile, into, channels);
            let code = self.Record(profile, found, channels.output);
            worst = Worse(worst, code);
        }

        return worst;
    }

    /// One profile's outcome, counted, and the code it contributes to the run.
    ///
    /// A promise is kept only by an output that is current. A half-present pair or an
    /// edited body has already printed its own line, and carrying that into the
    /// requirement summary is what stops the summary reporting a requirement as met by a
    /// file that just failed.
    fn Record(
        &mut self,
        profile: &'a Profile,
        found: Option<ExitCode>,
        output: &mut dyn std::io::Write,
    ) -> ExitCode
    {
        let promised = self.required.contains(&profile.id.as_str());

        let Some(code) = found
        else
        {
            return self.Absent(profile, promised, output);
        };

        self.checked = self.checked.saturating_add(1);
        if promised && !matches!(code, ExitCode::Ok)
        {
            self.unmet.push(profile.id.as_str());
        }

        return code;
    }

    /// A profile with neither half on disk: a broken promise, or simply not built here.
    fn Absent(
        &mut self,
        profile: &'a Profile,
        promised: bool,
        output: &mut dyn std::io::Write,
    ) -> ExitCode
    {
        if !promised
        {
            self.unbuilt.push(profile.id.as_str());

            return ExitCode::Ok;
        }

        let _ = writeln!(
            output,
            "{}: required here, and neither {} nor its stamp is on disk, so an output this \
             repository promises to ship was never written or has been deleted",
            profile.id, profile.output
        );
        self.unmet.push(profile.id.as_str());

        return ExitCode::Stale;
    }

    fn Report(
        &self,
        into: &Path,
        only: Option<&str>,
        worst: ExitCode,
        output: &mut dyn std::io::Write,
    ) -> ExitCode
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

        return self.Outcome(into, only, worst, output);
    }

    /// The code the run reports, once everything it looked at has been named.
    ///
    /// Asking about one profile that is not there is a question about a named file, and
    /// "no such file" is its answer. Asking about all of them over a build root that holds
    /// three is the ordinary case and not a failure. A profile that was *required* is
    /// neither: it has already been reported as a missing promise, and letting this answer
    /// for it would downgrade that finding to a lookup miss.
    fn Outcome(
        &self,
        into: &Path,
        only: Option<&str>,
        worst: ExitCode,
        output: &mut dyn std::io::Write,
    ) -> ExitCode
    {
        let Some(id) = only
        else
        {
            return worst;
        };

        if self.checked != 0 || !self.unmet.is_empty()
        {
            return worst;
        }

        let _ = writeln!(
            output,
            "{id} has not been built under {}, so there was nothing to compare",
            into.display()
        );

        return ExitCode::NotFound;
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
}

/// The code a run reports when its profiles disagreed about what happened.
///
/// [`ExitCode::Stale`] beats [`ExitCode::Absent`] deliberately: a definite finding about
/// one output is more actionable than a machine that could not check another, and the
/// text above has already said both.
pub(super) const fn Worse(carried: ExitCode, found: ExitCode) -> ExitCode
{
    return match (carried, found)
    {
        (ExitCode::StoreError, _) | (_, ExitCode::StoreError) => ExitCode::StoreError,
        (ExitCode::Stale, _) | (_, ExitCode::Stale) => ExitCode::Stale,
        (ExitCode::Absent, _) | (_, ExitCode::Absent) => ExitCode::Absent,
        _ => ExitCode::Ok,
    };
}
