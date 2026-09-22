//! Deciding every write before making any of them.
//!
//! Planning is where every refusal lives, and that is the design rather than a convenience.
//! `OD-PACKAGE-004` says a `UserOwned` target is never written "under any circumstance", and
//! a run that wrote three targets before reaching the fourth and refusing would have made
//! that promise true of one path and false of the tree. So an intent that cannot be
//! performed ends the run while the tree is still untouched.
//!
//! Planning also reads each target's prior contents, which is the other half of what the
//! rollback in [`super`] needs: by the time the first byte is written, what every target
//! held — or that it held nothing — is already in hand, and undoing needs no second read
//! from a filesystem that has just started failing.

use super::composed;
use crate::materialization_error::MaterializationError;
use crate::materialization_intent::MaterializationIntent;
use crate::materialization_roots::MaterializationRoots;
use crate::ownership_class::OwnershipClass;
use crate::placement_outcome::PlacementOutcome;
use crate::publication_scope::PublicationScope;
use crate::refusal::Refusal;
use crate::target;
use nomos_platform::FileSystem;
use std::path::{Path, PathBuf};

/// One decided write: where it goes, what it will hold, and what was there before it.
pub(crate) struct PlannedWrite
{
    /// The intent's surface label, passed through to the report unread.
    pub(crate) surface: String,
    /// The target as the intent spelled it, which is how a report and a refusal name it.
    pub(crate) target: String,
    /// The target resolved against the target root.
    pub(crate) path: PathBuf,
    /// The complete bytes the target will hold — for a `Composed` target, the whole file
    /// with only its owned region replaced.
    pub(crate) contents: String,
    /// What the target held, or `None` if it did not exist. The rollback's whole input.
    pub(crate) previous: Option<String>,
    /// The class that decided [`Self::contents`].
    pub(crate) ownership_class: OwnershipClass,
    /// The scope that decided nothing.
    pub(crate) publication_scope: PublicationScope,
    /// What the write will be reported as.
    pub(crate) outcome: PlacementOutcome,
}

/// Every intent decided, or the first one that cannot be.
pub(crate) fn Plan(
    intents: &[MaterializationIntent],
    roots: &MaterializationRoots,
    file_system: &impl FileSystem,
) -> Result<Vec<PlannedWrite>, MaterializationError>
{
    let mut planned = Vec::with_capacity(intents.len());
    for intent in intents
    {
        let write = Planned(intent, roots, file_system).map_err(|refusal| {
            return MaterializationError::Refused { target: intent.target.clone(), refusal };
        })?;

        planned.push(write);
    }

    return Ok(planned);
}

/// One intent decided.
fn Planned(
    intent: &MaterializationIntent,
    roots: &MaterializationRoots,
    file_system: &impl FileSystem,
) -> Result<PlannedWrite, Refusal>
{
    Admissible(intent)?;
    let source = Source_Text(intent, roots, file_system)?;
    let path = roots.Target_Path(&intent.target);
    let previous = Previous_Text(&path, file_system)?;
    let (contents, outcome) = Contents(intent, &source, previous.as_deref())?;

    return Ok(PlannedWrite {
        surface: intent.surface.clone(),
        target: intent.target.clone(),
        path,
        contents,
        previous,
        ownership_class: intent.ownership_class,
        publication_scope: intent.publication_scope,
        outcome,
    });
}

/// Whether an intent is one this mechanism may perform at all, before anything is read.
///
/// The publication scope is not consulted, here or anywhere below. `OD-PACKAGE-005` makes it
/// a property of where the asset may travel afterwards, and a mechanism that refused a write
/// on it would have folded two independent axes into one.
fn Admissible(intent: &MaterializationIntent) -> Result<(), Refusal>
{
    if intent.ownership_class == OwnershipClass::UserOwned
    {
        return Err(Refusal::UserOwnedTarget);
    }
    if target::Is_Absolute(&intent.target)
    {
        return Err(Refusal::AbsoluteTarget);
    }
    if target::Escapes_The_Root(&intent.target)
    {
        return Err(Refusal::EscapingTarget);
    }

    Source_Stays_Inside(&intent.source)?;

    return Region_Matches_The_Class(intent);
}

/// The source is judged by the same two predicates as the target: a mechanism that read
/// outside the tree it was pointed at would be placing content nobody declared.
fn Source_Stays_Inside(source: &str) -> Result<(), Refusal>
{
    if target::Is_Absolute(source)
    {
        return Err(Refusal::AbsoluteSource);
    }
    if target::Escapes_The_Root(source)
    {
        return Err(Refusal::EscapingSource);
    }

    return Ok(());
}

/// A region is required exactly where it means something and refused everywhere else.
///
/// Both directions refuse rather than default. A `Composed` intent with no region names no
/// bytes as its own, and a region declared against a class that overwrites the whole file is
/// two declarations that disagree — reading either one silently is how a file somebody meant
/// to protect half of gets replaced entirely.
fn Region_Matches_The_Class(intent: &MaterializationIntent) -> Result<(), Refusal>
{
    return match (intent.ownership_class, intent.owned_region.is_some())
    {
        (OwnershipClass::Composed, false) => Err(Refusal::UndeclaredOwnedRegion),
        (OwnershipClass::GeneratedOwned, true) => Err(Refusal::OwnedRegionOnAnUncomposedTarget),
        _ => Ok(()),
    };
}

/// The source's bytes, read as one file.
fn Source_Text(intent: &MaterializationIntent, roots: &MaterializationRoots, file_system: &impl FileSystem) -> Result<String, Refusal>
{
    return file_system.Read_To_String(&roots.Source_Path(&intent.source)).map_err(|error| {
        return Refusal::UnreadableSource { cause: error.to_string() };
    });
}

/// What the target holds now, or `None` if it holds nothing because it is not there.
///
/// An existing target that does not read is a refusal and not an absence: treating it as
/// absent would plan a write with nothing to restore, so a later rollback would delete a file
/// the run never created.
fn Previous_Text(path: &Path, file_system: &impl FileSystem) -> Result<Option<String>, Refusal>
{
    if !file_system.Exists(path)
    {
        return Ok(None);
    }

    return file_system.Read_To_String(path).map(Some).map_err(|error| {
        return Refusal::UnreadableTarget { cause: error.to_string() };
    });
}

/// The bytes the target will hold, and what that write is.
///
/// `OD-PACKAGE-004`'s three classes, one arm each.
fn Contents(intent: &MaterializationIntent, source: &str, previous: Option<&str>) -> Result<(String, PlacementOutcome), Refusal>
{
    return match intent.ownership_class
    {
        OwnershipClass::GeneratedOwned => Ok((source.to_owned(), Overwrite_Outcome(previous))),
        OwnershipClass::Composed => Composed_Contents(intent, source, previous),
        // Unreachable while `Admissible` runs first, and refused again rather than falling
        // through to a write. A guard that holds only because of the order of two calls is
        // one reordering away from being the mistake this whole crate exists to prevent.
        OwnershipClass::UserOwned => Err(Refusal::UserOwnedTarget),
    };
}

/// A `GeneratedOwned` write destroyed bytes, or it did not.
fn Overwrite_Outcome(previous: Option<&str>) -> PlacementOutcome
{
    if previous.is_some()
    {
        return PlacementOutcome::Overwritten;
    }

    return PlacementOutcome::Created;
}

/// The whole file, with only the declared owned region replaced.
fn Composed_Contents(intent: &MaterializationIntent, source: &str, previous: Option<&str>) -> Result<(String, PlacementOutcome), Refusal>
{
    let Some(region) = intent.owned_region.as_ref()
    else
    {
        return Err(Refusal::UndeclaredOwnedRegion);
    };
    let Some(previous) = previous
    else
    {
        return Err(Refusal::AbsentComposedTarget);
    };

    return composed::Spliced(previous, region, source).map(|contents| return (contents, PlacementOutcome::OwnedRegionReplaced));
}
