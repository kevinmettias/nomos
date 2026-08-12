//! The transition kind that identity comparison produces.

use crate::identity::IdentityTransitionKind;
use crate::transition::Transition;

/// A change of identity between two snapshots.
pub type IdentityTransition = Transition<IdentityTransitionKind>;
