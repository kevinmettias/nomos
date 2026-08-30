//! The transition kind that identity comparison produces.

use crate::IdentityTransitionKind;

/// A change of identity between two snapshots.
pub type Transition = crate::Transition<IdentityTransitionKind>;
