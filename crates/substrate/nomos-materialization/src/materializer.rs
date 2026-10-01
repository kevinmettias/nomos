//! Performing a planned run, and undoing it when a write fails partway through.
//!
//! Two phases, and the split is what makes the promise in [`crate`]'s own doc provable
//! rather than hopeful. **Nothing is written until everything is planned**: every refusal
//! `OD-PACKAGE-004` requires is reached with the tree untouched, every target's prior
//! contents are already in hand, and the only thing left to fail is the filesystem itself.
//! **A write that fails undoes what came before it**, in reverse order, restoring what was
//! there or removing what was not.
//!
//! Per-target atomicity is the port's, not this module's: `Replace_Atomically` promises a
//! temporary file and a rename, so a target is never half-written and a failed write leaves
//! its previous bytes intact. What this module adds is the guarantee *across* targets, which
//! no single-file primitive can give.

mod composed;
mod plan;

use crate::materialization_error::MaterializationError;
use crate::materialization_intent::MaterializationIntent;
use crate::materialization_report::MaterializationReport;
use crate::materialization_roots::MaterializationRoots;
use crate::placement::Placement;
use nomos_platform::FileSystem;
use plan::PlannedWrite;

/// Performs every declared intent against a real tree, or performs none of them.
///
/// `OD-PACKAGE-003`'s generic materializer. It is handed intents and two roots and knows
/// nothing else: not which package kind declared them, not what the targets are for, not
/// what a crate, a rule, a gate or a peer connection is.
///
/// On success every intent was performed, in declaration order, under the behaviour its
/// ownership class requires. On failure the tree is either untouched (a refusal) or restored
/// (a rollback), and the error says which.
///
/// # Errors
///
/// Returns [`MaterializationError::Refused`] when any intent cannot be performed — including
/// a `UserOwned` target, which is refused for the whole run rather than skipped within it —
/// with nothing written. Returns [`MaterializationError::RolledBack`] when a write failed
/// and every earlier write was undone, and [`MaterializationError::RollbackFailed`] when an
/// undo failed too.
pub fn Materialize(
    intents: &[MaterializationIntent],
    roots: &MaterializationRoots,
    file_system: &impl FileSystem,
) -> Result<MaterializationReport, MaterializationError>
{
    let planned = plan::Plan(intents, roots, file_system)?;

    return Perform(&planned, file_system);
}

/// Writes every planned target, undoing what it already wrote if one fails.
fn Perform(planned: &[PlannedWrite], file_system: &impl FileSystem) -> Result<MaterializationReport, MaterializationError>
{
    let mut written: Vec<&PlannedWrite> = Vec::new();
    for write in planned
    {
        if let Err(error) = file_system.Replace_Atomically(&write.path, &write.contents)
        {
            return Err(Undone(&written, file_system, &write.target, &error.to_string()));
        }

        written.push(write);
    }

    return Ok(MaterializationReport { placements: planned.iter().map(Placed).collect() });
}

/// Puts back everything already written, newest first, and names what the run ended as.
///
/// Reverse order because two intents may name one target: undoing forwards would leave the
/// earlier write standing as if it were the tree's original state.
fn Undone(written: &[&PlannedWrite], file_system: &impl FileSystem, target: &str, cause: &str) -> MaterializationError
{
    for write in written.iter().rev()
    {
        if let Err(unrestored_cause) = Restored(write, file_system)
        {
            return MaterializationError::RollbackFailed {
                target: target.to_owned(),
                cause: cause.to_owned(),
                unrestored_target: write.target.clone(),
                unrestored_cause,
            };
        }
    }

    return MaterializationError::RolledBack { target: target.to_owned(), cause: cause.to_owned() };
}

/// One target back to what it held before the run, or gone if it held nothing.
fn Restored(write: &PlannedWrite, file_system: &impl FileSystem) -> Result<(), String>
{
    let restored = match &write.previous
    {
        Some(previous) => file_system.Replace_Atomically(&write.path, previous),
        None => file_system.Remove_File(&write.path),
    };

    return restored.map_err(|error| return error.to_string());
}

/// What a completed write is reported as.
fn Placed(write: &PlannedWrite) -> Placement
{
    return Placement {
        surface: write.surface.clone(),
        target: write.target.clone(),
        ownership_class: write.ownership_class,
        publication_scope: write.publication_scope,
        outcome: write.outcome,
    };
}
