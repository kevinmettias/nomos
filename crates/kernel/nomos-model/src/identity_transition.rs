//! The transition kind that identity comparison produces.

use crate::identity_transition_kind::IdentityTransitionKind;
use crate::transition::Transition;

/// A change of identity between two snapshots.
pub type IdentityTransition = Transition<IdentityTransitionKind>;
