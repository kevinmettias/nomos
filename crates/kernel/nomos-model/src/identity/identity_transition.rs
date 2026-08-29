//! The transition kind that identity comparison produces.

use crate::IdentityTransitionKind;
use crate::Transition;

/// A change of identity between two snapshots.
pub type IdentityTransition = Transition<IdentityTransitionKind>;
