//! Running `gh api` against one already-identified pull-request review comment and
//! capturing GitHub's own response, exactly as it arrived.
//!
//! This is the generic-connector-substrate side of `ARC-CONNECTOR-001`'s seam: it hands
//! back raw vendor bytes and never interprets a field inside them.
//! [`crate::translation`] is the vendor-to-canonical side, and the two are deliberately not
//! one function -- the same split `OD-CONNECTOR-002`'s fixture rule depends on: a recording
//! taken here, replayed through that, exercises the translation on every run.
//!
//! `gh api`'s raw REST passthrough for this endpoint returns the object exactly as
//! GitHub's own API defines it, whole -- no `--json` field subset is requested.
//! Narrowing the fields read is [`crate::translation::Translate_Review_Comment`]'s job
//! entirely, so that this layer stays the one that knows nothing about a vendor field name
//! at all -- not even enough to ask for one by name.

use nomos_platform::{Command, ExitOutcome, ProgramLauncher};
use std::time::Duration;

/// `gh api` against one comment id is one HTTPS round trip to the GitHub API; warm, it
/// returns in a second or two. This bound is headroom, not the expected case.
const TIMEOUT: Duration = Duration::from_secs(30);

/// The vendor could not be reached, or answered with something other than a fetched review
/// comment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchError
{
    pub reason: String,
}

impl core::fmt::Display for FetchError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Fetches one pull-request review comment live and returns GitHub's own JSON response,
/// unparsed.
///
/// A caller already knows `comment_id` -- discovering which comments exist on a pull
/// request at all is not this connector's job.
///
/// # Errors
///
/// [`FetchError`] if `gh` cannot be run, is killed for exceeding [`TIMEOUT`] or going idle
/// that long, is terminated before finishing, or exits non-zero -- `gh api`'s own exit code
/// is a genuine verdict on whether the fetch succeeded (comment deleted, repository
/// private, token missing every scope `gh auth status` would have reported), not a report
/// this reader must relay as data.
pub fn Fetch_Review_Comment<Launcher: ProgramLauncher>(repository: &str, comment_id: u64, launcher: &Launcher) -> Result<Vec<u8>, FetchError>
{
    let command = Github_Api_Review_Comment_Command(repository, comment_id);
    let output = launcher.Run(&command).map_err(|error| FetchError {
        reason: format!("gh could not be run: {error}"),
    })?;

    Require_Succeeded(&output.outcome, &output.stderr)?;

    return Ok(output.stdout.into_bytes());
}

fn Github_Api_Review_Comment_Command(repository: &str, comment_id: u64) -> Command
{
    return Command::From_String_Arguments(
        vec![
            "gh".to_owned(),
            "api".to_owned(),
            format!("repos/{repository}/pulls/comments/{comment_id}"),
        ],
        TIMEOUT,
    );
}

fn Require_Succeeded(outcome: &ExitOutcome, stderr: &str) -> Result<(), FetchError>
{
    return match outcome
    {
        ExitOutcome::Exited { code: 0 } => Ok(()),
        ExitOutcome::Exited { code } =>
        {
            Err(FetchError {
                reason: format!("gh api pulls/comments exited {code}: {stderr}"),
            })
        }
        ExitOutcome::TimedOut =>
        {
            Err(FetchError {
                reason: format!("gh api pulls/comments was still running after {TIMEOUT:?} and was killed"),
            })
        }
        ExitOutcome::Stalled { idle_elapsed } =>
        {
            Err(FetchError {
                reason: format!("gh api pulls/comments produced no output for {idle_elapsed:?} and was judged stalled"),
            })
        }
        ExitOutcome::Terminated =>
        {
            Err(FetchError {
                reason: "gh api pulls/comments was terminated before it could finish".to_owned(),
            })
        }
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The real, public review comment this crate's own recorded fixture was captured from.
    const COMMENT_ID: u64 = 3_521_038_097;

    #[test]
    fn Test_The_Command_Should_Name_The_Repository_And_Comment_Id()
    {
        let command = Github_Api_Review_Comment_Command("coderabbitai/rabbits-playground", COMMENT_ID);

        assert_eq!(
            command.argv,
            vec!["gh", "api", "repos/coderabbitai/rabbits-playground/pulls/comments/3521038097"]
        );
    }

    #[test]
    fn Test_A_Nonzero_Exit_Should_Be_Refused()
    {
        let refusal = Require_Succeeded(&ExitOutcome::Exited { code: 1 }, "comment not found");
        assert!(refusal.is_err());
    }

    #[test]
    fn Test_A_Timeout_Should_Be_Refused()
    {
        assert!(Require_Succeeded(&ExitOutcome::TimedOut, "").is_err());
    }

    #[test]
    fn Test_A_Zero_Exit_Should_Succeed()
    {
        assert_eq!(Require_Succeeded(&ExitOutcome::Exited { code: 0 }, ""), Ok(()));
    }
}
