//! The path a rename left behind, and what became of removing it.

use std::path::PathBuf;

use crate::spec_outcome::VacateOutcome;

/// The path a rename left behind, and what became of removing it.
#[derive(Debug)]
pub struct Vacated
{
    /// The path the rename moved the record away from.
    pub path: PathBuf,
    /// What removing it did.
    pub outcome: VacateOutcome,
}
