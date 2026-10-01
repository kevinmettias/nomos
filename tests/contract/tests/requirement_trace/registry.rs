//! Reading the registry, which refuses rather than skips.
//!
//! `P42-REQUIREMENT-TRACE-STALENESS-RULE-2` promoted the refusing reader itself into
//! `nomos_cap_requirement_trace::{Entries, Parse, Is_Requirement_Id}` — this module is a
//! thin wrapper over it, not a second copy, threading the real disk
//! ([`nomos_platform_std::StdFileSystem`]) through the [`nomos_platform::FileSystem`] port
//! that crate's reader now takes so it is reachable from a real provider as well as from
//! this suite. [`Committed`] alone is this test suite's own: panicking on a refusal is
//! right for a corpus this repository's own authoring process controls completely, and
//! wrong for a provider judging an arbitrary repository, which is why it stays here rather
//! than moving with the rest.

use crate::assessment::Assessment;
use nomos_platform_std::StdFileSystem;
use std::path::Path;

/// Every committed assessment, or a panic naming the entry that would not read.
pub(crate) fn Committed(root: &Path) -> Vec<Assessment>
{
    return Entries(&root.join(nomos_cap_requirement_trace::REGISTRY))
        // `Entries` is fallible so a control can hand it a bad directory and read the refusal
        // back. The committed registry has no such caller: an entry that will not read is an
        // assessment that stopped being counted, and the floor would measure a set nobody
        // chose while still reporting a number.
        .unwrap_or_else(|refusal| panic!("the committed registry must read: {refusal}"));
}

/// Every assessment in a directory, sorted by requirement.
///
/// Takes the directory rather than finding it, so a control can hand it one.
pub(crate) fn Entries(directory: &Path) -> Result<Vec<Assessment>, String>
{
    return nomos_cap_requirement_trace::Assessments_In(directory, &StdFileSystem);
}

/// One entry, or the reason it is not one.
pub(crate) fn Parse(source: nomos_cap_requirement_trace::EntrySource<'_>) -> Result<Assessment, String>
{
    return nomos_cap_requirement_trace::Parse_Assessment(source);
}

/// Whether a string is a corpus requirement identifier: a family, then three digits.
pub(crate) fn Is_Requirement_Id(stem: &str) -> bool
{
    return nomos_cap_requirement_trace::Is_Requirement_Id(stem);
}
