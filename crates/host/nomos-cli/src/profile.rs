//! `nomos profile` — what a root holds before anything judges it, and what this host can
//! answer for it.
//!
//! The first-run verb. `990abac2` built `nomos_workspace_discovery::WorkspaceProfile::Of_Root`
//! and nothing called it: the profile existed as a function while a repository adopting this
//! tool still had no command to type. This group is that command, and it is a renderer over
//! that function the same way `check` is a renderer over `nomos_check_orchestration::Run` —
//! it walks nothing this crate does not already walk, judges nothing, and decides nothing
//! about the tree.
//!
//! # The two halves, and why the second is not a file listing
//!
//! The profile answers what is *there*: how many sources of each registered language the
//! walk finds, which language manifests sit at the root, which repository policy files are
//! present, absent or present-and-unreadable, and whether the root carries the marker. The
//! absent half is printed as plainly as the present half, because an absent policy file is
//! the one a person can act on.
//!
//! What a profile cannot answer is whether this machine can produce the facts those files
//! configure. A repository with a `Cargo.toml` and a `standards.json` still gets no lint
//! diagnostics from a host with no `cargo`, and today the only way to find that out is to
//! run a check and notice that a capability quietly went unanswered. [`availability`] is the
//! second half: per declared capability, what this host settles about answering it, with the
//! tool named where one is missing and the question left open where settling it would mean
//! running something. Its own module doc carries where that line is drawn and why this verb
//! does not cross it.
//!
//! # And a third thing it can do, once
//!
//! `--write-gate-policy` writes a starting `nomos-gate.json` for a root that has none, and
//! refuses rather than overwriting one that exists. [`starter_policy`] is what it writes and
//! why that file declares nothing at all.
//!
//! # What this group does not do
//!
//! It judges no tree, composes no rule, and changes nothing about any other group's
//! judgment or exit codes. It reads this build's capability registry —
//! `nomos_check_orchestration::Registered`, the same composition a real `check` run uses —
//! and asks it what is declared; it never runs it. `crate::vacuity`'s stance for this group
//! is `NotApplicable` for the reason recorded there: an empty directory profiles as a tree
//! with nothing in it, which is a true answer and the one a person adopting this tool from
//! an empty directory needs to read, not a clean judgment over a world nobody looked at.

mod availability;
mod capability_standing;
mod exit_code;
mod offer_standing;
mod parsing;
mod profile_command;
mod program_search;
mod provider_need;
mod provider_standing;
mod report;
mod starter_outcome;
mod starter_policy;

#[cfg(test)]
mod tests;

pub use parsing::Profile_Command_From_String_Arguments;

pub(crate) use capability_standing::CapabilityStanding;
pub(crate) use exit_code::ExitCode;
pub(crate) use offer_standing::OfferStanding;
pub(crate) use profile_command::ProfileCommand;
pub(crate) use program_search::ProgramSearch;
pub(crate) use provider_need::{Need_Of, ProviderNeed};
pub(crate) use provider_standing::ProviderStanding;
pub(crate) use starter_outcome::StarterOutcome;

use crate::arguments::Named_Value_From_String_Arguments;
use nomos_composer_std::{ENVIRONMENT, FILE_SYSTEM};
use nomos_workspace_discovery::WorkspaceProfile;
use std::io::Write;
use std::path::PathBuf;

/// Renders the profile of a root, what this host can answer for it, and whatever the
/// starter file came to.
pub fn Run(command: &ProfileCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    let profile = match WorkspaceProfile::Of_Root(&command.root)
    {
        Ok(profile) => profile,
        Err(refusal) => return report::Render_Refusal(&refusal, stderr),
    };

    report::Render_Profile(&command.root, &profile, stdout);

    let declared = Render_Availability(stdout, stderr);
    let starter = Render_Starter_If_Asked(command, stdout, stderr);

    if declared == ExitCode::Ok
    {
        return starter;
    }

    return declared;
}

/// What this host settles about answering each capability this build declares.
///
/// The registry is composed rather than run: `Registered` is the same function a real check
/// run composes from, so what is reported here is what would really be asked of this host
/// rather than a second list that could drift from it.
fn Render_Availability(stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    let registry = match nomos_check_orchestration::Registered()
    {
        Ok(registry) => registry,
        Err(error) => return report::Render_Contradictory(&error, stderr),
    };

    let standings = availability::Capability_Standings(&registry, &ENVIRONMENT, &FILE_SYSTEM);
    report::Render_Standings(&standings, stdout);

    return ExitCode::Ok;
}

/// Writes the starter policy file when it was asked for, and reports what came of it.
///
/// Reported after the profile rather than instead of it: the policy-file section above has
/// just said whether one is there, so a refusal to overwrite reads as the consequence of
/// something the reader has already seen.
fn Render_Starter_If_Asked(command: &ProfileCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    if !command.write_gate_policy
    {
        return ExitCode::Ok;
    }

    let outcome = starter_policy::Write_Starter_Gate_Policy(&command.root, &FILE_SYSTEM);

    return report::Render_Starter(&outcome, stdout, stderr);
}
