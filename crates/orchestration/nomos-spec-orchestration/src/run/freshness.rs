//! Resolving `nomos spec freshness`'s request, and comparing every profile it names against
//! what `filesystem` holds.
//!
//! Reads through [`FileSystem::Read_To_String`] rather than raw `std::fs`, the read half of
//! the same decision [`crate::run::render`] documents for the write half: a rendered body and
//! its sidecar are files this crate placed at paths it already knows, not an arbitrary
//! caller-named path and not a directory listing.

use nomos_platform::FileSystem;
use nomos_spec_project::{Catalogue, Check, Profile, SIDECAR_SUFFIX};
use std::path::Path;

use crate::corpus::Assembly;
use crate::spec_outcome::{FreshnessAnswer, FreshnessRefusal, ProfileOutcome, Verdict};
use crate::request::FreshnessRequest;

/// Every profile `request` names, compared against what `filesystem` holds under
/// `request.into`.
///
/// Moved from `nomos-cli::spec::verb::freshness`'s own `Freshness_Of`, `Examined`,
/// `Required_Profiles`, `Every_Requirement_Examined` and `Verdict`, minus the writing: which
/// profile is stale, edited, absent or half-present is decided here; how many were checked,
/// which requirements were kept and which exit code the run reports stays a rendering
/// decision, because it needs nothing this crate does not already hand back in
/// [`FreshnessAnswer`].
///
/// # Errors
///
/// Returns [`FreshnessRefusal::NoSuchProfile`] when `--profile` or a `--require` names an
/// identifier the catalogue does not carry, [`FreshnessRefusal::RequirementUnexamined`] when
/// `--require` names a profile `--profile` narrowed this run away from, and
/// [`FreshnessRefusal::Project`] when the embedded catalogue itself fails to parse.
pub fn Freshness_Of_Render<Filesystem: FileSystem>(
    assembly: &Assembly,
    request: &FreshnessRequest,
    filesystem: &Filesystem,
) -> Result<FreshnessAnswer, FreshnessRefusal>
{
    let catalogue = Catalogue::Shipped().map_err(FreshnessRefusal::Project)?;
    let only = request.profile.as_deref();
    let required = Required_Profiles(&catalogue, &request.require)?;
    let wanted = Wanted_Profiles(&catalogue, only)?;
    Every_Requirement_Examined(&required, &wanted, only)?;

    let examined = wanted
        .iter()
        .map(|profile| {
            return ProfileOutcome {
                profile: (*profile).clone(),
                verdict: Verdict_Of(assembly, profile, &request.into, filesystem),
            };
        })
        .collect();

    return Ok(FreshnessAnswer { examined, required });
}

/// The profiles a run was told it must find, resolved before any disk is read.
///
/// Resolving first is what keeps an unknown `--require` a question about a profile rather
/// than an answer about a file.
fn Required_Profiles(catalogue: &Catalogue, require: &[String]) -> Result<Vec<String>, FreshnessRefusal>
{
    let mut required = Vec::new();
    for id in require
    {
        required.push(Resolved_Profile(catalogue, id)?.id.clone());
    }

    return Ok(required);
}

/// The shipped profile that identifier names, or the refusal saying which ones exist.
fn Resolved_Profile<'a>(catalogue: &'a Catalogue, profile: &str) -> Result<&'a Profile, FreshnessRefusal>
{
    return catalogue.Named(profile).ok_or_else(|| {
        return FreshnessRefusal::NoSuchProfile {
            requested: profile.to_owned(),
            known: catalogue.Profiles().iter().map(|shipped| return shipped.id.clone()).collect(),
        };
    });
}

/// Which profiles this run will look at.
fn Wanted_Profiles<'a>(catalogue: &'a Catalogue, only: Option<&str>) -> Result<Vec<&'a Profile>, FreshnessRefusal>
{
    return match only
    {
        Some(id) => Ok(vec![Resolved_Profile(catalogue, id)?]),
        None => Ok(catalogue.Profiles().iter().collect()),
    };
}

/// Refuses a run that was promised an output it would never have looked at.
///
/// `--profile a --require b` asks for one profile to be examined and a different one to be
/// guaranteed. Answering it would mean reporting success over a requirement nothing checked.
fn Every_Requirement_Examined(
    required: &[String],
    wanted: &[&Profile],
    only: Option<&str>,
) -> Result<(), FreshnessRefusal>
{
    let Some(unexamined) = required
        .iter()
        .find(|id| return !wanted.iter().any(|profile| return &profile.id == *id))
    else
    {
        return Ok(());
    };

    return Err(FreshnessRefusal::RequirementUnexamined {
        requested: unexamined.clone(),
        only: only.map(str::to_owned),
    });
}

/// One profile's answer, read off `filesystem` and compared against the store when both
/// halves are there.
fn Verdict_Of<Filesystem: FileSystem>(
    assembly: &Assembly,
    profile: &Profile,
    into: &Path,
    filesystem: &Filesystem,
) -> Verdict
{
    let body_path = into.join(&profile.output);
    let sidecar_path = into.join(format!("{}{SIDECAR_SUFFIX}", profile.output));
    let body = filesystem.Read_To_String(&body_path).ok();
    let sidecar = filesystem.Read_To_String(&sidecar_path).ok();

    return match (body, sidecar)
    {
        (None, None) => Verdict::Absent,
        (Some(_), None) => Verdict::Unstamped,
        (None, Some(_)) => Verdict::Unbodied,
        (Some(body), Some(sidecar)) =>
        {
            let comparison = Check(&assembly.store, profile, Some(&body), Some(&sidecar));
            Verdict::Compared(comparison)
        }
    };
}
