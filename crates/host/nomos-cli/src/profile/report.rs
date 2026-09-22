//! Turning a workspace profile, a set of capability standings and a starter file's outcome
//! into text and an [`ExitCode`].
//!
//! Every list renders every entry, including the ones that are not there. A report that
//! omitted the absent manifest and the absent policy file would be shorter and would answer
//! the wrong question: a person who already knew which files to look for would not be
//! running this verb, and the whole value of the absent half is that it names a file they
//! can go and write.

use super::{CapabilityStanding, ExitCode, OfferStanding, ProviderStanding, StarterOutcome};
use nomos_capability::RegistryError;
use nomos_workspace_discovery::{LanguageManifest, PolicyFile, PolicyFilePresence, ProfileRefusal, SourceCount, WorkspaceProfile};
use std::io::Write;
use std::path::Path;

/// What a file that is there reads as.
const PRESENT: &str = "present";

/// What a file that is not there reads as.
const ABSENT: &str = "absent";

/// A capability some provider on this host can answer.
const AVAILABLE: &str = "available";

/// A capability no provider on this host can answer.
const UNAVAILABLE: &str = "unavailable";

/// A capability this verb did not settle either way.
const UNDETERMINED: &str = "undetermined";

/// The root could not be profiled at all.
pub(super) fn Render_Refusal(refusal: &ProfileRefusal, stderr: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(stderr, "cannot profile: {refusal}");

    return ExitCode::Unusable;
}

/// This build's own capability registry is self-contradictory, so there is no set of
/// declared capabilities to report standings for -- a defect in the composition, not in the
/// tree being profiled.
pub(super) fn Render_Contradictory(error: &RegistryError, stderr: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(
        stderr,
        "this build's own composition is contradictory, so what it declares cannot be \
         reported: {error}"
    );

    return ExitCode::Unusable;
}

/// What the root holds, before anything judges it.
pub(super) fn Render_Profile(root: &Path, profile: &WorkspaceProfile, stdout: &mut impl Write)
{
    let _ignored = writeln!(stdout, "root: {}", root.display());
    let _ignored = writeln!(stdout, "repository root marker: {}", Presence_Word(profile.has_root_marker));

    Render_Sources(&profile.sources, stdout);
    Render_Manifests(&profile.manifests, stdout);
    Render_Policy_Files(&profile.policy_files, stdout);
}

/// How many sources of each registered language the walk found, a zero included.
fn Render_Sources(sources: &[SourceCount], stdout: &mut impl Write)
{
    let _ignored = writeln!(stdout, "\nsources the walk found");

    for count in sources
    {
        let _ignored = writeln!(stdout, "  {}: {}", count.extension, count.sources);
    }
}

/// Which language manifests sit at the root, and which do not.
fn Render_Manifests(manifests: &[LanguageManifest], stdout: &mut impl Write)
{
    let _ignored = writeln!(stdout, "\nlanguage manifests at the root");

    for manifest in manifests
    {
        let _ignored = writeln!(stdout, "  {}: {} ({})", manifest.name, Presence_Word(manifest.is_present), manifest.extension);
    }
}

/// Which repository policy files are there, which are not, and which are there and
/// unreadable.
fn Render_Policy_Files(policy_files: &[PolicyFile], stdout: &mut impl Write)
{
    let _ignored = writeln!(stdout, "\nrepository policy files at the root");

    for policy_file in policy_files
    {
        let _ignored = writeln!(stdout, "  {}: {}", policy_file.name, Presence_Sentence(&policy_file.presence));
    }
}

/// Every declared capability, its verdict, and a row per provider offering it.
///
/// Two levels rather than one, because a capability with several offers has several
/// answers: the verdict line says whether anything on this host can answer at all, and the
/// rows under it say which provider that was and what is true of every other. A reader who
/// has a Rust workspace and reads `available` off a verdict whose only available provider
/// reads `go.mod` is one line away from seeing it.
pub(super) fn Render_Standings(standings: &[CapabilityStanding], stdout: &mut impl Write)
{
    let _ignored = writeln!(
        stdout,
        "\ncapabilities this workspace declares, and what this host settles about answering them"
    );

    for standing in standings
    {
        let _ignored = writeln!(stdout, "  {}: {}", standing.capability, Verdict_Word(&standing.Summary()));
        Render_Offers(&standing.offers, stdout);
    }
}

/// One row per provider offering a capability, or the one line a capability with no offers
/// gets instead.
fn Render_Offers(offers: &[OfferStanding], stdout: &mut impl Write)
{
    if offers.is_empty()
    {
        let _ignored = writeln!(stdout, "    no provider offers this capability, so nothing can answer it");
        return;
    }

    for offer in offers
    {
        let _ignored = writeln!(stdout, "    {}: {}", offer.provider, Standing_Sentence(&offer.standing));
    }
}

/// What came of being asked for a starter policy file, and what that leaves the shell.
pub(super) fn Render_Starter(outcome: &StarterOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match *outcome
    {
        StarterOutcome::Written { ref path } => Render_Written(path, stdout),
        StarterOutcome::AlreadyDeclared { ref path } => Render_Already_Declared(path, stderr),
        StarterOutcome::Unwritable { ref path, ref reason } => Render_Unwritable(path, reason, stderr),
    };
}

/// The starter file was written.
fn Render_Written(path: &Path, stdout: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(
        stdout,
        "\nwrote {}. It declares nothing, so a run over this root answers exactly as it did \
         before: no suppression, no accepted debt, no rule calibration, no coverage floor \
         and no phase. Every key its reader accepts is spelled in it, which is what there is \
         to edit.",
        path.display()
    );

    return ExitCode::Ok;
}

/// Something is already at the path, so nothing was written.
fn Render_Already_Declared(path: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(
        stderr,
        "\n{} already exists, and nothing was written. A starter file declares nothing, so \
         writing one over a declaration would retire every suppression, accepted debt and \
         phase it carried -- which is a decision for whoever wrote it.",
        path.display()
    );

    return ExitCode::Refused;
}

/// Nothing was there and it could not be written.
fn Render_Unwritable(path: &Path, reason: &str, stderr: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(stderr, "\n{} could not be written: {reason}", path.display());

    return ExitCode::Unusable;
}

/// Whether a file is there, in the one vocabulary this report uses for it.
const fn Presence_Word(is_present: bool) -> &'static str
{
    if is_present
    {
        return PRESENT;
    }

    return ABSENT;
}

/// What a first run found of one policy file, as a phrase.
///
/// The unreadable case carries the operating system's own account rather than collapsing
/// into either of the other two: absent means a run proceeds on defaults and present means
/// it reads the file, and a present-but-unreadable file is the one case where a reader
/// refuses -- which is what the person needs to hear.
fn Presence_Sentence(presence: &PolicyFilePresence) -> String
{
    return match *presence
    {
        PolicyFilePresence::Present => PRESENT.to_owned(),
        PolicyFilePresence::Absent => ABSENT.to_owned(),
        PolicyFilePresence::PresentButUnreadable { ref reason } => format!("{PRESENT}, and could not be read: {reason}"),
    };
}

/// One capability's verdict: whether anything on this host can answer it.
fn Verdict_Word(summary: &ProviderStanding) -> &'static str
{
    return match *summary
    {
        ProviderStanding::InThisBinary => AVAILABLE,
        ProviderStanding::Undetermined { .. } => UNDETERMINED,
        ProviderStanding::ToolMissing { .. } | ProviderStanding::NothingOffered => UNAVAILABLE,
    };
}

/// One provider's standing, as a verdict word and the reason behind it.
fn Standing_Sentence(standing: &ProviderStanding) -> String
{
    return match *standing
    {
        ProviderStanding::InThisBinary => format!("{AVAILABLE} -- answers from inside this binary and launches nothing"),
        ProviderStanding::ToolMissing { ref tool } =>
        {
            format!("{UNAVAILABLE} -- runs `{tool}`, and no directory on this host's search path holds it")
        }
        ProviderStanding::Undetermined { ref because } => format!("{UNDETERMINED} -- {because}"),
        ProviderStanding::NothingOffered => format!("{UNAVAILABLE} -- nothing offers this capability"),
    };
}
