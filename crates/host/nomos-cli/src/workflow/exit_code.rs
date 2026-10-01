//! What `nomos workflow` tells the shell.

/// `ExitCode::Ok`: the step ran and answered.
const OK_CODE: isize = 0;

/// `ExitCode::Refused`: the one step declared itself incoherent, or a gate step failed.
const REFUSED_CODE: isize = 1;

/// `ExitCode::Usage`: the command line was wrong.
const USAGE_CODE: isize = 2;

/// `ExitCode::Unavailable`: the named root, registry, executor or model backend could not be
/// read, run or answered at all.
const UNAVAILABLE_CODE: isize = 5;

/// `ExitCode::Vacuous`: the check, correction or gate step found nothing to judge.
const VACUOUS_CODE: isize = 6;

/// What `nomos workflow` tells the shell.
///
/// The numbers are shared with every other group on this binary -- see `check::ExitCode`'s own
/// doc for the fuller statement of that discipline. [`ExitCode::Unavailable`] carries the
/// meaning `agent`'s own `Unavailable` and `check`'s own `Unreadable` already give `5`: the
/// thing this step needed could not be read, started, or answered at all, whichever of the
/// three bodies it was.
///
/// Each discriminant is a named constant rather than a spelled digit, because the number is
/// the one thing this enum shares with five sibling groups and the name is what says which
/// shared meaning it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode
{
    /// The step ran and answered -- a `Judged` check regardless of its findings, or a real
    /// response from `ClaudeCode`/`Ollama`.
    Ok = OK_CODE,
    /// The one step this command composed declared itself incoherent -- never reachable
    /// through this command today, since the one declaration this verb composes is fixed, but
    /// a real answer this match must still give.
    Refused = REFUSED_CODE,
    /// The command line was wrong.
    Usage = USAGE_CODE,
    /// The named root could not be read as a directory or a workspace state, this build's
    /// capability registry is self-contradictory, or the chosen executor/model backend could
    /// not be started, exited non-zero, or timed out.
    Unavailable = UNAVAILABLE_CODE,
    /// A check step found no source under its root, or no syntax fact was materialized for any
    /// of it.
    Vacuous = VACUOUS_CODE,
}

impl ExitCode
{
    /// The numeric code.
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}
