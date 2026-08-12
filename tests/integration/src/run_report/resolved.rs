//! Which provider the registry chose, and how far its answer can be stood behind.

use nomos_capability::Selection;
use nomos_contracts::Applicability;

/// Which provider the registry chose, and how far its answer can be stood behind.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
pub struct Resolved
{
    pub selection: Selection,
    pub applicability: Applicability,
}
