//! Where a run's own record of what it observed lives, and how it gets there.

use nomos_platform::FileSystem;
use std::path::Path;

use super::OccurrenceHistory;

/// The record a run reads its own history from, relative to the run's own root.
///
/// Beside `nomos-gate.json` and named after it, because the two answer adjacent questions about
/// one tree: the policy file says what a repository decided to tolerate, and this says what has
/// actually been there. JSON for the reason the policy file is JSON -- the format this workspace
/// already reads for `standards.json` and `work/ledger.json`.
///
/// # Its presence is the opt-in, and that is the whole configuration
///
/// A run reads this file if it is there and writes it back if it was. A tree without one is
/// never given one, so every caller that predates this -- including this repository's own
/// `gate run --root .` -- is byte-for-byte unchanged, and no repository acquires a file it did
/// not ask for. A repository starts recording by writing `{"scopes": []}` here and running the
/// gate.
///
/// That is deliberately not a key in `nomos-gate.json`. A policy key states what a repository
/// *decided*; this file is evidence a run *collected*, and `OD-GATE-030` is explicit that the
/// two are different things -- it declines to decide whether an undetermined continuity blocks
/// precisely because that one is a policy question and this one is not.
pub(in crate::gate_environment) const GATE_HISTORY_FILE: &str = "nomos-gate-history.json";

/// The record under `root`, or `None` when there is none to read.
///
/// `None` for three cases that are one case to a caller: no file, a file that cannot be read,
/// and a file that is not this shape. Each of them leaves a run with no evidence, which
/// `OD-GATE-030`'s floor already has an answer for -- every baselined finding is reported
/// undetermined and tolerated exactly as it is today.
///
/// A malformed record is deliberately not refused the way a malformed `nomos-gate.json` is.
/// That file is policy, and a build that passed under policy nobody authored is the failure its
/// refusal exists to stop; this one is evidence, and a run that cannot read it has less evidence
/// rather than the wrong policy. What protects the file itself is that
/// [`Record_Occurrence_History`] is only reached with a record that was read, so a file this
/// function could not parse is never overwritten by one derived from nothing.
pub(in crate::gate_environment) fn Resolve_Occurrence_History<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Option<OccurrenceHistory>
{
    let text = filesystem.Read_To_String(&root.join(GATE_HISTORY_FILE)).ok()?;

    return serde_json::from_str(&text).ok();
}

/// Replaces the record under `root` with `history`.
///
/// A failure to serialize or to write is left where it fell rather than reported, and the
/// direction that leaves things in is the argument for it: the previous record stays, so the
/// next run reads a shorter history and establishes *less*. Every outcome of a failed write is
/// therefore an occurrence reported undetermined that could have been reported continuous, and
/// none of them is a tolerance lapsing or a recreation announced that did not happen. There is
/// also nowhere honest to report it from here -- a run's verdict is about the tree, and a gate
/// that went red because a scratch file could not be written would be failing a build over its
/// own bookkeeping.
pub(in crate::gate_environment) fn Record_Occurrence_History<Fs: FileSystem>(root: &Path, filesystem: &Fs, history: &OccurrenceHistory)
{
    let Ok(text) = serde_json::to_string_pretty(history)
    else
    {
        return;
    };

    let _written = filesystem.Replace_Atomically(&root.join(GATE_HISTORY_FILE), &text);
}
